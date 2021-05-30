use crate::board::{self, Board, Piece};

#[allow(dead_code)]
pub fn print_path(piece: u8, path: Vec<u8>) {
    for row in (0..10).rev() {
        let mut row_str = String::new();
        let in_row: Vec<&u8> = path.iter().filter(|x| (*x - 1) / 10 == row).collect();
        for col in 1..11 {
            let values: Vec<&&u8> = in_row
                .iter()
                .filter(|x| board::get_column(***x) == col)
                .collect();

            let tile = 10 * row + col;
            if values.len() > 0 {
                row_str.push_str(&format!("\x1b[32m{} \x1b[0m", to_double_digit(tile)));
            } else if tile == piece {
                row_str.push_str("\x1b[35mPP \x1b[0m");
            } else if board::WATER_TILES.contains(&tile) {
                row_str.push_str("\x1b[34mSS \x1b[0m");
            } else {
                row_str.push_str("00 ");
            }
        }

        println!("{}", row_str);
    }
}

#[allow(dead_code)]
pub fn print_board(board: &Board) {
    println!("---------------------");
    for row in 0..10 {
        let mut row_str = String::new();
        let in_row: Vec<&Piece> = board
            .iter()
            .filter(|x| (x.location - 1) / 10 == row)
            .collect();
        for col in 1..11 {
            let values: Vec<&&Piece> = in_row
                .iter()
                .filter(|x| board::get_column(x.location) == col)
                .collect();

            let tile = 10 * row + col;
            if board::WATER_TILES.contains(&tile) {
                row_str.push_str("\x1b[34mSS \x1b[0m");
            } else if let Some(piece) = values.iter().find(|x| x.location == tile) {
                row_str.push_str(&format!("\x1b[32m{} \x1b[0m", piece.pig.print()));
            } else {
                row_str.push_str("00 ");
            }
        }

        println!("{}", row_str);
    }
}

pub fn to_double_digit(num: u8) -> String {
    if num < 10 {
        return String::from(format!("0{}", num));
    } else if num >= 100 {
        return String::from("00");
    } else {
        return num.to_string();
    }
}
