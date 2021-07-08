//! Specs for this implementation can be found at https://www.sv-zanshin.com/r/manuals/victron-ve-direct-protocol.pdf

use crate::types::*;
use crate::utils::*;
use crate::ve_error::VeError;
use crate::{checksum, constants::*};
use crate::{firmware_version::FirmwareVersion, parser::Field};
use crate::{map::Map, serial_number::SerialNumber};
use std::{collections::hash_map::HashMap, str::FromStr};

use super::base;

/// MPPT solar charge controllers Frame definition.
#[derive(Debug)]
pub struct MpptFrame {
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

    /// Main (battery) voltage.
    ///
    /// Specs:
    /// - Frame Label: V
    /// - Frame unit: mV
    /// - Field unit: V
    pub voltage: Volt,

    /// Battery current converted to A
    ///
    /// Specs:
    /// - Frame Label: I
    /// - Frame Unit: Unit: mA
    /// - Field unit: A
    pub current: Current,

    /// Panel voltage, converted to V.
    ///
    /// Specs:
    /// - Frame Label: VPV
    /// - Frame Unit: mV
    /// - Field unit: V
    pub vpv: Volt,

    /// Panel Power
    ///
    /// Specs:
    /// - Frame Label: PPV
    /// - Frame Unit: W
    /// - Field unit: W
    pub ppv: Watt,

    /// State of Operation
    ///
    /// Specs:
    /// - Frame Label: CS
    pub charge_state: ChargeState,

    // Tracker operation state
    /// Label: MPPT
    /// See [MpptOperationState]
    pub mppt: MpptOperationState,

    // Off reason, this field described why a unit is switched off.
    /// Label: OR
    /// See [OffReason]
    pub off_reason: OffReason,

    /// Error code    
    ///
    /// Specs:
    /// - Frame Label: ERR
    pub error: Err,

    /// Whether the load is turned ON(true) or OFF(false)
    ///
    /// Specs:
    /// - Frame Label: LOAD
    pub load: Option<bool>,

    /// Load current, converted to A
    ///
    /// Specs:
    /// - Frame Label: IL
    /// - Frame Unit: mA
    /// - Field unit: A
    pub load_current: Option<Current>,

    /// Yield total (user resettable counter) in 0.01 kWh converted to kWh
    ///
    /// Specs:
    /// - Frame Label: H19
    /// - Frame Unit: 0.01 kWh
    /// - Field unit: kWh
    pub yield_total: kWh,

    /// Yield today in 0.01 kWh converted to kWh
    ///
    /// Specs:
    /// - Frame Label: H20
    /// - Frame Unit: 0.01 kWh
    /// - Field unit: kWh
    pub yield_today: kWh,

    /// Maximum power today
    ///
    /// Specs:
    /// - Frame Label: H21
    /// - Frame Unit: W
    /// - Field unit: W
    pub max_power_today: Watt,

    /// Yield today in 0.01 kWh converted to kWh
    ///
    /// Specs:
    /// - Frame Label: H22
    /// - Frame Unit: 0.01 kWh
    /// - Field unit: kWh
    pub yield_yesterday: kWh,

    /// Maximum power today
    ///
    /// Specs:
    /// - Frame Label: H23
    /// - Frame Unit: W
    /// - Field unit: W
    pub max_power_yesterday: Watt,

    /// Historical data. The day sequence number, a change in this number indicates a new day. This
    /// implies that the historical data has changed. Range 0..364.
    ///
    /// Note: The HSDS field is available in the MPPT charger since version v1.16.
    ///
    /// Specs:
    /// - Frame Label: HSDS
    pub hsds: u16,

    /// The checksum
    ///
    /// Specs:
    /// - Frame label: Checksum
    pub checksum: u8,
}

pub trait VictronProduct {
    /// Get the ProductId of the device
    fn get_product_id(&self) -> VictronProductId;

    /// Get the product name
    fn get_name(&self) -> String;

