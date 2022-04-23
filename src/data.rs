use std::{collections::hash_map::HashMap, str::from_utf8};

use strum_macros::FromRepr;

use crate::VEError;

// Data types
type Watt = i32;
type Percent = f32;
type Volt = f32;
type Ampere = f32;
type Minute = i32;
type KiloWattHours = i32;

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

#[derive(FromRepr, PartialEq, Eq, Debug)]
pub enum OffReason {
    None = 0x0,
    NoInputPower = 0x00000001,
    SwitchedOffPowerSwitch = 0x00000002,
    SwitchedOffDMR = 0x00000004,
    RemoteInput = 0x00000008,
    ProtectionActive = 0x000000010,
    Paygo = 0x00000020,
    BMS = 0x000000040,
    EngineShutdownDetection = 0x00000080,
    AnalysingInputVoltage = 0x000000100,
}

#[derive(FromRepr, PartialEq, Eq, Debug)]
pub enum TrackerOperationMode {
    Off = 0,
    VoltageOrCurrentLimited = 1,
    MPPTrackerActive = 2,
}

#[derive(FromRepr, PartialEq, Eq, Debug)]
pub enum ErrorCode {
    NoError = 0,
    BatteryVoltageTooHigh = 2,
    ChargerTemperatureTooHigh = 17,
    ChargerOverCurrent = 18,
    ChargerCurrentReversed = 19,
    BulkTimeLimitExceeded = 20,
    CurrentSensorIssue = 21,
    TerminalsOverheatd = 26,
    ConverterIssue = 28,
    InputVoltageTooHigh = 33,
    InputCurrentTooHigh = 34,
    InputShutdownBatVoltage = 38,
    InputShutdownCurrentFlow = 39,
    LostComWithDevices = 65,
    SynchronisedChargingIssue = 66,
    BMSConnectionLost = 67,
    NetworkMisconfigured = 68,
    FactoryCalibrationDataLost = 116,
    InvalidFirmware = 117,
    UserSettingsInvalid = 119,
}

#[derive(FromRepr, PartialEq, Eq, Debug)]
pub enum StateOfOperation {
    Off = 0,
    LowPower = 1,
    Fault = 2,
    Bulk = 3,
    Absorption = 4,
    Float = 5,
    Storage = 6,
    Equalize = 7,
    Inverting = 9,
    PowerSupply = 11,
    StartingUp = 245,
    RepeatedAbsorption = 246,
    AutoEqualize = 247,
    BatterySafe = 248,
    ExternalControl = 252,
}

pub trait VEDirectData {
    fn fill(fields: &HashMap<String, Vec<u8>>) -> Result<Self, VEError>
    where
        Self: Sized;
}

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

impl VEDirectData for Bmv700 {
    fn fill(fields: &HashMap<String, Vec<u8>>) -> Result<Self, VEError> {
        Ok(Bmv700 {
            voltage: convert_volt(fields, "V", 10.0)?,
            power: convert_watt(fields, "P")?,
            consumed: Some(convert_string(fields, "CE")?),
            soc: convert_percentage(fields, "SOC")?,
            ttg: convert_ttg(fields, "TTG")?,
        })
    }
}

/// Data for all MPPT solar charge controller
#[derive(Debug)]
pub struct MPPT {
    pub channel1_voltage: Volt,
    pub panel_voltage: Volt,
    pub panel_power: Watt,
    pub battery_current: Ampere,
    pub load_current: Ampere,
    pub load_output_state: bool,
    pub relay_state: Option<bool>,
    pub off_reason: OffReason,
    pub yield_total: KiloWattHours,
    pub yield_today: KiloWattHours,
    pub max_power_today: Watt,
    pub yield_yesterday: KiloWattHours,
    pub max_power_yesterday: Watt,
    pub error_code: ErrorCode,
    pub state_of_operation: StateOfOperation,
    pub firmware: u16,
    pub product_id: String,
    pub serial_number: String,
    pub day_sequence: u16,
    pub tracker_mode: TrackerOperationMode,
}

