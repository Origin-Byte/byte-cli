use serde::{de::Visitor, Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

use crate::err::GutenError;

#[derive(Debug, Clone)]
pub enum Tag {
    Art,
    ProfilePicture,
    Collectible,
    GameAsset,
    TokenisedAsset,
    DomainName,
    Music,
    Video,
    Ticket,
    License,
    Custom(String),
}

impl Tag {
    fn function_name(&self) -> &'static str {
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

    pub fn write_move(&self) -> String {
        match self {
            Tag::Custom(tag) => format!(
                "
        std::vector::push_back(&mut tags, std::string::utf8(b\"{tag}\"));"
            ),
            tag => {
                let function_name = tag.function_name();
                format!(
                    "
        std::vector::push_back(&mut tags, nft_protocol::tags::{function_name}());"
                )
            }
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

impl FromStr for Tag {
    type Err = ();

    fn from_str(input: &str) -> Result<Tag, Self::Err> {
        match input {
            "Art" => Ok(Tag::Art),
            "ProfilePicture" => Ok(Tag::ProfilePicture),
            "Collectible" => Ok(Tag::Collectible),
            "GameAsset" => Ok(Tag::GameAsset),
            "TokenisedAsset" => Ok(Tag::TokenisedAsset),
            "DomainName" => Ok(Tag::DomainName),
            "Music" => Ok(Tag::Music),
            "Video" => Ok(Tag::Video),
            "Ticket" => Ok(Tag::Ticket),
            "License" => Ok(Tag::License),
            tag => Ok(Tag::Custom(tag.to_string())),
        }
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
                Tag::from_str(v)
                    .map_err(|_err| E::custom("Could not parse tag"))
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Tags(Vec<Tag>);

impl Tags {
    pub fn new(tags: &[String]) -> Self {
        let tags = tags
            .iter()
            .map(|string| Tag::from_str(string).unwrap())
            .collect::<Vec<Tag>>();

        Tags(tags)
    }

    /// Generates Move code to push tags to a Move `vector` structure
    pub fn write_move_init(&self) -> String {
        let mut code = String::from(
            "

        let tags: vector<std::string::String> = std::vector::empty();",
        );

        for tag in self.0.iter() {
            code.push_str(&tag.write_move());
        }

        code
    }

    pub fn push_tag(&mut self, tag_string: String) -> Result<(), GutenError> {
        let tag = Tag::from_str(&tag_string).unwrap();
        self.0.push(tag);

        Ok(())
    }

    pub fn has_tags(&self) -> bool {
        !self.0.is_empty()
    }
}
