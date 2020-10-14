use crate::map::Map;
use crate::parser::Field;
use crate::types::*;
use crate::utils::*;
use crate::ve_error::VeError;
use std::collections::hash_map::HashMap;
use crate::constants::*;

// PID     0xA053
// FW      150
// SER#    HQ9999ABCDE
// V       12000
// I       0
// VPV     10
// PPV     0
// CS      0
// MPPT    0
// OR      0x00000001
// ERR     0
// LOAD    OFF
// IL      0
// H19     10206
// H20     0
// H21     0
// H22     2
// H23     8
// HSDS    279
// Checksum        �

/// Data for all MPPT solar charge controller
#[derive(Debug)]
pub struct Mppt75_15 {
    /// Label: PID, Product ID
    pub pid: String,

    /// Label: FW, Firmware version
    pub firmware: String, // TODO: check if that could be a semver => yes it is. 150 = v1.50

    /// Label: SER#, Serial Number
    /// The serial number of the device. The notation is LLYYMMSSSSS, where LL=location code,
    /// YYWW=production datestamp (year, week) and SSSSS=unique part of the serial number.
    /// Example: HQ1328Y6TF6
    pub serial_number: String,
    
    /// Label: V, Unit: mV, Main (battery) voltage
    pub voltage: Volt,
    
    /// Label: I, Unit: mA, Battery current converted to A
    pub current: Current,
    
    /// Label: VPV, Unit: mV, Panel voltage, converted to V.
    pub vpv: Volt, 

    /// Label: PPV, Unit: W, Panel Power
    pub ppv: Watt,

    /// Label: CS, State of Operation
    pub charge_state: ChargeState,

    // TODO: check what MPPT is
    /// Label: MPPT, Unsure what this is so catching it as String for now
    pub mppt: String,

    // TODO: check what OR is
    /// Label: OR, Unsure what this is so catching it as String for now
    pub or: String,

    /// Label: ERR, Error code    
    pub error: Err,

    // TODO: only available for models with load output and since Firmware v1.12 so we should probably make that an Option<bool> or manage versionning
    /// Label: LOAD, Whether the load is turned ON(true) or OFF(false)
    pub load: bool,
    
    // TODO: only available for models with load output and since Firmware v1.15 so we should probably make that an Option<bool> or manage versionning
    /// Label: IL, Unit: mA, Load current, converted to A
    pub load_current: Current,

    /// Label: H19, Yield total (user resettable counter) in 0.01 kWh converted to kWh
    pub yield_total: kWh,

    /// Label: H20, Yield today in 0.01 kWh converted to kWh
    pub yield_today: kWh,

    /// Label: H21, Maximum power today
    pub max_power_today: Watt,

    /// Label: H22, Yield today in 0.01 kWh converted to kWh
    pub yield_yesterday: kWh,

    /// Label: H23, Maximum power today
    pub max_power_yesterday: Watt,

    /// Label: HSDS
    /// Historical data. The day sequence number, a change in this number indicates a new day. This
    /// implies that the historical data has changed. Range 0..364.
    /// Note: The HSDS field is available in the MPPT charger since version v1.16.
    pub hsds: u16,

    /// label: Checksum, the checksum
    pub checksum: u8,
}

impl ToString for Mppt75_15 {
    fn to_string(&self) -> String {
        format!("\r\nPID\t{}\r\nFW\t{}\r\nSER#\t{}\r\nV\t{}\r\nI\t{}\r\nVPV\t{}\r\nPPV\t{}\r\nCS\t{}\r\nMPPT\t{}\r\nOR\t{}\r\nERR\t{}\r\nLOAD\t{}\r\nIL\t{}\r\nH19\t{}\r\nH20\t{}\r\nH21\t{}\r\nH22\t{}\r\nH23\t{}\r\nHSDS\t{}\r\nChecksum\t{}", 
        self.pid, 
        self.firmware,
        self.serial_number,
        self.voltage,
        self.current,
        self.vpv,
        self.ppv,
        self.charge_state as u32,
        self.mppt,
        self.or,
        self.error as u32,
        if self.load { "ON" } else { "OFF" },
        self.load_current,
        self.yield_total,
        self.yield_today,
        self.max_power_today,
        self.yield_yesterday,
        self.max_power_yesterday,
        self.hsds,
        self.checksum,
        ) 
    }
}

impl Default for Mppt75_15 {
    fn default() -> Self {
        Self {
            pid: "0x0000".into(),
            firmware: "150".into(),
            serial_number: "HQ1328Y6TF6".into(),
            voltage: 0.0,
            current: 0.0,
            vpv: 0.0,
            ppv: 0,
            charge_state: ChargeState::Off,
            mppt: "0".into(),
            or: "0x00000001".into(),
            load_current: 0.0,

            yield_total: 0,
            yield_today: 0,
            max_power_today: 0,
            yield_yesterday: 0,
            max_power_yesterday: 0,

            error: Err::NoError,
            load: false,

            hsds: 0,
            checksum: 0,
        }
    }
}

