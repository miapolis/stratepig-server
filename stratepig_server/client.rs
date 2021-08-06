use crate::player::*;
use crate::Endpoint;

pub struct Client {
    pub id: usize,
    pub endpoint: Endpoint,
    pub game_room_id: usize,
    pub room_player: Option<RoomPlayer>,
    pub player: Option<Player>,
}

impl Client {
    pub fn new(id: usize, endpoint: Endpoint) -> Self {
        Self {
            id,
            endpoint,
            game_room_id: 0,
            room_player: None,
            player: None,
        }
    }

    pub fn set_game_room(&mut self, id: usize) {
        self.game_room_id = id;
    }

    pub fn set_player(&mut self, player: Player) {
        self.player = Some(player);
    }

    pub fn reset(&mut self) {
        if self.player.is_none() || self.room_player.is_none() {
            return;
        }
        self.room_player.as_mut().unwrap().reset();
        self.player.as_mut().unwrap().reset();
    }
}
