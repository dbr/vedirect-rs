use num_derive::FromPrimitive;
use std::fmt;
use std::fmt::Display;
// use num_traits::FromPrimitive;

#[derive(Debug, FromPrimitive, Copy, Clone, PartialEq)]
pub enum AlarmReason {
    LowVoltage = 1,
    HighVoltage = 2,
    LowSOC = 4,
    LowStarterVoltage = 8,
    HighStarterVoltage = 16,
    LowTemperature = 32,
    HighTemperature = 64,
    MidVoltage = 128,
    Overload = 256,
    DCRipple = 512,
    LowVACout = 1024,
    HighVACout = 2048,
}

#[allow(non_camel_case_types)]
#[derive(Debug, FromPrimitive, Copy, Clone, PartialEq)]
pub enum VictronProductId {
    BMV700 = 0x203,
    BMV702 = 0x204,
    BMV700H = 0x205,

    BlueSolar_MPPT_75_10 = 0xA04C,
    BlueSolar_MPPT_150_100 = 0xA047,
    BlueSolar_MPPT_70_15 = 0x300,
    BlueSolar_MPPT_75_15 = 0xA042,
    BlueSolar_MPPT_100_15 = 0xA043,
    BlueSolar_MPPT_100_30_rev1 = 0xA044,
    BlueSolar_MPPT_100_30_rev2 = 0xA04A,
    BlueSolar_MPPT_150_35_rev1 = 0xA041,
    BlueSolar_MPPT_150_35_rev2 = 0xA04B,
    BlueSolar_MPPT_150_45 = 0xA04D,
    BlueSolar_MPPT_150_60 = 0xA04E,
    BlueSolar_MPPT_150_70 = 0xA046,
    BlueSolar_MPPT_150_85 = 0xA04F,
    BlueSolar_MPPT_75_50 = 0xA040,
    BlueSolar_MPPT_100_50_rev1 = 0xA045,
    BlueSolar_MPPT_100_50_rev2 = 0xA049,

    SmartSolar_MPPT_150_100 = 0xA051,
    SmartSolar_MPPT_250_100 = 0xA050,

    Phoenix_Inverter_12V_250VA_230V = 0xA201,
    Phoenix_Inverter_24V_250VA_230V = 0xA202,
    Phoenix_Inverter_48V_250VA_230V = 0xA204,
    Phoenix_Inverter_12V_375VA_230V = 0xA211,
    Phoenix_Inverter_24V_375VA_230V = 0xA212,
    Phoenix_Inverter_48V_375VA_230V = 0xA214,
    Phoenix_Inverter_12V_500VA_230V = 0xA221,
    Phoenix_Inverter_24V_500VA_230V = 0xA222,
    Phoenix_Inverter_48V_500VA_230V = 0xA224,
}

#[allow(non_camel_case_types)]
#[derive(Debug, FromPrimitive, Copy, Clone, PartialEq)]
pub enum DeviceMode {
    VE_REG_MODE_INVERTER = 2,
    VE_REG_MODE_OFF = 4,
    VE_REG_MODE_ECO = 5,
}

#[derive(Debug, FromPrimitive, Copy, Clone, PartialEq)]
pub enum ChargeState {
    Off = 0,
    LowPower = 1,
    Fault = 2,
    Bulk = 3,
    Absorption = 4,
    Float = 5,
    Inverting = 9,
}

impl Display for ChargeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, FromPrimitive, Copy, Clone, PartialEq)]
pub enum Err {
    NoError = 0,
    BatteryVoltageTooHigh = 2,
    ChargerTemperatureTooHigh = 17,
    ChargerOverCurrent = 18,
    ChargerCurrentReversed = 19,
    BulkTimeLimitExceeded = 20,
    CurrentSensorIssue = 21,
    TerminalsOverheated = 26,
    SolarInputVoltageTooHigh = 33,
    SolarInputCurrentTooHigh = 34,
    InputShutdown = 38,
    FactoryCalibrationDataLost = 116,
    InvalidFirmware = 117,
    UserSettingsInvalid = 119,
}

impl Display for Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
