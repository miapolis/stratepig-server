use bincode::Options;
use serde::{Deserialize, Serialize};

use crate::stratepig_derive::{client_packet, server_packet};
use crate::PacketBody;
use stratepig_core;

////////////////////////////////////////
////// SERVER PACKETS //////////////////
////////////////////////////////////////

#[server_packet(1)]
pub struct WelcomePacket {
    pub version: String,
    pub my_id: String, // TODO: use usize eventually
}

#[server_packet(2)]
pub struct KickedPacket {
    pub msg: String,
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

#[server_packet(13)]
pub struct PigConfigValueChangedPacket {
    pub turn_time: u32,
    pub buffer_time: u32,
    pub pig_config: Vec<(u32, u32)>,
}

#[server_packet(14)]
pub struct RoomTimerUpdatePacket {
    pub timestamp: i64,
}

#[server_packet(20)]
pub struct TurnInitPacket {
    pub role: u32,
}

////////////////////////////////////////
////// CLIENT PACKETS //////////////////
////////////////////////////////////////

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

#[client_packet(10)]
pub struct LeaveGamePacket {
    pub my_id: String,
}

#[allow(dead_code)]
/// Messages that the server can send to the client
pub enum ServerMessage {
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
