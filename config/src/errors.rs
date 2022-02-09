use std::backtrace::Backtrace;

use std;
use std::fmt::{self, Display};
use thiserror::Error;

use serde::{de, ser};

// This is a bare-bones implementation. A real library would provide additional
// information in its error type, for example the line and column at which the
// error occurred, the byte offset into the input, or the current key being
// processed.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    Message(String),

    IntOutOfRange { value: isize, min: isize, max: isize },
    FloatOutOfRange { value: f64, min: f64, max: f64 },
    KeyDoesNotExist,
    Syntax,
    ValueRequired,

    ExpectedBoolean,
    ExpectedInteger,
    ExpectedFloat,
    ExpectedString,
    ExpectedStringOrPath,
    ExpectedPath,
    ExpectedOption,
    ExpectedVec,
    ExpectedStruct,
    ExpectedTuple,
    ExpectedMap,
    ExpectedEnum,
    ExpectedUnitVariant,
    ExpectedNewTypeVariant,
    ExpectedTupleVariant,
    ExpectedStructVariant,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            /* and so forth */
            _ => todo!(),
        }
    }
}

impl std::error::Error for Error {}
