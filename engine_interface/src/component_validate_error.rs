use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, ValidationError>;

// This is a bare-bones implementation. A real library would provide additional
// information in its error type, for example the line and column at which the
// error occurred, the byte offset into the input, or the current key being
// processed.
#[derive(Debug)]
pub enum ValidationError {
    Sequence,
    Enum,
    Option,
}

impl ser::Error for ValidationError {
    // Don't ask... This stuff just exists to make the compiler happy.
    fn custom<T: Display>(_msg: T) -> Self {
        panic!("Custom error unsupported")
    }
}

impl de::Error for ValidationError {
    // Don't ask... This stuff just exists to make the compiler happy.
    fn custom<T: Display>(_msg: T) -> Self {
        panic!("Custom error unsupported")
    }
}

impl Display for ValidationError {
    // Don't ask... This stuff just exists to make the compiler happy.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::Sequence => write!(f, "Sequence (vector or string) of arbitrary size"),
            ValidationError::Enum => write!(f, "Enum of arbitrary size"),
            ValidationError::Option => write!(
                f,
                "Enum of arbitrary size. Consider using FixedOption instead!"
            ),
        }
    }
}

impl std::error::Error for ValidationError {}
