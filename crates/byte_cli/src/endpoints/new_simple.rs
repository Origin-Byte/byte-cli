use std::collections::BTreeSet;

use byte_cli::models::project::Project;
use convert_case::{Case, Casing};
use gutenberg::{
    models::{
        collection::{
            CollectionData, MintCap, Orderbook, RequestPolicies, RoyaltyPolicy,
            Share, Supply,
        },
        nft::{Burn, MintPolicies, NftData},
        Address,
    },
    Schema,
};

pub async fn init_schema(
    name: &String,
    supply: usize,
    royalty_bps: usize,
) -> Result<(Schema, Project), anyhow::Error> {
    let keystore = rust_sdk::utils::get_keystore().await?;
    let sender = rust_sdk::utils::get_active_address(&keystore)?;
    let sender_string = Address::new(sender.to_string())?;

    let nft_type = name.as_str().to_case(Case::Pascal);

    let project = Project::new(name.clone(), sender);

    let royalties = Some(RoyaltyPolicy::new(
        BTreeSet::from([Share::new(sender_string, 10_000)]),
        royalty_bps as u64,
    ));

    let schema = Schema::new(
        CollectionData::new(
            name.clone(),
            None,
            None,
            None,
            vec![],
            Supply::Untracked,
            MintCap::new(Some(supply as u64)),
            royalties,
            None,
            RequestPolicies::new(true, false, false),
            Orderbook::Protected,
        ),
        NftData::new(
            nft_type,
            Burn::Permissionless,
            false,
            MintPolicies::new(true, true),
        ),
    );

    Ok((schema, project))
}
