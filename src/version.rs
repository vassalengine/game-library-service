use serde::Deserialize;
use std::{
    cmp::Ordering,
    fmt,
    str::FromStr
};
use thiserror::Error;

#[derive(Debug, Eq, Error, PartialEq)]
#[error("{0} is malformed")]
pub struct MalformedVersion(pub String);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(try_from = "&str")]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
    pub pre: Option<String>,
    pub build: Option<String>
}

impl Version {
    pub fn new(version: &semver::Version) -> Option<Version> {
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

    fn get_prerelease(&self) -> semver::Prerelease {
        self.pre
            .as_deref()
            .map(semver::Prerelease::new)
            .unwrap_or(Ok(semver::Prerelease::EMPTY))
            .unwrap()
    }

    fn get_build(&self) -> semver::BuildMetadata {
        self.build
            .as_deref()
            .map(semver::BuildMetadata::new)
            .unwrap_or(Ok(semver::BuildMetadata::EMPTY))
            .unwrap()
    }
}

impl From<&Version> for String {
    fn from(v: &Version) -> Self {
        v.to_string()
    }
}

impl TryFrom<&str> for Version {
    type Error = MalformedVersion;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse::<semver::Version>()
            .ok()
            .as_ref()
            .and_then(Version::new)
            .ok_or_else(|| MalformedVersion(s.into()))
    }
}

impl FromStr for Version {
    type Err = MalformedVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::try_from(s)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.pre {
            None => write!(f, "{}.{}.{}", self.major, self.minor, self.patch),
            Some(pre) => match &self.build {
                None => write!(
                    f,
                    "{}.{}.{}-{}",
                    self.major,
                    self.minor,
                    self.patch,
                    pre
                ),
                Some(build) => write!(
                    f,
                    "{}.{}.{}-{}+{}",
                    self.major,
                    self.minor,
                    self.patch,
                    pre,
                    build
                )
            }
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.major, self.minor, self.patch).cmp(
            &(other.major, other.minor, other.patch)
        ) {
            Ordering::Equal => {
                match self.get_prerelease().cmp(&other.get_prerelease()) {
                    Ordering::Equal => {
                        self.get_build().cmp(&other.get_build())
                    },
                    ord => ord
                }
            },
            ord => ord
        }
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
        assert!(
            matches!(
                "".parse::<Version>().unwrap_err(),
                MalformedVersion(_)
            )
        );
    }

    #[test]
    fn string_to_version_bogus() {
        assert!(
            matches!(
                "bogus".parse::<Version>().unwrap_err(),
                MalformedVersion(_)
            )
        );
    }

    #[test]
    fn string_to_version_whitespace() {
        assert!(
            matches!(
                " 1.2.3 ".parse::<Version>().unwrap_err(),
                MalformedVersion(_)
            )
        );
    }

    #[test]
    fn string_to_version_too_few_components() {
        assert!(
            matches!(
                "0.1".parse::<Version>().unwrap_err(),
                MalformedVersion(_)
            )
        );
    }

    #[test]
    fn string_to_version_too_many_components() {
        assert!(
            matches!(
                "0.1.2.3".parse::<Version>().unwrap_err(),
                MalformedVersion(_)
            )
        );
    }

    #[test]
    fn string_to_version_bad_pre() {
        assert!(
            matches!(
                "0.1.2-".parse::<Version>().unwrap_err(),
                MalformedVersion(_)
            )
        );
    }

    #[test]
    fn string_to_version_major_too_large() {
        let v = format!("{}.0.0", i64::MAX as u64 + 1);
        assert!(
            matches!(
                v.parse::<Version>().unwrap_err(),
                MalformedVersion(_)
            )
        );
    }

    #[test]
    fn string_to_version_minor_too_large() {
        let v = format!("0.{}.0", i64::MAX as u64 + 1);
        assert!(
            matches!(
                v.parse::<Version>().unwrap_err(),
                MalformedVersion(_)
            )
        );
    }

    #[test]
    fn string_to_version_patch_too_large() {
        let v = format!("0.0.{}", i64::MAX as u64 + 1);
        assert!(
            matches!(
                v.parse::<Version>().unwrap_err(),
                MalformedVersion(_)
            )
        );
    }

    #[test]
    fn version_to_string() {
        let v = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: None,
            build: None
        };
        assert_eq!(String::from(&v), "1.2.3");
    }

    #[test]
    fn version_to_string_pre() {
        let v = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Some("abc".into()),
            build: None
        };
        assert_eq!(String::from(&v), "1.2.3-abc");
    }

    #[test]
    fn version_to_string_pre_build() {
        let v = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Some("abc".into()),
            build: Some("def".into())
        };
        assert_eq!(String::from(&v), "1.2.3-abc+def");
    }

    #[test]
    fn ordering() {
        let vers: Vec<_> = [
            "1.2.3-alpha.1",
            "1.2.3-alpha.1+foo",
            "1.2.3",
            "1.2.3+foo"
        ]
        .iter()
        .map(|v| v.parse::<Version>().unwrap())
        .collect();

        for i in 0..vers.len() {
            for j in 0..vers.len() {
                assert_eq!(vers[i].cmp(&vers[j]), i.cmp(&j));
            }
        }
    }
}
