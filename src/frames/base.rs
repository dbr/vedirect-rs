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

/// The base frame common to all devices
#[derive(Debug)]
pub struct Base {
    /// Label: PID, Product ID
    pid: VictronProductId,

    /// Label: FW, Firmware version
    firmware: FirmwareVersion,

    /// The serial number of the device. The notation is LLYYMMSSSSS, where LL=location code,
    /// YYWW=production datestamp (year, week) and SSSSS=unique part of the serial number.
    /// Example: HQ1328A1B2C
    ///
    /// Specs:
    /// - Frame Label: SER#
    serial_number: SerialNumber,

    /// This is the last complete frame
    frame: Option<Box<FrameBytes>>,
    // This buffer fills as we get data, once a frame is found, the current [frame] is replaced and the [buffer] is emptied
    buffer: Box<DataBytes>,
}

// TODO: this is redundant with MpptFrame for instance. Can be solved using a macro.
impl VictronProduct for Base {
    fn get_product_id(&self) -> VictronProductId {
        self.pid
    }

    fn get_name(&self) -> String {
        get_name(self.pid)
    }

    fn get_serial_number(&self) -> &SerialNumber {
        &self.serial_number
    }

    fn get_firmware_version(&self) -> &FirmwareVersion {
        &self.firmware
    }

    fn get_frame(&self) -> Frame {
        match self.pid {
            VictronProductId::BlueSolar_MPPT_70_15
            | VictronProductId::BlueSolar_MPPT_75_10
            | VictronProductId::BlueSolar_MPPT_75_15 => {
                let frame = &*self.frame.unwrap(); // TODO: That a very strong assumption (as in wrong...)
                Frame::Mppt(MpptFrame::new(&frame).unwrap())
            }

            _ => panic!("We don't support this product ID yet"),
        }
    }
}

impl Map<Base> for Base {
    fn map_fields(fields: &Vec<Field>, _checksum: u8) -> Result<Self, VeError> {
        let mut hm: HashMap<&str, &str> = HashMap::new();
        for f in fields {
            hm.insert(f.key, f.value);
        }

        let sn = convert_string(&hm, "SER#").unwrap();

        Ok(Base {
            pid: convert_product_id(&hm, "PID")?,
            firmware: FirmwareVersion::from_str(&convert_string(&hm, "FW")?).unwrap(),
            serial_number: SerialNumber::from_str(&sn).unwrap(),
            frame: Some(Box::new([])), // TODO: change that
            buffer: Box::new([]),      // TODO: change that
        })
    }
}

impl Display for Base {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - s/n: {} - Firmware v{}",
            self.get_name(),
            self.serial_number.to_string(),
            self.firmware.version.to_string()
        )
    }
}

