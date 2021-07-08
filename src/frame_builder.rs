use std::{collections::HashMap, fmt, fmt::Display, str::FromStr};

use crate::{
    firmware_version::FirmwareVersion,
    parser::Field,
    serial_number::SerialNumber,
    types::DataBytes,
    types::{Frame, FrameBytes},
    utils::{convert_product_id, convert_string},
    ve_error::VeError,
    Map, MpptFrame, VictronProduct, VictronProductId,
};

/// The builder is receiving data, tries to make frames out of it,
/// recognize the type of device and builds a frame accordingly.
#[derive(Debug)]
pub struct FrameBuilder {
    /// This is the last complete frame
    frame: Option<Box<FrameBytes>>,

    // This buffer fills as we get data, once a frame is found, the current [frame] is replaced and the [buffer] is emptied
    buffer: Box<DataBytes>,
}

impl FrameBuilder {
    pub fn from_frame_data(frame: &FrameBytes) -> Option<Frame> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checksum;

    #[test]
    fn test_mpppt_frame() {
        let frame_data = "\r\nPID\t0xA042\r\nFW\t150\r\nSER#\tHQ1328A1B2C\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nH19\t0\r\nH20\t0\r\nH21\t0\r\nH22\t0\r\nH23\t0\r\nHSDS\t0\r\nChecksum\t".as_bytes();
        let frame_data = &checksum::append_checksum(&frame_data);

        let frame = FrameBuilder::from_frame_data(&frame_data).unwrap();

        // let (fields, checksum, _remainder) = crate::parser::parse(&frame).unwrap();
        // let device = Base::map_fields(&fields, checksum).unwrap();

        match frame {
            Frame::Mppt(f) => println!("frame: {:?}", f),
            _ => panic!("not done"),
        }
    }
}
