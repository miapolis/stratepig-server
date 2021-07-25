use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{RwLock, RwLockReadGuard};
use std::time::Duration;
use tokio::time;

use crate::client::Client;
use crate::packet::{RoomTimerUpdatePacket, TurnInitPacket, TurnSecondUpdatePacket, WinPacket};
use crate::player::{Player, PlayerRole};
use crate::util::unix_now;
use crate::util::unix_timestamp_to;
use crate::win::WinType;
use crate::GameServer;

use crate::message_room;

use stratepig_game::Pig;

#[derive(Debug)]
pub struct GameRoomInner {
    pub id: usize,
    pub code: String,
    pub client_ids: Vec<usize>,
    pub in_game: bool,
    pub game_phase: u8,
    pub game_ended: bool,
    pub settings: GameRoomSettings,
    pub fake_enemy: Option<Player>,
    pub last_seen_at: u64,

    pub current_turn: PlayerRole,
    pub room_ticker: Option<tokio::task::JoinHandle<()>>,
    pub game_ticker: Option<tokio::task::JoinHandle<()>>,
    pub last_buffer_timestamp: Option<u64>,
    pub game_start_timestamp: Option<u64>,
}

type Inner = Arc<RwLock<GameRoomInner>>;
#[derive(Debug)]
pub struct GameRoom(Inner);

impl GameRoom {
    pub fn new(id: usize, code: String) -> Self {
        Self(Arc::new(RwLock::new(GameRoomInner {
            id,
            code,
            client_ids: Vec::new(),
            in_game: false,
            game_phase: 1,
            game_ended: false,
            settings: GameRoomSettings::new(GameMode::Original, 600, 15, 300),
            fake_enemy: None,
            last_seen_at: unix_now(),

            current_turn: PlayerRole::One,
            room_ticker: None,
            game_ticker: None,
            last_buffer_timestamp: None,
            game_start_timestamp: None,
        })))
    }

    pub fn get(&self) -> &Inner {
        &self.0
    }

    pub fn inner(&self) -> RwLockReadGuard<GameRoomInner> {
        self.0.read().unwrap()
    }

    pub fn id(&self) -> usize {
        self.0.read().unwrap().id
    }

    pub fn clients(&self) -> Vec<usize> {
        self.0.read().unwrap().client_ids.clone()
    }

    pub fn load_default_settings(&self) {
        self.0.write().unwrap().settings = GameRoomSettings::default();
    }

    pub fn other_id(&self, id: usize) -> usize {
        let clients = self.clients();
        let other: Vec<&usize> = clients.iter().filter(|x| **x != id).collect();
        *other[0]
    }

    pub fn get_active_id(&self, game: &GameServer) -> usize {
        let role = self.inner().current_turn;
        for id in self.clients().iter() {
            let player = game.get_player(*id).unwrap();
            if player.role == role {
                return *id;
            }
        }
        panic!("Client options exhausted!");
    }

    pub fn store_seen(&self) {
        self.get().write().unwrap().last_seen_at = unix_now();
    }

    pub async fn start(&self, game: &GameServer, in_secs: u64) {
        let inner = self.get().clone();
        let duration = Duration::from_secs(in_secs);
        let timestamp = unix_timestamp_to(duration);

        let packet = RoomTimerUpdatePacket {
            timestamp: timestamp as i64,
        };
        game.message_room(self, packet).await;

        let handle = tokio::task::spawn(async move {
            time::sleep(duration).await;
            inner.write().unwrap().in_game = true;
        });
        self.get().write().unwrap().room_ticker = Some(handle);
    }

    pub fn cancel_start(&self) {
        let mut write = self.get().write().unwrap();
        if let Some(t) = &write.room_ticker {
            t.abort();
            write.room_ticker = None;
        }
    }

    pub async fn start_phase_two(&self) {
        let mut write = self.get().write().unwrap();
        write.game_phase = 2;
        write.game_start_timestamp = Some(unix_now());
    }

