//! Module containing enums that facilitate the generation of Move
//! code. Fields in the yaml file, such as `NftType`, are represented via a
//! String but should match to a value in a given Enum. Such Enums represent
//! the type of NFTs available or the type of Markets available on our
//! OriginByte protocol.
use serde::{Deserialize, Serialize};

use crate::{err::GutenError, utils::validate_address};

use super::default_admin;

/// Contains the market configurations of the marketplace
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Marketplace {
    #[serde(default = "default_admin")]
    pub admin: String,
    #[serde(default = "default_admin")]
    pub receiver: String,
    // Default fee in basis points
    pub default_fee: u64,
}

impl Marketplace {
    pub fn new(
        admin: String,
        receiver: String,
        default_fee: u64,
    ) -> Result<Self, GutenError> {
        validate_address(&admin)?;
        validate_address(&receiver)?;
        // TODO: Validate default fee basis points

        Ok(Marketplace {
            admin,
            receiver,
            default_fee,
        })
    }
}
