mod burn;
mod dynamic;
mod fields;
mod mint_cap;
mod minting;
mod orderbook;

use crate::{DefArgs, InitArgs, MoveDefs, MoveInit, MoveTests, TestArgs};
use gutenberg_types::models::{
    collection::{CollectionData, Tags},
    nft::NftData,
};

// TODO: Merge `cfg(feature = "full")` and `cfg(not(feature = "full"))` definitions, requires manually
// implementing derives

impl MoveInit for NftData {
    fn write_move_init(&self, args: InitArgs) -> String {
        let collection_data = init_args(args);

        let witness = &self.witness_name();
        let type_name = &self.type_name();

        let collection_init = collection_data
            .write_move_init(InitArgs::CollectionData { type_name });

        let transfer_fns = write_move_transfer_fns(self);

        // Write MintCap instantiation
        //
        // If using non-full version of Gutenberg, a MintCap with supply
        // limited to 100 will always be instantiated

        let mint_cap_init = self
            .mint_cap
            .write_move_init(InitArgs::MintCap { witness, type_name });

        let mut misc_init = String::new();
        misc_init.push_str(&write_move_display(self, collection_data.tags()));
        misc_init.push_str(&write_move_policies(
            self,
            collection_data.has_royalties(),
        ));
        misc_init.push_str(
            &self
                .orderbook
                .map(|orderbook| orderbook.write_move_init(type_name))
                .unwrap_or_default(),
        );

        format!(
            "

    fun init(witness: {witness}, ctx: &mut sui::tx_context::TxContext) {{
        let delegated_witness = ob_permissions::witness::from_witness(Witness {{}});{collection_init}{mint_cap_init}

        let publisher = sui::package::claim(witness, ctx);{misc_init}

        sui::transfer::public_transfer(publisher, sui::tx_context::sender(ctx));{transfer_fns}
    }}"
        )
    }
}

impl MoveDefs for NftData {
    fn write_move_defs(
        &self,
        args: DefArgs,
        // collection_data: &CollectionData,
    ) -> String {
        let collection_data = def_args(args);

        let fields = self.fields();
        let type_name = &self.type_name();
        let requires_collection = collection_data.requires_collection();

        let mut defs_str = String::new();
        defs_str.push_str(&write_move_struct(self));
        defs_str.push_str(
            &self.write_move_init(InitArgs::NftData { collection_data }),
        );
        defs_str.push_str(&self.mint_policies.write_move_defs(
            DefArgs::MintPolicies {
                fields,
                type_name,
                requires_collection,
            },
        ));
        defs_str.push_str(
            &self
                .dynamic
                .write_move_defs(DefArgs::Dynamic { fields, type_name }),
        );
        defs_str.push_str(
            &self
                .burn
                .map(|burn| {
                    burn.write_move_defs(DefArgs::Burn {
                        fields,
                        type_name,
                        requires_collection,
                        requires_listing: self.mint_policies.has_launchpad(),
                        requires_confirm: !self.request_policies.has_withdraw(),
                    })
                })
                .unwrap_or_default(),
        );

        defs_str
    }
}

impl MoveTests for NftData {
    fn write_move_tests(
        &self,
        args: TestArgs,
        // collection_data: &CollectionData
    ) -> String {
        let collection_data = test_args(args);

        let fields = self.fields();
        let type_name = &self.type_name();
        let witness_name = &self.witness_name();
        let requires_collection = collection_data.requires_collection();

        let mut tests_str = String::new();
        tests_str.push_str(&self.mint_policies.write_move_tests(
            TestArgs::MintPolicies {
                fields,
                type_name,
                witness_name,
                requires_collection,
            },
        ));
        tests_str.push_str(&self.dynamic.write_move_tests(TestArgs::Dynamic {
            fields,
            type_name,
            witness_name,
            requires_collection,
        }));
        tests_str.push_str(
            &self
                .burn
                .map(|burn| {
                    burn.write_move_tests(TestArgs::Burn {
                        fields,
                        type_name,
                        witness_name,
                        requires_collection,
                    })
                })
                .unwrap_or_default(),
        );
        tests_str.push_str(
            &self
                .orderbook
                .map(|orderbook| {
                    orderbook.write_move_tests(
                        fields,
                        &type_name,
                        &witness_name,
                        requires_collection,
                        collection_data.has_royalties(),
                    )
                })
                .unwrap_or_default(),
        );
        tests_str
    }
}

