use std::collections::BTreeSet;

use crate::{consts::BPS_100_PERCENT, models::FromPrompt};

use byte_cli::models::project::Project;
use console::style;
use convert_case::{Case, Casing};
use gutenberg::{
    models::{
        collection::CollectionData,
        nft::{burn::Burn, NftData},
        settings::{
            royalties::Share, MintPolicies, Orderbook, RequestPolicies,
            RoyaltyPolicy, Settings,
        },
        Address,
    },
    schema::SchemaBuilder,
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
        BTreeSet::from([Share::new(sender_string, BPS_100_PERCENT)]),
        royalty_bps as u64,
    ));

    let schema = Schema::new(
        CollectionData::new(name.clone(), None, None, None, vec![], None),
        NftData::new(nft_type, Burn::Permissionless, false),
        Settings::new(
            royalties,
            MintPolicies::new(Some(supply as u64), true, true),
            RequestPolicies::new(true, false, false),
            None,
            Orderbook::Protected,
        ),
    );

    Ok((schema, project))
}