    /// Get the serial number of the device
    fn get_serial_number(&self) -> &SerialNumber;

    /// Get the firmware version reported by the device
    fn get_firmware_version(&self) -> &FirmwareVersion;

    // Based on the ProductId, this function returns the appropriate variant for the Frame
    // fn get_frame(&self) -> Frame;
}

impl VictronProduct for MpptFrame {
    fn get_product_id(&self) -> VictronProductId {
        self.pid
    }

    fn get_name(&self) -> String {
        base::get_name(self.pid)
    }

    fn get_serial_number(&self) -> &SerialNumber {
        &self.serial_number
    }

    fn get_firmware_version(&self) -> &FirmwareVersion {
        &self.firmware
    }

    // TODO: it makes no sense to have this implementation here
    // fn get_frame(&self) -> Frame {
    //     todo!()
    // }
}

impl ToString for MpptFrame {
    /// Returns the whole string for the frame except the checksum that is likely not a valid utf8 char
    fn to_string(&self) -> String {
        format!("{pid}{fw}{ser}{v}{i}{vpv}{ppv}{cs}{mppt}{or}{err}{load}{il}{h19}{h20}{h21}{h22}{h23}{hsds}{checksum}",
        pid = get_field_string("PID", Some(format!("0x{:X}", self.pid as u32))),
        fw = get_field_string("FW", Some(&self.firmware.to_encoded_version())),
        ser = get_field_string("SER#", Some(&self.serial_number)),
        v = get_field_string("V", Some(self.voltage)),
        i = get_field_string("I", Some(self.current)),
        vpv= get_field_string("VPV", Some(self.vpv)),
        ppv = get_field_string("PPV", Some(self.ppv)),
        cs = get_field_string("CS", Some(self.charge_state as u32)),
        mppt= get_field_string("MPPT", Some(self.mppt as u32)),
        or = get_field_string("OR", Some( format!("0x{:08x}", self.off_reason as u32))),
        err = get_field_string("ERR", Some(self.error as u32)),

        load = get_field_string("LOAD", match self.load {
            Some(state) => if state { Some("ON") } else { Some("OFF") },
            None => None,
        }),
        il = get_field_string("IL", self.load_current),

        h19 = get_field_string("H19", Some(self.yield_total)),
        h20 = get_field_string("H20", Some(self.yield_today)),
        h21 = get_field_string("H21", Some(self.max_power_today)),
        h22 = get_field_string("H22", Some(self.yield_yesterday)),
        h23 = get_field_string("H23", Some(self.max_power_yesterday)),
        hsds = get_field_string("HSDS", Some(self.hsds)),
        checksum = format!("\r\nChecksum\t"),
        )
    }
}

impl Default for MpptFrame {
    fn default() -> Self {
        Self {
            pid: VictronProductId::BlueSolar_MPPT_75_15,
            firmware: FirmwareVersion::from_str("150").unwrap(),
            serial_number: SerialNumber::from_str("HQ1328A1B2C").unwrap(),
            voltage: 0.0,
            current: 0.0,
            vpv: 0.0,
            ppv: 0,
            charge_state: ChargeState::Off,
            mppt: MpptOperationState::Off,
            off_reason: OffReason::NoInputPower,

            yield_total: 0_f32,
            yield_today: 0_f32,
            max_power_today: 0,
            yield_yesterday: 0_f32,
            max_power_yesterday: 0,

            load: None,
            load_current: None,

            hsds: 0,
            error: Err::NoError,
            checksum: 0,
        }
    }
}

