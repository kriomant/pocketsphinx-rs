use std;

pub struct Error;

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PocketSphinx error, see log for details")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str { "PocketSphinx error"}
    fn cause(&self) -> Option<&std::error::Error> { None }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PocketSphinx error, see log for details")
    }
}

pub type Result<T> = std::result::Result<T, Error>;
