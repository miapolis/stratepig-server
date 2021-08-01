use crate::player::PlayerRole;
use crate::util::unix_now;
use crate::win::WinType;
use crate::{GameRoom, GameServer};

impl GameServer {
    pub async fn broadcast_win(&self, room: &GameRoom, role: PlayerRole, win_type: WinType) {
        self.broadcast_win_i(room, role, win_type, win_type.immediate())
            .await;
    }

    pub async fn broadcast_win_i(
        &self,
        room: &GameRoom,
        role: PlayerRole,
        win_type: WinType,
        immediate: bool,
    ) {
        // Win terminates all tickers
        room.get().write().unwrap().abort_all_tickers();
        room.store_seen();

        let start = room.inner().game_start_timestamp.unwrap_or(unix_now());
        let elapsed = unix_now() - start;

        self.send_win(room, role, win_type, elapsed, immediate)
            .await;

        let client_ids = room.inner().client_ids.clone();
        for id in client_ids.iter() {
            let read = room.inner();

            let opp_player;
            if client_ids.len() == 1 && self.config.one_player {
                opp_player = read.fake_enemy.as_ref().unwrap();
            } else if client_ids.len() == 2 {
                opp_player = self
                    .get_client(room.other_id(*id))
                    .unwrap()
                    .player
                    .as_ref()
                    .unwrap();
            } else {
                return;
            }

            let mut setup = Vec::new();
            if opp_player.init_board.len() > 0 {
                setup = opp_player
                    .init_board
                    .iter()
                    .map(|p| (p.id, p.pig as u8))
                    .collect();
            }
            drop(read);

            self.send_enemy_piece_data(*id, setup).await;
        }
    }
}
