use std::error;
use std::fmt;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum Error {
    IdNotFound,
    NameNotFound,
    Other,
    TypeError,
    LuaError(rlua::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        match self {
            Error::IdNotFound => write!(f, "node ID not found"),
            Error::NameNotFound => write!(f, "name not found"),
            Error::Other => write!(f, "unspecified error"),
            Error::TypeError => write!(f, "type error"),
            Error::LuaError(ref err) => write!(f, "lua error: {}", err),
        }
    }
}

impl error::Error for Error {}

impl From<rlua::Error> for Error {
    fn from(err: rlua::Error) -> Self {
        Error::LuaError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