impl VEDirectData for MPPT {
    fn fill(fields: &HashMap<String, Vec<u8>>) -> Result<Self, VEError> {
        Ok(MPPT {
            channel1_voltage: convert_volt(fields, "V", 1000.0)?,
            panel_voltage: convert_volt(fields, "VPV", 1000.0)?,
            panel_power: convert_watt(fields, "PPV")?,
            battery_current: convert_ampere(fields, "I", 1000.0)?,
            load_current: convert_ampere(fields, "IL", 1000.0)?,
            load_output_state: convert_bool(fields, "LOAD")?,
            relay_state: if fields.contains_key("Relay") {
                Some(convert_bool(fields, "Relay")?)
            } else {
                None
            },
            off_reason: convert_off_reason(fields, "OR")?,
            yield_total: convert_watt(fields, "H19")?,
            yield_today: convert_watt(fields, "H20")?,
            max_power_today: convert_watt(fields, "H21")?,
            yield_yesterday: convert_watt(fields, "H22")?,
            max_power_yesterday: convert_watt(fields, "H23")?,
            error_code: convert_error_code(fields, "ERR")?,
            state_of_operation: convert_state_of_operation(fields, "CS")?,
            firmware: convert_u16(fields, "FW")?,
            product_id: convert_string(fields, "PID")?,
            serial_number: convert_string(fields, "SER#")?,
            day_sequence: convert_u16(fields, "HSDS")?,
            tracker_mode: convert_tracker_mode(fields, "MPPT")?,
        })
    }
}

/// Data for Phoenix Inverters
// struct PhoenixInverter {}

/// Data for Phoenix Chargers
// struct PhoenixCharger {}

/// Data for all devices
// pub struct Everything {}

/// "When the BMV is not synchronised, these statistics have no meaning, so "---" will be sent instead of a value"
fn convert_percentage(
    rawkeys: &HashMap<String, Vec<u8>>,
    label: &str,
) -> Result<Option<Percent>, VEError> {
    let raw = rawkeys
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;

    let s = from_utf8(&raw)
    .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?
    ;
    if s == "---" {
        Ok(None)
    } else {
        Ok(Some(s.parse::<Percent>()? / 10.0))
    }
}

fn convert_volt(
    rawkeys: &HashMap<String, Vec<u8>>,
    label: &str,
    factor: f32,
) -> Result<Volt, VEError> {
    let raw = rawkeys
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;
    let cleaned = from_utf8(raw)
    .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?
    .parse::<Volt>()? / factor;
    Ok(cleaned)
}

fn convert_ampere(
    rawkeys: &HashMap<String, Vec<u8>>,
    label: &str,
    factor: f32,
) -> Result<Ampere, VEError> {
    let raw = (*rawkeys)
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;
    let cleaned = from_utf8(raw)
    .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?
    .parse::<Ampere>()? / factor;
    Ok(cleaned)
}

fn convert_watt(rawkeys: &HashMap<String, Vec<u8>>, label: &str) -> Result<Watt, VEError> {
    let raw = (*rawkeys)
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;
    let cleaned = from_utf8(raw)
    .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?
    .parse::<Watt>()?;
    Ok(cleaned)
}

fn convert_u16(rawkeys: &HashMap<String, Vec<u8>>, label: &str) -> Result<u16, VEError> {
    let raw = (*rawkeys)
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;
    let cleaned = from_utf8(raw)
        .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?
        .parse::<u16>()?;
    Ok(cleaned)
}

fn convert_string(rawkeys: &HashMap<String, Vec<u8>>, label: &str) -> Result<String, VEError> {
    let raw = &*rawkeys
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;
    Ok(String::from_utf8(raw.clone())
        .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?
    )
}

fn convert_bool(rawkeys: &HashMap<String, Vec<u8>>, label: &str) -> Result<bool, VEError> {
    let raw = &*rawkeys
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;
    let s = from_utf8(&raw)
    .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?
    ;
    if s == "ON" {
        Ok(true)
    } else if s == "OFF" {
        Ok(false)
    } else {
        Err(VEError::OnOffExpected(String::from(s)))
    }
}

fn convert_ttg(rawkeys: &HashMap<String, Vec<u8>>, label: &str) -> Result<Minute, VEError> {
    let raw = rawkeys
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;
    let cleaned = from_utf8(&raw)
        .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?
        .parse::<Minute>()?;
    Ok(cleaned)
}

fn convert_error_code(
    rawkeys: &HashMap<String, Vec<u8>>,
    label: &str,
) -> Result<ErrorCode, VEError> {
    let raw = rawkeys
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;
    let cleaned = from_utf8(&raw)
        .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?
        .parse::<usize>()?;
    Ok(ErrorCode::from_repr(cleaned).unwrap_or(ErrorCode::NoError))
}