fn write_move_struct(data: &NftData) -> String {
    let type_name = data.type_name();
    let fields: String = data
        .fields()
        .iter()
        .map(|field| {
            format!(
                "
    {name}: {field_type},",
                name = field.name(),
                field_type = field.field_type().write_move_init(InitArgs::None)
            )
        })
        .collect();

    format!(
        "

struct {type_name} has key, store {{
    id: sui::object::UID,{fields}
}}"
    )
}

fn write_move_policies(data: &NftData, has_royalties: bool) -> String {
    let type_name = data.type_name();

    let mut policies_str = String::new();

    if data.requires_transfer() {
        let royalty_strategy_str = has_royalties.then_some("
    nft_protocol::royalty_strategy_bps::enforce(&mut transfer_policy, &transfer_policy_cap);").unwrap_or_default();

        policies_str.push_str(&format!(
            "

    let (transfer_policy, transfer_policy_cap) = ob_request::transfer_request::init_policy<{type_name}>(
        &publisher, ctx,
    );{royalty_strategy_str}
    nft_protocol::transfer_allowlist::enforce(&mut transfer_policy, &transfer_policy_cap);"
        ));
    }

    if data.requires_borrow() {
        policies_str.push_str(&format!(
            "

    let (borrow_policy, borrow_policy_cap) = ob_request::borrow_request::init_policy<{type_name}>(
        &publisher, ctx,
    );"
        ));
    }

    if data.requires_withdraw() {
        policies_str.push_str(&format!(
            "

    let (withdraw_policy, withdraw_policy_cap) = ob_request::withdraw_request::init_policy<{type_name}>(
        &publisher, ctx,
    );"
        ));

        // When `NftData` requires a withdraw policy we must be careful to
        // protect it such that a malicious actor may not withdraw
        // arbitrarily
        if !data.request_policies.has_withdraw() {
            policies_str.push_str(&format!(
                "
    ob_request::request::enforce_rule_no_state<ob_request::request::WithNft<{type_name}, ob_request::withdraw_request::WITHDRAW_REQ>, Witness>(
        &mut withdraw_policy, &withdraw_policy_cap,
    );"
            ));
        }
    }

    policies_str
}

fn write_move_transfer_fns(data: &NftData) -> String {
    let mut code = String::new();

    if data.requires_transfer() {
        code.push_str(
            "

    sui::transfer::public_transfer(transfer_policy_cap, sui::tx_context::sender(ctx));
    sui::transfer::public_share_object(transfer_policy);"
        );
    }

    if data.requires_withdraw() {
        code.push_str(
            "

    sui::transfer::public_transfer(withdraw_policy_cap, sui::tx_context::sender(ctx));
    sui::transfer::public_share_object(withdraw_policy);"
        );
    }

    if data.requires_borrow() {
        code.push_str(
            "

    sui::transfer::public_transfer(borrow_policy_cap, sui::tx_context::sender(ctx));
    sui::transfer::public_share_object(borrow_policy);"
        );
    }

    code
}

fn write_move_display(data: &NftData, tags: &Option<Tags>) -> String {
    let type_name = data.type_name();

    let tags_str = tags.as_ref().map(|tags| {
        let tags_str = tags.write_move_init(InitArgs::None);
        format!("{tags_str}

    sui::display::add(&mut display, std::string::utf8(b\"tags\"), ob_utils::display::from_vec(tags));")
    }).unwrap_or_default();

    format!("

    let display = sui::display::new<{type_name}>(&publisher, ctx);
    sui::display::add(&mut display, std::string::utf8(b\"name\"), std::string::utf8(b\"{{name}}\"));
    sui::display::add(&mut display, std::string::utf8(b\"description\"), std::string::utf8(b\"{{description}}\"));
    sui::display::add(&mut display, std::string::utf8(b\"image_url\"), std::string::utf8(b\"{{url}}\"));
    sui::display::add(&mut display, std::string::utf8(b\"attributes\"), std::string::utf8(b\"{{attributes}}\"));{tags_str}
    sui::display::update_version(&mut display);

    sui::transfer::public_transfer(display, sui::tx_context::sender(ctx));"
    )
}

fn def_args(args: DefArgs) -> &CollectionData {
    match args {
        DefArgs::NftData { collection_data } => collection_data,
        _ => panic!("Incorrect DefArgs variant"),
    }
}

fn test_args(args: TestArgs) -> &CollectionData {
    match args {
        TestArgs::NftData { collection_data } => collection_data,
        _ => panic!("Incorrect TestArgs variant"),
    }
}

fn init_args(args: InitArgs) -> &CollectionData {
    match args {
        InitArgs::NftData { collection_data } => collection_data,
        _ => panic!("Incorrect InitArgs variant"),
    }
}
