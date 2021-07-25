use crate::board::{get_adjacent, get_scout};
use crate::interaction::InteractionResult;

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
        rank_eval!(me, target)
    }
    fn defense_override(&self, _attacker: Pig) -> Option<InteractionResult> {
        None
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
    fn defense_override(&self, attacker: Pig) -> Option<InteractionResult> {
        if attacker == Pig::Miner {
            return Some(InteractionResult::Lose);
        }
        Some(InteractionResult::Win)
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
    fn defense_override(&self, _attacker: Pig) -> Option<InteractionResult> {
        Some(InteractionResult::Lose)
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

    pub fn immovable(&self) -> bool {
        match self {
            Pig::Bomb | Pig::Flag => true,
            _ => false
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
