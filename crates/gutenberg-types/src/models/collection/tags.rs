use serde::{de::Visitor, Deserialize, Serialize};
use std::fmt::{self, Display};

/// Represents various tags associated with an NFT.
#[derive(Debug, Clone)]
pub enum Tag {
    /// Tag representing Art.
    Art,
    /// Tag representing a Profile Picture.
    ProfilePicture,
    /// Tag representing a Collectible.
    Collectible,
    /// Tag representing a Game Asset.
    GameAsset,
    /// Tag representing a Tokenised Asset.
    TokenisedAsset,
    /// Tag representing a Domain Name.
    DomainName,
    /// Tag representing Music.
    Music,
    /// Tag representing Video.
    Video,
    /// Tag representing a Ticket.
    Ticket,
    /// Tag representing a License.
    License,
    /// Custom tag with a provided string.
    Custom(String),
}

impl Tag {
    /// Creates a new `Tag` instance based on a string.
    ///
    /// # Arguments
    /// * `tag` - The string representing the tag.
    ///
    /// # Returns
    /// * `Tag` - A new instance of `Tag` based on the input string.
    pub fn new(tag: &str) -> Self {
        match tag {
            "Art" => Tag::Art,
            "ProfilePicture" => Tag::ProfilePicture,
            "Collectible" => Tag::Collectible,
            "GameAsset" => Tag::GameAsset,
            "TokenisedAsset" => Tag::TokenisedAsset,
            "DomainName" => Tag::DomainName,
            "Music" => Tag::Music,
            "Video" => Tag::Video,
            "Ticket" => Tag::Ticket,
            "License" => Tag::License,
            tag => Tag::Custom(tag.to_string()),
        }
    }

    /// Returns the corresponding function name for the tag.
    pub fn function_name(&self) -> &'static str {
        match self {
            Tag::Art => "art",
            Tag::ProfilePicture => "profile_picture",
            Tag::Collectible => "collectible",
            Tag::GameAsset => "game_asset",
            Tag::TokenisedAsset => "tokenised_asset",
            Tag::DomainName => "domain_name",
            Tag::Music => "music",
            Tag::Video => "video",
            Tag::Ticket => "ticket",
            Tag::License => "license",
            Tag::Custom(_) => "",
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag = match self {
            Tag::Art => "Art",
            Tag::ProfilePicture => "ProfilePicture",
            Tag::Collectible => "Collectible",
            Tag::GameAsset => "GameAsset",
            Tag::TokenisedAsset => "TokenisedAsset",
            Tag::DomainName => "DomainName",
            Tag::Music => "Music",
            Tag::Video => "Video",
            Tag::Ticket => "Ticket",
            Tag::License => "License",
            Tag::Custom(tag) => tag,
        };

        f.write_str(tag)
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TagVisitor;

        impl<'v> Visitor<'v> for TagVisitor {
            type Value = Tag;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Tag::new(v))
            }
        }

        deserializer.deserialize_str(TagVisitor {})
    }
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{self}"))
    }
}

/// Represents a collection of `Tag` instances.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Tags(pub Vec<Tag>);

impl Tags {
    /// Creates a new `Tags` instance from a vector of strings.
    ///
    /// # Arguments
    /// * `tags` - A vector of strings representing tags.
    ///
    /// # Returns
    /// * `Tags` - A new instance of `Tags` containing `Tag` instances based on the input strings.
    pub fn new(tags: &[String]) -> Self {
        let tags = tags
            .iter()
            .map(|string| Tag::new(string))
            .collect::<Vec<Tag>>();

        Tags(tags)
    }
}
