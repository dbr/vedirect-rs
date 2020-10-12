use crate::types::*;

// PID     0xA053
// FW      150
// SER#    HQ1835CBDRQ
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
pub struct MPPT {
    pub pid: String,

    pub voltage: Volt,

    pub load: bool,
}

#[cfg(test)]
mod tests_mppt {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_mppt() {
        let sample_frame = "PID\t0xA053\r\nFW\t150\r\nSER#\tHQ1835CBDRQ\r\nV\t12000\r\nI\t0\r\nVPV\t10\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nLOAD\tOFF\r\nIL\t0\r\nH19\t10206\r\nH20\t0\r\nH21\t0\r\nH22\t2\r\nH23\t8\r\nHSDS\t279\r\nChecksum\t12".as_bytes();
        let (_raw, _remainder) = crate::parser::parse(sample_frame).unwrap();

        // let data = map_fields_Bmv700(&raw).unwrap();
        // assert_eq!(data.power, 123);
        // assert_eq!(data.consumed, Some("53".into()));
        // assert_eq!(data.soc, Some(45.2));
        // assert_eq!(data.ttg, 60);
        // assert_eq!(data.voltage, 23.2);
    }
}
