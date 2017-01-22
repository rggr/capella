use std::fmt;
use std::error::Error as StdError;
use std::io;
use std::num::{ParseFloatError, ParseIntError};

use self::Error::Parse;

pub type CapellaResult<T> = Result<T, Error>;

/// `Error` is used for server side errors that may occur.
#[derive(Debug)]
pub enum Error {
    Parse
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Parse => "Error parsing metric",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

// The following `From` conversions are used for parsing integers or floats while destructuring
// a packet.
impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Error {
        Error::Parse
    }
}

impl From<ParseFloatError> for Error {
    fn from(_: ParseFloatError) -> Error {
        Error::Parse
    }
}

// TODO: Should I be doing this for a library type?
impl From<Error> for io::Error {
    fn from(e: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, e.description())
    }
}
