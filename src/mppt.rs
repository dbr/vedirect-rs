use crate::map::Map;
use crate::types::*;
use crate::parser::Field;
use std::collections::hash_map::HashMap;
use crate::ve_error::VeError;
use crate::utils::*;

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

    pub voltage: Volt,

    // pub load: bool,
}

impl Map<Mppt75_15> for Mppt75_15 {
    fn map_fields(fields: &Vec<Field>) -> Result<Self, VeError> {
        let mut hm: HashMap<&str, &str> = HashMap::new();
        for f in fields {
            hm.insert(f.key, f.value);
        }

        Ok(Mppt75_15 {
            pid: convert_string(&hm, "PID")?,
            voltage: convert_volt(&hm, "V")?/100f32,
            // power: convert_watt(&hm, "P")?,
            // consumed: Some(convert_string(&hm, "CE")?),
            // soc: convert_percentage(&hm, "SOC")?,
            // ttg: convert_ttg(&hm, "TTG")?,
        })
    }
}

#[cfg(test)]
mod tests_mppt {
    use super::*;

    #[test]
    fn test_mppt() {
        // let sample_frame = "\r\nPID\t0xA053\r\nFW\t150\r\nSER#\tHQ9999ABCDE\r\nV\t12000\r\nI\t0\r\nVPV\t10\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nLOAD\tOFF\r\nIL\t0\r\nH19\t10206\r\nH20\t0\r\nH21\t0\r\nH22\t2\r\nH23\t8\r\nHSDS\t279\r\nChecksum\t12".as_bytes();
        let sample_frame = "\r\nPID\t0xA053\r\nV\t12000\r\nChecksum\t12".as_bytes();
        let (raw, _remainder) = crate::parser::parse(sample_frame).unwrap();

        let data = Mppt75_15::map_fields(&raw).unwrap();
        assert_eq!(data.pid, String::from("0xA053"));
        assert_eq!(data.voltage, 12.0);
    }
}
