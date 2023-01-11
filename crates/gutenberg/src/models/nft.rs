use serde::Deserialize;

/// Enum representing the NFT types currently available in the protocol
#[derive(Debug, Deserialize)]
pub enum NftType {
    // TODO: Need to add support for Soulbound
    Classic,
    // TODO: To be added back
    // Collectible,
    // CNft,
}

impl NftType {
    /// Writes Move code for an entry function meant to be called by
    /// the Creators to mint NFTs. Depending on the NFTtype the function
    /// parameters change, therefore pattern match the NFT type.
    pub fn mint_func(&self, witness: &str) -> String {
        let func = match self {
            NftType::Classic => format!(
                "public entry fun mint_nft(
                    name: String,
                    description: String,
                    url: vector<u8>,
                    attribute_keys: vector<String>,
                    attribute_values: vector<String>,
                    mint_cap: &mut MintCap<{witness}>,
                    inventory: &mut Inventory,
                    ctx: &mut TxContext,
                ) {{
                    let nft = nft::new<{witness}>(tx_context::sender(ctx), ctx);

                    collection::increment_supply(mint_cap, 1);

                    display::add_display_domain(
                        &mut nft,
                        name,
                        description,
                        ctx,
                    );

                    display::add_url_domain(
                        &mut nft,
                        url::new_unsafe_from_bytes(url),
                        ctx,
                    );

                    display::add_attributes_domain_from_vec(
                        &mut nft,
                        attribute_keys,
                        attribute_values,
                        ctx,
                    );

                    inventory::deposit_nft(inventory, nft);
                }}",
                witness = witness,
            ),
        };
        func
    }
}