/// Given a [VictronProductId], this function return the name of the device
/// as a [String].
pub fn get_name(pid: VictronProductId) -> String {
    match pid {
        VictronProductId::BMV700 => "BMV-700".into(),
        VictronProductId::BMV702 => "BMV-702".into(),
        VictronProductId::BMV700H => "BMV-700H".into(),
        VictronProductId::BlueSolar_MPPT_75_10 => "BlueSolar MPPT 75/10".into(),
        VictronProductId::BlueSolar_MPPT_150_100 => "BlueSolar MPPT 150/100".into(),
        VictronProductId::BlueSolar_MPPT_70_15 => "BlueSolar MPPT 70/15".into(),
        VictronProductId::BlueSolar_MPPT_75_15 => "BlueSolar MPPT 75/15".into(),
        VictronProductId::BlueSolar_MPPT_100_15 => "BlueSolar MPPT 100/15".into(),
        VictronProductId::BlueSolar_MPPT_100_30_rev1 => "BlueSolar MPPT 100/30 rev1".into(),
        VictronProductId::BlueSolar_MPPT_100_30_rev2 => "BlueSolar MPPT 100/30 rev2".into(),
        VictronProductId::BlueSolar_MPPT_150_35_rev1 => "BlueSolar MPPT 150/35 rev1".into(),
        VictronProductId::BlueSolar_MPPT_150_35_rev2 => "BlueSolar MPPT 150/35 rev2".into(),
        VictronProductId::BlueSolar_MPPT_150_45 => "BlueSolar MPPT 150/45".into(),
        VictronProductId::BlueSolar_MPPT_150_60 => "BlueSolar MPPT 150/60".into(),
        VictronProductId::BlueSolar_MPPT_150_70 => "BlueSolar MPPT 150/70".into(),
        VictronProductId::BlueSolar_MPPT_150_85 => "BlueSolar MPPT 150/85".into(),
        VictronProductId::BlueSolar_MPPT_75_50 => "BlueSolar MPPT 75/50".into(),
        VictronProductId::BlueSolar_MPPT_100_50_rev1 => "BlueSolar MPPT 100/50 rev1".into(),
        VictronProductId::BlueSolar_MPPT_100_50_rev2 => "BlueSolar MPPT 100/50 rev2".into(),
        VictronProductId::SmartSolar_MPPT_150_100 => "SmartSolar MPPT 150/100".into(),
        VictronProductId::SmartSolar_MPPT_250_100 => "SmartSolar MPPT 250/100".into(),
        VictronProductId::Phoenix_Inverter_12V_250VA_230V => {
            "Phoenix Inverter 12V 250VA 230V".into()
        }
        VictronProductId::Phoenix_Inverter_24V_250VA_230V => {
            "Phoenix Inverter 24V 250VA 230V".into()
        }
        VictronProductId::Phoenix_Inverter_48V_250VA_230V => {
            "Phoenix Inverter 48V 250VA 230V".into()
        }
        VictronProductId::Phoenix_Inverter_12V_375VA_230V => {
            "Phoenix Inverter 12V 375VA 230V".into()
        }
        VictronProductId::Phoenix_Inverter_24V_375VA_230V => {
            "Phoenix Inverter 24V 375VA 230V".into()
        }
        VictronProductId::Phoenix_Inverter_48V_375VA_230V => {
            "Phoenix Inverter 48V 375VA 230V".into()
        }
        VictronProductId::Phoenix_Inverter_12V_500VA_230V => {
            "Phoenix Inverter 12V 500VA 230V".into()
        }
        VictronProductId::Phoenix_Inverter_24V_500VA_230V => {
            "Phoenix Inverter 24V 500VA 230V".into()
        }
        VictronProductId::Phoenix_Inverter_48V_500VA_230V => {
            "Phoenix Inverter 48V 500VA 230V".into()
        } // _ => "Unknown".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checksum;

    #[test]
    fn test_mpppt_frame() {
        let frame = "\r\nPID\t0xA042\r\nFW\t150\r\nSER#\tHQ1328A1B2C\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nH19\t0\r\nH20\t0\r\nH21\t0\r\nH22\t0\r\nH23\t0\r\nHSDS\t0\r\nChecksum\t".as_bytes();
        let frame = &checksum::append_checksum(frame);
        let (fields, checksum, _remainder) = crate::parser::parse(&frame).unwrap();
        let device = Base::map_fields(&fields, checksum).unwrap();
        println!("{}", device);
    }

    #[test]
    fn test_bmv_frame() {
        let frame = "\r\nPID\t0x203\r\nFW\tC208\r\nSER#\tHQ1328A1B2C\r\nP\t123\r\nCE\t53\r\nSOC\t452\r\nTTG\t60\r\nRelay\tOFF\r\nAlarm\tOFF\r\nV\t232\r\nChecksum\t".as_bytes();
        let frame = &checksum::append_checksum(frame);
        let (fields, checksum, _remainder) = crate::parser::parse(&frame).unwrap();
        let device = Base::map_fields(&fields, checksum).unwrap();
        println!("{}", device);
    }

    #[test]
    fn test_phoenix_frame() {
        let frame = "\r\nPID\t0xA212\r\nFW\tC208\r\nSER#\tHQ1328A1B2C\r\nP\t123\r\nCE\t53\r\nSOC\t452\r\nTTG\t60\r\nRelay\tOFF\r\nAlarm\tOFF\r\nV\t232\r\nChecksum\t".as_bytes();
        let frame = &checksum::append_checksum(frame);
        let (fields, checksum, _remainder) = crate::parser::parse(&frame).unwrap();
        let device = Base::map_fields(&fields, checksum).unwrap();
        println!("{}", device);
    }
}
