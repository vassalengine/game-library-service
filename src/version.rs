use serde::Deserialize;
use std::str::FromStr;

use crate::errors::AppError;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(try_from = "&str")]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
    pub pre: Option<String>,
    pub build: Option<String>
}

impl Version {
    pub fn new(version: semver::Version) -> Option<Version> {
        Some(
            Version {
                major: i64::try_from(version.major).ok()?,
                minor: i64::try_from(version.minor).ok()?,
                patch: i64::try_from(version.patch).ok()?,
                pre: match version.pre.as_str() {
                    "" => None,
                    s => Some(s.into())
                },
                build: match version.build.as_str() {
                    "" => None,
                    s => Some(s.into())
                }
            }
        )
    }
}

impl TryFrom<&str> for Version {
    type Error = AppError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse::<semver::Version>()
            .ok()
            .and_then(Version::new)
            .ok_or(AppError::MalformedVersion)
    }
}

impl FromStr for Version {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::try_from(s)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn string_to_version_ok() {
        assert_eq!(
            "1.2.3".parse::<Version>().unwrap(),
            Version { major: 1, minor: 2, patch: 3, pre: None, build: None }
        );
    }

    #[test]
    fn string_to_version_pre_ok() {
        assert_eq!(
            "1.2.3-beta.1".parse::<Version>().unwrap(),
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                pre: Some("beta.1".into()),
                build: None
            }
        );
    }

    #[test]
    fn string_to_version_build_ok() {
        assert_eq!(
            "1.2.3+foobar".parse::<Version>().unwrap(),
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                pre: None,
                build: Some("foobar".into())
            }
        );
    }

    #[test]
    fn string_to_version_pre_build_ok() {
        assert_eq!(
            "1.2.3-beta.1+foobar".parse::<Version>().unwrap(),
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                pre: Some("beta.1".into()),
                build: Some("foobar".into())
            }
        );
    }

    #[test]
    fn string_to_version_empty() {
        assert_eq!(
            "".parse::<Version>().unwrap_err(),
            AppError::MalformedVersion
        );
    }

    #[test]
    fn string_to_version_bogus() {
        assert_eq!(
            "bogus".parse::<Version>().unwrap_err(),
            AppError::MalformedVersion
        );
    }

    #[test]
    fn string_to_version_whitespace() {
        assert_eq!(
            " 1.2.3 ".parse::<Version>().unwrap_err(),
            AppError::MalformedVersion
        );
    }

    #[test]
    fn string_to_version_too_few_components() {
        assert_eq!(
            "0.1".parse::<Version>().unwrap_err(),
            AppError::MalformedVersion
        );
    }

    #[test]
    fn string_to_version_too_many_components() {
        assert_eq!(
            "0.1.2.3".parse::<Version>().unwrap_err(),
            AppError::MalformedVersion
        );
    }

    #[test]
    fn string_to_version_bad_pre() {
        assert_eq!(
            "0.1.2-".parse::<Version>().unwrap_err(),
            AppError::MalformedVersion
        );
    }

    #[test]
    fn string_to_version_major_too_large() {
        let v = format!("{}.0.0", i64::MAX as u64 + 1);
        assert_eq!(
            v.parse::<Version>().unwrap_err(),
            AppError::MalformedVersion
        );
    }

    #[test]
    fn string_to_version_minor_too_large() {
        let v = format!("0.{}.0", i64::MAX as u64 + 1);
        assert_eq!(
            v.parse::<Version>().unwrap_err(),
            AppError::MalformedVersion
        );
    }

    #[test]
    fn string_to_version_patch_too_large() {
        let v = format!("0.0.{}", i64::MAX as u64 + 1);
        assert_eq!(
            v.parse::<Version>().unwrap_err(),
            AppError::MalformedVersion
        );
    }
}
