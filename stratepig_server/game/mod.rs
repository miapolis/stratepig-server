use std::convert::TryInto;
use stratepig_core::{Packet, PacketBody};

use crate::packet::FinishedSceneLoadPacket;
use crate::GameServer;
use crate::StratepigError;

mod game;
mod operations;
mod send;
mod start;
mod win;

impl GameServer {
    pub async fn handle_client_finish_scene_load(
        &mut self,
        id: usize,
        packet: Packet,
    ) -> Result<(), StratepigError> {
        let data = FinishedSceneLoadPacket::deserialize(&packet.body)?;
        let (_client, room) = self.get_context(id).unwrap();
        let room_id = room.id();

        drop(room);

        if data.scene_index <= 2 {
            self.all_clients
                .get_mut(&id)
                .unwrap()
                .player
                .as_mut()
                .unwrap()
                .scene_index = data.scene_index.try_into().unwrap_or(2);
        }

        let reference = self.get_room(room_id).unwrap();
        reference.store_seen();

        if data.scene_index == 2 {
            // Game
            if let Some(opp) = self.get_other_player(&reference, id) {
                if opp.player.as_ref().unwrap().scene_index == 2 {
                    self.both_clients_loaded_game(&reference).await;
                }
            } else if self.config.one_player {
                self.both_clients_loaded_game(&reference).await;
            }
        } else if data.scene_index == 1 {
            if let Some(opp) = self.get_other_player(&reference, id) {
                if opp.player.as_ref().unwrap().scene_index == 1 {
                    self.room_player_add(&reference).await;
                    self.send_game_info(&reference, None).await;
                }
            } else if self.config.one_player {
                self.room_player_add(&reference).await;
                self.send_game_info(&reference, None).await;
            }
        }

        Ok(())
    }
}
