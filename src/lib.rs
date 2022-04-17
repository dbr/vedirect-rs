mod parser;
mod data;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEError {
    #[error("error parsing data")]
    Parse(String),

    #[error("Need more data to parse successfully")]
    NeedMoreData,

    #[error("checksum did not match recieved data")]
    ChecksumError,

    #[error("missing field from recieved data")]
    MissingField(String),
}

// Re-export
pub use parser::parse;
pub use data::map_fields_bmv700;
