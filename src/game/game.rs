use std::convert::TryInto;

use crate::board::{self, Pig};
// use crate::test_util;
use crate::board::InteractionResult;
use crate::unwrap_ret;
use crate::GameServer;
use crate::Packet;
use crate::win::WinType;

impl GameServer {
    pub async fn move_received(&mut self, id: usize, mut packet: Packet) {
        let id_check = packet.read_string().unwrap_or(String::new());
        if id.to_string() != id_check {
            return;
        }
        let ctx = self.get_context(id);
        if let None = ctx {
            return;
        }
        let (client, room) = ctx.unwrap();

        if client.player.as_ref().is_none() {
            return;
        }
        if room.inner().game_phase != 2 || room.inner().game_ended {
            return;
        }

        let current_turn = room.inner().current_turn;
        if client.player.as_ref().unwrap().role != current_turn {
            return;
        }

        let from_location: u8 = packet.read_u32().unwrap_or(0).try_into().unwrap_or(0);
        let to_location: u8 = packet.read_u32().unwrap_or(0).try_into().unwrap_or(0);

        if from_location == to_location
            || !board::in_bounds(from_location as i16)
            || !board::in_bounds(to_location as i16)
        {
            return;
        }

        let _guess = Pig::from(packet.read_u32().unwrap_or(0));

        let player = client.player.as_ref().unwrap();

        // unsafe
        let opp_client = self.get_other_player(&room, id).unwrap();
        let opp_id = opp_client.id;
        let opponent = opp_client.player.as_ref().unwrap();

        let mut local_board = player.board.clone();
        let mut opponent_board = board::flip_board(&opponent.board);
        let total_board = board::sum_boards(&local_board, &opponent_board);

        let initiator = unwrap_ret!(local_board.iter().find(|x| x.location == from_location));
        let target_opt = local_board.iter().find(|x| x.location == to_location);
        let target_opp_opt = opponent_board.iter().find(|x| x.location == to_location);
        let attack = target_opp_opt.is_some();

        // test_util::print_board(&total_board);

        if target_opt.is_some() {
            return;
        }

        // Ensure that this is a valid move (ignores rest of pigs on the board)
        // Prevents jumping over water tiles
        if !initiator
            .pig
            .get_behavior()
            .allow_move(from_location, to_location)
        {
            return;
        }
        // Ensure that there are no pigs (friend or enemy) in between the from and to locations
        // No actions can be done THROUGH other pigs
        if board::pig_in_path(&total_board, from_location, to_location) {
            return;
        }

        macro_rules! index {
            ($loc:expr, $board:expr) => {
                $board.iter().position(|x| x.location == $loc).unwrap();
            };
        }

        // Move, not an attack of any sort
        if target_opp_opt.is_none() {
            // Checks are already in place to ensure that move is valid for the specific
            // pig, and that there are not pigs in between the initiator and the target
            let i = index!(from_location, local_board);
            local_board[i].move_to(to_location);
            self.send_move_data(&room, player.role, from_location, to_location)
                .await;

            drop(room);
            self.get_player_mut(id).unwrap().board = local_board;
        } else {
            // An attack
            let interaction: InteractionResult;

            let target = target_opp_opt.unwrap();
            let target_behavior = target.pig.get_behavior();
            if let Some(result) = target_behavior.defense_override(initiator.pig) {
                interaction = result.invert() // Target winning = current losing... inverse required
            } else {
                let initiator_behavior = initiator.pig.get_behavior();
                interaction = initiator_behavior.attack(initiator.pig, target.pig);
            }

            let init_type = initiator.pig;
            let target_type = target.pig;

            // TODO: Allow for infiltration and other conditions to occur
            if target_type == Pig::Flag {
                self.broadcast_win(&room, player.role, WinType::FlagCapture).await;
            }

            if let InteractionResult::Tie = interaction {
                local_board.remove(index!(from_location, local_board));
                opponent_board.remove(index!(to_location, opponent_board));
            } else if let InteractionResult::Win = interaction {
                opponent_board.remove(index!(to_location, opponent_board));
                let i = index!(from_location, local_board);
                local_board[i].move_to(to_location);
            } else {
                local_board.remove(index!(from_location, local_board));
            }

            self.send_move_data_attack(
                &room,
                player.role,
                from_location,
                to_location,
                interaction,
                init_type,
                target_type,
            )
            .await;

            drop(room);

            self.get_player_mut(id).unwrap().board = local_board;
            self.get_player_mut(opp_id).unwrap().board = board::flip_board(&opponent_board);
        }

        let room = self.get_room(1).unwrap();
        let room_id = room.id();
        room.get().write().unwrap().current_turn = current_turn.opp();
        drop(room);

        self.turn_start(room_id, attack).await;
    }
}
