use std::error::Error as StdError;

#[derive(Debug)]
pub struct Error {
    pub msg: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.msg)
    }
}

impl StdError for Error {}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: String::from(msg),
        }
    }
}
