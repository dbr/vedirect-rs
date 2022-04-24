//! Parser for the Victron Energy "VE.Direct" text protocol.

#![forbid(unsafe_code)]
//#![warn(missing_docs)]
#![allow(clippy::upper_case_acronyms)]

mod data;
mod parser;

use thiserror::Error;

/// Errors from the parser
#[derive(Error, Debug)]
pub enum VEError {
    /// General parsing errors
    #[error("error parsing data: {0}")]
    Parse(String),

    /// A non-fatal error. The parser can be fed a stream of data in
    /// chunks; and a given chunk might, for example, stop in the
    /// middle of a field name (in which case the parser needs more
    /// data to continue working)
    #[error("Need more data to parse successfully")]
    NeedMoreData,

    /// Malformed data
    #[error("checksum did not match recieved data")]
    ChecksumError,

    /// A required field was missing from the received data
    #[error("missing field from received data")]
    MissingField(String),

    /// A field was expected to be a boolean type but received
    /// differently formatted data
    #[error("ON or OFF value expected")]
    OnOffExpected(String),

    /// Some fields are encoded as hexidecimal codes, and this error
    /// occurs if the received code is not recognized
    #[error("Unknown enum code")]
    UnknownCode(String),
}

// Re-export
pub use data::Bmv700;
pub use data::MPPT;
pub use parser::Events;
pub use parser::Parser;
