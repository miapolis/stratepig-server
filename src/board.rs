pub type Board = Vec<Piece>;

pub struct Piece {
    pub pig: Pig,
    pub location: u8, // At most 100 so this is fine
}

#[allow(dead_code)]
impl Piece {
    pub fn new(pig: Pig, location: u8) -> Self {
        Self { pig, location }
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
}
