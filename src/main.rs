use log::info;
use parking_lot::{Mutex, MutexGuard};
use std::collections::VecDeque;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::thread;
use std::time;
use vec_map::VecMap;

use stratepig_core::server::{Server, ServerEvent};
use stratepig_core::Packet;

mod board;
mod client;
mod config;
mod constants;
mod game;
mod gameroom;
mod lobby;
mod log_init;
mod macros;
mod packet;
mod player;
mod test_util;
mod util;
mod version;
mod win;
use client::Client;
use config::Config;
use gameroom::{GameRoom, GameRoomError};
use packet::{ClientMessage::*, ServerMessage};
use player::{Player, PlayerRole};

type PacketHandler = fn(&mut GameServer, usize, Packet) -> Pin<Box<dyn Future<Output = ()> + '_>>;

pub struct GameServer {
    server: Arc<Mutex<Server>>,
    config: Config,
    packet_handlers: VecMap<PacketHandler>,
    all_clients: VecMap<Client>,
    game_rooms: Arc<Mutex<VecMap<GameRoom>>>,
    free_game_room_ids: Arc<Mutex<VecDeque<usize>>>,
    next_game_room_id: Arc<Mutex<usize>>,
    game_room_codes: Arc<Mutex<Vec<String>>>,
}

const MAX_ROOMS: usize = 1000;
const PRUNE_INTERVAL_SECS: u64 = 180;
const MAX_PRUNE_AGE_SECS: u64 = 300;

impl GameServer {
    fn new(config: Config) -> Self {
        Self {
            config,
            server: Arc::new(Mutex::new(Server::new())),
            packet_handlers: VecMap::new(),
            all_clients: VecMap::new(),
            game_rooms: Arc::new(Mutex::new(VecMap::new())),
            next_game_room_id: Arc::new(Mutex::new(0)),
            free_game_room_ids: Arc::new(Mutex::new(VecDeque::new())),
            game_room_codes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn register_packet_handlers(&mut self) {
        macro_rules! register {
            ($id:expr, $p:expr) => {{
                self.packet_handlers
                    .insert($id as usize, |g, id, p| Box::pin($p(g, id, p)));
            }};
        }

        // rustfmt friendly
        register!(GameRequestSent, Self::handle_game_request);
        register!(LeaveGame, Self::handle_client_leave);
        register!(UpdateReadyState, Self::handle_ready_state_change);
        register!(UpdatePigIcon, Self::handle_update_icon);
        register!(UpdateSettingsValue, Self::handle_settings_value_update);
        register!(FinishedSceneLoad, Self::handle_client_finish_scene_load);
        register!(GamePlayerReadyData, Self::handle_game_player_ready);
        register!(Move, Self::move_received);
        register!(PlayAgain, Self::handle_client_play_again);
    }

    async fn start(&mut self) {
        self.run_prune_cycle();
        // Core loop
        loop {
            let result = self.server.lock().scan();
            match result {
                (id, ServerEvent::Connected) => self.handle_connection(id).await,
                (id, ServerEvent::Disconnected) => self.handle_disconnect(id).await,
                (_, ServerEvent::IoError(err)) => eprintln!("IO error: {}", err),
                (id, ServerEvent::Data(data)) => self.handle_data(id, data).await,
                (_, ServerEvent::Empty) => {}
            }
        }
    }

    async fn handle_connection(&mut self, id: usize) {
        self.all_clients.insert(id, Client::new(id));

        // Send to the player version info and their ID
        let mut packet = Packet::new_id(ServerMessage::Welcome as i32);
        packet.write_str(version::VERSION);
        packet.write_str(&id.to_string());
        self.message_one(id, packet).await;
    }

    async fn handle_disconnect(&mut self, id: usize) {
        if let Some(client) = self.all_clients.get(id) {
            let game_room_id = client.game_room_id;
            if game_room_id != 0 {
                self.handle_client_disconnect(game_room_id, id).await;
            }

            self.all_clients.remove(id);
        }
    }

    async fn handle_client_disconnect(&mut self, room_id: usize, id: usize) {
        let result = self.get_room(room_id);
        if let Some(_) = result {
            let room = result.unwrap();
            let mut client_ids = room.inner().client_ids.clone();
            if !client_ids.contains(&id) {
                return;
            }

            let mut write = room.get().write().unwrap();

            write
                .client_ids
                .remove(client_ids.iter().position(|x| *x == id).unwrap());
            write.in_game = false;
            write.abort_all_tickers(); // Nothing is functional with only one player, tickers don't need to be running
            client_ids = write.client_ids.clone();

            // Drop write before room to prevent deadlock
            drop(write);

            if client_ids.len() >= 1 {
                self.client_disconnected(&room, id).await;
            }

            // Now we can drop the room
            drop(room);

            if client_ids.len() == 1 {
                // If there is still someone left, we have to worry about whether or not
                // they need to be made the host of the room
                self.handle_transfer_ownership(id, client_ids[0]).await;
            }
        }
    }

    async fn handle_transfer_ownership(&mut self, leave: usize, stay: usize) {
        if let Some(player) = &self.all_clients.get(leave).unwrap().player {
            // Host left the game, ownership needs to be transferred
            if player.role == PlayerRole::One {
                let client = self.all_clients.get_mut(stay).unwrap();
                client.player.as_mut().unwrap().role = PlayerRole::One;
            }
        }
    }

    async fn handle_data(&mut self, id: usize, data: Vec<u8>) {
        let mut packet = Packet::new_from_bytes(data);
        let mut packet_length = 0;

        if packet.unread_len() >= 4 {
            packet_length = packet.read_i32().unwrap_or(0);
            if packet_length <= 0 {
                return;
            }
        }

        let packet_bytes = packet
            .read_u8s(packet_length as usize)
            .unwrap_or(&[])
            .to_vec();
        let packet_handlers = self.packet_handlers.clone();
        let mut new_packet = Packet::new_from_bytes(packet_bytes);
        match new_packet.read_i32() {
            Ok(pid) => {
                if let Some(func) = packet_handlers.get(pid as usize) {
                    func(self, id, new_packet).await;
                }
            }
            Err(_) => (),
        }
    }

    pub async fn message_one(&self, id: usize, mut packet: Packet) {
        packet.write_length();
        self.server.lock().message_one(id, packet.to_array());
    }

    pub async fn message_room(&self, room: &GameRoom, mut packet: Packet) {
        packet.write_length();
        let bytes = packet.to_array();
        let server = self.server.lock();
        for id in room.clients().iter() {
            server.message_one(*id, bytes);
        }
    }

    pub fn new_room(&mut self) -> Result<impl Deref<Target = GameRoom> + '_, &str> {
        let mut game_rooms = self.game_rooms.lock();
        if game_rooms.len() >= MAX_ROOMS {
            return Err("There are too many rooms at the moment. Try again later.");
        }

        let id = match self.free_game_room_ids.lock().pop_front() {
            Some(id) => id,
            None => {
                let mut next = self.next_game_room_id.lock();
                *next += 1;
                *next
            }
        };

        let mut code = util::gen_game_room_code();
        // Ensure code is unique, as a 1/456976 chance is still possible
        while self.game_room_codes.lock().contains(&code) {
            code = util::gen_game_room_code();
        }

        let room = GameRoom::new(id, code);
        game_rooms.insert(id, room);
        Ok(MutexGuard::map(game_rooms, |g| g.get_mut(id).unwrap()))
    }

    pub fn get_room(&self, id: usize) -> Option<impl Deref<Target = GameRoom> + '_> {
        let game_rooms = self.game_rooms.lock();
        if game_rooms.get(id).is_none() {
            return None;
        }
        Some(MutexGuard::map(game_rooms, |g| g.get_mut(id).unwrap()))
    }

