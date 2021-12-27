use log::{info, trace, warn};
use message_io::network::{Endpoint, Transport};
use message_io::node::{
    self, NodeHandler, NodeListener, StoredNetEvent, StoredNodeEvent as NodeEvent,
};
use parking_lot::{Mutex, MutexGuard};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::thread;
use std::time;
use vec_map::VecMap;

use stratepig_cli::{self, CliConfig};
use stratepig_core::{Packet, PacketBody};
use stratepig_macros;

mod client;
mod constants;
mod error;
mod game;
mod gameroom;
mod guard;
mod lobby;
mod log_init;
mod macros;
mod packet;
mod player;
mod util;
mod version;
mod win;
use client::Client;
use error::StratepigError;
use gameroom::{GameRoom, GameRoomError};
use guard::{Guard, InGameGuard, InGameStrictGuard, InRoomGuard};
use packet::{ClientMessage::*, *};
use player::{Player, PlayerRole};

type PacketHandler = fn(
    &mut GameServer,
    usize,
    Packet,
) -> Pin<Box<dyn Future<Output = Result<(), StratepigError>> + '_>>;

pub struct GameServer {
    handler: Arc<Mutex<NodeHandler<()>>>,
    config: CliConfig,
    packet_handlers: VecMap<PacketHandler>,
    guards: VecMap<Option<Box<dyn Guard>>>,
    endpoints: Arc<Mutex<HashMap<Endpoint, usize>>>,
    all_clients: HashMap<usize, Client>,
    next_client_id: usize,
    free_client_ids: VecDeque<usize>,
    pub game_rooms: Arc<Mutex<VecMap<GameRoom>>>,
    free_game_room_ids: Arc<Mutex<VecDeque<usize>>>,
    next_game_room_id: Arc<Mutex<usize>>,
    game_room_codes: Arc<Mutex<Vec<String>>>,
}

const MAX_ROOMS: usize = 1000;
const PRUNE_INTERVAL_SECS: u64 = 180;
const MAX_PRUNE_AGE_SECS: u64 = 300;

impl GameServer {
    fn register_packet_handlers(&mut self) {
        macro_rules! register {
            ($id:expr, $p:expr) => {{
                self.packet_handlers
                    .insert($id as usize, (|g, id, p| Box::pin($p(g, id, p))));
                self.guards.insert($id as usize, None);
            }};
        }

        macro_rules! register_guarded {
            ($id:expr, $p:expr, $g:expr) => {{
                self.packet_handlers
                    .insert($id as usize, (|g, id, p| Box::pin($p(g, id, p))));
                self.guards.insert($id as usize, Some(Box::new($g)));
            }};
        }

        register!(GameRequestSent, Self::handle_game_request);

        register_guarded!(
            UpdateReadyState,
            Self::handle_ready_state_change,
            InRoomGuard
        );
        register_guarded!(UpdatePigIcon, Self::handle_update_icon, InRoomGuard);
        register_guarded!(
            UpdateSettingsValue,
            Self::handle_settings_value_update,
            InRoomGuard
        );
        register_guarded!(
            UpdatePigItemValue,
            Self::handle_pig_item_update,
            InRoomGuard
        );
        register_guarded!(
            FinishedSceneLoad,
            Self::handle_client_finish_scene_load,
            InRoomGuard
        );

        register_guarded!(
            GamePlayerReadyData,
            Self::handle_game_player_ready,
            InGameGuard
        );

        register_guarded!(Surrender, Self::handle_surrender, InGameGuard);
        register_guarded!(LeaveGame, Self::handle_client_leave, InGameGuard);
        register_guarded!(PlayAgain, Self::handle_client_play_again, InGameGuard);
        register_guarded!(Move, Self::move_received, InGameStrictGuard);
    }