    pub async fn start_player_turn(&self, game: &GameServer, delay: bool) {
        let role = self.inner().current_turn;

        let inner = self.get().clone();
        let server = game.server.clone();

        let player = game.get_player(self.get_active_id(game));
        if player.is_none() {
            return;
        }
        let player_buffer = player.unwrap().current_buffer;
        let mut write = self.get().write().unwrap();

        let handle = tokio::task::spawn(async move {
            if delay {
                time::sleep(Duration::from_secs(4)).await;
            }

            let packet = TurnInitPacket { role: role as u32 };
            {
                // For some weird reason this is required to be in a separate scope
                message_room!(server, inner, packet);
            }

            let turn_duration =
                Duration::from_secs(inner.read().unwrap().settings.turn_time as u64);
            let turn_timestamp = unix_timestamp_to(turn_duration);

            let packet = TurnSecondUpdatePacket {
                role: role as u32,
                turn_timestamp,
                is_buffer: false,
            };
            {
                message_room!(server, inner, packet);
            }

            time::sleep(turn_duration).await;

            let buffer_duration = Duration::from_secs(player_buffer as u64);
            let buffer_timestamp = unix_timestamp_to(buffer_duration);

            let packet = TurnSecondUpdatePacket {
                role: role as u32,
                turn_timestamp: buffer_timestamp,
                is_buffer: true,
            };
            {
                message_room!(server, inner, packet);
            }

            inner.write().unwrap().last_buffer_timestamp = Some(unix_now());

            time::sleep(buffer_duration).await;

            inner.write().unwrap().game_ended = true;

            {
                let read = inner.read().unwrap();
                let start = read.game_start_timestamp.unwrap_or(unix_now());
                let elapsed = unix_now() - start;

                let packet = WinPacket {
                    role: read.current_turn.opp() as u32,
                    win_type: WinType::OutOfTime as u32,
                    elapsed,
                    immediate: WinType::OutOfTime.immediate(),
                };

                drop(read);

                message_room!(server, inner, packet);
            }
        });
        write.game_ticker = Some(handle);
    }

    pub fn reset(&self) {
        let mut write = self.get().write().unwrap();

        write.current_turn = PlayerRole::One;
        write.game_phase = 1;
        write.in_game = false;
        write.game_ended = false;

        write.last_buffer_timestamp = None;
        write.game_start_timestamp = None;

        write.abort_all_tickers();
    }
}

impl GameRoomInner {
    pub fn abort_all_tickers(&mut self) {
        if let Some(t) = &self.room_ticker {
            t.abort();
            self.room_ticker = None;
        }
        if let Some(t) = &self.game_ticker {
            t.abort();
            self.game_ticker = None;
        }
    }
}

impl Drop for GameRoomInner {
    fn drop(&mut self) {
        self.client_ids.clear();
        self.abort_all_tickers();
    }
}

impl GameServer {
    pub fn get_client_by_name(&self, room: &GameRoom, username: &str) -> Option<&Client> {
        for id in room.clients() {
            let client = self.all_clients.get(id).unwrap();
            if let None = client.room_player {
                continue;
            }
            if client.room_player.as_ref().unwrap().username.eq(username) {
                return Some(client);
            }
        }
        None
    }

    pub fn generate_safe_username(&self, room: &GameRoom, username: &str) -> String {
        let mut final_username = String::from(username);
        if let Some(_) = self.get_client_by_name(room, &final_username) {
            let mut i = 1;
            while let Some(_) = self.get_client_by_name(room, &final_username) {
                final_username = format!("{} {}", username, i);
                i += 1;
            }
        }
        final_username
    }

    pub fn get_other_player(&self, room: &GameRoom, id: usize) -> Option<&Client> {
        let result: Vec<usize> = room.clients().into_iter().filter(|x| *x != id).collect();
        if result.len() == 0 {
            return None;
        } else {
            return Some(self.all_clients.get(result[0]).unwrap());
        }
    }
}

#[derive(Debug)]
pub struct GameRoomSettings {
    pub game_mode: GameMode,
    pub placement_time: u32,
    pub turn_time: u32,
    pub buffer_time: u32,

    pub pig_config: HashMap<Pig, u8>,
}

