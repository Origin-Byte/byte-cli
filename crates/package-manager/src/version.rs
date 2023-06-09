use anyhow::{anyhow, Result};
use std::{cmp::Ordering, fmt, marker::PhantomData};

use serde::{
    de::{self, Unexpected, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    pub fn from_string(string: &str) -> Result<Self> {
        let version: Vec<&str> = string.split('.').collect();

        if version.len() != 3 {
            return Err(anyhow!("Version semantics is incorrect"));
        }

        let version = version
            .iter()
            .map(|v| {
                let v_string = v.to_string();
                v_string.parse::<u8>().unwrap()
            })
            .collect::<Vec<u8>>();

        Ok(Version {
            major: version[0],
            minor: version[1],
            patch: version[2],
        })
    }
}

struct VersionVisitor {
    marker: PhantomData<fn() -> Version>,
}

impl VersionVisitor {
    fn new() -> Self {
        VersionVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string containing the package version in semantic versioning format xx.yy.zz")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let version: Vec<&str> = s.split('.').collect();

        if version.len() != 3 {
            return Err(de::Error::invalid_value(Unexpected::Str(s), &self));
        }

        let version = version
            .iter()
            .map(|v| {
                let v_string = v.to_string();
                v_string.parse::<u8>().unwrap()
            })
            .collect::<Vec<u8>>();

        Ok(Version {
            major: version[0],
            minor: version[1],
            patch: version[2],
        })
    }
}

// This is the trait that informs Serde how to deserialize Version.
impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate VersionVisitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of Version.
        deserializer.deserialize_str(VersionVisitor::new())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let major_ord = self.major.cmp(&other.major);
        if major_ord != Ordering::Equal {
            return major_ord;
        }

        let minor_ord = self.minor.cmp(&other.minor);
        if minor_ord != Ordering::Equal {
            return minor_ord;
        }

        self.patch.cmp(&other.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let major_ord = self.major.partial_cmp(&other.major);
        if major_ord.unwrap() != Ordering::Equal {
            return major_ord;
        }

        let minor_ord = self.minor.partial_cmp(&other.minor);
        if minor_ord.unwrap() != Ordering::Equal {
            return minor_ord;
        }

        self.patch.partial_cmp(&other.patch)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod test_version {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_order() -> Result<()> {
        let version_a = Version::from_string(&String::from("1.0.1"))?;
        let version_b = Version::from_string(&String::from("1.0.0"))?;
        assert!(version_a > version_b);

        let version_a = Version::from_string(&String::from("1.1.0"))?;
        let version_b = Version::from_string(&String::from("1.0.0"))?;
        assert!(version_a > version_b);

        let version_a = Version::from_string(&String::from("1.0.0"))?;
        let version_b = Version::from_string(&String::from("1.0.0"))?;
        assert!(version_a == version_b);

        let version_a = Version::from_string(&String::from("2.0.0"))?;
        let version_b = Version::from_string(&String::from("1.5.0"))?;
        assert!(version_a > version_b);

        let version_a = Version::from_string(&String::from("0.0.30"))?;
        let version_b = Version::from_string(&String::from("0.5.0"))?;
        assert!(version_a < version_b);

        Ok(())
    }
}