fn convert_off_reason(
    rawkeys: &HashMap<String, Vec<u8>>,
    label: &str,
) -> Result<OffReason, VEError> {
    let raw = rawkeys
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;
    let cleaned = from_utf8(&raw)
        .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?;
    match cleaned {
        "0x00000000" => Ok(OffReason::None),
        "0x00000001" => Ok(OffReason::NoInputPower),
        "0x00000002" => Ok(OffReason::SwitchedOffPowerSwitch),
        "0x00000004" => Ok(OffReason::SwitchedOffDMR),
        "0x00000008" => Ok(OffReason::RemoteInput),
        "0x00000010" => Ok(OffReason::ProtectionActive),
        "0x00000020" => Ok(OffReason::Paygo),
        "0x00000040" => Ok(OffReason::BMS),
        "0x00000080" => Ok(OffReason::EngineShutdownDetection),
        "0x00000100" => Ok(OffReason::AnalysingInputVoltage),
        _ => Err(VEError::UnknownCode(cleaned.to_string())),
    }
}

fn convert_state_of_operation(
    rawkeys: &HashMap<String, Vec<u8>>,
    label: &str,
) -> Result<StateOfOperation, VEError> {
    let raw = rawkeys
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;
    let cleaned = from_utf8(&raw)
        .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?
        .parse::<usize>()?;
    Ok(StateOfOperation::from_repr(cleaned).unwrap_or(StateOfOperation::Off))
}

fn convert_tracker_mode(
    rawkeys: &HashMap<String, Vec<u8>>,
    label: &str,
) -> Result<TrackerOperationMode, VEError> {
    let raw = rawkeys
        .get(label)
        .ok_or(VEError::MissingField(label.into()))?;
    let cleaned = from_utf8(&raw)
        .map_err(|e| VEError::Parse(format!("Failed to parse {} from {:?} - {}", label, &raw, e)))?
        .parse::<usize>()?;
    Ok(TrackerOperationMode::from_repr(cleaned).unwrap_or(TrackerOperationMode::Off))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Events;

    struct CheckerBmv700;

    impl Events<Bmv700> for CheckerBmv700 {
        fn on_complete_block(&mut self, data: Bmv700) {
            assert_eq!(data.power, 123);
            assert_eq!(data.consumed, Some("53".into()));
            assert_eq!(data.soc, Some(45.2));
            assert_eq!(data.ttg, 60);
            assert_eq!(data.voltage, 23.2);
        }

        fn on_parse_error(&mut self, _error: VEError, _parse_buf: &Vec<u8>) {
            assert!(false);
        }
    }

    #[test]
    fn test_mapping() {
        let input = "\r\nP\t123\r\nCE\t53\r\nSOC\t452\r\nTTG\t60\r\nRelay\tOFF\r\nAlarm\tOFF\r\nV\t232\r\nChecksum\t12";
        let mut checker = CheckerBmv700 {};
        let mut parser = crate::Parser::new(&mut checker);
        parser.feed(input.as_bytes()).unwrap();
    }

    struct CheckerMPPT;

    impl Events<MPPT> for CheckerMPPT {
        fn on_complete_block(&mut self, data: MPPT) {
            assert_eq!(data.channel1_voltage, 12.54);
            assert_eq!(data.battery_current, 0.04);
            assert_eq!(data.panel_voltage, 18.54);
            assert_eq!(data.panel_power, 5);
            assert_eq!(data.load_current, 0.3);
            assert_eq!(data.load_output_state, true);
            assert_eq!(data.yield_total, 144);
            assert_eq!(data.yield_today, 1);
            assert_eq!(data.yield_yesterday, 4);
            assert_eq!(data.max_power_today, 6);
            assert_eq!(data.max_power_yesterday, 14);
            assert_eq!(data.day_sequence, 16);
            assert_eq!(data.firmware, 159);
            assert_eq!(data.tracker_mode, TrackerOperationMode::MPPTrackerActive);
        }

        fn on_parse_error(&mut self, _error: VEError, _parse_buf: &Vec<u8>) {
            assert!(false);
        }
    }
    #[test]
    fn test_mapping_mppt() {
        let input = "\r\nPID\t0xA053\r\nFW\t159\r\nSER#\tHQ2132QY2KR\r\nV\t12540\r\nI\t40\r\nVPV\t18540\r\nPPV\t5\r\nCS\t3\r\nMPPT\t2\r\nOR\t0x00000000\r\nERR\t0\r\nLOAD\tON\r\nIL\t300\r\nH19\t144\r\nH20\t1\r\nH21\t6\r\nH22\t4\r\nH23\t14\r\nHSDS\t16\r\nChecksum\t?";
        let mut checker = CheckerMPPT {};
        let mut parser = crate::Parser::new(&mut checker);
        parser.feed(input.as_bytes()).unwrap();
    }
}
