//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.

pub mod royalties;
pub mod supply;
pub mod tags;

use crate::{InitArgs, MoveInit};
use gutenberg_types::models::collection::CollectionData;

impl MoveInit for CollectionData {
    fn write_move_init(&self, args: InitArgs) -> String {
        let type_name = init_args(args);

        let mut domains_str = String::new();

        domains_str.push_str(write_move_creators(self).as_str());
        domains_str.push_str(
            write_move_collection_display_info(self)
                .unwrap_or_default()
                .as_str(),
        );
        domains_str.push_str(
            write_move_collection_symbol(self)
                .unwrap_or_default()
                .as_str(),
        );
        domains_str.push_str(
            write_move_collection_url(self).unwrap_or_default().as_str(),
        );
        domains_str.push_str(&self.supply.write_move_init(InitArgs::None));
        domains_str.push_str(
            self.royalties
                .as_ref()
                .map(|royalties| royalties.write_move_init(InitArgs::None))
                .unwrap_or_default()
                .as_str(),
        );

        // Opt for `collection::create` over `collection::create_from_otw` in
        // order to statically assert `DelegatedWitness` gets created for the
        // `Collection<T>` type `T`.
        format!("

        let collection = nft_protocol::collection::create<{type_name}>(delegated_witness, ctx);
        let collection_id = sui::object::id(&collection);{domains_str}

        sui::transfer::public_share_object(collection);"
        )
    }
}

fn write_move_collection_display_info(data: &CollectionData) -> Option<String> {
    data.name().map(|name| {
        let description = data.description().unwrap_or_default();

        format!(
            "

    nft_protocol::collection::add_domain(
        delegated_witness,
        &mut collection,
        nft_protocol::display_info::new(
            std::string::utf8(b\"{name}\"),
            std::string::utf8(b\"{description}\"),
        ),
    );"
        )
    })
}

fn write_move_collection_url(data: &CollectionData) -> Option<String> {
    data.url().as_ref().map(|url| {
        format!(
            "

    nft_protocol::collection::add_domain(
        delegated_witness,
        &mut collection,
        sui::url::new_unsafe_from_bytes(b\"{url}\"),
    );"
        )
    })
}

fn write_move_collection_symbol(data: &CollectionData) -> Option<String> {
    data.symbol().as_ref().map(|symbol| {
        format!(
            "

    nft_protocol::collection::add_domain(
        delegated_witness,
        &mut collection,
        nft_protocol::symbol::new(std::string::utf8(b\"{symbol}\")),
    );",
        )
    })
}

// TODO: Separate out into `creators` module
fn write_move_creators(data: &CollectionData) -> String {
    let mut code = String::new();

    if !data.creators.is_empty() {
        code.push_str(
            "

    let creators = sui::vec_set::empty();",
        );
        for address in data.creators.iter() {
            code.push_str(&format!(
                "
    sui::vec_set::insert(&mut creators, @{address});"
            ));
        }

        code.push_str(
            "

    nft_protocol::collection::add_domain(
        delegated_witness,
        &mut collection,
        nft_protocol::creators::new(creators),
    );",
        );
    };

    code
}

fn init_args(args: InitArgs) -> &str {
    match args {
        InitArgs::CollectionData { type_name } => type_name,
        _ => panic!("Incorrect InitArgs variant"),
    }
}