impl Map<MpptFrame> for MpptFrame {
    fn map_fields(fields: &Vec<Field>, checksum: u8) -> Result<Self, VeError> {
        let mut hm: HashMap<&str, &str> = HashMap::new();
        for f in fields {
            hm.insert(f.key, f.value);
        }

        let sn = convert_string(&hm, "SER#").unwrap();

        Ok(MpptFrame {
            pid: convert_product_id(&hm, "PID")?,
            firmware: FirmwareVersion::from_str(&convert_string(&hm, "FW")?).unwrap(),
            serial_number: SerialNumber::from_str(&sn).unwrap(),
            voltage: convert_f32(&hm, "V")? / 1000_f32,
            current: convert_f32(&hm, "I")? / 1000_f32,
            vpv: convert_f32(&hm, "VPV")? / 1000_f32,
            ppv: convert_watt(&hm, "PPV")?,
            charge_state: convert_enum::<ChargeState>(&hm, "CS")?,
            mppt: convert_enum::<MpptOperationState>(&hm, "MPPT")?,
            off_reason: convert_enum::<OffReason>(&hm, "OR")?,
            error: convert_enum::<Err>(&hm, "ERR")?,

            load: convert_load(&hm, "LOAD")?,
            load_current: convert_load_current(&hm, "IL")?,

            yield_total: convert_yield(&hm, "H19")?,
            yield_today: convert_yield(&hm, "H20")?,
            max_power_today: convert_watt(&hm, "H21")?,
            yield_yesterday: convert_yield(&hm, "H22")?,
            max_power_yesterday: convert_watt(&hm, "H23")?,
            hsds: convert_u32(&hm, "HSDS")? as u16,
            checksum,
        })
    }
}

impl Into<Vec<u8>> for MpptFrame {
    fn into(self) -> Vec<u8> {
        let str = self.to_string();
        let raw = str.as_bytes();
        let checksum = checksum::calculate(raw);
        checksum::append(raw, checksum)
    }
}

impl MpptFrame {
    /// Creates a new device based on the provided frame.
    pub fn new(frame: &[u8]) -> Result<Self, VeError> {
        let (raw, checksum, _remainder) = crate::parser::parse(frame)?;
        MpptFrame::map_fields(&raw, checksum)
    }

    /// This ctor is mainly used for some of the tests to prevent having to generate frames.
    pub fn init(
        pid: VictronProductId,
        firmware: FirmwareVersion,
        serial_number: SerialNumber,
        voltage: Volt,
        current: Current,
        vpv: Volt,
        ppv: Watt,
        charge_state: ChargeState,
        mppt: MpptOperationState,
        off_reason: OffReason,
        error: Err,
        load: Option<bool>,
        load_current: Option<Current>,
        yield_total: kWh,
        yield_today: kWh,
        max_power_today: Watt,
        yield_yesterday: kWh,
        max_power_yesterday: Watt,
        hsds: u16,
        checksum: u8,
    ) -> Self {
        Self {
            pid,
            firmware,
            serial_number,
            voltage,
            current,
            vpv,
            ppv,
            charge_state,
            mppt,
            off_reason,
            error,
            load,
            load_current,
            yield_total,
            yield_today,
            max_power_today,
            yield_yesterday,
            max_power_yesterday,
            hsds,
            checksum,
        }
    }
}

#[cfg(test)]
mod tests_mppt {
    use super::*;

    #[test]
    fn test_mppt_to_string() {
        let mppt = MpptFrame::default();
        let frame = mppt.to_string();
        let default_frame = "\r\nPID\t0xA042\r\nFW\t150\r\nSER#\tHQ1328A1B2C\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nH19\t0\r\nH20\t0\r\nH21\t0\r\nH22\t0\r\nH23\t0\r\nHSDS\t0\r\nChecksum\t";
        assert_eq!(frame, default_frame);
    }

    #[test]
    fn test_mppt_to_bytes_fw() {
        let mppt = MpptFrame::default();
        let bytes: Vec<u8> = mppt.into();
        let bytes_no_checksum = bytes.split_last().unwrap().1;
        let frame_without_checksum = "\r\nPID\t0xA042\r\nFW\t150\r\nSER#\tHQ1328A1B2C\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nH19\t0\r\nH20\t0\r\nH21\t0\r\nH22\t0\r\nH23\t0\r\nHSDS\t0\r\nChecksum\t".as_bytes();

        assert_eq!(bytes_no_checksum, frame_without_checksum);
        assert_eq!(bytes.split_last().unwrap().0, &68);
    }

