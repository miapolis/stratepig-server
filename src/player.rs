use crate::board::*;
use crate::client::Client;

pub struct Player {
    pub role: PlayerRole,
    pub scene_index: u8,

    pub is_ready: bool,
    pub play_again: bool,

    pub current_time: u32,
    pub current_buffer: u32,

    pub board: Board,
    pub init_board: Board,
}

#[derive(PartialEq, Copy, Clone)]
pub enum PlayerRole {
    One = 1,
    Two = 2,
}

impl Player {
    /// Constructs a new player instance given a role
    fn new(role: PlayerRole) -> Self {
        Self {
            role,
            scene_index: 1,
            is_ready: false,
            play_again: false,
            current_time: 0,
            current_buffer: 0,
            board: Board::new(),
            init_board: Board::new(),
        }
    }

    pub fn initialize_setup(&mut self, setup: Vec<Piece>) {
        self.init_board = setup.clone();
        self.board = setup;
    }
}

pub struct RoomPlayer {
    pub username: String,
    pub ready: bool,
    pub icon: u8,
}

impl RoomPlayer {
    /// Constructs a new room player instance
    pub fn new(role: PlayerRole, username: String, icon: u8, client: &mut Client) -> Self {
        client.set_player(Player::new(role));
        Self {
            username,
            ready: false,
            icon,
        }
    }
}
