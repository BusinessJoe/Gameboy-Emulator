use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    AddresssingError {
        address: usize,
        source: Option<String>,
    },
    Message(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::result::Result<(), std::fmt::Error> {
        match self {
            Error::AddresssingError { address, source } => {
                if let Some(source) = source {
                    write!(f, "AddressingError at {:x} from {}", address, source)
                } else {
                    write!(f, "AddressingError at {:x}", address)
                }
            }
            Error::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl StdError for Error {}

pub type Result<T> = std::result::Result<T, Error>;
/*
impl Error {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: String::from(msg),
        }
    }
}
*/
impl Error {
    pub fn from_address(address: usize) -> Self {
        Error::AddresssingError {
            address,
            source: None,
        }
    }

    pub fn from_address_with_source(address: usize, source: String) -> Self {
        Error::AddresssingError {
            address,
            source: Some(source),
        }
    }

    pub fn from_message(msg: String) -> Self {
        Error::Message(msg)
    }
}

impl From<crate::cartridge::AddressingError> for Error {
    fn from(value: crate::cartridge::AddressingError) -> Self {
        Error::AddresssingError {
            address: value.0,
            source: None,
        }
    }
}

impl From<String> for Error {
    fn from(str: String) -> Self {
        Error::from_message(str)
    }
}
