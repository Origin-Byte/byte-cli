use crate::err::GutenError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Nft {
    nft_type: NftType,
    supply_policy: bool,
    fields: Fields,
    mint_strategy: MintStrategy,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Fields {
    display: bool,
    url: bool,
    attributes: bool,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct MintStrategy {
    direct: bool,
    airdrop: bool,
    launchpad: bool,
}

pub enum Mint {
    Direct,
    Airdrop,
    Launchpad,
}

/// Enum representing the NFT types currently available in the protocol
#[derive(Debug, Deserialize, Serialize)]
pub enum NftType {
    // TODO: Need to add support for Soulbound
    Classic,
    // Composable,
}

impl FromStr for NftType {
    type Err = ();

    fn from_str(input: &str) -> Result<NftType, Self::Err> {
        match input {
            "Classic" => Ok(NftType::Classic),
            // "Composable" => Ok(NftType::Composable),
            _ => Err(()),
        }
    }
}

impl Nft {
    pub fn write_domains(&self) -> String {
        let s = serde_json::to_string(&self.mint_strategy).unwrap();
        let domains: HashMap<String, bool> = serde_json::from_str(&s).unwrap();

        let code = domains
            .iter()
            .filter(|(k, v)| **v == true)
            .map(|(k, _)| {
                let display = "display".to_string();
                let url = "url".to_string();
                let attributes = "attributes".to_string();

                match k {
                    display => {
                        "display::add_display_domain(
                            &mut nft,
                            name,
                            description,
                            ctx,
                        );"
                    }
                    airdrop => {
                        "display::add_url_domain(
                            &mut nft,
                            url::new_unsafe_from_bytes(url),
                            ctx,
                        );"
                    }
                    attributes => {
                        "display::add_attributes_domain_from_vec(
                            &mut nft,
                            attribute_keys,
                            attribute_values,
                            ctx,
                        );"
                    }
                }
            })
            .collect();

        code
    }

    pub fn write_fields(&self) -> String {
        let s = serde_json::to_string(&self.mint_strategy).unwrap();
        let domains: HashMap<String, bool> = serde_json::from_str(&s).unwrap();

        let code = domains
            .iter()
            .filter(|(k, v)| **v == true)
            .map(|(k, _)| {
                let display = "display".to_string();
                let url = "url".to_string();
                let attributes = "attributes".to_string();

                match k {
                    display => {
                        "name: String,
                        description: String,
                        "
                    }
                    url => "url: vector<u8>,",
                    attributes => {
                        "attribute_keys: vector<String>,
                        attribute_values: vector<String>,
                        "
                    }
                }
            })
            .collect();

        code
    }

    pub fn mint_fn(&self, witness: &str, mint_strategy: Mint) -> String {
        let fun_name: String;
        let to_whom: String;
        let transfer: String;

        let fun = match mint_strategy {
            Mint::Direct => {
                fun_name = "direct_mint".to_string();
                to_whom = "receiver: address,".to_string();
                transfer = "transfer::transfer(nft, receiver);".to_string();
            }
            Mint::Airdrop => {
                fun_name = "airdrop_mint".to_string();
                to_whom = format!(
                    "
                mint_cap: &mut MintCap<{witness}>,
                receiver: address,",
                    witness = witness,
                );
                transfer = "transfer::transfer(nft, receiver);".to_string();
            }
            Mint::Launchpad => {
                fun_name = "warehouse_mint".to_string();
                to_whom = format!(
                    "
                mint_cap: &mut MintCap<{witness}>,
                inventory: &mut Inventory,",
                    witness = witness,
                );
                transfer =
                    "inventory::deposit_nft(inventory, nft);".to_string();
            }
        };

        let domains = self.write_domains();
        let fields = self.write_fields();

        let end_of_signature = "ctx: &mut TxContext,
        ) {{
            let nft = nft::new<{witness}>(tx_context::sender(ctx), ctx);

            collection::increment_supply(mint_cap, 1);
        "
        .to_string();

        [
            format!("public entry fun{fun_name}(", fun_name = fun_name),
            fields,
            to_whom,
            end_of_signature,
            domains,
            transfer,
            "}}".to_string(),
        ]
        .join("\n")
    }
}

impl NftType {
    pub fn new(nft_type: &str) -> Result<NftType, GutenError> {
        let nft = NftType::from_str(nft_type)
            .map_err(|_| GutenError::UnsupportedNFftype)?;
        Ok(nft)
    }

    // /// Writes Move code for an entry function meant to be called by
    // /// the Creators to mint NFTs. Depending on the NFTtype the function
    // /// parameters change, therefore pattern match the NFT type.
    // pub fn mint_func(&self, witness: &str) -> String {
    //     let func = match self {
    //         NftType::Classic => format!(
    //             "public entry fun mint_nft(
    //                 name: String,
    //                 description: String,
    //                 url: vector<u8>,
    //                 attribute_keys: vector<String>,
    //                 attribute_values: vector<String>,
    //                 mint_cap: &mut MintCap<{witness}>,
    //                 inventory: &mut Inventory,
    //                 ctx: &mut TxContext,
    //             ) {{
    //                 let nft = nft::new<{witness}>(tx_context::sender(ctx), ctx);

    //                 collection::increment_supply(mint_cap, 1);

    //                 display::add_display_domain(
    //                     &mut nft,
    //                     name,
    //                     description,
    //                     ctx,
    //                 );

    //                 display::add_url_domain(
    //                     &mut nft,
    //                     url::new_unsafe_from_bytes(url),
    //                     ctx,
    //                 );

    //                 display::add_attributes_domain_from_vec(
    //                     &mut nft,
    //                     attribute_keys,
    //                     attribute_values,
    //                     ctx,
    //                 );

    //                 inventory::deposit_nft(inventory, nft);
    //             }}",
    //             witness = witness,
    //         ),
    //     };
    //     func
    // }
}