    #[test]
    fn test_mppt_1() {
        let frame = "\r\nPID\t0xA042\r\nFW\t150\r\nSER#\tHQ1328A1B2C\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nH19\t0\r\nH20\t0\r\nH21\t0\r\nH22\t0\r\nH23\t0\r\nHSDS\t0\r\nChecksum\t".as_bytes();
        let frame = &checksum::append(frame, 68);
        let (fields, checksum, _remainder) = crate::parser::parse(&frame).unwrap();
        let device = MpptFrame::map_fields(&fields, checksum).unwrap();

        assert_eq!(device.pid, VictronProductId::BlueSolar_MPPT_75_15);
        assert_eq!(device.firmware, FirmwareVersion::from_str("150").unwrap());
        assert_eq!(
            device.serial_number,
            SerialNumber::from_str("HQ1328A1B2C").unwrap()
        );
        assert_eq!(device.voltage, 0.0);
        assert_eq!(device.current, 0.0);
        assert_eq!(device.load, None);
        assert_eq!(device.load_current, None);
        assert_eq!(device.vpv, 0.0);
        assert_eq!(device.ppv, 0);
        assert_eq!(device.charge_state, ChargeState::Off);
        assert_eq!(device.mppt, MpptOperationState::Off);
        assert_eq!(device.off_reason, OffReason::NoInputPower);
        assert_eq!(device.error, Err::NoError);
        assert_eq!(device.yield_total, 0_f32);
        assert_eq!(device.yield_today, 0_f32);
        assert_eq!(device.max_power_today, 0);
        assert_eq!(device.yield_yesterday, 0_f32);
        assert_eq!(device.max_power_yesterday, 0);
        assert_eq!(device.hsds, 0);
        assert_eq!(device.checksum, 68);

        assert_eq!(device.get_name(), "BlueSolar MPPT 75/15");
    }

    #[test]
    fn test_mppt_older_versions() {
        let frame = "\r\nPID\t0xA042\r\nFW\t150\r\nSER#\tHQ1328A1B2C\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nH19\t0\r\nH20\t0\r\nH21\t0\r\nH22\t0\r\nH23\t0\r\nHSDS\t0\r\nChecksum\t".as_bytes();
        let frame = &checksum::append(frame, 68);
        let (raw, checksum, _remainder) = crate::parser::parse(frame).unwrap();
        let device = MpptFrame::map_fields(&raw, checksum).unwrap();

        assert_eq!(device.pid, VictronProductId::BlueSolar_MPPT_75_15);
        assert_eq!(device.firmware, FirmwareVersion::from_str("150").unwrap());
        assert_eq!(
            device.serial_number,
            SerialNumber::from_str("HQ1328A1B2C").unwrap()
        );
        assert_eq!(device.voltage, 0.0);
        assert_eq!(device.current, 0.0);
        assert_eq!(device.vpv, 0.0);
        assert_eq!(device.ppv, 0);
        assert_eq!(device.charge_state, ChargeState::Off);
        assert_eq!(device.mppt, MpptOperationState::Off);
        assert_eq!(device.off_reason, OffReason::NoInputPower);
        assert_eq!(device.error, Err::NoError);
        assert_eq!(device.yield_total, 0_f32);
        assert_eq!(device.yield_today, 0_f32);
        assert_eq!(device.max_power_today, 0);
        assert_eq!(device.yield_yesterday, 0_f32);
        assert_eq!(device.max_power_yesterday, 0);
        assert_eq!(device.hsds, 0);
        assert_eq!(device.checksum, 68);
    }

