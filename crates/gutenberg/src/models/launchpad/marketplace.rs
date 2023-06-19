//! Module containing enums that facilitate the generation of Move
//! code. Fields in the yaml file, such as `NftType`, are represented via a
//! String but should match to a value in a given Enum. Such Enums represent
//! the type of NFTs available or the type of Markets available on our
//! OriginByte protocol.
use package_manager::Address;
use serde::{Deserialize, Serialize};

/// Contains the market configurations of the marketplace
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Marketplace {
    admin: Address,
    receiver: Address,
    // Default fee in basis points
    default_fee: u64,
}

impl Marketplace {
    pub fn new(admin: Address, receiver: Address, default_fee: u64) -> Self {
        // TODO: Validate default fee basis points
        Marketplace {
            admin,
            receiver,
            default_fee,
        }
    }
}