impl Map<Mppt75_15> for Mppt75_15 {
    fn map_fields(fields: &Vec<Field>) -> Result<Self, VeError> {
        let mut hm: HashMap<&str, &str> = HashMap::new();
        for f in fields {
            hm.insert(f.key, f.value);
        }

        Ok(Mppt75_15 {
            pid: convert_string(&hm, "PID")?,
            firmware: convert_string(&hm, "FW")?,
            serial_number: convert_string(&hm, "SER#")?,
            voltage: convert_volt(&hm, "V")? / 100f32,
            current: convert_volt(&hm, "I")? / 100f32,
            load_current : convert_volt(&hm, "IL")?, // TODO: fix that
            vpv: convert_volt(&hm, "VPV")? / 100f32,
            ppv: convert_watt(&hm, "PPV")?,
            charge_state: convert_charge_state(&hm, "CS")?,
            mppt: convert_string(&hm, "MPPT")?,
            or: convert_string(&hm, "OR")?,
            error: convert_err(&hm, "ERR")?,
            load: convert_bool(&hm, "LOAD")?,
            yield_total: convert_yield(&hm, "H19")?,
            yield_today: convert_yield(&hm, "H20")?,
            max_power_today: convert_watt(&hm, "H21")?,
            yield_yesterday: convert_yield(&hm, "H22")?,
            max_power_yesterday: convert_watt(&hm, "H23")?,
            hsds: convert_u32(&hm, "HSDS")? as u16,
            checksum: convert_u32(&hm, "HSDS")? as u8,
        })
    }
}

#[cfg(test)]
mod tests_mppt {
    use super::*;

    #[test]
    fn test_mppt_to_string() {
        let mppt = Mppt75_15::default();
        let frame = mppt.to_string();
        let default_frame = "\r\nPID\t0x0000\r\nFW\t150\r\nSER#\tHQ1328Y6TF6\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nLOAD\tOFF\r\nIL\t0\r\nH19\t0\r\nH20\t0\r\nH21\t0\r\nH22\t0\r\nH23\t0\r\nHSDS\t0\r\nChecksum\t0";
        assert_eq!(frame, default_frame);
    }

    #[test]
    fn test_mppt_1() {
        let mppt = Mppt75_15::default();
        let frame = mppt.to_string();
        let sample_frame = frame.as_bytes();
        let (raw, _remainder) = crate::parser::parse(sample_frame).unwrap();
        let data = Mppt75_15::map_fields(&raw).unwrap();

        assert_eq!(data.pid, String::from("0x0000"));
        assert_eq!(data.firmware, String::from("150"));
        assert_eq!(data.serial_number, "HQ1328Y6TF6");
        assert_eq!(data.voltage, 0.0);
        assert_eq!(data.current, 0.0);
        assert_eq!(data.load_current, 0.0);
        assert_eq!(data.vpv, 0.0);
        assert_eq!(data.ppv, 0);
        assert_eq!(data.charge_state, ChargeState::Off);
        assert_eq!(data.mppt, "0");
        assert_eq!(data.or, "0x00000001");
        assert_eq!(data.error, Err::NoError);
        assert_eq!(data.load, false);
        assert_eq!(data.yield_total, 0);
        assert_eq!(data.yield_today, 0);
        assert_eq!(data.max_power_today, 0);
        assert_eq!(data.yield_yesterday, 0);
        assert_eq!(data.max_power_yesterday, 0);
        assert_eq!(data.hsds, 0);
        assert_eq!(data.checksum, 0);
    }

    #[test]
    fn test_mppt_older_versions() {
        let mppt = Mppt75_15::default();
        let frame = mppt.to_string();
        let sample_frame = frame.as_bytes();
        let (raw, _remainder) = crate::parser::parse(sample_frame).unwrap();
        let data = Mppt75_15::map_fields(&raw).unwrap();

        let default_frame = "\r\nPID\t0x0000\r\nFW\t150\r\nSER#\tHQ1328Y6TF6\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nLOAD\tOFF\r\nIL\t0\r\nH19\t0\r\nH20\t0\r\nH21\t0\r\nH22\t0\r\nH23\t0\r\nHSDS\t0\r\nChecksum\t0";

        
        assert_eq!(data.pid, String::from("0x0000"));
        assert_eq!(data.firmware, String::from("150"));
        assert_eq!(data.serial_number, "HQ1328Y6TF6");
        assert_eq!(data.voltage, 0.0);
        assert_eq!(data.current, 0.0);
        assert_eq!(data.load_current, 0.0);
        assert_eq!(data.vpv, 0.0);
        assert_eq!(data.ppv, 0);
        assert_eq!(data.charge_state, ChargeState::Off);
        assert_eq!(data.mppt, "0");
        assert_eq!(data.or, "0x00000001");
        assert_eq!(data.error, Err::NoError);
        assert_eq!(data.load, false);
        assert_eq!(data.yield_total, 0);
        assert_eq!(data.yield_today, 0);
        assert_eq!(data.max_power_today, 0);
        assert_eq!(data.yield_yesterday, 0);
        assert_eq!(data.max_power_yesterday, 0);
        assert_eq!(data.hsds, 0);
        assert_eq!(data.checksum, 0);
    }
}
