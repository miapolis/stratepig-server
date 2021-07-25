mod board;
mod interaction;
mod pig;
mod test_util;

pub use board::*;
pub use interaction::InteractionResult;
pub use pig::{Pig, PigBehavior};

#[derive(Debug, Clone)]
pub struct Piece {
    pub pig: Pig,
    pub location: u8,
    pub id: u8,
}

impl Piece {
    pub fn new(pig: Pig, location: u8) -> Self {
        Self {
            pig,
            location,
            id: location,
        }
    }

    pub fn move_to(&mut self, location: u8) {
        self.location = location;
    }
}
