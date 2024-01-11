//! Module containing enums that facilitate the generation of Move
//! code. Fields in the yaml file, such as `NftType`, are represented via a
//! String but should match to a value in a given Enum. Such Enums represent
//! the type of NFTs available or the type of Markets available on our
//! OriginByte protocol.
use super::Address;
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
    /// Creates a new `Marketplace` instance.
    ///
    /// # Arguments
    /// * `admin` - The `Address` that will act as the admin of the marketplace.
    /// * `receiver` - The `Address` where transaction fees or other payments will be sent.
    /// * `default_fee` - The default fee for transactions in the marketplace, in basis points.
    ///
    /// # Returns
    /// * `Marketplace` - A new instance of `Marketplace`.
    ///
    /// # TODO
    /// Validate the default fee in basis points to ensure it falls within an acceptable range.
    pub fn new(admin: Address, receiver: Address, default_fee: u64) -> Self {
        Marketplace {
            admin,
            receiver,
            default_fee,
        }
    }
}
