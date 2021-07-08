use crate::{Bmv700, MpptFrame};

///! Data types

// TODO: are those i32 correct? Do we really get negative values ?

// TODO: we should fix the mix between units: Volt / Voltage / Current / Ampere / Watt / Power...
pub type Watt = i32;

#[allow(non_camel_case_types)]
pub type kWh = f32;

pub type Percent = f32;
pub type Volt = f32;
pub type Current = f32;
pub type Minute = i32;

/// This should not change
pub type DataType = u8;

/// This type alias is mainly used to distinguished between bytes forming a proper
/// Frame vs random Data bytes.
/// See also [DataBytes].
pub type FrameBytes = [DataType];

/// See also [FrameBytes].
pub type DataBytes = [DataType];

pub type Checksum = DataType;

pub enum Frame {
    Mppt(MpptFrame),
    Bmv(Bmv700), // TODO: that should become BMV and not BMV-700

    /* Phoenix(PhoenixFrame) */ // TODO: Bring that later
}
