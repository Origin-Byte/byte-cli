use anyhow::{anyhow, Result};
use std::{cmp::Ordering, fmt, marker::PhantomData, str::FromStr};
use serde::{
    de::{self, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

/// Represents a semantic version number.
///
/// This struct is used to handle versioning in the format major.minor.patch.
/// Each component of the version number is an unsigned 8-bit integer.
///
/// # Fields
/// * `major` - Major version number.
/// * `minor` - Minor version number.
/// * `patch` - Patch version number.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u8,
}

impl Version {
    /// Constructs a new `Version`.
    ///
    /// # Arguments
    /// * `major` - Major version number.
    /// * `minor` - Minor version number.
    /// * `patch` - Patch version number.
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

/// Enables parsing a `Version` from a string slice.
///
/// This implementation allows for creating a `Version` instance from a string
/// that follows the semantic versioning format (major.minor.patch).
impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let version: Vec<&str> = s.split('.').collect();

        if version.len() != 3 {
            return Err(anyhow!("Version semantics is incorrect"));
        }

        let version = version
            .iter()
            .map(|v| {
                let v_string = v.to_string();
                v_string
                    .parse::<u8>()
                    .expect("Unable to convert version substring to u8")
            })
            .collect::<Vec<u8>>();

        Ok(Version {
            major: version[0],
            minor: version[1],
            patch: version[2],
        })
    }
}

// Visitor struct and implementation for custom deserialization of `Version`.
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
                v_string
                    .parse::<u8>()
                    .expect("Unable to convert version substring to u8")
            })
            .collect::<Vec<u8>>();

        Ok(Version {
            major: version[0],
            minor: version[1],
            patch: version[2],
        })
    }
}

/// Implementing custom deserialization for `Version` using Serde.
///
/// This allows for converting a string following the semantic versioning format
/// into a `Version` instance during deserialization.
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

/// Implementing custom ordering for `Version`.
///
/// This enables comparison between two `Version` instances based on their
/// major, minor, and patch numbers.
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

/// Implementing custom display formatting for `Version`.
///
/// This allows for a `Version` instance to be converted into a string
/// in the format major.minor.patch.
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Implementing custom serialization for `Version` using Serde.
///
/// This allows for converting a `Version` instance into a string
/// following the semantic versioning format during serialization.
impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let version_str = self.major.to_string()
            + "."
            + self.minor.to_string().as_str()
            + "."
            + self.patch.to_string().as_str();

        serializer.serialize_str(version_str.as_str())
    }
}

#[cfg(test)]
mod test_version {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_order() -> Result<()> {
        let version_a = Version::from_str("1.0.1")?;
        let version_b = Version::from_str("1.0.0")?;
        assert!(version_a > version_b);

        let version_a = Version::from_str("1.1.0")?;
        let version_b = Version::from_str("1.0.0")?;
        assert!(version_a > version_b);

        let version_a = Version::from_str("1.0.0")?;
        let version_b = Version::from_str("1.0.0")?;
        assert!(version_a == version_b);

        let version_a = Version::from_str("2.0.0")?;
        let version_b = Version::from_str("1.5.0")?;
        assert!(version_a > version_b);

        let version_a = Version::from_str("0.0.30")?;
        let version_b = Version::from_str("0.5.0")?;
        assert!(version_a < version_b);

        Ok(())
    }
}
