#[derive(Copy, Clone)]
pub enum WinType {
    FlagCapture = 1,
    Disconnect = 2,
    OutOfMoves = 3,
    OutOfTime = 4,
    Surrender = 5,
}

impl WinType {
    /// Whether or not win type should be displayed immediately
    /// or shortly cached on the client until animations stop
    pub fn immediate(&self) -> bool {
        match self {
            Self::FlagCapture => false,
            Self::Disconnect => true,
            Self::OutOfMoves => false,
            Self::OutOfTime => true,
            Self::Surrender => true,
        }
    }
}
