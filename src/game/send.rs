use crate::*;

impl GameServer {
    pub async fn both_clients_loaded_game(&self, room: &GameRoom) {
        let packet = Packet::new_id(ServerMessage::BothClientsLoadedGame as i32);
        self.message_room(room, packet).await;
    }
}
