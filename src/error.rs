use std::error::Error;
use std::fmt;

pub type TubResult<T> = Result<T, TubError>;

#[derive(Debug, Clone)]
pub enum TubError {
    OsError(String),
    ContextCreationError(String),
    IconLoadError(u16)
}

impl fmt::Display for TubError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TubError::*;

        match *self {
            OsError(ref s) => write!(f, "{}", s),
            ContextCreationError(ref s) => write!(f, "{}", s),
            IconLoadError(size) => write!(f, "Could not load {0}x{0} icon", size)
        }
    }
}

impl Error for TubError {
    fn description<'a>(&'a self) -> &'a str {
        use self::TubError::*;

        match *self {
            OsError(ref s) => s,
            ContextCreationError(ref s) => s,
            IconLoadError(_) => "Icon load error"
        }
    }
}