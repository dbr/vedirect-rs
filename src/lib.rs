//! Crate to parse Victron VE.Direct frames and provide data in a rusty way.
//!
//! Library to parse the Victron Energy "VE.Direct" protocol and map the data to useful structs with clear units.
#![feature(test)]

mod bmv;
mod checksum;
mod constants;
mod firmware_version;
mod frames;
mod map;
mod parser;
mod serial_number;
mod types;
mod utils;
mod ve_error;

pub use bmv::Bmv700;
pub use constants::*;
pub use frames::mppt_frame::*;
pub use map::Map;
pub use parser::parse;