    async fn start(&mut self, listener: NodeListener<()>) {
        self.run_prune_cycle();
        // Core loop
        let packet_handlers = self.packet_handlers.clone();
        let guards = self.clone_guards();

        let (_task, mut receiver) = listener.enqueue();

        loop {
            match receiver.receive() {
                NodeEvent::Network(event) => match event {
                    StoredNetEvent::Accepted(endpoint, _listener) => {
                        let id = match self.free_client_ids.pop_front() {
                            Some(id) => id,
                            None => {
                                self.next_client_id += 1;
                                self.next_client_id
                            }
                        };
                        self.handle_connection(endpoint, id).await;
                    }
                    StoredNetEvent::Message(endpoint, data) => {
                        if let Ok(header) = stratepig_core::deserialize_packet_header(&data) {
                            let packet_size =
                                stratepig_core::PACKET_HEADER_SIZE + (header.size as usize);
                            let bytes: &[u8] =
                                &data[stratepig_core::PACKET_HEADER_SIZE..packet_size];
                            let body = bytes.to_vec();
                            let packet = Packet { header, body };

                            let endpoints = self.endpoints.lock();
                            let result = endpoints.get(&endpoint);
                            if let Some(id) = result {
                                let id = *id;
                                drop(endpoints);
                                self.handle_data(id, packet, &packet_handlers, &guards)
                                    .await;
                            }
                        }
                    }
                    StoredNetEvent::Disconnected(endpoint) => {
                        self.handle_disconnect(endpoint).await;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    async fn handle_connection(&mut self, endpoint: Endpoint, id: usize) {
        self.endpoints.lock().insert(endpoint, id);
        self.all_clients.insert(id, Client::new(id, endpoint));

        let packet = WelcomePacket {
            version: version::VERSION.to_owned(),
            my_id: id.to_string(),
        };
        self.message_one(id, packet).await;
    }

    async fn handle_disconnect(&mut self, endpoint: Endpoint) {
        let mut endpoints = self.endpoints.lock();
        let id = endpoints.get(&endpoint);
        if let Some(id) = id {
            let id = *id;
            if let Some(client) = self.all_clients.get(&id) {
                let client_id = client.id;
                let game_room_id = client.game_room_id;

                endpoints.remove(&endpoint);
                drop(endpoints);

                if game_room_id != 0 {
                    let id = client.id;
                    self.handle_client_disconnect(game_room_id, id, endpoint)
                        .await;
                }

                self.all_clients.remove(&client_id);
            }
        }
    }

    async fn handle_client_disconnect(&mut self, room_id: usize, id: usize, endpoint: Endpoint) {
        let result = self.get_room(room_id);
        if let Some(_) = result {
            let room = result.unwrap();
            let mut client_ids = room.inner().client_ids.clone();
            if !client_ids.contains(&(id, endpoint)) {
                return;
            }

            let mut write = room.get().write().unwrap();

            write
                .client_ids
                .remove(client_ids.iter().position(|x| x.0 == id).unwrap());
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
                self.handle_transfer_ownership(id, client_ids[0].0).await;
            }
        }
    }

    async fn handle_transfer_ownership(&mut self, leave: usize, stay: usize) {
        if let Some(player) = &self.all_clients.get(&leave).unwrap().player {
            // Host left the game, ownership needs to be transferred
            if player.role == PlayerRole::One {
                let client = self.all_clients.get_mut(&stay).unwrap();
                client.player.as_mut().unwrap().role = PlayerRole::One;
            }
        }
    }

    async fn handle_data(
        &mut self,
        id: usize,
        packet: Packet,
        handlers: &VecMap<PacketHandler>,
        guards: &VecMap<Option<Box<dyn Guard>>>,
    ) {
        let packet_id = packet.header.id as usize;
        if let Some(func) = handlers.get(packet_id) {
            {
                // Evaluate guards
                if let Some(guard) = guards.get(packet_id).unwrap() {
                    if self.config.log_packet_output {
                        info!("Checking guard '{}'", guard.name());
                    }

                    if let Err(err) = guard.guard(id, packet.clone(), &self) {
                        warn!("Guard failed: {:?}", err);
                        return;
                    }
                }

                let res = func(self, id, packet.clone()).await;
                if self.config.log_packet_output {
                    info!(
                        "Client {}: {:?} ==> {:?}",
                        id,
                        ClientMessage::from(packet.header.id),
                        res
                    );
                }
            }
        }
    }

    pub async fn message_one(&self, id: usize, packet: impl PacketBody) {
        if self.config.log_packet_output {
            info!("OUTBOUND({}) => {:?}", id, ServerMessage::from(packet.id()));
        }
        if let Some(client) = self.get_client(id) {
            let endpoint = client.endpoint;
            self.handler.lock().network().send(
                endpoint,
                &stratepig_core::serialize_packet(Box::new(packet)).unwrap(),
            );
        }
    }

    pub async fn message_room(&self, room: &GameRoom, packet: impl PacketBody) {
        let client_ids = room.clients();
        if self.config.log_packet_output {
            info!(
                "OUTBOUND({:?}) => {:?}",
                client_ids,
                ServerMessage::from(packet.id())
            );
        }

        let bytes = &stratepig_core::serialize_packet(Box::new(packet)).unwrap();
        for (id, _endpoint) in client_ids.into_iter() {
            if let Some(client) = self.get_client(id) {
                let endpoint = client.endpoint;
                self.handler.lock().network().send(endpoint, bytes);
            }
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

        let room = GameRoom::new(id, code.clone());
        game_rooms.insert(id, room);
        trace!("New room '{}' created with ID {}", code, id);
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
        self.all_clients.get(&id)
    }

    pub fn get_player(&self, id: usize) -> Option<&Player> {
        self.get_client(id)?.player.as_ref()
    }

    pub fn get_client_mut(&mut self, id: usize) -> Option<&mut Client> {
        self.all_clients.get_mut(&id)
    }

    pub fn get_player_mut(&mut self, id: usize) -> Option<&mut Player> {
        self.get_client_mut(id)?.player.as_mut()
    }

    pub fn get_context(&self, id: usize) -> Option<(&Client, impl Deref<Target = GameRoom> + '_)> {
        let client = self.all_clients.get(&id).unwrap();
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

    fn clone_guards(&self) -> VecMap<Option<Box<dyn Guard>>> {
        let mut map = VecMap::new();

        for (id, guard) in self.guards.iter() {
            if guard.is_none() {
                map.insert(id, None);
            } else {
                let cloned = dyn_clone::clone(guard.as_ref().unwrap());
                map.insert(id, Some(cloned));
            }
        }

        map
    }

    fn run_prune_cycle(&mut self) {
        let game_rooms = self.game_rooms.clone();
        let free_game_room_ids = self.free_game_room_ids.clone();
        let handler = self.handler.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(time::Duration::from_secs(PRUNE_INTERVAL_SECS));

                let now = util::unix_now();

                let mut to_prune = Vec::new();
                for (id, room) in game_rooms.lock().iter_mut() {
                    if !room.inner().in_game || room.inner().game_ended {
                        if now > (room.inner().last_seen_at + MAX_PRUNE_AGE_SECS).into() {
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

                    let packet = KickedPacket {
                        msg: "Room closed due to inactivity.".to_owned(),
                    };
                    let bytes = &stratepig_core::serialize_packet(Box::new(packet)).unwrap();

                    let handler = handler.lock();
                    let endpoints: Vec<Endpoint> =
                        room.clients().into_iter().map(|x| x.1).collect();
                    for endpoint in endpoints.into_iter() {
                        handler.network().send(endpoint, bytes);
                    }

                    drop(room);
                    pruned += 1;
                }

                info!("Pruned {} room(s) | ({})", pruned, game_rooms.len());
            }
        });
    }
}

#[tokio::main]
async fn main() {
    log_init::init();
    info!("Starting Stratepig Server...");

    let config = CliConfig::new();
    config.log();

    let (handler, listener) = node::split::<()>();
    handler
        .network()
        .listen(Transport::Tcp, "0.0.0.0:32500")
        .unwrap();
    let handler = Arc::new(Mutex::new(handler));

    let mut server = GameServer {
        handler,
        config,
        packet_handlers: VecMap::new(),
        guards: VecMap::new(),
        endpoints: Arc::new(Mutex::new(HashMap::new())),
        all_clients: HashMap::new(),
        next_client_id: 1,
        free_client_ids: VecDeque::new(),
        game_rooms: Arc::new(Mutex::new(VecMap::new())),
        next_game_room_id: Arc::new(Mutex::new(0)),
        free_game_room_ids: Arc::new(Mutex::new(VecDeque::new())),
        game_room_codes: Arc::new(Mutex::new(Vec::new())),
    };

    let endpoints = server.endpoints.clone();
    let game_rooms = server.game_rooms.clone();

    thread::spawn(move || loop {
        let result = stratepig_cli::wait_for_command();
        if result.is_err() {
            continue;
        }
        match result.unwrap().as_str() {
            "ss stats" => {
                let len_clients = endpoints.lock().len();
                let len_game_rooms = game_rooms.lock().len();

                println!("--- SERVER STATS ---");
                println!("Number of clients: {}", len_clients);
                println!("Number of rooms: {}", len_game_rooms);
            }
            _ => {}
        }
    });

    ctrlc::set_handler(|| {
        println!("Received exit signal");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    server.register_packet_handlers();
    server.start(listener).await;
}
