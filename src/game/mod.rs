use std::convert::TryInto;

use crate::GameServer;
use crate::Packet;
use crate::StratepigError;
mod game;
mod send;
mod start;
mod win;

impl GameServer {
    pub async fn handle_client_finish_scene_load(
        &mut self,
        id: usize,
        mut packet: Packet,
    ) -> Result<(), StratepigError> {
        // let id_check = packet.read_string().unwrap_or(String::new());
        // let scene_index = packet.read_u32().unwrap_or(2);
        // if id.to_string() != id_check {
        //     return;
        // }
        // let ctx = self.get_context(id);
        // if let None = ctx {
        //     return;
        // }
        // let (_client, room) = ctx.unwrap();
        // if room.inner().game_phase == 2 {
        //     return;
        // }
        // let room_id = room.id();
        // drop(room);

        // if scene_index <= 2 {
        //     self.all_clients
        //         .get_mut(id)
        //         .unwrap()
        //         .player
        //         .as_mut()
        //         .unwrap()
        //         .scene_index = scene_index.try_into().unwrap_or(2);
        // }

        // let reference = self.get_room(room_id).unwrap();
        // reference.store_seen();

        // if scene_index == 2 {
        //     // Game
        //     if let Some(opp) = self.get_other_player(&reference, id) {
        //         if opp.player.as_ref().unwrap().scene_index == 2 {
        //             self.both_clients_loaded_game(&reference).await;
        //         }
        //     } else if self.config.one_player {
        //         self.both_clients_loaded_game(&reference).await;
        //     }
        // } else if scene_index == 1 {
        //     if let Some(opp) = self.get_other_player(&reference, id) {
        //         if opp.player.as_ref().unwrap().scene_index == 1 {
        //             self.room_player_add(&reference).await;
        //             self.send_game_info(&reference, None).await;
        //         }
        //     } else if self.config.one_player {
        //         self.room_player_add(&reference).await;
        //         self.send_game_info(&reference, None).await;
        //     }
        // }
        Ok(())
    }
}
