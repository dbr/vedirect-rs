use crate::constants::*;
use crate::types::*;
use crate::ve_error::VeError;
use num_traits::FromPrimitive;
use std::{
    collections::hash_map::HashMap,
    fmt::{Debug, Display},
};

// TODO: all those convert functions should be one or more impl of a struct holding the rawkeys

// TODO: Lots of duplicate code here, we probably can do better. See function below.
// pub fn convert_number<T: FromStr + Debug>(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<T, VeError> {
//     let raw = (*rawkeys)
//         .get(label)
//         .ok_or(VeError::MissingField(label.into()))?;
//     let cleaned = raw.parse::<T>()?;
//     Ok(cleaned)
// }

/// "When the BMV is not synchronised, these statistics have no meaning, so "---" will be sent instead of a value"
pub fn convert_percentage(
    rawkeys: &HashMap<&str, &str>,
    label: &str,
) -> Result<Option<Percent>, VeError> {
    let raw = *(*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;

    if raw == "---" {
        Ok(None)
    } else {
        Ok(Some(raw.parse::<Percent>()? / 10.0))
    }
}

pub fn convert_f32(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<f32, VeError> {
    let raw = (*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;
    let cleaned = raw.parse::<f32>()?;
    Ok(cleaned)
}

pub fn convert_u32(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<u32, VeError> {
    let raw = (*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;
    let cleaned = raw.parse::<u32>()?;
    Ok(cleaned)
}

/// This function simplifies (=remove) all the \r\n and \t we have all over the place.
/// It also helps with the generation of frames where some values (such as the load) are optionnal.
pub fn get_field_string<T: Display>(label: &str, value: Option<T>) -> String {
    match value {
        Some(v) => format!("\r\n{}\t{}", label, v),
        None => String::from(""),
    }
}

/// This function converts the LOAD field. This field is special as it *may* not be present.
/// In that case, this is not an error but probably either a model with no load or an older firmware.
pub fn convert_load(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<Option<bool>, VeError> {
    let raw = (*rawkeys).get(label);
    match raw {
        Some(field) => Ok(Some(String::from(*field).contains("ON"))),
        None => Ok(None),
    }
}

/// This function converts the IL field (Load Current). This field is special as it *may* not be present.
/// In that case, this is not an error but probably either a model with no load or an older firmware.
pub fn convert_load_current(
    rawkeys: &HashMap<&str, &str>,
    label: &str,
) -> Result<Option<Current>, VeError> {
    let raw = (*rawkeys).get(label);
    match raw {
        Some(field) => Ok(Some(field.parse::<Current>()? / 1000_f32)),
        None => Ok(None),
    }
}

/// This function tries itS best to convert:
/// - "0" or "7", "15" in to 0, 7 or 15
/// - "0x00000001" into 1 and "0x00000100" into 256
/// and then match with an element of the enum T
pub fn convert_enum<T>(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<T, VeError>
where
    T: FromPrimitive + Debug,
{
    let raw = (*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;

    let cleaned = match raw.contains("0x") {
        true => {
            let raw = &raw.replace("0x", "");
            u32::from_str_radix(raw, 16)
        }
        false => raw.parse::<u32>(),
    };

    let index = cleaned.unwrap();
    let hit = FromPrimitive::from_u32(index);

    match hit {
        Some(x) => Ok(x),
        None => Err(VeError::Parse(format!(
            "Error parsing integer into T: {}",
            raw
        ))),
    }
}

pub fn convert_watt(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<Watt, VeError> {
    let raw = (*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;
    let cleaned = raw.parse::<Watt>()?;
    Ok(cleaned)
}

pub fn convert_yield(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<kWh, VeError> {
    let raw = (*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;
    let cleaned = raw.parse::<kWh>()? / 100_f32;
    Ok(cleaned)
}

pub fn convert_string(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<String, VeError> {
    let raw = *(*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;
    Ok(raw.into())
}

/// From a produc ID in hex such as 0xA042, returns the VictronProductId
pub fn convert_product_id(
    rawkeys: &HashMap<&str, &str>,
    label: &str,
) -> Result<VictronProductId, VeError> {
    let raw = *(*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;
    let id = u32::from_str_radix(raw.trim_start_matches("0x"), 16)?;
    let pid = FromPrimitive::from_u32(id);
    match pid {
        Some(x) => Ok(x),
        None => Err(VeError::Parse(format!(
            "Error parsing integer into VictronProductId: {}",
            raw
        ))),
    }
}

pub fn convert_ttg(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<Minute, VeError> {
    let raw = *(*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;
    let cleaned = raw.parse::<Minute>()?;
    Ok(cleaned)
}

#[cfg(test)]
mod tests_utils {
    use super::*;
    macro_rules! seq {
        ($($x:expr),+) => {
            [$($x,)+].iter().map(|&x| x).collect()
        }
    }

    #[test]
    fn test_convert_percentage() {
        let hm: HashMap<&str, &str> = seq!(("P", "031"));
        assert_eq!(convert_percentage(&hm, "P").unwrap(), Some(3.1_f32));
    }

    #[test]
    fn test_convert_percentage2() {
        let hm: HashMap<&str, &str> = seq!(("P", "---"));
        assert_eq!(convert_percentage(&hm, "P").unwrap(), None);
    }

    #[test]
    fn test_get_field_string_some() {
        assert_eq!(get_field_string("ABC", Some(3.14)), "\r\nABC\t3.14");
    }

    #[test]
    fn test_get_field_string_none() {
        assert_eq!(get_field_string::<u32>("ABC", None), "");
    }

    #[test]
    fn test_get_or_field_0x00000001() {
        assert_eq!(
            convert_enum::<OffReason>(&seq!(("OR", "0x00000001")), "OR").unwrap(),
            OffReason::NoInputPower
        );
    }

    #[test]
    fn test_get_or_field_0x00000100() {
        assert_eq!(
            convert_enum::<OffReason>(&seq!(("OR", "0x00000100")), "OR").unwrap(),
            OffReason::AnalysingInputVoltage
        );
    }

    #[test]
    fn test_get_mppt_field_1() {
        assert_eq!(
            convert_enum::<MpptOperationState>(&seq!(("MPPT", "1")), "MPPT").unwrap(),
            MpptOperationState::VoltageOrCurrentLimited
        );
    }
}
