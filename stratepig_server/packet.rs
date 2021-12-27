use serde::{Deserialize, Serialize};

use crate::stratepig_macros::{client_packet, server_packet};
use crate::PacketBody;
use stratepig_core;

////////////////////////////////////////
////// SERVER PACKETS //////////////////
////////////////////////////////////////

#[server_packet(0)]
pub struct KeepAlivePacket;

#[server_packet(1)]
pub struct WelcomePacket {
    pub version: String,
    pub my_id: String, // TODO: use usize eventually
}

#[server_packet(2)]
pub struct KickedPacket {
    pub msg: String,
}

#[server_packet(3)]
pub struct ClientDisconnectPacket {
    pub id: String,
    pub timestamp: u64,
}

#[server_packet(4)]
pub struct RoomPlayerAddPacket {
    pub id: String,
    pub client_count: i32,
    pub username: String,
    pub ready: bool,
    pub icon: i32,
}

#[server_packet(5)]
pub struct RoomPlayerUpdatedReadyStatePacket {
    pub id: String,
    pub ready: bool,
}

#[server_packet(6)]
pub struct FailCreateGamePacket;

#[server_packet(7)]
pub struct ErrJoinGamePacket {
    pub msg: String,
}

#[server_packet(8)]
pub struct ClientInfoPacket {
    pub role: u32,
}

#[server_packet(9)]
pub struct GameInfoPacket {
    pub code: String,
    pub game_mode: i32,
    pub placement_time: u32,
    pub turn_time: u32,
    pub buffer_time: u32,
    pub pig_config: Vec<(u32, u32)>,
}

#[server_packet(10)]
pub struct UpdatedPigIconPacket {
    pub id: String,
    pub icon: i32,
}

#[server_packet(11)]
pub struct SettingsValueChangedPacket {
    pub id: u32,
    pub value: u32,
}

#[server_packet(12)]
pub struct PigItemValueChangedPacket {
    pub pig: u32,
    pub amount: u32,
}

#[server_packet(13)]
pub struct PigConfigValueChangedPacket {
    pub turn_time: u32,
    pub buffer_time: u32,
    pub pig_config: Vec<(u32, u32)>,
}

#[server_packet(14)]
pub struct RoomTimerUpdatePacket {
    pub timestamp: i128,
    pub server_now: u128,
}

#[server_packet(15)]
pub struct BothClientsLoadedGamePacket;

#[server_packet(17)]
pub struct GamePlayerUpdatedReadyStatePacket {
    pub id: String,
    pub ready: bool,
}

#[server_packet(18)]
pub struct OpponentPigPlacementPacket {
    pub locations: Vec<u8>,
}

#[server_packet(19)]
pub struct MoveDataPacket {
    pub role: u32,
    pub from: u8,
    pub to: u8,
    pub bundle_null: bool,
}

#[server_packet(19)]
pub struct MoveDataAttackPacket {
    pub role: u32,
    pub from: u8,
    pub to: u8,
    pub bundle_null: bool,
    pub result: i32,
    pub init_type: u32,
    pub target_type: u32,
}

#[server_packet(20)]
pub struct TurnInitPacket {
    pub role: u32,
}

#[server_packet(21)]
pub struct TurnSecondUpdatePacket {
    pub role: u32,
    pub turn_timestamp: u128,
    pub server_now: u128,
    pub is_buffer: bool,
}

#[server_packet(22)]
pub struct WinPacket {
    pub role: u32,
    pub win_type: u32,
    pub elapsed: u64,
    pub immediate: bool,
}

#[server_packet(23)]
pub struct EnemyPieceDataPacket {
    pub data: Vec<(u8, u8)>,
}

#[server_packet(24)]
pub struct ClientPlayAgainPacket {
    pub id: String,
}

////////////////////////////////////////
////// CLIENT PACKETS //////////////////
////////////////////////////////////////

#[client_packet(0)]
pub struct BaseGuardPacket {
    pub my_id: String,
}

#[client_packet(1)]
pub struct GameRequestDefaultPacket {
    pub my_id: String,
    pub is_hosting: bool,
    pub username: String,
    pub icon: i32,
    pub code: String,
    pub data_null: bool,
}

#[client_packet(1)]
pub struct GameRequestFullPacket {
    pub my_id: String,
    pub is_hosting: bool,
    pub username: String,
    pub icon: i32,
    pub code: String,
    pub data_null: bool,
    pub game_mode: i32,
    pub placement_secs: i32,
    pub turn_secs: i32,
    pub buffer_secs: i32,
    pub pig_config: Vec<(i32, i32)>,
}

