use serde::{de::Visitor, Deserialize, Serialize};
use std::fmt::{self, Display};

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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Tags(Vec<Tag>);

impl Tags {
    pub fn new(tags: &[String]) -> Self {
        let tags = tags
            .iter()
            .map(|string| Tag::new(string))
            .collect::<Vec<Tag>>();

        Tags(tags)
    }

    /// Generates Move code to push tags to a Move `vector` structure
    pub fn write_move_init(&self) -> String {
        let mut code = String::from(
            "

        let tags = std::vector::empty();",
        );

        for tag in self.0.iter() {
            code.push_str(&tag.write_move());
        }

        code
    }
}
