use std::{fmt, fmt::Display, str::FromStr};

/// The serial number of a device.
///
/// The notation is LLYYWWSSSSS, where LL=location code,
/// YYWW=production datestamp (year, week) and SSSSS=unique part of the serial number.
/// Example: HQ1328Y6TF6
#[derive(Debug, Clone, PartialEq)]
pub struct SerialNumber {
    pub location: String,
    pub year: u8,
    pub week: u8,
    pub id: String,
}

impl SerialNumber {
    pub fn new(location: &str, year: u8, week: u8, id: &str) -> Self {
        let location = String::from(location).to_ascii_uppercase();
        assert_eq!(
            location.len(),
            2,
            "The location should be 2 alpha chars exactly"
        );
        let id = id.to_ascii_uppercase();
        assert_eq!(
            id.len(),
            5,
            "The location should be 5 alphanum chars exactly"
        );

        Self {
            location,
            year,
            week,
            id,
        }
    }
}

impl FromStr for SerialNumber {
    type Err = std::num::ParseIntError;

    // Parses serial number form 'LLYYWWSSSSS' into an
    // instance of [SerialNumber].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // u8::from_str_radix(src: &str, radix: u32) converts a string
        // slice in a given base to u8
        let location = s[0..2].to_ascii_uppercase();
        let year: u8 = u8::from_str_radix(&s[2..4], 10)?;
        let week: u8 = u8::from_str_radix(&s[4..6], 10)?;
        let id = s[6..11].to_ascii_uppercase();

        Ok(Self {
            location,
            year,
            week,
            id,
        })
    }
}

impl Display for SerialNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{LL}{YY}{WW}{SSSSS}",
            LL = self.location,
            YY = self.year,
            WW = self.week,
            SSSSS = self.id,
        )
    }
}

#[cfg(test)]
mod tests_serial_number {
    use super::*;

    #[test]
    fn test_new() {
        let sn = SerialNumber::new("HQ", 20, 49, "A1B2C");
        assert_eq!(sn.to_string(), "HQ2049A1B2C");
    }

    #[test]

    fn test_from_str_valid() {
        let sn = SerialNumber::from_str("HQ2049A1B2C").unwrap();
        assert_eq!(sn.to_string(), "HQ2049A1B2C");
    }

    #[test]
    #[should_panic]
    fn test_from_str_invalid() {
        let _sn = SerialNumber::from_str("HQXY49A1B2C").unwrap();
    }
}
