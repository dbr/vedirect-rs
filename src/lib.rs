mod parser;
mod data;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEError {
    #[error("error parsing data")]
    Parse(String),

    #[error("checksum did not match recieved data")]
    ChecksumError,

    #[error("missing field from recieved data")]
    MissingField(String),
}

// Re-export
pub use parser::parse;
pub use data::mapraw;
