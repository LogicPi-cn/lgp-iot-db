use std::{error, fmt, io, num};

#[derive(Debug)]
pub struct PkgError {
    kind: String,
    message: String,
}

impl PkgError {
    pub fn new(kind: String, message: String) -> Self {
        PkgError { kind, message }
    }
}

impl fmt::Display for PkgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind as &str {
            "pkg" => write!(
                f,
                "Packcage Error:{{Kind: {}, Message: {}}}",
                self.kind, self.message
            ),
            _ => write!(f, "Sorry, something is wrong! Please Try Again!"),
        }
    }
}

impl error::Error for PkgError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self)
    }
}

impl From<io::Error> for PkgError {
    fn from(error: io::Error) -> Self {
        PkgError {
            kind: String::from("io"),
            message: error.to_string(),
        }
    }
}

impl From<num::ParseIntError> for PkgError {
    fn from(error: num::ParseIntError) -> Self {
        PkgError {
            kind: String::from("parse"),
            message: error.to_string(),
        }
    }
}
