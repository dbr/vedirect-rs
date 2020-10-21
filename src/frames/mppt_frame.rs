//! Specs for this implementation can be found at https://www.sv-zanshin.com/r/manuals/victron-ve-direct-protocol.pdf

use crate::parser::Field;
use crate::types::*;
use crate::utils::*;
use crate::ve_error::VeError;
use crate::{checksum, constants::*};
use crate::{map::Map, serial_number::SerialNumber};
use std::{collections::hash_map::HashMap, str::FromStr};

/// MPPT solar charge controllers Frame definition.
#[derive(Debug)]
pub struct MpptFrame {
    /// Label: PID, Product ID
    pub pid: VictronProductId,

    /// Label: FW, Firmware version
    pub firmware: String, // TODO: check if that could be a semver => yes it is. 150 = v1.50

    /// The serial number of the device. The notation is LLYYMMSSSSS, where LL=location code,
    /// YYWW=production datestamp (year, week) and SSSSS=unique part of the serial number.
    /// Example: HQ1328A1B2C
    ///
    /// Specs:
    /// - Frame Label: SER#
    pub serial_number: SerialNumber,

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
    fn get_name(&self) -> String;
}

impl VictronProduct for MpptFrame {
    fn get_name(&self) -> String {
        match self.pid {
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
}

impl ToString for MpptFrame {
    /// Returns the whole string for the frame except the checksum that is likely not a valid utf8 char
    fn to_string(&self) -> String {
        format!("{pid}{fw}{ser}{v}{i}{vpv}{ppv}{cs}{mppt}{or}{err}{load}{il}{h19}{h20}{h21}{h22}{h23}{hsds}{checksum}",
        pid = get_field_string("PID", Some(format!("0x{:X}", self.pid as u32))),
        fw = get_field_string("FW", Some(&self.firmware)),
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
            firmware: "150".into(),
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

        println!("ALIVE");
        println!("{}: {:#?} ", checksum, fields);

        let sn = convert_string(&hm, "SER#").unwrap();
        println!("ALIVE1");

        Ok(MpptFrame {
            pid: convert_product_id(&hm, "PID")?,
            firmware: convert_string(&hm, "FW")?,
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
        // [raw.iter().cloned().collect(), vec![checksum]].concat()
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
        firmware: String,
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
    fn test_mppt_to_bytes() {
        let mppt = MpptFrame::default();
        let bytes: Vec<u8> = mppt.into();
        let frame_without_checksum = "\r\nPID\t0xA042\r\nFW\t150\r\nSER#\tHQ1328A1B2C\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nH19\t0\r\nH20\t0\r\nH21\t0\r\nH22\t0\r\nH23\t0\r\nHSDS\t0\r\nChecksum\t".as_bytes();
        assert_eq!(bytes.split_last().unwrap().1, frame_without_checksum);
        assert_eq!(bytes.split_last().unwrap().0, &68);
    }

    #[test]
    fn test_mppt_1() {
        let frame = "\r\nPID\t0xA042\r\nFW\t150\r\nSER#\tHQ1328A1B2C\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nH19\t0\r\nH20\t0\r\nH21\t0\r\nH22\t0\r\nH23\t0\r\nHSDS\t0\r\nChecksum\t".as_bytes();
        let frame = &checksum::append(frame, 68);
        let (fields, checksum, _remainder) = crate::parser::parse(&frame).unwrap();
        let device = MpptFrame::map_fields(&fields, checksum).unwrap();

        assert_eq!(device.pid, VictronProductId::BlueSolar_MPPT_75_15);
        assert_eq!(device.firmware, String::from("150"));
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
        assert_eq!(device.firmware, String::from("150"));
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
        assert_eq!(device.firmware, String::from("150"));
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
            "420".into(),
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
        assert_eq!(device.firmware, String::from("420"));
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
