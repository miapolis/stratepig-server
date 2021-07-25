#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),

    FailedToSendBytes,
    FailedToRegisterForEvents,
    InvalidData(String),
    ConnectionNotFound,

    #[doc(hidden)]
    __Nonexhaustive,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}
