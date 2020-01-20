use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct MrtError {
    msg: String,
}

pub fn new(msg: &str) -> MrtError {
    MrtError { msg: msg.to_string() }
}

impl fmt::Display for MrtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Something wrong")
    }
}

impl error::Error for MrtError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}