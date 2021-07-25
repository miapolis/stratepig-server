use crate::Piece;

pub type Board = Vec<Piece>;

pub const BOTTOM_LEFT_TILE: u8 = 1;
pub const TOP_RIGHT_TILE: u8 = 100;
pub const STARTING_TERRITORY_VALUE: u8 = 40;
pub const WATER_TILES: [u8; 8] = [43, 44, 47, 48, 53, 54, 57, 58];

pub fn in_bounds(pos: i16) -> bool {
    pos >= BOTTOM_LEFT_TILE as i16 && pos <= TOP_RIGHT_TILE as i16
}

pub fn in_starting_bounds(pos: i16) -> bool {
    pos >= BOTTOM_LEFT_TILE as i16 && pos <= STARTING_TERRITORY_VALUE as i16
}

#[allow(dead_code)]
pub fn get_adjacent(pos: u8) -> Vec<u8> {
    let pos = pos as i16;
    let initial = vec![pos + 1, pos - 1, pos + 10, pos - 10];
    let mut result: Vec<u8> = initial
        .into_iter()
        .filter(|x| in_bounds(*x) && !WATER_TILES.contains(&(*x as u8)))
        .map(|x| x as u8)
        .collect();

    if pos % 10 == 0 {
        result = result.into_iter().filter(|x| *x != pos as u8 + 1).collect();
    }
    if pos - 1 == 0 || (pos - 1) % 10 == 0 {
        result = result.into_iter().filter(|x| *x != pos as u8 - 1).collect();
    }

    result
}

pub fn get_column(pos: u8) -> u8 {
    let pos = pos % 10;
    if pos != 0 {
        return pos;
    } else {
        return 10;
    }
}

#[allow(dead_code)]
pub fn get_scout(pos: u8) -> Vec<u8> {
    let row = (pos - 1) / 10;
    let column = get_column(pos);
    let mut result = Vec::new();

    for x in column..11 {
        let val = row * 10 + x;
        if WATER_TILES.contains(&val) {
            break;
        }
        result.push(val);
    }
    for x in (1..column).rev() {
        let val = row * 10 + x;
        if WATER_TILES.contains(&val) {
            break;
        }
        result.push(val);
    }
    for y in row + 1..10 {
        let val = y * 10 + column;
        if WATER_TILES.contains(&val) {
            break;
        }
        result.push(val);
    }
    for y in (0..row).rev() {
        let val = y * 10 + column;
        if WATER_TILES.contains(&val) {
            break;
        }
        result.push(val);
    }

    result.into_iter().filter(|x| *x != pos).collect()
}

pub fn flip_tile(pos: u8) -> u8 {
    100 - pos + 1
}

pub fn flip_board(board: &Board) -> Board {
    let mut result = Board::new();
    for piece in board.iter() {
        result.push(Piece::new(piece.pig, flip_tile(piece.location)))
    }
    result
}

pub fn sum_boards(local: &Board, opp: &Board) -> Board {
    let mut board = Board::new();
    board.append(&mut local.clone());
    board.append(&mut opp.clone());
    board
}

pub fn pig_in_path(total_board: &Board, from: u8, to: u8) -> bool {
    let right_or_up = to > from;
    let row_from = (from - 1) / 10;
    let row_to = (to - 1) / 10;

    macro_rules! check {
        ($loc:expr) => {
            if total_board.iter().any(|x| x.location == $loc) {
                return true;
            }
        };
    }

    if right_or_up {
        if row_from == row_to {
            for i in from + 1..to {
                check!(i);
            }
        } else {
            for i in (from + 10..to).step_by(10) {
                check!(i);
            }
        }
    } else {
        if row_from == row_to {
            for i in to + 1..from {
                check!(i);
            }
        } else {
            for i in (to + 10..from).step_by(10) {
                check!(i);
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;

    #[test]
    #[rustfmt::skip]
    fn check_scout_path() {
        let tests: Vec<u8> = vec![18, 26, 40, 46, 63];
        let solutions: Vec<Vec<u8>> = vec![
            vec![8, 11, 12, 13, 14, 15, 16, 17, 19, 20, 28, 38],
            vec![6, 16, 21, 22, 23, 24, 25, 27, 28, 29, 30, 36, 46, 56, 66, 76, 86, 96],
            vec![10, 20, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 50, 60, 70, 80, 90, 100],
            vec![6, 16, 26, 36, 45, 56, 66, 76, 86, 96],
            vec![61, 62, 64, 65, 66, 67, 68, 69, 70, 73, 83, 93],
        ];

        for (index, test) in tests.into_iter().enumerate() {
            let result = get_scout(test);
            println!("---------------------------");
            println!("--- Testing Scout at {} ---", test);
            println!("---------------------------");
            test_util::print_path(test, result.clone());

            assert!(
                result.len() == solutions[index].len()
                    && result.iter().all(|x| solutions[index].contains(x))
            );
        }
    }

    #[test]
    fn check_regular_path() {
        let tests: Vec<u8> = vec![1, 8, 10, 18, 33, 56, 100];
        let solutions: Vec<Vec<u8>> = vec![
            vec![2, 11],
            vec![7, 9, 18],
            vec![9, 20],
            vec![8, 17, 19, 28],
            vec![23, 32, 34],
            vec![46, 55, 66],
            vec![90, 99],
        ];

        for (index, test) in tests.into_iter().enumerate() {
            let result = get_adjacent(test);
            println!("---------------------------");
            println!("--- Testing Regular at {} ---", test);
            println!("---------------------------");
            test_util::print_path(test, result.clone());

            assert!(
                result.len() == solutions[index].len()
                    && result.iter().all(|x| solutions[index].contains(x))
            );
        }
    }
}
