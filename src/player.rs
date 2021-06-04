use crate::board::*;
use crate::client::Client;

pub struct Player {
    pub role: PlayerRole,
    pub scene_index: u8,

    pub is_ready: bool,
    pub play_again: bool,

    pub current_buffer: u64,

    pub board: Board,
    pub init_board: Board,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PlayerRole {
    One = 1,
    Two = 2,
}

impl PlayerRole {
    pub fn opp(&self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::One,
        }
    }
}

impl Player {
    /// Constructs a new player instance given a role
    fn new(role: PlayerRole) -> Self {
        Self {
            role,
            scene_index: 1,
            is_ready: false,
            play_again: false,
            current_buffer: 0,
            board: Board::new(),
            init_board: Board::new(),
        }
    }

    pub fn initialize_setup(&mut self, setup: Vec<Piece>) {
        self.init_board = setup.clone();
        self.board = setup;
    }

    pub fn reset(&mut self) {
        self.is_ready = false;
        self.board = Board::new();
        self.init_board = Board::new();
        self.play_again = false;
        self.current_buffer = 0;
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

    pub fn reset(&mut self) {
        self.ready = false;
    }
}
