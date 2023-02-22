use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

use crate::{contract::modules::TagsMod, err::GutenError};

#[derive(Debug, Serialize, Deserialize)]
pub enum Tag {
    Art,
    ProfilePicture,
    Collectible,
    GameAsset,
    TokenisedAsset,
    Ticker,
    DomainName,
    Music,
    Video,
    Ticket,
    License,
}

// The ToString trait is automatically implemented for any type which
// implements the Display trait. As such, ToString shouldn't be
// implemented directly.
impl Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tag = match self {
            Tag::Art => "art",
            Tag::ProfilePicture => "profile_picture",
            Tag::Collectible => "collectible",
            Tag::GameAsset => "game_asset",
            Tag::TokenisedAsset => "tokenised_asset",
            Tag::Ticker => "ticker",
            Tag::DomainName => "domain_name",
            Tag::Music => "music",
            Tag::Video => "video",
            Tag::Ticket => "ticket",
            Tag::License => "license",
        };

        write!(f, "{}", tag)
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
            "Ticker" => Ok(Tag::Ticker),
            "DomainName" => Ok(Tag::DomainName),
            "Music" => Ok(Tag::Music),
            "Video" => Ok(Tag::Video),
            "Ticket" => Ok(Tag::Ticket),
            "License" => Ok(Tag::License),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tags(Vec<Tag>);

impl Tags {
    pub fn new(tags: &[String]) -> Result<Self, GutenError> {
        let tags = tags
            .iter()
            .map(|string| {
                Tag::from_str(string).map_err(|_| GutenError::UnsupportedTag)
            })
            .collect::<Result<Vec<Tag>, GutenError>>()?;

        Ok(Tags(tags))
    }

    /// Generates Move code to push tags to a Move `vector` structure
    pub fn write_domain(&self, is_collection: bool) -> String {
        let mut code = String::from(
            "
        let tags = nft_protocol::tags::empty(ctx);\n",
        );

        for tag in self.0.iter().map(Tag::to_string) {
            code.push_str(&format!(
                "        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::{tag}());\n",
            ));
        }

        if is_collection {
            code.push_str(TagsMod::add_collection_domain());
        } else {
            code.push_str(TagsMod::add_nft_domain());
        }

        code
    }

    pub fn push_tag(&mut self, tag_string: String) -> Result<(), GutenError> {
        let tag = Tag::from_str(tag_string.as_str())
            .map_err(|_| GutenError::UnsupportedTag)?;

        self.0.push(tag);

        Ok(())
    }

    pub fn has_tags(&self) -> bool {
        !self.0.is_empty()
    }
}
