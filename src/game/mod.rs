use crate::GameServer;
use crate::Packet;
mod send;

impl GameServer {
    pub async fn handle_client_finish_scene_load(&mut self, id: usize, mut packet: Packet) {
        let id_check = packet.read_string().unwrap_or(String::new());
        let scene_index = packet.read_u32().unwrap_or(2);
        if id.to_string() != id_check {
            return;
        }
        let ctx = self.get_context(id);
        if let None = ctx {
            return;
        }
        let (client, room) = ctx.unwrap();
        let room_id = room.id();
        drop(room);

        if scene_index <= 2 {
            client.player.as_mut().unwrap().scene_index = 2;
        }

        let reference = self.get_room(room_id).unwrap();

        if scene_index == 2 {
            // Game
            if let Some(opp) = self.get_other_player(&reference, id) {
                if opp.player.as_ref().unwrap().scene_index == 2 {
                    self.both_clients_loaded_game(&reference).await;
                }
            } else if self.config.one_player {
                self.both_clients_loaded_game(&reference).await;
            }
        }
    }
}