#[client_packet(2)]
pub struct UpdateReadyStatePacket {
    pub my_id: String,
    pub ready: bool,
}

#[client_packet(3)]
pub struct UpdatePigIconPacket {
    pub my_id: String,
    pub icon: u32,
}

#[client_packet(4)]
pub struct UpdateSettingsValue {
    pub my_id: String,
    pub settings_id: u32,
    pub increased: bool,
}

#[client_packet(5)]
pub struct UpdatePigItemValuePacket {
    pub my_id: String,
    pub pig: u32,
    pub increased: bool,
}

#[client_packet(6)]
pub struct FinishedSceneLoadPacket {
    pub my_id: String,
    pub scene_index: u32,
}

#[client_packet(7)]
pub struct GamePlayerReadyDataDefaultPacket {
    pub my_id: String,
    pub ready: bool,
}

#[client_packet(7)]
pub struct GamePlayerReadyDataFullPacket {
    pub my_id: String,
    pub ready: bool,
    pub board: Vec<(u32, u32)>,
}

#[client_packet(8)]
pub struct MovePacket {
    pub my_id: String,
    pub from_location: u8,
    pub to_location: u8,
}

// Useless packets
#[client_packet(9)]
pub struct SurrenderPacket;
#[client_packet(10)]
pub struct LeaveGamePacket;
#[client_packet(11)]
pub struct PlayAgainPacket;

#[allow(dead_code)]
#[derive(Debug)]
/// Messages that the server can send to the client
pub enum ServerMessage {
    KeepAlive = 0,
    Welcome = 1,
    Kicked = 2,
    ClientDisconnect = 3,
    RoomPlayerAdd = 4,
    RoomPlayerUpdatedReadyState = 5,
    FailCreateGame = 6,
    ErrorJoinGame = 7,
    ClientInfo = 8,
    GameInfo = 9,
    UpdatedPigIcon = 10,
    SettingsValueChanged = 11,
    PigItemValueChanged = 12,
    PigConfigValueChanged = 13,
    RoomTimerUpdate = 14,
    BothClientsLoadedGame = 15,
    GameTimerUpdate = 16,
    GamePlayerUpdatedReadyState = 17,
    OpponentPigPlacement = 18,
    MoveData = 19,
    TurnInit = 20,
    TurnSecondUpdate = 21,
    Win = 22,
    EnemyPieceData = 23,
    ClientPlayAgain = 24,
    Null,
}

impl ServerMessage {
    pub fn from(id: u8) -> Self {
        match id {
            1 => Self::Welcome,
            2 => Self::Kicked,
            3 => Self::ClientDisconnect,
            4 => Self::RoomPlayerAdd,
            5 => Self::RoomPlayerUpdatedReadyState,
            6 => Self::FailCreateGame,
            7 => Self::ErrorJoinGame,
            8 => Self::ClientInfo,
            9 => Self::GameInfo,
            10 => Self::UpdatedPigIcon,
            11 => Self::SettingsValueChanged,
            12 => Self::PigItemValueChanged,
            13 => Self::PigConfigValueChanged,
            14 => Self::RoomTimerUpdate,
            15 => Self::BothClientsLoadedGame,
            16 => Self::GameTimerUpdate,
            17 => Self::GamePlayerUpdatedReadyState,
            18 => Self::OpponentPigPlacement,
            19 => Self::MoveData,
            20 => Self::TurnInit,
            21 => Self::TurnSecondUpdate,
            22 => Self::Win,
            23 => Self::EnemyPieceData,
            24 => Self::ClientPlayAgain,
            _ => Self::Null,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
/// Messages the client can send to the server
pub enum ClientMessage {
    GameRequestSent = 1,
    UpdateReadyState = 2,
    UpdatePigIcon = 3,
    UpdateSettingsValue = 4,
    UpdatePigItemValue = 5,
    FinishedSceneLoad = 6,
    GamePlayerReadyData = 7,
    Move = 8,
    Surrender = 9,
    LeaveGame = 10,
    PlayAgain = 11,
    Null,
}

impl ClientMessage {
    pub fn from(id: u8) -> Self {
        match id {
            1 => Self::GameRequestSent,
            2 => Self::UpdateReadyState,
            3 => Self::UpdatePigIcon,
            4 => Self::UpdateSettingsValue,
            5 => Self::UpdatePigItemValue,
            6 => Self::FinishedSceneLoad,
            7 => Self::GamePlayerReadyData,
            8 => Self::Move,
            9 => Self::Surrender,
            10 => Self::LeaveGame,
            11 => Self::PlayAgain,
            _ => Self::Null,
        }
    }
}
