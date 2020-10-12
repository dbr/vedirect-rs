use crate::ve_error::VeError;

/// This trait is common to all devices.
/// Each device has its own implementation for map_fields.
pub trait Map<T> {
    fn map_fields(fields: &Vec<crate::parser::Field>) -> Result<T, VeError>;
}
