use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

use crate::{err::GutenError, models::collection::CollectionData};

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Burn {
    None,
    Permissioned,
    Permissionless,
}

impl Default for Burn {
    /// Not being able to burn NFTs is a sensible default as it does not introduce
    /// any potential attack vectors against a creator's collection and burn
    /// funcitons can be introduced via a contract upgrade.
    fn default() -> Self {
        Burn::None
    }
}

impl Display for Burn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Burn::None => "None",
            Burn::Permissioned => "Permissioned",
            Burn::Permissionless => "Permissionless",
        };

        f.write_str(string)
    }
}

impl FromStr for Burn {
    type Err = GutenError;

    fn from_str(level: &str) -> Result<Burn, Self::Err> {
        match level {
            "None" => Ok(Burn::None),
            "Permissioned" => Ok(Burn::Permissioned),
            "Permissionless" => Ok(Burn::Permissionless),
            level => Err(GutenError::UnsupportedSettings(
                format!("Burn level of `{level}` is unsupported. Supported levels include: [`None`, `Permissioned`, `Permissionless`].")
            ))
        }
    }
}

impl Burn {
    pub fn is_none(&self) -> bool {
        matches!(self, Burn::None)
    }

    pub fn is_permissioned(&self) -> bool {
        matches!(self, Burn::Permissioned)
    }

    pub fn is_permissionless(&self) -> bool {
        matches!(self, Burn::Permissionless)
    }

    pub fn write_move_defs(
        &self,
        nft_type_name: &str,
        collection_data: &CollectionData,
    ) -> String {
        let mut code = String::new();

        if let Burn::None = self {
            return code;
        }

        let supply = collection_data.supply();
        let decrements = supply.requires_collection();

        let mut_str = decrements.then_some("mut ").unwrap_or_default();

        let collection_decrement_str = decrements
            .then(|| supply.write_move_decrement())
            .unwrap_or_default();

        code.push_str(&format!(
            "

    public fun burn_nft(
        delegated_witness: ob_permissions::witness::Witness<{nft_type_name}>,
        collection: &{mut_str}nft_protocol::collection::Collection<{nft_type_name}>,
        nft: {nft_type_name},
    ) {{
        let guard = nft_protocol::mint_event::start_burn(delegated_witness, &nft);
        let {nft_type_name} {{ id, name: _, description: _, url: _, attributes: _ }} = nft;
        nft_protocol::mint_event::emit_burn(guard, sui::object::id(collection), id);{collection_decrement_str}
    }}"
        ));

        code.push_str(&format!(
            "

    public entry fun burn_nft_in_listing(
        publisher: &sui::package::Publisher,
        collection: &{mut_str}nft_protocol::collection::Collection<{nft_type_name}>,
        listing: &mut ob_launchpad::listing::Listing,
        inventory_id: sui::object::ID,
        ctx: &mut sui::tx_context::TxContext,
    ) {{
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        let nft = ob_launchpad::listing::admin_redeem_nft(listing, inventory_id, ctx);
        burn_nft(delegated_witness, collection, nft);
    }}"
        ));

        code.push_str(&format!(
            "

    public entry fun burn_nft_in_listing_with_id(
        publisher: &sui::package::Publisher,
        collection: &{mut_str}nft_protocol::collection::Collection<{nft_type_name}>,
        listing: &mut ob_launchpad::listing::Listing,
        inventory_id: sui::object::ID,
        nft_id: sui::object::ID,
        ctx: &mut sui::tx_context::TxContext,
    ) {{
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        let nft = ob_launchpad::listing::admin_redeem_nft_with_id(listing, inventory_id, nft_id, ctx);
        burn_nft(delegated_witness, collection, nft);
    }}"
        ));

        if self == &Burn::Permissionless {
            code.push_str(&format!(
            "

    public entry fun burn_own_nft(
        collection: &{mut_str}nft_protocol::collection::Collection<{nft_type_name}>,
        nft: {nft_type_name},
    ) {{
        let delegated_witness = ob_permissions::witness::from_witness(Witness {{}});
        burn_nft(delegated_witness, collection, nft);
    }}

    public entry fun burn_own_nft_in_kiosk(
        collection: &{mut_str}nft_protocol::collection::Collection<{nft_type_name}>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<{nft_type_name}, ob_request::withdraw_request::WITHDRAW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {{
        let (nft, withdraw_request) = ob_kiosk::ob_kiosk::withdraw_nft_signed(kiosk, nft_id, ctx);
        ob_request::withdraw_request::add_receipt(&mut withdraw_request, &Witness {{}});
        ob_request::withdraw_request::confirm(withdraw_request, policy);

        burn_own_nft(collection, nft);
    }}"
        ));
        }

        code
    }

    pub fn write_move_tests(
        &self,
        type_name: &str,
        witness_name: &str,
        requires_collection: bool,
    ) -> String {
        match self {
            Burn::None => String::new(),
            Burn::Permissioned => String::new(),
            Burn::Permissionless => {
                let collection_param_str = requires_collection
                    .then_some(
                        "
            &mut collection,",
                    )
                    .unwrap_or_default();

                let collection_mut_str =
                    requires_collection.then_some("mut ").unwrap_or_default();

                format!(
                "

    #[test]
    fun it_burns_own_nft() {{
        let scenario = sui::test_scenario::begin(CREATOR);
        init({witness_name} {{}}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<{type_name}>>(
            &scenario,
            CREATOR,
        );

        let publisher = sui::test_scenario::take_from_address<sui::package::Publisher>(
            &scenario,
            CREATOR,
        );

        let collection = sui::test_scenario::take_shared<nft_protocol::collection::Collection<{type_name}>>(
            &scenario
        );

        let withdraw_policy = sui::test_scenario::take_shared<
            ob_request::request::Policy<
                ob_request::request::WithNft<{type_name}, ob_request::withdraw_request::WITHDRAW_REQ>
            >
        >(&scenario);

        let nft = mint(
            std::string::utf8(b\"TEST NAME\"),
            std::string::utf8(b\"TEST DESCRIPTION\"),
            b\"https://originbyte.io/\",
            vector[std::ascii::string(b\"avg_return\")],
            vector[std::ascii::string(b\"24%\")],
            &mut mint_cap,{collection_param_str}
            sui::test_scenario::ctx(&mut scenario)
        );
        let nft_id = sui::object::id(&nft);

        let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));
        ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, sui::test_scenario::ctx(&mut scenario));

        burn_own_nft_in_kiosk(
            &{collection_mut_str}collection,
            &mut kiosk,
            nft_id,
            &withdraw_policy,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::return_to_address(CREATOR, publisher);
        sui::test_scenario::return_shared(collection);
        sui::test_scenario::return_shared(withdraw_policy);
        sui::transfer::public_share_object(kiosk);
        sui::test_scenario::end(scenario);
    }}")
            }
        }
    }
}
