use crate::types::*;
use crate::ve_error::VeError;
use std::collections::hash_map::HashMap;

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

pub fn convert_volt(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<Volt, VeError> {
    let raw = (*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;
    let cleaned = raw.parse::<Volt>()? / 10.0;
    Ok(cleaned)
}

pub fn convert_watt(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<Watt, VeError> {
    let raw = (*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;
    let cleaned = raw.parse::<Watt>()?;
    Ok(cleaned)
}

pub fn convert_string(rawkeys: &HashMap<&str, &str>, label: &str) -> Result<String, VeError> {
    let raw = *(*rawkeys)
        .get(label)
        .ok_or(VeError::MissingField(label.into()))?;
    Ok(raw.into())
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
}
