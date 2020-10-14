use thiserror::Error;

#[derive(Error, Debug)]
pub enum VeError {
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
impl From<std::num::ParseIntError> for VeError {
    fn from(src: std::num::ParseIntError) -> VeError {
        VeError::Parse(format!("Error parsing integer: {}", src))
    }
}

impl From<std::num::ParseFloatError> for VeError {
    fn from(src: std::num::ParseFloatError) -> VeError {
        VeError::Parse(format!("Error parsing float: {}", src))
    }
}

impl From<std::str::ParseBoolError> for VeError {
    fn from(src: std::str::ParseBoolError) -> VeError {
        VeError::Parse(format!("Error parsing bool: {}", src))
    }
}
