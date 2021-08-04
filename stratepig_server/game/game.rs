use stratepig_core::{Packet, PacketBody};
use stratepig_game::{InteractionResult, Pig};

use crate::packet::{MovePacket, PlayAgainPacket, SurrenderPacket};
use crate::unwrap_ret;
use crate::win::WinType;
use crate::GameServer;
use crate::StratepigError;

impl GameServer {
    pub async fn move_received(&mut self, id: usize, packet: Packet) -> Result<(), StratepigError> {
        let data = MovePacket::deserialize(&packet.body)?;

        if id.to_string() != data.my_id {
            return Err(StratepigError::AssumeWrongId);
        }

        let ctx = self.get_context(id);
        if let None = ctx {
            return Err(StratepigError::MissingContext);
        }
        let (client, room) = ctx.unwrap();
        let room_id = room.id();

        if client.player.as_ref().is_none() {
            return Err(StratepigError::with("missing player object on client"));
        }
        if room.inner().game_phase != 2 || room.inner().game_ended {
            return Err(StratepigError::with(
                "game not in correct state to allow move",
            ));
        }
        let current_turn = room.inner().current_turn;
        if !self.config.ignore_turns && client.player.as_ref().unwrap().role != current_turn {
            return Err(StratepigError::with("not at correct turn to allow move"));
        }
        if data.from_location == data.to_location
            || !stratepig_game::in_bounds(data.from_location as i16)
            || !stratepig_game::in_bounds(data.to_location as i16)
        {
            return Err(StratepigError::with("move data not in bounds"));
        }

        // let _guess = Pig::from(packet.read_u32().unwrap_or(0));

        let player = client.player.as_ref().unwrap();

        let mut local_board = player.board.clone();

        let mut opp_id = 0;
        let mut opponent_board;
        if !self.config.one_player {
            let opp_client = self.get_other_player(&room, id).unwrap();
            opp_id = opp_client.id;
            let opponent = opp_client.player.as_ref().unwrap();
            opponent_board = stratepig_game::flip_board(&opponent.board);
        } else {
            opponent_board =
                stratepig_game::flip_board(&room.inner().fake_enemy.as_ref().unwrap().board);
        }

        let total_board = stratepig_game::sum_boards(&local_board, &opponent_board);

        let initiator = unwrap_ret!(local_board
            .iter()
            .find(|x| x.location == data.from_location));
        let target_opt = local_board.iter().find(|x| x.location == data.to_location);
        let target_opp_opt = opponent_board
            .iter()
            .find(|x| x.location == data.to_location);
        let attack = target_opp_opt.is_some();

        // test_util::print_board(&total_board);

        if target_opt.is_some() {
            return Err(StratepigError::with("attempting attack on friend piece"));
        }

        // Ensure that this is a valid move (ignores rest of pigs on the board)
        // Prevents jumping over water tiles
        if !initiator
            .pig
            .get_behavior()
            .allow_move(data.from_location, data.to_location)
        {
            return Err(StratepigError::with(
                "pig prevents moving in the desired way",
            ));
        }
        // Ensure that there are no pigs (friend or enemy) in between the from and to locations
        // No actions can be done THROUGH other pigs
        if stratepig_game::pig_in_path(&total_board, data.from_location, data.to_location) {
            return Err(StratepigError::with(
                "pig found in between to and from locations",
            ));
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
            let i = index!(data.from_location, local_board);
            local_board[i].move_to(data.to_location);
            self.send_move_data(&room, player.role, data.from_location, data.to_location)
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
                room.get().write().unwrap().game_ended = true;
                self.broadcast_win(&room, player.role, WinType::FlagCapture)
                    .await;
            }

            if let InteractionResult::Tie = interaction {
                local_board.remove(index!(data.from_location, local_board));
                opponent_board.remove(index!(data.to_location, opponent_board));
            } else if let InteractionResult::Win = interaction {
                opponent_board.remove(index!(data.to_location, opponent_board));
                let i = index!(data.from_location, local_board);
                local_board[i].move_to(data.to_location);
            } else {
                local_board.remove(index!(data.from_location, local_board));
            }

            self.send_move_data_attack(
                &room,
                player.role,
                data.from_location,
                data.to_location,
                interaction,
                init_type,
                target_type,
            )
            .await;

            drop(room);

            self.get_player_mut(id).unwrap().board = local_board;

            if !self.config.one_player {
                self.get_player_mut(opp_id).unwrap().board =
                    stratepig_game::flip_board(&opponent_board);
            } else {
                let reference = self.get_room(room_id).unwrap();
                reference
                    .get()
                    .write()
                    .unwrap()
                    .fake_enemy
                    .as_mut()
                    .unwrap()
                    .board = stratepig_game::flip_board(&opponent_board);
            }
        }

        let room = self.get_room(room_id).unwrap();

        if room.inner().game_ended {
            return Ok(());
        }

        room.get().write().unwrap().current_turn = current_turn.opp();
        self.run_operations(&room, false).await;

        drop(room);

        if !(self.config.one_player || self.config.ignore_turns) {
            self.turn_start(room_id, attack).await;
        }

        Ok(())
    }

    pub async fn handle_client_play_again(
        &mut self,
        id: usize,
        packet: Packet,
    ) -> Result<(), StratepigError> {
        let data = PlayAgainPacket::deserialize(&packet.body)?;

        if id.to_string() != data.my_id {
            return Err(StratepigError::AssumeWrongId);
        }

        let ctx = self.get_context(id);
        if let None = ctx {
            return Err(StratepigError::MissingContext);
        }
        let (client, room) = ctx.unwrap();
        let room_id = room.id();

        if client.player.as_ref().is_none() {
            return Err(StratepigError::with("missing player object on client"));
        }
        if !room.inner().game_ended {
            return Err(StratepigError::with(
                "game not in correct state to allow play again",
            ));
        }
        if client.player.as_ref().unwrap().play_again {
            return Err(StratepigError::with("client already set to play again"));
        }

        self.client_play_again(&room, id).await;
        drop(room);
        self.get_player_mut(id).unwrap().play_again = true;
        let room = self.get_room(room_id).unwrap();

        if self
            .get_other_player(&room, id)
            .unwrap()
            .player
            .as_ref()
            .unwrap()
            .play_again
        {
            room.reset();
            room.store_seen();
            let clients = room.clients();
            drop(room);

            for client_id in clients {
                self.get_client_mut(client_id.0).unwrap().reset();
            }
        }

        Ok(())
    }

    pub async fn handle_surrender(
        &mut self,
        id: usize,
        packet: Packet,
    ) -> Result<(), StratepigError> {
        let data = SurrenderPacket::deserialize(&packet.body)?;

        if id.to_string() != data.my_id {
            return Err(StratepigError::AssumeWrongId);
        }

        let ctx = self.get_context(id);
        if let None = ctx {
            return Err(StratepigError::MissingContext);
        }
        let (client, room) = ctx.unwrap();

        if client.player.as_ref().is_none() {
            return Err(StratepigError::with("missing player object on client"));
        }
        if !room.inner().in_game || room.inner().game_ended {
            return Err(StratepigError::with(
                "game not in correct state to allow surrender",
            ));
        }

        let winning_role = client.player.as_ref().unwrap().role.opp();
        room.get().write().unwrap().game_ended = true;
        self.broadcast_win(&room, winning_role, WinType::Surrender)
            .await;

        Ok(())
    }
}
