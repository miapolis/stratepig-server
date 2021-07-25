pub enum InteractionResult {
    Win = 1,
    Lose = 0,
    Tie = -1,
}

impl InteractionResult {
    pub fn invert(&self) -> Self {
        match self {
            Self::Win => Self::Lose,
            Self::Lose => Self::Win,
            _ => Self::Tie,
        }
    }
}
