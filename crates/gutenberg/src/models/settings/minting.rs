use serde::{Deserialize, Serialize};

use crate::contract::modules::DisplayInfoMod;

pub enum MintType {
    Airdrop,
    Launchpad,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MintPolicies {
    pub supply: Option<u64>,
    pub launchpad: bool,
    pub airdrop: bool,
}

impl MintPolicies {
    pub fn new(supply: Option<u64>, launchpad: bool, airdrop: bool) -> Self {
        Self {
            supply,
            launchpad,
            airdrop,
        }
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
        let mut extra_fun = String::new();

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
                            "        warehouse: &mut ob_launchpad::warehouse::Warehouse<{}>,\n",
                            nft_type_name
                        )
                        .as_str(),
                    );
                    transfer.push_str(
                        "ob_launchpad::warehouse::deposit_nft(warehouse, nft);",
                    );
                    fun_name.push_str("mint_nft");
                    args.push_str(
                        "        ctx: &mut sui::tx_context::TxContext,",
                    );
                }
                MintType::Airdrop => {
                    args.push_str(
                        "        receiver: &mut sui::kiosk::Kiosk,\n",
                    );
                    transfer.push_str(
                        "ob_kiosk::ob_kiosk::deposit(receiver, nft, ctx);",
                    );
                    fun_name.push_str("airdrop_nft");
                    args.push_str(
                        "        ctx: &mut sui::tx_context::TxContext,",
                    );
                    extra_fun = self.write_airdrop_nft_into_new_kiosk(
                        nft_type_name,
                        &params,
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
    }}{extra_fun}"
            );
        } else {
            return_type.push_str(format!(": {}", nft_type_name).as_str());
            transfer.push_str("nft");

            args.push_str("        ctx: &mut sui::tx_context::TxContext,\n");

            let nft = format!(
                "let nft = {nft_type_name} {{
            id: sui::object::new(ctx),
            name,
            description,
            url: sui::url::new_unsafe_from_bytes(url),
            attributes: nft_protocol::attributes::from_vec(attribute_keys, attribute_values)
        }};"
            );

            code = format!(
                "

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

    pub fn write_airdrop_nft_into_new_kiosk(
        &self,
        nft_type_name: &str,
        params: &str,
    ) -> String {
        format!(
            "

    public entry fun airdrop_nft_into_new_kiosk(
        name: std::string::String,
        description: std::string::String,
        url: vector<u8>,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<{nft_type_name}>,
        receiver: address,
        ctx: &mut sui::tx_context::TxContext,
    ) {{
        let nft = mint(
{params}
        );

        let (kiosk, _) = ob_kiosk::ob_kiosk::new_for_address(receiver, ctx);
        ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, ctx);
        sui::transfer::public_share_object(kiosk);
    }}"
        )
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

    pub fn write_collection_create_with_mint_cap(
        &self,
        witness: &str,
        nft_type_name: &str,
    ) -> String {
        match self.supply {
            Some(supply) => format!(
"let (collection, mint_cap) = nft_protocol::collection::create_with_mint_cap<{witness}, {nft_type_name}>(
            &witness, std::option::some({supply}), ctx
        );"),
            None =>
            format!(
"let (collection, mint_cap) = nft_protocol::collection::create_with_mint_cap<{witness}, {nft_type_name}>(
            &witness, std::option::none(), ctx
        );")
        }
    }
}
