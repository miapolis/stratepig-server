use crate::board::{self, Pig};
use crate::test_util;
use crate::unwrap_ret;
use crate::{GameServer};
use crate::Packet;
use std::convert::TryInto;

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
        if room.inner().current_phase != 2 {
            return;
        }

        let from_location: u8 = packet.read_u32().unwrap_or(0).try_into().unwrap_or(0);
        println!("FROM {}", from_location);
        let to_location: u8 = packet.read_u32().unwrap_or(0).try_into().unwrap_or(0);
        println!("TO {}", to_location);

        if from_location == to_location
            || !board::in_bounds(from_location as i16)
            || !board::in_bounds(to_location as i16)
        {
            return;
        }

        let _guess = Pig::from(packet.read_u32().unwrap_or(0));

        let player = client.player.as_ref().unwrap();
        // unsafe
        let opponent = self
            .get_other_player(&room, id)
            .unwrap()
            .player
            .as_ref()
            .unwrap();

        let mut local_board = player.board.clone();
        let opponent_board = board::flip_board(&opponent.board);
        let total_board = board::sum_boards(&local_board, &opponent_board);

        let initiator = unwrap_ret!(local_board.iter().find(|x| x.location == from_location));
        let target_opt = local_board.iter().find(|x| x.location == to_location);
        let target_opp_opt = opponent_board.iter().find(|x| x.location == to_location);

        test_util::print_board(&total_board);

        if target_opt.is_some() {
            return;
        }

        // Ensure that this is a valid move (ignores rest of pigs on the board)
        if !initiator
            .pig
            .get_behavior()
            .allow_move(from_location, to_location)
        {
            return;
        }
        // Ensure that there are no pigs (friend or enemy) in between the from and to locations
        // No actions can be done THROUGH other pigs
        if total_board
            .iter()
            .any(|x| x.location > from_location && x.location < to_location)
        {
            return;
        }

        // Move, not an attack of any sort
        if target_opp_opt.is_none() {
            // Checks are already in place to ensure that move is valid for the specific
            // pig, and that there are not pigs in between the initiator and the target
            let index = local_board.iter().position(|x| x.location == from_location).unwrap();
            local_board[index].move_to(to_location);
            self.send_move_data(&room, player.role, from_location, to_location).await;
            drop(room);
            self.get_player_mut(id).unwrap().board = local_board;
        }
    }
}
