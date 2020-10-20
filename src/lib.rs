//! Crate to parse Victron VE.Direct frames and provide data in a rusty way.
//!
//! Library to parse the Victron Energy "VE.Direct" protocol and map the data to useful structs with clear units.

mod bmv;
mod constants;
mod map;
mod mppt;
mod parser;
mod types;
mod utils;
mod ve_error;

pub use bmv::Bmv700;
pub use constants::*;
pub use map::Map;
pub use mppt::Mppt;
pub use parser::parse;
