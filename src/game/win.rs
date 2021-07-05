use crate::player::PlayerRole;
use crate::util::unix_now;
use crate::win::WinType;
use crate::{GameRoom, GameServer};

impl GameServer {
    pub async fn broadcast_win(&self, room: &GameRoom, role: PlayerRole, win_type: WinType) {
        // Win terminates all tickers
        room.get().write().unwrap().abort_all_tickers();
        room.store_seen();

        let start = room.inner().game_start_timestamp.unwrap_or(unix_now());
        let elapsed = unix_now() - start;

        self.send_win(room, role, win_type, elapsed, win_type.immediate())
            .await;
    }
}
