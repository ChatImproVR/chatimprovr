use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, ValidationError>;

// This is a bare-bones implementation. A real library would provide additional
// information in its error type, for example the line and column at which the
// error occurred, the byte offset into the input, or the current key being
// processed.
#[derive(Debug)]
pub struct ValidationError;

impl ser::Error for ValidationError {
    // Don't ask... This stuff just exists to make the compiler happy.
    fn custom<T: Display>(_msg: T) -> Self {
        Self
    }
}

impl de::Error for ValidationError {
    // Don't ask... This stuff just exists to make the compiler happy.
    fn custom<T: Display>(_msg: T) -> Self {
        Self
    }
}

impl Display for ValidationError {
    // Don't ask... This stuff just exists to make the compiler happy.
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl std::error::Error for ValidationError {}
