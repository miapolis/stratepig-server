pub type Board = Vec<Piece>;

const BOTTOM_LEFT_TILE: u8 = 1;
const TOP_RIGHT_TILE: u8 = 100;
const STARTING_TERRITORY_VALUE: u8 = 40;
pub const WATER_TILES: [u8; 8] = [43, 44, 47, 48, 53, 54, 57, 58];

pub enum InteractionResult {
    Win,
    Lose,
    Tie,
}

macro_rules! rank_eval {
    ($me:expr, $target:expr) => {{
        let self_rank = $me.rank();
        let target_rank = $target.rank();
        if self_rank > target_rank {
            return InteractionResult::Win;
        } else if self_rank < target_rank {
            return InteractionResult::Lose;
        } else {
            return InteractionResult::Tie;
        }
    }};
}

pub trait PigBehavior {
    fn allow_move(&self, from: u8, to: u8) -> bool {
        get_adjacent(from).contains(&to)
    }
    fn attack(&self, me: Pig, target: Pig) -> InteractionResult {
        // TODO: make sure this is included in main code!
        // let target_behavior = target.get_behavior();
        // if let Some(result) = target_behavior.defense_override(me) {
        //     return !result // Target winning = current losing... inverse required
        // }

        rank_eval!(me, target)
    }
    fn defense_override(&self, _attacker: Pig) -> Option<bool> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub pig: Pig,
    pub location: u8,
}

impl Piece {
    pub fn new(pig: Pig, location: u8) -> Self {
        Self { pig, location }
    }

    pub fn move_to(&mut self, location: u8) {
        self.location = location;
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Pig {
    Bomb = 0,
    Spy = 1,
    Infiltrator = 2,
    Flag = 3,
    Scout = 4,
    Miner = 5,
    Sergeant = 6,
    Lieutenant = 7,
    Chemist = 8,
    Major = 9,
    Colonel = 10,
    General = 11,
    Kingo = 12,
    Empty = -1,
}

// Special pigs
struct Bomb;
impl PigBehavior for Bomb {
    fn allow_move(&self, _from: u8, _to: u8) -> bool {
        false
    }
    fn defense_override(&self, attacker: Pig) -> Option<bool> {
        if attacker == Pig::Miner {
            return Some(false);
        }
        Some(true)
    }
}
struct Spy;
impl PigBehavior for Spy {
    fn attack(&self, me: Pig, target: Pig) -> InteractionResult {
        if target == Pig::Kingo {
            return InteractionResult::Win;
        }
        rank_eval!(me, target)
    }
}
struct Flag;
impl PigBehavior for Flag {
    fn allow_move(&self, _from: u8, _to: u8) -> bool {
        false
    }
    fn defense_override(&self, _attacker: Pig) -> Option<bool> {
        Some(false)
    }
}
struct Scout;
impl PigBehavior for Scout {
    fn allow_move(&self, from: u8, to: u8) -> bool {
        get_scout(from).contains(&to)
    }
}

// Normies
struct Miner;
impl PigBehavior for Miner {}
struct Sergeant;
impl PigBehavior for Sergeant {}
struct Lieutenant;
impl PigBehavior for Lieutenant {}
struct Chemist;
impl PigBehavior for Chemist {}
struct Major;
impl PigBehavior for Major {}
struct Colonel;
impl PigBehavior for Colonel {}
struct General;
impl PigBehavior for General {}
struct Kingo;
impl PigBehavior for Kingo {}

#[allow(dead_code)]
impl Pig {
    pub fn from(val: u32) -> Pig {
        match val {
            0 => Pig::Bomb,
            1 => Pig::Spy,
            2 => Pig::Infiltrator,
            3 => Pig::Flag,
            4 => Pig::Scout,
            5 => Pig::Miner,
            6 => Pig::Sergeant,
            7 => Pig::Lieutenant,
            8 => Pig::Chemist,
            9 => Pig::Major,
            10 => Pig::Colonel,
            11 => Pig::General,
            12 => Pig::Kingo,
            _ => Pig::Empty,
        }
    }

    pub fn rank(&self) -> u8 {
        match self {
            Pig::Bomb => 0,
            Pig::Spy => 1,
            Pig::Infiltrator => 1,
            Pig::Flag => 0,
            Pig::Scout => 2,
            Pig::Miner => 3,
            Pig::Sergeant => 4,
            Pig::Lieutenant => 5,
            Pig::Chemist => 6,
            Pig::Major => 7,
            Pig::Colonel => 8,
            Pig::General => 9,
            Pig::Kingo => 10,
            _ => 0,
        }
    }

    pub fn get_behavior(&self) -> Box<dyn PigBehavior> {
        match self {
            Pig::Bomb => Box::new(Bomb),
            Pig::Spy => Box::new(Spy),
            Pig::Infiltrator => Box::new(Spy),
            Pig::Flag => Box::new(Flag),
            Pig::Scout => Box::new(Scout),
            Pig::Miner => Box::new(Miner),
            Pig::Sergeant => Box::new(Sergeant),
            Pig::Lieutenant => Box::new(Lieutenant),
            Pig::Chemist => Box::new(Chemist),
            Pig::Major => Box::new(Major),
            Pig::Colonel => Box::new(Colonel),
            Pig::General => Box::new(General),
            Pig::Kingo => Box::new(Kingo),
            _ => Box::new(Sergeant),
        }
    }

    pub fn print(&self) -> String {
        match self {
            Pig::Bomb => String::from("BB"),
            Pig::Spy => String::from("SS"),
            Pig::Infiltrator => String::from("II"),
            Pig::Flag => String::from("FF"),
            Pig::Scout => String::from("22"),
            Pig::Miner => String::from("33"),
            Pig::Sergeant => String::from("44"),
            Pig::Lieutenant => String::from("55"),
            Pig::Chemist => String::from("66"),
            Pig::Major => String::from("77"),
            Pig::Colonel => String::from("88"),
            Pig::General => String::from("99"),
            Pig::Kingo => String::from("KK"),
            _ => String::from("00"),
        }
    }
}

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
