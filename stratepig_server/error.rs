#[derive(Debug)]
pub enum StratepigError {
    Core(stratepig_core::Error),

    AssumeWrongId,
    MissingContext,
    Unspecified,
    Default(String),
}

impl StratepigError {
    pub fn with(msg: &str) -> Self {
        Self::Default(msg.to_owned())
    }
}

impl From<stratepig_core::Error> for StratepigError {
    fn from(err: stratepig_core::Error) -> StratepigError {
        StratepigError::Core(err)
    }
}
