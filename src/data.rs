use std::collections::hash_map::HashMap;

use crate::VEError;

// Data types
type Watt = i32;
type Percent = f32;
type Volt = f32;
type Minute = i32;

// Type conversion errors
impl From<std::num::ParseIntError> for VEError {
    fn from(src: std::num::ParseIntError) -> VEError {
        VEError::Parse(format!("Error parsing integer: {}", src))
    }
}

impl From<std::num::ParseFloatError> for VEError {
    fn from(src: std::num::ParseFloatError) -> VEError {
        VEError::Parse(format!("Error parsing float: {}", src))
    }
}

/// Data for BMV 600 battery monitor series
// struct Bmv600 {}

/// Data for BMV 700 battery monitor series
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

/// Data for all MPPT solar charge controller
// struct MPPT {}

/// Data for Phoenix Inverters
// struct PhoenixInverter {}

/// Data for Phoenix Chargers
// struct PhoenixCharger {}

/// Data for all devices
// pub struct Everything {}

/// "When the BMV is not synchronised, these statistics have no meaning, so "---" will be sent instead of a value"
fn convert_percentage(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<Option<Percent>, VEError> {
    let raw = *(*rawkeys).get(label).ok_or(VEError::MissingField(label.into()))?;

    if raw == "---" {
        Ok(None)
    } else {
        Ok(Some(raw.parse::<Percent>()? / 10.0))
    }
}

fn convert_volt(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<Volt, VEError> {
    let raw = (*rawkeys).get(label).ok_or(VEError::MissingField(label.into()))?;
    let cleaned = raw.parse::<Volt>()? / 10.0;
    Ok(cleaned)
}

fn convert_watt(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<Watt, VEError> {
    let raw = (*rawkeys).get(label).ok_or(VEError::MissingField(label.into()))?;
    let cleaned = raw.parse::<Watt>()?;
    Ok(cleaned)
}

fn convert_string(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<String, VEError> {
    let raw = *(*rawkeys).get(label).ok_or(VEError::MissingField(label.into()))?;
    Ok(raw.into())
}

fn convert_ttg(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<Minute, VEError> {
    let raw = *(*rawkeys).get(label).ok_or(VEError::MissingField(label.into()))?;
    let cleaned = raw.parse::<Minute>()?;
    Ok(cleaned)
}

pub fn mapraw(fields: Vec<crate::parser::Field>) -> Result<Bmv700, VEError> {
    // Convert from list into map
    let mut hm: HashMap<&str, &str> = HashMap::new();
    for f in fields {
        hm.insert(f.key, f.value);
    }

    Ok(Bmv700 {
        voltage: convert_volt(&hm, "V")?,
        power: convert_watt(&hm, "P")?,
        consumed: Some(convert_string(&hm, "CE")?),
        soc: convert_percentage(&hm, "SOC")?,
        ttg: convert_ttg(&hm, "TTG")?,
    })
}

#[test]
fn test_mapping() {
    let raw = crate::parser::parse(
        "\r\nP\t123\r\nCE\t53\r\nSOC\t452\r\nTTG\t60\r\nRelay\tOFF\r\nAlarm\tOFF\r\nV\t232\r\nChecksum\t12".as_bytes(),
    )
    .unwrap();
    let data = mapraw(raw).unwrap();
    assert_eq!(data.power, 123);
    assert_eq!(data.consumed, Some("53".into()));
    assert_eq!(data.soc, Some(45.2));
    assert_eq!(data.ttg, 60);
    assert_eq!(data.voltage, 23.2);
}
