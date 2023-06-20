use serde::{Deserialize, Serialize};

use crate::models::collection::CollectionData;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MintPolicies {
    launchpad: bool,
    airdrop: bool,
}

impl Default for MintPolicies {
    fn default() -> Self {
        Self {
            launchpad: true,
            airdrop: true,
        }
    }
}

impl MintPolicies {
    pub fn new(launchpad: bool, airdrop: bool) -> Self {
        Self { launchpad, airdrop }
    }

    pub fn write_move_defs(
        &self,
        type_name: &str,
        collection_data: &CollectionData,
    ) -> String {
        let requires_collection = collection_data.requires_collection();

        let mut mint_fns = String::new();

        let mut base_params = vec!["name".to_string()];
        base_params.extend(self.params().into_iter());
        base_params.push("mint_cap".to_string());

        if requires_collection {
            base_params.push("collection".to_string());
        }

        let mut nft_params = base_params.clone();
        nft_params.push("ctx".to_string());

        let mut base_param_types = vec!["std::string::String".to_string()];
        base_param_types.extend(self.param_types().into_iter());
        base_param_types
            .push(format!("&mut nft_protocol::mint_cap::MintCap<{type_name}>"));

        if requires_collection {
            base_param_types.push(format!(
                "&mut nft_protocol::collection::Collection<{type_name}>"
            ));
        }

        let mut nft_param_types = base_param_types.clone();
        nft_param_types.push("&mut sui::tx_context::TxContext".to_string());

        // Mint NFT to Warehouse
        //
        // TODO: Mint NFT to Listing
        if self.launchpad {
            let mut params = base_params.clone();
            params.push("warehouse".to_string());
            params.push("ctx".to_string());

            let mut param_types = base_param_types.clone();
            param_types.push(format!(
                "&mut ob_launchpad::warehouse::Warehouse<{type_name}>"
            ));
            param_types.push("&mut sui::tx_context::TxContext".to_string());

            mint_fns.push_str(&write_move_fn(
                "mint_nft_to_warehouse",
                params.as_slice(),
                param_types.as_slice(),
                true,
                true,
                None,
                || {
                    format!(
                        "
        let nft = mint(
            {params_str},
        );

        ob_launchpad::warehouse::deposit_nft(warehouse, nft);",
                        params_str = nft_params.join(",\n            ")
                    )
                },
            ));
        }

        // Airdrop NFT into Kiosks
        if self.airdrop {
            // Write `mint_nft_to_kiosk`
            let mut params = base_params.clone();
            params.push("receiver".to_string());
            params.push("ctx".to_string());

            let mut param_types = base_param_types.clone();
            param_types.push("&mut sui::kiosk::Kiosk".to_string());
            param_types.push("&mut sui::tx_context::TxContext".to_string());

            mint_fns.push_str(&write_move_fn(
                "mint_nft_to_kiosk",
                params.as_slice(),
                param_types.as_slice(),
                true,
                true,
                None,
                || {
                    format!(
                        "
        let nft = mint(
            {params_str},
        );

        ob_kiosk::ob_kiosk::deposit(receiver, nft, ctx);",
                        params_str = nft_params.join(",\n            "),
                    )
                },
            ));

            // Write `mint_nft_to_new_kiosk`
            let mut params = base_params.clone();
            params.push("receiver".to_string());
            params.push("ctx".to_string());

            let mut param_types = base_param_types.clone();
            param_types.push("address".to_string());
            param_types.push("&mut sui::tx_context::TxContext".to_string());

            mint_fns.push_str(&write_move_fn(
                "mint_nft_to_new_kiosk",
                params.as_slice(),
                param_types.as_slice(),
                true,
                true,
                None,
                || {
                    format!(
                        "
        let nft = mint(
            {params_str},
        );

        let (kiosk, _) = ob_kiosk::ob_kiosk::new_for_address(receiver, ctx);
        ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, ctx);
        sui::transfer::public_share_object(kiosk);",
                        params_str = nft_params.join(",\n            ")
                    )
                },
            ));
        }

        mint_fns.push_str(&write_move_fn(
            "mint",
            nft_params.as_slice(),
            nft_param_types.as_slice(),
            false,
            false,
            Some(type_name.to_string()),
            || {
                let collection_increment_str = requires_collection
                    .then(|| {
                        #[cfg(feature = "full")]
                        let supply_str = collection_data.supply().write_move_increment();
                        #[cfg(not(feature = "full"))]
                        let supply_str = "";

                        supply_str
                    })
                    .unwrap_or_default();

                format!(
                    "
        let delegated_witness = ob_permissions::witness::from_witness(Witness {{}});

        let nft = {type_name} {{
            id: sui::object::new(ctx),
            name,
            description,
            url: sui::url::new_unsafe_from_bytes(url),
            attributes: nft_protocol::attributes::from_vec(attribute_keys, attribute_values)
        }};

        nft_protocol::mint_event::emit_mint(
            delegated_witness,
            nft_protocol::mint_cap::collection_id(mint_cap),
            &nft,
        );{collection_increment_str}

        nft_protocol::mint_cap::increment_supply(mint_cap, 1);

        nft",
                )
            },
        ));

        mint_fns
    }

    fn params(&self) -> Vec<String> {
        vec!["description", "url", "attribute_keys", "attribute_values"]
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    fn param_types(&self) -> Vec<String> {
        vec![
            "std::string::String",
            "vector<u8>",
            "vector<std::ascii::String>",
            "vector<std::ascii::String>",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }
}

fn write_move_fn<F>(
    name: &str,
    params: &[String],
    param_types: &[String],
    is_public: bool,
    is_entry: bool,
    returns: Option<String>,
    body_fn: F,
) -> String
where
    F: FnOnce() -> String,
{
    let is_public_str = match is_public {
        true => "public ",
        false => "",
    };

    let is_entry_str = match is_entry {
        true => "entry ",
        false => "",
    };

    let args_str = params
        .iter()
        .zip(param_types)
        .map(|(param, param_type)| format!("        {param}: {param_type},\n"))
        .collect::<Vec<String>>()
        .join("");

    let returns_str = returns
        .map(|returns| format!(": {returns}"))
        .unwrap_or_default();
    let body_str = body_fn();

    format!(
        "

    {is_public_str}{is_entry_str}fun {name}(
{args_str}    ){returns_str} {{{body_str}
    }}"
    )
}
