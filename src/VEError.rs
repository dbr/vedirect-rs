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

// Type conversion errors
impl From<std::num::ParseIntError> for VEError {
    fn from(src: std::num::ParseIntError) -> VEError {
        VEError::Parse(format!("Error parsing integer: {}", src))
    }
}

impl From<std::num::ParseFloatError> for VEError {
    fn from(src: std::num::ParseFloatError) -> VEError {
        VEError::Parse(format!("Error parsing float: {}", src))
    }
}