use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{RwLock, RwLockReadGuard};
use tokio::time;

use crate::board::Pig;
use crate::client::Client;
use crate::util::unix_now;
use crate::util::unix_timestamp_to;
use crate::GameServer;
use crate::Packet;
use crate::ServerMessage;

#[derive(Debug)]
pub struct GameRoomInner {
    pub id: usize,
    pub code: String,
    pub client_ids: Vec<usize>,
    pub has_started: bool,
    pub settings: GameRoomSettings,

    pub last_seen_at: u64,

    pub room_ticker: Option<tokio::task::JoinHandle<()>>,
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
            has_started: false,
            settings: GameRoomSettings::new(GameMode::Original, 600, 15, 300),

            last_seen_at: unix_now(),

            room_ticker: None,
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

    pub async fn start(&self, game: &GameServer) {
        let inner = self.get().clone();
        let duration = std::time::Duration::from_secs(5);
        let timestamp = unix_timestamp_to(duration);

        let mut packet = Packet::new_id(ServerMessage::RoomTimerUpdate as i32);
        packet.write_u64(timestamp);
        game.message_room(self, packet).await;

        let handle = tokio::task::spawn(async move {
            time::sleep(duration).await;
        });
        inner.write().unwrap().room_ticker = Some(handle);
    }

    pub fn cancel_start(&self) {
        let mut write = self.get().write().unwrap();
        if let Some(t) = &write.room_ticker {
            t.abort();
            write.room_ticker = None;
        }
    }
}

impl GameRoomInner {
    pub fn abort_all_tickers(&mut self) {
        if let Some(t) = &self.room_ticker {
            t.abort();
            self.room_ticker = None;
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
    if mode == GameMode::Original || mode == GameMode::Infiltrator {
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
