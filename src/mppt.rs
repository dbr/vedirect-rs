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
    pub firmware: String, // TODO: check if that could be a semver

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

    // Label: MPPT
    
    // or
    
    /// Error code    
    pub error: Err,

    /// Whether the load is turned ON(true) or OFF(false)
    pub load: bool,
    
    // Label: IL, Unit: mA, Load current, converted to A
    pub load_current: Current,

    // Label: H19, Yield total (user resettable counter)
    // pub yield_total



    // Label: HSDS
    // Historical data. The day sequence number, a change in this number indicates a new day. This
    // implies that the historical data has changed. Range 0..364.
    // Note: The HSDS field is available in the MPPT charger since version v1.16.
    // pub hsds

    // label: Checksum
}

impl ToString for Mppt75_15 {
    fn to_string(&self) -> String {
        format!("\r\nPID\t{}\r\nFW\t{}\r\nSER#\t{}\r\nV\t{}\r\nI\t{}\r\nVPV\t{}\r\nPPV\t{}\r\nCS\t{}\r\nERR\t{}\r\nLOAD\t{}\r\nChecksum\t{}", 
        self.pid, 
        self.firmware,
        self.serial_number,
        self.voltage,
        self.current,
        self.vpv,
        self.ppv,
        self.charge_state as u32,
        self.error as u32,
        if self.load { "ON" } else { "OFF" } , 
        42) // TODO: fix that
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
            load_current: 0.0,
            error: Err::NoError,
            load: false,
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
            error: convert_err(&hm, "ERR")?,
            load: convert_bool(&hm, "LOAD")?,
        })
    }
}

#[cfg(test)]
mod tests_mppt {
    use super::*;

    #[test]
    fn test_mppt_1() {
        // let sample_frame = "\r\nPID\t0xA053\r\nFW\t150\r\nSER#\tHQ9999ABCDE\r\nV\t12000\r\nI\t0\r\nVPV\t10\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nLOAD\tOFF\r\nIL\t0\r\nH19\t10206\r\nH20\t0\r\nH21\t0\r\nH22\t2\r\nH23\t8\r\nHSDS\t279\r\nChecksum\t12".as_bytes();
        // let sample_frame = "\r\nPID\t0xA053\r\nFW\t150\r\nV\t12000\r\nI\t0\r\nVPV\t10\r\nPPV\t0\r\nERR\t0\r\nLOAD\tOFF\r\nChecksum\t12".as_bytes();
        let sample_frame = "\r\nPID\t0xA053\r\nFW\t150\r\nSER#\tHQ1328Y6TF6\r\nV\t12340\r\nI\t01230\r\nVPV\t10\r\nPPV\t0\r\nCS\t0\r\nERR\t0\r\nLOAD\tOFF\r\nIL\t0\r\nChecksum\t42".as_bytes();

        // let sample_frame = "\r\nPID\t0xA053\r\nV\t12000\r\nLOAD\tOFF\r\nChecksum\t12".as_bytes();
        let (raw, _remainder) = crate::parser::parse(sample_frame).unwrap();

        let data = Mppt75_15::map_fields(&raw).unwrap();
        assert_eq!(data.pid, String::from("0xA053"));
        assert_eq!(data.voltage, 12.34);
        assert_eq!(data.current, 1.23);
        assert_eq!(data.load, false);
        assert_eq!(data.load_current, 0.0);
        assert_eq!(data.serial_number, "HQ1328Y6TF6");
    }

    #[test]
    fn test_mppt_to_string() {
        let mppt = Mppt75_15::default();

        let frame = mppt.to_string();
        // println!("{}", frame);
        let default_frame = "\r\nPID\t0x0000\r\nFW\t150\r\nSER#\tHQ1328Y6TF6\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nERR\t0\r\nLOAD\tOFF\r\nChecksum\t42";
        assert_eq!(frame, default_frame);
    }
}
