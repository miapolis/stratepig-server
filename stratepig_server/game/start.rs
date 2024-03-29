use std::collections::HashMap;
use std::convert::TryInto;

use crate::packet::{GamePlayerReadyDataDefaultPacket, GamePlayerReadyDataFullPacket};
use crate::player::{Player, PlayerRole};
use crate::util;
use crate::GameServer;
use crate::StratepigError;
use stratepig_core::{Packet, PacketBody};
use stratepig_game::{Piece, Pig};

impl GameServer {
    pub async fn handle_game_player_ready(
        &mut self,
        id: usize,
        packet: Packet,
    ) -> Result<(), StratepigError> {
        let data = GamePlayerReadyDataDefaultPacket::deserialize(&packet.body)?;
        let (_client, room) = self.get_context(id).unwrap();
        let room_id = room.id();
        drop(room);

        if !data.ready {
            let player = self
                .all_clients
                .get_mut(&id)
                .unwrap()
                .player
                .as_mut()
                .unwrap();
            player.is_ready = false;
            let reference = self.get_room(room_id).unwrap();
            self.game_player_ready_state(&reference, id, false).await;

            return Ok(());
        }

        let data = GamePlayerReadyDataFullPacket::deserialize(&packet.body)?;

        let mut pig_locations = Vec::<Piece>::new();
        let mut provided_config = HashMap::new();

        for (pig, location) in data.board.into_iter() {
            let pig = Pig::from(pig);
            if let Pig::Empty = pig {
                return Err(StratepigError::with("invalid pig"));
            }
            if !stratepig_game::in_starting_bounds(location.try_into().unwrap_or(0)) {
                return Err(StratepigError::with("location out of bounds"));
            }
            if pig_locations.iter().any(|x| x.location == location as u8) {
                return Err(StratepigError::with("duplicate location placement"));
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
                return Err(StratepigError::with(
                    "board config does not agree with settings",
                ));
            }
        }
        drop(reference);

        let player = self
            .all_clients
            .get_mut(&id)
            .unwrap()
            .player
            .as_mut()
            .unwrap();
        player.is_ready = true;
        player.initialize_setup(pig_locations.clone());
        let reference = self.get_room(room_id).unwrap();
        self.game_player_ready_state(&reference, id, true).await;

        if let Some(res) = self.get_other_player(&reference, id) {
            drop(reference);
            if let Some(player) = &res.player {
                if player.is_ready {
                    self.register_board_data(room_id).await;
                }
            }
        } else if self.config.one_player {
            let mut fake_enemy = Player::new(PlayerRole::Two);
            fake_enemy.is_ready = true;
            fake_enemy.initialize_setup(pig_locations);
            reference.get().write().unwrap().fake_enemy = Some(fake_enemy);

            drop(reference);
            self.register_board_data(room_id).await;
        }

        Ok(())
    }

    async fn register_board_data(&mut self, room_id: usize) {
        let room = self.get_room(room_id).unwrap();

        for id in room.inner().client_ids.iter() {
            let locations;
            if self.config.one_player {
                locations = room
                    .inner()
                    .fake_enemy
                    .as_ref()
                    .unwrap()
                    .board
                    .iter()
                    .map(|x| x.location)
                    .collect();
            } else {
                let opp_board = &self
                    .get_other_player(&room, id.0)
                    .unwrap()
                    .player
                    .as_ref()
                    .unwrap()
                    .board;
                locations = opp_board.iter().map(|x| x.location).collect();
            }

            self.opponent_pig_placement(id.0, locations).await;
        }

        self.run_operations(&room, true).await;

        room.start_phase_two().await;
        let clients = room.clients();
        let buffer = room.inner().settings.buffer_time;
        drop(room);

        for id in clients {
            let player = self.get_player_mut(id.0).unwrap();
            player.current_buffer = buffer as u128;
        }

        if !(self.config.one_player || self.config.ignore_turns) {
            self.turn_start(room_id, false).await;
        }
    }

    pub async fn turn_start(&mut self, room_id: usize, delay: bool) {
        let room = self.get_room(room_id).unwrap();

        let mut write = room.get().write().unwrap();
        if let Some(t) = &write.game_ticker {
            t.abort();
            write.game_ticker = None;
        }
        drop(write);

        room.start_player_turn(self, delay).await;

        // Set the remaining buffer time for the other player
        // (start of new turn marks end of previous turn)
        let timestamp = room.inner().last_buffer_timestamp;
        let other_id = room.other_id(room.get_active_id(self));

        if let Some(timestamp) = timestamp {
            room.get().write().unwrap().last_buffer_timestamp = None;
            drop(room);

            let diff = ((util::unix_now() - timestamp) as f32 / 1000.0).ceil() as u128;
            let player = self.get_player_mut(other_id).unwrap();
            player.current_buffer -= diff;
        }
    }
}
