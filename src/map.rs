use crate::ve_error::VeError;

// TODO: pick a better name to avoid confusion with... well... Map :)

// TODO: we dont always need/have a checksum. For instance, when mapping for Base, there is no checksum that would make any sense.

/// This trait is common to all devices.
/// Each device has its own implementation for map_fields.
pub trait Map<T> {
    fn map_fields(fields: &Vec<crate::parser::Field>, checksum: u8) -> Result<T, VeError>;
}