impl GameRoomSettings {
    pub fn new(game_mode: GameMode, placement_time: u32, turn_time: u32, buffer_time: u32) -> Self {
        Self {
            game_mode,
            placement_time,
            turn_time,
            buffer_time,
            pig_config: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn new_with_pigs(
        game_mode: GameMode,
        placement_time: u32,
        turn_time: u32,
        buffer_time: u32,
        pig_config: HashMap<Pig, u8>,
    ) -> Self {
        Self {
            game_mode,
            placement_time,
            turn_time,
            buffer_time,
            pig_config,
        }
    }

    pub fn default() -> Self {
        Self {
            game_mode: GameMode::Original,
            placement_time: 300,
            turn_time: 15,
            buffer_time: 300,
            pig_config: get_pig_config_for_mode(GameMode::Original).unwrap(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum GameMode {
    Original = 1,
    Infiltrator = 2,
    Duel = 3,
    Custom = 4,
}

impl GameMode {
    pub const MAX: u8 = 4;

    pub fn from(val: u8) -> Self {
        match val {
            1 => Self::Original,
            2 => Self::Infiltrator,
            3 => Self::Duel,
            _ => Self::Custom,
        }
    }
}

pub struct SettingsGroup {
    pub loopable: bool,
    pub min_val: i32,
    pub max_val: i32,
    pub interval: u32,
    pub default: i32,
}

lazy_static! {
    pub static ref SETTINGS_GROUPS: HashMap<u8, SettingsGroup> = {
        let mut map = HashMap::new();
        map.insert(
            1,
            SettingsGroup {
                loopable: false,
                min_val: 30,
                max_val: 600,
                interval: 30,
                default: 300,
            },
        );
        map.insert(
            2,
            SettingsGroup {
                loopable: true,
                min_val: 0,
                max_val: 30,
                interval: 1,
                default: 15,
            },
        );
        map.insert(
            3,
            SettingsGroup {
                loopable: false,
                min_val: 0,
                max_val: 900,
                interval: 30,
                default: 300,
            },
        );
        map
    };
}

pub fn get_pig_config_for_mode(mode: GameMode) -> Option<HashMap<Pig, u8>> {
    let mut map = HashMap::new();
    match mode {
        GameMode::Original => {
            map.insert(Pig::Bomb, 6);
            map.insert(Pig::Spy, 1);
            map.insert(Pig::Infiltrator, 0);
            map.insert(Pig::Flag, 1);
            map.insert(Pig::Scout, 8);
            map.insert(Pig::Miner, 5);
            map.insert(Pig::Sergeant, 4);
            map.insert(Pig::Lieutenant, 4);
            map.insert(Pig::Chemist, 4);
            map.insert(Pig::Major, 3);
            map.insert(Pig::Colonel, 2);
            map.insert(Pig::General, 1);
            map.insert(Pig::Kingo, 1);
            Some(map)
        }
        GameMode::Infiltrator => {
            map.insert(Pig::Bomb, 6);
            map.insert(Pig::Spy, 1);
            map.insert(Pig::Infiltrator, 1);
            map.insert(Pig::Flag, 1);
            map.insert(Pig::Scout, 7);
            map.insert(Pig::Miner, 5);
            map.insert(Pig::Sergeant, 4);
            map.insert(Pig::Lieutenant, 4);
            map.insert(Pig::Chemist, 4);
            map.insert(Pig::Major, 3);
            map.insert(Pig::Colonel, 2);
            map.insert(Pig::General, 1);
            map.insert(Pig::Kingo, 1);
            Some(map)
        }
        GameMode::Duel => {
            map.insert(Pig::Bomb, 2);
            map.insert(Pig::Spy, 1);
            map.insert(Pig::Infiltrator, 0);
            map.insert(Pig::Flag, 1);
            map.insert(Pig::Scout, 2);
            map.insert(Pig::Miner, 2);
            map.insert(Pig::Sergeant, 0);
            map.insert(Pig::Lieutenant, 0);
            map.insert(Pig::Chemist, 0);
            map.insert(Pig::Major, 0);
            map.insert(Pig::Colonel, 0);
            map.insert(Pig::General, 1);
            map.insert(Pig::Kingo, 1);
            Some(map)
        }
        _ => None,
    }
}

pub fn get_settings_vars(mode: GameMode) -> SettingsVars {
    if mode == GameMode::Duel {
        return SettingsVars {
            buffer_time: 180,
            ..Default::default()
        };
    }
    return Default::default();
}

pub struct SettingsVars {
    pub turn_time: u32,
    pub buffer_time: u32,
}

impl std::default::Default for SettingsVars {
    fn default() -> Self {
        Self {
            turn_time: 15,
            buffer_time: 300,
        }
    }
}

#[derive(Debug)]
pub enum GameRoomError {
    NotFound,
    Started,
    Full,
}
