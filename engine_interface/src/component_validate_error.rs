use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, ValidationError>;

// TODO: Make this more descriptive
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
                "Option<T> is variable size. Consider using FixedOption<T> instead!"
            ),
        }
    }
}

impl std::error::Error for ValidationError {}
