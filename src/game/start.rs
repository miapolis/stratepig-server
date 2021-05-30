use crate::board::{self, Piece, Pig};
use crate::Packet;
use crate::{GameRoom, GameServer};
use std::collections::HashMap;
use std::convert::TryInto;

impl GameServer {
    pub async fn handle_game_player_ready(&mut self, id: usize, mut packet: Packet) {
        let id_check = packet.read_string().unwrap_or(String::new());
        if id.to_string() != id_check {
            return;
        }
        let ctx = self.get_context(id);
        if let None = ctx {
            return;
        }
        let (client, room) = ctx.unwrap();
        if !room.inner().has_started {
            return;
        }

        let room_id = room.id();
        drop(room); // TODO: FIX THIS NOW
        if client.player.as_ref().is_none() {
            return;
        }

        let ready = packet.read_bool().unwrap_or(false);

        if !ready {
            let player = self
                .all_clients
                .get_mut(id)
                .unwrap()
                .player
                .as_mut()
                .unwrap();
            player.is_ready = false;
            let reference = self.get_room(room_id).unwrap();
            self.game_player_ready_state(&reference, id, false).await;
            return;
        }

        let mut pig_locations = Vec::<Piece>::new();
        let mut provided_config = HashMap::new();
        let length = packet.read_u32().unwrap_or(0);

        for _ in 0..length {
            let pig = Pig::from(packet.read_u32().unwrap_or(0));
            let location = packet.read_u32().unwrap_or(0);
            if !board::in_starting_bounds(location.try_into().unwrap_or(0)) {
                return;
            }
            if pig_locations.iter().any(|x| x.location == location as u8) {
                return;
            }

            // Safe to cast using as, since above checks ensures location is between 1 and 40
            pig_locations.push(Piece::new(pig, location as u8));
            let value = *provided_config.get(&pig).unwrap_or(&0);
            provided_config.insert(pig, value + 1);
        }

        // Fill in the rest with blanks
        for i in 0..13 {
            let pig = Pig::from(i);
            if !provided_config.contains_key(&pig) {
                provided_config.insert(pig, 0);
            }
        }

        // Ensure provided board agrees with config
        let reference = self.get_room(room_id).unwrap();
        let config = reference.inner().settings.pig_config.clone();
        for (pig, amount) in config {
            if provided_config.get(&pig).unwrap_or(&0) != &amount {
                return;
            }
        }
        drop(reference);

        let player = self
            .all_clients
            .get_mut(id)
            .unwrap()
            .player
            .as_mut()
            .unwrap();
        player.is_ready = true;
        player.initialize_setup(pig_locations);
        let reference = self.get_room(room_id).unwrap();
        self.game_player_ready_state(&reference, id, true).await;

        if let Some(res) = self.get_other_player(&reference, id) {
            if let Some(player) = &res.player {
                if player.is_ready {
                    self.register_board_data(&reference).await;
                    drop(reference);
                }
            }
        }
    }

    async fn register_board_data(&self, room: &GameRoom) {
        for id in room.inner().client_ids.iter() {
            // let player = self.all_clients.get(*id).unwrap().player.as_ref().unwrap();
            let mut locations = Vec::new();

            if self.config.one_player {
                // TODO: Allow for one player games
            } else {
                let opp_board = &self
                    .get_other_player(&room, *id)
                    .unwrap()
                    .player
                    .as_ref()
                    .unwrap()
                    .board;
                locations = opp_board.iter().map(|x| x.location).collect();
            }

            self.opponent_pig_placement(*id, locations).await;
        }

        room.get().write().unwrap().current_phase = 2;
    }
}
