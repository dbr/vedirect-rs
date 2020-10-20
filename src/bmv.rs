use crate::map::Map;
use crate::parser::Field;
use crate::types::*;
use crate::utils::*;
use crate::ve_error::VeError;
use std::collections::hash_map::HashMap;

// TODO: add bmv600
/// Data for BMV 600 battery monitor series
// struct Bmv600 {}

/// Data for BMV 700 battery monitor series
#[derive(Debug)]
pub struct Bmv700 {
    /// Main (channel 1) battery voltage. Labelled `V`
    /// Units: V
    /// Available on: BMV 600, BMV 700, MPPT, Inverter
    pub voltage: Volt,

    /// Instantaneous power. Labelled `P`
    /// Units: W
    /// Available on: BMV 700
    pub power: Watt,

    /// Consumed Amp Hours. Labelled `CE`
    /// Units: mAh (When the BMV is not synchronised, these statistics have no meaning, so "---" will be sent instead of a value)
    pub consumed: Option<String>,

    /// State of charge. Labelled `SOC`
    /// Unit: Percent (When the BMV is not synchronised, these statistics have no meaning, so "---" will be sent instead of a value)
    /// Available on: BMV 600, BMV 700
    pub soc: Option<Percent>,

    /// Time-to-go. Labelled `TTG`
    /// Units: Minutes (When the battery is not discharging the time-to-go is infinite. This is represented as -1)
    /// Available on: BMV 600, BMV 700
    pub ttg: Minute,
}

impl Map<Bmv700> for Bmv700 {
    /// "When the BMV is not synchronised, these statistics have no meaning, so "---" will be sent instead of a value"
    /// Take a list of fields and creates an easier to use structure
    fn map_fields(fields: &Vec<Field>, _checksum: u8) -> Result<Self, VeError> {
        // Convert from list into map
        let mut hm: HashMap<&str, &str> = HashMap::new();
        for f in fields {
            hm.insert(f.key, f.value);
        }

        Ok(Bmv700 {
            voltage: convert_f32(&hm, "V")? / 1000_f32,
            power: convert_watt(&hm, "P")?,
            consumed: Some(convert_string(&hm, "CE")?),
            soc: convert_percentage(&hm, "SOC")?,
            ttg: convert_ttg(&hm, "TTG")?,
        })
    }
}

#[cfg(test)]
mod tests_mppt {
    use super::*;
    use crate::checksum;

    #[test]
    fn test_mapping() {
        let frame = checksum::append("\r\nP\t123\r\nCE\t53\r\nSOC\t452\r\nTTG\t60\r\nRelay\tOFF\r\nAlarm\tOFF\r\nV\t232\r\nChecksum\t".as_bytes(), 149);
        let (raw, checksum, _remainder) = crate::parser::parse(&frame).unwrap();
        let data = Bmv700::map_fields(&raw, checksum).unwrap();
        assert_eq!(data.power, 123);
        assert_eq!(data.consumed, Some("53".into()));
        assert_eq!(data.soc, Some(45.2));
        assert_eq!(data.ttg, 60);
        assert_eq!(data.voltage, 0.232);
    }
}
