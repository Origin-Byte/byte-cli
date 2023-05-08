use serde::{Deserialize, Serialize};

use crate::{
    contract::modules::DisplayInfoMod,
    models::collection::{supply::SupplyPolicy, CollectionData},
};

pub enum MintType {
    Direct,
    Airdrop,
    Launchpad,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MintPolicies {
    #[serde(default)]
    pub launchpad: bool,
    #[serde(default)]
    pub airdrop: bool,
    #[serde(default)]
    pub direct: bool,
}

impl Default for MintPolicies {
    fn default() -> Self {
        Self {
            launchpad: false,
            airdrop: false,
            direct: false,
        }
    }
}

impl MintPolicies {
    pub fn is_empty(&self) -> bool {
        !self.launchpad && !self.airdrop && !self.direct
    }

    pub fn write_mint_fn(
        &self,
        collection: &CollectionData,
        mint_policy: Option<MintType>,
    ) -> String {
        let code: String;
        let witness = collection.witness_name();

        let mut return_type = String::new();
        let mut args = String::new();
        let mut domains = String::new();
        let mut params = String::new();
        let mut transfer = String::new();

        // Name and URL are mandatory as they are static display fields on the
        // NFT
        args.push_str("        name: std::string::String,\n");
        params.push_str("            name,\n");

        args.push_str("        url: vector<u8>,\n");
        params.push_str("            url,\n");

        args.push_str(DisplayInfoMod::add_display_args());
        domains.push_str(DisplayInfoMod::add_nft_display());
        params.push_str(DisplayInfoMod::add_display_params());

        domains.push_str(DisplayInfoMod::add_nft_url());

        args.push_str(DisplayInfoMod::add_attributes_args());
        domains.push_str(DisplayInfoMod::add_nft_attributes());
        params.push_str(DisplayInfoMod::add_attributes_params());

        let mint_cap = match collection.supply_policy {
            SupplyPolicy::Unlimited => format!(
                "        mint_cap: &nft_protocol::mint_cap::UnregulatedMintCap<{witness}>,\n"
            ),
            SupplyPolicy::Limited { .. } => format!(
                "        mint_cap: &mut nft_protocol::mint_cap::RegulatedMintCap<{witness}>,\n"
            ),
            SupplyPolicy::Undefined => format!(
                "        mint_cap: &nft_protocol::mint_cap::MintCap<{witness}>,\n"
            ),
        };
        args.push_str(&mint_cap);

        params.push_str("            mint_cap,\n");
        params.push_str("            ctx,");

        if let Some(mint_policy) = mint_policy {
            let mut fun_name = String::new();

            match mint_policy {
                MintType::Launchpad => {
                    args.push_str(
                        format!(
                            "        warehouse: &mut nft_protocol::warehouse::Warehouse<{}>,\n",
                            witness
                        )
                        .as_str(),
                    );
                    transfer.push_str(
                        "nft_protocol::warehouse::deposit_nft(warehouse, nft);",
                    );
                    fun_name.push_str("mint_to_launchpad");
                    args.push_str(
                        "        ctx: &mut sui::tx_context::TxContext,",
                    );
                }
                MintType::Airdrop => {
                    args.push_str("        receiver: address,\n");
                    transfer
                        .push_str("sui::transfer::transfer(nft, receiver);");
                    fun_name.push_str("mint_to_address");
                    args.push_str(
                        "        ctx: &mut sui::tx_context::TxContext,",
                    );
                }
                MintType::Direct => unimplemented!(),
            }

            code = format!(
                "\n
    public entry fun {fun_name}(
{args}
    ){return_type} {{
        let nft = mint(
{params}
        );

        {transfer}
    }}"
            );
        } else {
            return_type.push_str(
                format!(": nft_protocol::nft::Nft<{}>", witness).as_str(),
            );
            transfer.push_str("nft");

            args.push_str("        ctx: &mut sui::tx_context::TxContext,\n");

            let nft = match collection.supply_policy {
                SupplyPolicy::Unlimited => format!(
                    "let nft = nft_protocol::nft::from_unregulated(
            mint_cap,
            name,
            sui::url::new_unsafe_from_bytes(url),
            ctx,
        );"
                ),
                SupplyPolicy::Limited { .. } => format!(
                    "let nft = nft_protocol::nft::from_regulated(
            mint_cap,
            name,
            sui::url::new_unsafe_from_bytes(url),
            ctx,
        );"
                ),
                SupplyPolicy::Undefined => format!(
                    "let nft = nft_protocol::nft::from_mint_cap(
            mint_cap,
            name,
            sui::url::new_unsafe_from_bytes(url),
            ctx,
        );"
                ),
            };

            code = format!(
                "\n
    fun mint(
{args}    ){return_type} {{
        {nft}
        let delegated_witness = nft_protocol::witness::from_witness<{witness}, Witness>(&Witness {{}});
{domains}
        {transfer}
    }}"
            );
        }

        code
    }

    pub fn write_mint_fns(&self, collection: &CollectionData) -> String {
        let mut mint_fns = String::new();

        if self.launchpad {
            mint_fns.push_str(
                &self.write_mint_fn(collection, Some(MintType::Launchpad)),
            );
        }

        // TODO: For now the flow are indistinguishable
        if self.airdrop || self.direct {
            mint_fns.push_str(
                &self.write_mint_fn(collection, Some(MintType::Airdrop)),
            );
        }

        mint_fns.push_str(&self.write_mint_fn(collection, None));

        mint_fns
    }
}
