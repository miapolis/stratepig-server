use crate::win::WinType;
use crate::PlayerRole;
use crate::{GameRoom, GameServer};

impl GameServer {
    pub async fn run_operations(&self, room: &GameRoom, is_placement: bool) {
        if self.config.ignore_turns {
            return;
        }

        let client_ids = room.inner().client_ids.clone();
        let id = client_ids.get(0).unwrap();
        let player = self.get_client(*id).unwrap().player.as_ref().unwrap();
        let opp_player = self
            .get_client(room.other_id(*id))
            .unwrap()
            .player
            .as_ref()
            .unwrap();

        let local_board = player.board.clone();
        let enemy_board = stratepig_game::flip_board(&opp_player.board);
        let total_board = stratepig_game::sum_boards(&local_board, &enemy_board);

        let mut local_success = false;
        let mut enemy_success = false;

        'outer: for i in 1..3 {
            let board;
            if i == 1 {
                board = &local_board;
            } else {
                board = &enemy_board;
            }

            for piece in board.into_iter() {
                if piece.pig.immovable() {
                    continue;
                }
                let pos = piece.location as i16;

                let mut surrounding = vec![pos + 1, pos - 1, pos + 10, pos - 10];
                surrounding = surrounding
                    .into_iter()
                    .filter(|x| *x > 0 && *x < 100)
                    .collect();

                if pos % 10 == 0 {
                    surrounding = surrounding.into_iter().filter(|x| *x != pos + 1).collect();
                }
                if pos - 1 == 0 || (pos - 1) % 10 == 0 {
                    surrounding = surrounding.into_iter().filter(|x| *x != pos - 1).collect();
                }

                for tile in surrounding {
                    let tile = tile as u8;
                    if stratepig_game::WATER_TILES.contains(&tile) {
                        continue;
                    }
                    // Another pig is here
                    if total_board.iter().any(|x| x.location == tile) {
                        if i == 1 {
                            if enemy_board.iter().any(|x| x.location == tile)
                                && !local_board.iter().any(|x| x.location == tile)
                            {
                                local_success = true;
                                continue 'outer;
                            } else {
                                continue;
                            }
                        } else {
                            if local_board.iter().any(|x| x.location == tile)
                                && !enemy_board.iter().any(|x| x.location == tile)
                            {
                                enemy_success = true;
                                continue 'outer;
                            } else {
                                continue;
                            }
                        }
                    }

                    if i == 1 {
                        local_success = true;
                        continue 'outer;
                    } else {
                        enemy_success = true;
                        continue 'outer;
                    }
                }

                if i == 1 {
                    local_success = false;
                } else {
                    enemy_success = false;
                }
            }
        }

        if !(local_success && enemy_success) {
            room.get().write().unwrap().game_ended = true;
            if !local_success && enemy_success {
                self.broadcast_win_i(&room, PlayerRole::Two, WinType::OutOfMoves, is_placement)
                    .await;
            } else if !enemy_success && local_success {
                self.broadcast_win_i(&room, PlayerRole::One, WinType::OutOfMoves, is_placement)
                    .await;
            } else {
                self.broadcast_win_i(&room, PlayerRole::Tie, WinType::OutOfMoves, is_placement)
                    .await;
            }
        }
    }
}
