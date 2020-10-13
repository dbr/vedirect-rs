use crate::map::Map;
use crate::parser::Field;
use crate::types::*;
use crate::utils::*;
use crate::ve_error::VeError;
use std::collections::hash_map::HashMap;

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
pub struct Mppt75_15 {
    pub pid: String,
    pub firmware: String, // TODO: check if that could be a semver
    pub serial_number: String,
    pub voltage: Volt,
    pub current: Current,
    pub vpv: Volt,
    pub ppv: Watt,
    // cs
    // mppt
    // or
    pub errors: u32,
    pub load: bool,
    // il
    // H19..23
    // hsds
    // checksum
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

            vpv: convert_volt(&hm, "VPV")? / 100f32,
            ppv: convert_watt(&hm, "PPV")?,
            errors: convert_number(&hm, "ERR")?,
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
        let sample_frame = "\r\nPID\t0xA053\r\nFW\t150\r\nSER#\tHQ9999ABCDE\r\nV\t12530\r\nI\t01230\r\nVPV\t10\r\nPPV\t0\r\nERR\t0\r\nLOAD\tOFF\r\nChecksum\t12".as_bytes();

        // let sample_frame = "\r\nPID\t0xA053\r\nV\t12000\r\nLOAD\tOFF\r\nChecksum\t12".as_bytes();
        let (raw, _remainder) = crate::parser::parse(sample_frame).unwrap();

        let data = Mppt75_15::map_fields(&raw).unwrap();
        assert_eq!(data.pid, String::from("0xA053"));
        assert_eq!(data.voltage, 12.53);
        assert_eq!(data.current, 1.23);
        assert_eq!(data.load, false);
        assert_eq!(data.serial_number, "HQ9999ABCDE");
    }
}
