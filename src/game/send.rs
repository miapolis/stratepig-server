use crate::*;

impl GameServer {
    pub async fn both_clients_loaded_game(&self, room: &GameRoom) {
        let packet = Packet::new_id(ServerMessage::BothClientsLoadedGame as i32);
        self.message_room(room, packet).await;
    }

    pub async fn game_player_ready_state(&self, room: &GameRoom, id: usize, ready: bool) {
        let mut packet = Packet::new_id(ServerMessage::GamePlayerUpdatedReadyState as i32);
        packet.write_str(&id.to_string());
        packet.write_bool(ready);
        self.message_room(room, packet).await;
    }

    pub async fn opponent_pig_placement(&self, id: usize, locations: Vec<u8>) {
        let mut packet = Packet::new_id(ServerMessage::OpponentPigPlacement as i32);
        packet.write_u32(locations.len() as u32);
        for location in locations {
            packet.write_u32(location as u32);
        }
        self.message_one(id, packet).await;
    }

    pub async fn send_move_data(
        &self,
        room: &GameRoom,
        initiator_role: PlayerRole,
        from: u8,
        to: u8,
    ) {
        let mut packet = Packet::new_id(ServerMessage::MoveData as i32);

        packet.write_u32(initiator_role as u32);
        packet.write_u32(from as u32);
        packet.write_u32(to as u32);
        packet.write_bool(true); // Bundle null

        self.message_room(room, packet).await;
    }

    pub async fn send_move_data_attack(
        &self,
        room: &GameRoom,
        initiator_role: PlayerRole,
        from: u8,
        to: u8,
        result: board::InteractionResult,
        init_type: board::Pig,
        target_type: board::Pig,
    ) {
        let mut packet = Packet::new_id(ServerMessage::MoveData as i32);

        packet.write_u32(initiator_role as u32);
        packet.write_u32(from as u32);
        packet.write_u32(to as u32);
        packet.write_bool(false); // Bundle null
        packet.write_i32(result as i32);
        packet.write_u32(init_type as u32);
        packet.write_u32(target_type as u32);
        packet.write_u32(13);

        self.message_room(room, packet).await;
    }

    pub async fn send_win(
        &self,
        room: &GameRoom,
        role: PlayerRole,
        win_type: win::WinType,
        elapsed: u64,
        immediate: bool,
    ) {
        let mut packet = Packet::new_id(ServerMessage::Win as i32);

        packet.write_u32(role as u32);
        packet.write_u32(win_type as u32);
        packet.write_u64(elapsed);
        packet.write_bool(immediate);

        self.message_room(room, packet).await;
    }

    pub async fn client_play_again(&self, room: &GameRoom, id: usize) {
        let mut packet = Packet::new_id(ServerMessage::ClientPlayAgain as i32);
        packet.write_str(&id.to_string());
        self.message_room(room, packet).await;
    }
}
