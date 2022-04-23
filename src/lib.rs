mod data;
mod parser;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEError {
    #[error("error parsing data: {0}")]
    Parse(String),

    #[error("Need more data to parse successfully")]
    NeedMoreData,

    #[error("checksum did not match recieved data")]
    ChecksumError,

    #[error("missing field from recieved data")]
    MissingField(String),

    #[error("ON or OFF value expected")]
    OnOffExpected(String),

    #[error("Unknown enum code")]
    UnknownCode(String),
}

// Re-export
pub use parser::Events;
pub use parser::Parser;
pub use data::Bmv700;
pub use data::MPPT;
