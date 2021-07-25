use crate::*;
use stratepig_game::*;

impl GameServer {
    pub async fn both_clients_loaded_game(&self, room: &GameRoom) {
        let packet = BothClientsLoadedGamePacket;
        self.message_room(room, packet).await;
    }

    pub async fn game_player_ready_state(&self, room: &GameRoom, id: usize, ready: bool) {
        let packet = GamePlayerUpdatedReadyStatePacket {
            id: id.to_string(),
            ready,
        };
        self.message_room(room, packet).await;
    }

    pub async fn opponent_pig_placement(&self, id: usize, locations: Vec<u8>) {
        let packet = OpponentPigPlacementPacket { locations };
        self.message_one(id, packet).await;
    }

    pub async fn send_move_data(
        &self,
        room: &GameRoom,
        initiator_role: PlayerRole,
        from: u8,
        to: u8,
    ) {
        let packet = MoveDataPacket {
            role: initiator_role as u32,
            from,
            to,
            bundle_null: true,
        };
        self.message_room(room, packet).await;
    }

    pub async fn send_move_data_attack(
        &self,
        room: &GameRoom,
        initiator_role: PlayerRole,
        from: u8,
        to: u8,
        result: InteractionResult,
        init_type: Pig,
        target_type: Pig,
    ) {
        let packet = MoveDataAttackPacket {
            role: initiator_role as u32,
            from,
            to,
            bundle_null: false,
            result: result as i32,
            init_type: init_type as u32,
            target_type: target_type as u32,
        };
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
        let packet = WinPacket {
            role: role as u32,
            win_type: win_type as u32,
            elapsed,
            immediate,
        };
        self.message_room(room, packet).await;
    }

    pub async fn client_play_again(&self, room: &GameRoom, id: usize) {
        let packet = ClientPlayAgainPacket { id: id.to_string() };
        self.message_room(room, packet).await;
    }

    pub async fn send_enemy_piece_data(&self, id: usize, data: Vec<(u8, u8)>) {
        let packet = EnemyPieceDataPacket { data };
        self.message_one(id, packet).await;
    }
}