    pub fn get_room_by_code<'ret, 'me: 'ret>(
        &'me self,
        code: &str,
    ) -> Option<impl Deref<Target = GameRoom> + 'ret> {
        let mut game_rooms: MutexGuard<'ret, VecMap<GameRoom>> = self.game_rooms.lock();
        for (id, room) in game_rooms.iter_mut() {
            if room.inner().code == code {
                return Some(MutexGuard::map(game_rooms, |g| g.get_mut(id).unwrap()));
            }
        }
        None
    }

    pub fn try_join_room(
        &self,
        code: &String,
    ) -> Result<impl Deref<Target = GameRoom> + '_, GameRoomError> {
        let room = self.get_room_by_code(code);
        match room {
            Some(room) => {
                if room.inner().in_game {
                    return Err(GameRoomError::Started);
                } else if room.clients().len() >= 2 {
                    return Err(GameRoomError::Full);
                }
                Ok(room)
            }
            None => Err(GameRoomError::NotFound),
        }
    }

    pub fn get_client(&self, id: usize) -> Option<&Client> {
        self.all_clients.get(id)
    }

    pub fn get_player(&self, id: usize) -> Option<&Player> {
        self.get_client(id)?.player.as_ref()
    }

    pub fn get_client_mut(&mut self, id: usize) -> Option<&mut Client> {
        self.all_clients.get_mut(id)
    }

    pub fn get_player_mut(&mut self, id: usize) -> Option<&mut Player> {
        self.get_client_mut(id)?.player.as_mut()
    }

    pub fn get_context(&self, id: usize) -> Option<(&Client, impl Deref<Target = GameRoom> + '_)> {
        let client = self.all_clients.get(id).unwrap();
        let room_id = client.game_room_id;
        if room_id == 0 || client.room_player.is_none() {
            return None;
        }

        let game_rooms = self.game_rooms.lock();
        if let None = game_rooms.get(room_id) {
            return None;
        }

        Some((
            client,
            MutexGuard::map(game_rooms, |g| g.get_mut(room_id).unwrap()),
        ))
    }

    fn run_prune_cycle(&mut self) {
        let game_rooms = self.game_rooms.clone();
        let free_game_room_ids = self.free_game_room_ids.clone();
        let server = self.server.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(time::Duration::from_secs(PRUNE_INTERVAL_SECS));

                let now = util::unix_now();

                let mut to_prune = Vec::new();
                for (id, room) in game_rooms.lock().iter_mut() {
                    if !room.inner().in_game || room.inner().game_ended {
                        if now > room.inner().last_seen_at + MAX_PRUNE_AGE_SECS {
                            to_prune.push(id);
                        }
                    }
                }

                let mut pruned: usize = 0;
                let mut game_rooms = game_rooms.lock();
                for room_id in to_prune {
                    // Inform each client that they were kicked
                    let room = game_rooms.remove(room_id).unwrap();
                    free_game_room_ids.lock().push_back(room_id);

                    let server = server.lock();

                    for id in room.inner().client_ids.iter() {
                        let mut packet = Packet::new_id(ServerMessage::Kicked as i32);
                        packet.write_str("Room closed due to inactivity.");
                        packet.write_length();
                        server.message_one(*id, packet.to_array());
                    }
                    drop(room);
                    pruned += 1;
                }

                info!("Pruned {} room(s)", pruned);
            }
        });
    }
}

#[tokio::main]
async fn main() {
    log_init::init();
    info!("Starting Stratepig Server...");

    let config = Config::new();
    config.log();
    let mut server = GameServer::new(config);

    server.register_packet_handlers();
    server.start().await;
}