    #[test]
    fn test_mppt_new() {
        let frame = "\r\nPID\t0xA042\
            \r\nFW\t150\
            \r\nSER#\tHQ1328A1B2C\
            \r\nV\t12340\
            \r\nI\t1230\
            \r\nVPV\t36630\
            \r\nPPV\t99\
            \r\nCS\t0\
            \r\nMPPT\t0\
            \r\nOR\t0x00000001\
            \r\nLOAD\tON\
            \r\nIL\t5430\
            \r\nERR\t26\
            \r\nH19\t1234\
            \r\nH20\t2345\
            \r\nH21\t99\
            \r\nH22\t4567\
            \r\nH23\t98\
            \r\nHSDS\t0\
            \r\nChecksum\t"
            .as_bytes();
        let frame = &checksum::append(&frame, 217);
        let device = MpptFrame::new(frame).unwrap();

        assert_eq!(device.pid, VictronProductId::BlueSolar_MPPT_75_15);
        assert_eq!(device.firmware, FirmwareVersion::from_str("150").unwrap());
        assert_eq!(
            device.serial_number,
            SerialNumber::from_str("HQ1328A1B2C").unwrap()
        );
        assert_eq!(device.voltage, 12.34);
        assert_eq!(device.current, 1.23);
        assert_eq!(device.vpv, 36.63);
        assert_eq!(device.ppv, 99);
        assert_eq!(device.load, Some(true));
        assert_eq!(device.load_current, Some(5.43));
        assert_eq!(device.charge_state, ChargeState::Off);
        assert_eq!(device.mppt, MpptOperationState::Off);
        assert_eq!(device.off_reason, OffReason::NoInputPower);
        assert_eq!(device.error, Err::TerminalsOverheated);
        assert_eq!(device.yield_total, 12.34);
        assert_eq!(device.yield_today, 23.45);
        assert_eq!(device.max_power_today, 99);
        assert_eq!(device.yield_yesterday, 45.67);
        assert_eq!(device.max_power_yesterday, 98);
        assert_eq!(device.hsds, 0);
        assert_eq!(device.checksum, 217);
    }

    #[test]
    fn test_mppt_init() {
        let device = MpptFrame::init(
            VictronProductId::BlueSolar_MPPT_75_15,
            FirmwareVersion::from_str("420").unwrap(),
            SerialNumber::from_str("HQ1328A1B2C").unwrap(),
            12.34,
            1.23,
            36.63,
            99,
            ChargeState::Float,
            MpptOperationState::VoltageOrCurrentLimited,
            OffReason::ProtectionActive,
            Err::SolarInputCurrentTooHigh,
            Some(true),
            Some(17.45),
            10000_f32,
            500_f32,
            98,
            489_f32,
            97,
            4,
            12,
        );

        assert_eq!(device.pid, VictronProductId::BlueSolar_MPPT_75_15);
        assert_eq!(device.firmware, FirmwareVersion::from_str("420").unwrap());
        assert_eq!(
            device.serial_number,
            SerialNumber::from_str("HQ1328A1B2C").unwrap()
        );
        assert_eq!(device.voltage, 12.34);
        assert_eq!(device.current, 1.23);
        assert_eq!(device.vpv, 36.63);
        assert_eq!(device.ppv, 99);
        assert_eq!(device.charge_state, ChargeState::Float);
        assert_eq!(device.mppt, MpptOperationState::VoltageOrCurrentLimited);
        assert_eq!(device.off_reason, OffReason::ProtectionActive);
        assert_eq!(device.error, Err::SolarInputCurrentTooHigh);
        assert_eq!(device.yield_total, 10000_f32);
        assert_eq!(device.yield_today, 500_f32);
        assert_eq!(device.max_power_today, 98);
        assert_eq!(device.yield_yesterday, 489_f32);
        assert_eq!(device.max_power_yesterday, 97);
        assert_eq!(device.hsds, 4);
        assert_eq!(device.checksum, 12);
    }
}
