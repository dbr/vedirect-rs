use std::str::FromStr;

use regex::Captures;
use regex::Regex;

/// Firmware Version used by fields FW and FWE
///
/// According to the specs:
///
/// # FW
/// The firmware version of the device. The version is reported as a whole number, e.g. 208
/// for firmware version 2.08.The value C208 means release candidate C for version 2.08.
/// Note: This field is available in the BMV since version 2.08
///
/// # FWE
/// The firmware version of the device. The version contains up to 6 digits, 0 padding on the
/// left side is not mandatory. Examples: 0208FF or 208FF for firmware version 2.08 (last digit
/// FF indicates an official release), 020801 for firmware version 2.08-beta-01.
///
/// Notes: In general, Victron seems to not be using patch numbers (at least from what is released).
use semver::*;

/// The specs are blurry for those so we need to make assumptions
#[derive(Debug, PartialEq)]
pub enum VersionType {
    /// Coded on 3..4 digits: <Pre:1?> <Major:1> <Minor:2>
    FW,

    /// Coded on 5..6 digits: <Major:1..2> <Minor:2> <Pre:2>
    FWE,
}

#[derive(Debug)]
struct FirmwareVersion {
    version: Version,
    version_type: VersionType,
}

impl FromStr for FirmwareVersion {
    type Err = semver::SemVerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const MAJ: &str = "Major";
        const MIN: &str = "Minor";
        const PRE: &str = "Pre";

        let version_type = match s.len() {
            3..=4 => VersionType::FW,
            5..=6 => VersionType::FWE,
            _ => unreachable!("It seems that our assumptions about version parsing were wrong..."),
        };

        let version = match version_type {
            VersionType::FW => {
                let re = Regex::new(&format!(
                    "(?P<{PRE}>[A-F])?(?P<{MAJ}>\\d)(?P<{MIN}>\\d{{2}})",
                    MAJ = MAJ,
                    MIN = MIN,
                    PRE = PRE
                ))
                .unwrap();
                let caps: Captures = re.captures(s).unwrap();
                let major = caps.name(MAJ).map_or("", |m| m.as_str());
                let minor = caps.name(MIN).map_or("", |m| m.as_str());
                let pre = caps.name(PRE).map_or("", |m| m.as_str());
                println!("caps: {:?}", caps);
                println!("major: {:?}", major);
                println!("minor: {:?}", minor);
                println!("pre: {:?}", pre);
                Version::new(1, 2, 3)
                // let s = format!("{:.2}.0", u32::from_str(s).unwrap() as f32 / 100f32);
            }
            VersionType::FWE => {
                // let re =
                // Regex::new(r"(?<Major>\d{1,2})(?<Minor>\d{2})(?<Pre>[0-9A-F]{2})").unwrap();
                Version::new(1, 2, 3)
            }
        };

        Ok(Self {
            version: Version::parse(&s.to_string()).unwrap(),
            version_type,
        })
    }
}

#[cfg(test)]
mod test_frame_finder {
    use super::*;

    #[test]
    fn test_fw_150() {
        let fwv = FirmwareVersion::from_str("150").unwrap();

        assert_eq!(fwv.version_type, VersionType::FW);
        assert_eq!(
            fwv.version,
            Version {
                major: 1,
                minor: 50,
                patch: 0,
                pre: vec!(),
                build: vec!(),
            }
        );
    }

    #[test]
    fn test_fw_c208() {
        let fwv = FirmwareVersion::from_str("C208").unwrap();

        assert_eq!(fwv.version_type, VersionType::FW);
        assert_eq!(
            fwv.version,
            Version {
                major: 2,
                minor: 8,
                patch: 0,
                pre: vec![Identifier::AlphaNumeric("C".into())],
                build: vec!(),
            }
        );
    }

    #[test]
    fn test_fw_0208ff() {
        let fwv = FirmwareVersion::from_str("0208FF").unwrap();

        assert_eq!(fwv.version_type, VersionType::FWE);
        assert_eq!(
            fwv.version,
            Version {
                major: 2,
                minor: 8,
                patch: 0,
                pre: vec!(),
                build: vec!(),
            }
        );
    }

    #[test]
    fn test_fw_208ff() {
        let fwv = FirmwareVersion::from_str("208FF").unwrap();

        assert_eq!(fwv.version_type, VersionType::FWE);
        assert_eq!(
            fwv.version,
            Version {
                major: 2,
                minor: 8,
                patch: 0,
                pre: vec!(),
                build: vec!(),
            }
        );
    }

    #[test]
    fn test_fw_020801() {
        let fwv = FirmwareVersion::from_str("020801").unwrap();

        assert_eq!(fwv.version_type, VersionType::FWE);
        assert_eq!(
            fwv.version,
            Version {
                major: 2,
                minor: 8,
                patch: 0,
                pre: vec![Identifier::AlphaNumeric("beta-01".into())],
                build: vec!(),
            }
        );
    }
}
