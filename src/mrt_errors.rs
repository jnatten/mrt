use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct MrtError {
    msg: String,
}

impl MrtError {
    pub fn new(msg: &str) -> MrtError {
        MrtError {
            msg: msg.to_string(),
        }
    }
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

impl From<std::io::Error> for MrtError {
    fn from(err: std::io::Error) -> Self {
        MrtError::new(err.to_string().as_str())
    }
}

impl From<rayon::ThreadPoolBuildError> for MrtError {
    fn from(err: rayon::ThreadPoolBuildError) -> Self {
        MrtError::new(err.to_string().as_str())
    }
}
