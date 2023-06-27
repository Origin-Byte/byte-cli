//! Module containing enums that facilitate the generation of Move
//! code. Fields in the yaml file, such as `NftType`, are represented via a
//! String but should match to a value in a given Enum. Such Enums represent
//! the type of NFTs available or the type of Markets available on our
//! OriginByte protocol.
use package_manager::{Address, AddressError};
use serde::{Deserialize, Serialize};

/// Contains the market configurations of the marketplace
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Marketplace {
    pub admin: Address,
    pub receiver: Address,
    // Default fee in basis points
    pub default_fee: u64,
}

impl Marketplace {
    pub fn new(
        admin: &str,
        receiver: &str,
        default_fee: u64,
    ) -> Result<Self, AddressError> {
        // TODO: Validate default fee basis points

        Ok(Marketplace {
            admin: Address::new(admin)?,
            receiver: Address::new(receiver)?,
            default_fee,
        })
    }
}
