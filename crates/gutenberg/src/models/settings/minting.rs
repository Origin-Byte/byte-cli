use serde::{Deserialize, Serialize};

use crate::contract::modules::DisplayInfoMod;

pub enum MintType {
    Airdrop,
    Launchpad,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MintPolicies {
    pub launchpad: bool,
    pub airdrop: bool,
}

impl MintPolicies {
    pub fn is_empty(&self) -> bool {
        !self.launchpad && !self.airdrop
    }

    pub fn write_mint_fn(
        &self,
        mint_policy: Option<MintType>,
        nft_type_name: &str,
    ) -> String {
        let code: String;
        let mut return_type = String::new();
        let mut args = String::new();
        let mut params = String::new();
        let mut transfer = String::new();

        args.push_str("        name: std::string::String,\n");
        params.push_str("            name,\n");

        args.push_str(DisplayInfoMod::add_display_args());
        params.push_str(DisplayInfoMod::add_display_params());

        args.push_str("        url: vector<u8>,\n");
        params.push_str("            url,\n");

        args.push_str(DisplayInfoMod::add_attributes_args());
        params.push_str(DisplayInfoMod::add_attributes_params());

        let mint_cap = format!(
                "        mint_cap: &mut nft_protocol::mint_cap::MintCap<{nft_type_name}>,\n"
            );
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
                            nft_type_name
                        )
                        .as_str(),
                    );
                    transfer.push_str(
                        "nft_protocol::warehouse::deposit_nft(warehouse, nft);",
                    );
                    fun_name.push_str("mint_nft");
                    args.push_str(
                        "        ctx: &mut sui::tx_context::TxContext,",
                    );
                }
                MintType::Airdrop => {
                    args.push_str(
                        "        receiver: &mut ob_kiosk::ob_kiosk::Kiosk,\n",
                    );
                    transfer.push_str(
                        "ob_kiosk::ob_kiosk::deposit(receiver, nft, ctx);",
                    );
                    fun_name.push_str("airdrop_nft");
                    args.push_str(
                        "        ctx: &mut sui::tx_context::TxContext,",
                    );
                }
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
            return_type.push_str(format!(": {}", nft_type_name).as_str());
            transfer.push_str("nft");

            args.push_str("        ctx: &mut sui::tx_context::TxContext,\n");

            let nft = format!(
                "
        let nft = {nft_type_name} {{
            id: sui::object::new(ctx),
            name,
            description,
            url: sui::url::new_unsafe_from_bytes(url),
            attributes: nft_protocol::attributes::from_vec(attribute_keys, attribute_values)
        }};"
            );

            code = format!(
                "\n
    fun mint(
{args}    ){return_type} {{
        {nft}
        nft_protocol::mint_event::emit_mint(
            ob_permissions::witness::from_witness(Witness {{}}),
            nft_protocol::mint_cap::collection_id(mint_cap),
            &nft,
        );
        nft_protocol::mint_cap::increment_supply(mint_cap, 1);
        {transfer}
    }}"
            );
        }

        code
    }

    pub fn write_mint_fns(&self, nft_type_name: &str) -> String {
        let mut mint_fns = String::new();

        if self.launchpad {
            mint_fns.push_str(
                &self.write_mint_fn(Some(MintType::Launchpad), nft_type_name),
            );
        }

        if self.airdrop {
            mint_fns.push_str(
                &self.write_mint_fn(Some(MintType::Airdrop), nft_type_name),
            );
        }

        mint_fns.push_str(&self.write_mint_fn(None, nft_type_name));

        mint_fns
    }
}
