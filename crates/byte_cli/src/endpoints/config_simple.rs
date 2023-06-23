use crate::models::project::Project;
use convert_case::{Case, Casing};
#[cfg(feature = "full")]
use gutenberg::models::{
    collection::Supply,
    nft::{Burn, Dynamic, MintCap, Orderbook},
};
use gutenberg::{
    models::{
        collection::CollectionData,
        nft::{MintPolicies, NftData, RequestPolicies},
    },
    Schema,
};

#[cfg(feature = "full")]
pub async fn init_schema(
    name: &String,
) -> Result<(Schema, Project), anyhow::Error> {
    use crate::models::FromPrompt;

    let keystore = rust_sdk::utils::get_keystore().await?;
    let sender = rust_sdk::utils::get_active_address(&keystore)?;
    let sender_string = gutenberg::models::Address::new(sender.to_string())?;

    let nft_type = name.as_str().to_case(Case::Pascal);

    let project = Project::new(name.clone(), sender);

    let mut royalties = crate::models::royalties::get_policy_type()?;
    royalties.add_beneficiary_vecs(&[sender_string], &[10000]);

    let collection_data = CollectionData::new(
        name.clone(),
        None,
        None,
        None,
        Vec::new(),
        Supply::Untracked,
        Some(royalties),
        None,
    );

    let nft_data = NftData::new(
        nft_type,
        Burn::Permissionless,
        Dynamic::new(false),
        MintCap::from_prompt(())?,
        MintPolicies::new(true, true),
        RequestPolicies::new(true, false, false),
        Orderbook::Unprotected,
    );

    Ok((Schema::new(collection_data, nft_data), project))
}

#[cfg(not(feature = "full"))]
pub async fn init_schema(
    name: &String,
) -> Result<(Schema, Project), anyhow::Error> {
    let keystore = rust_sdk::utils::get_keystore().await?;
    let sender = rust_sdk::utils::get_active_address(&keystore)?;

    let nft_type = name.as_str().to_case(Case::Pascal);
    let project = Project::new(name.clone(), sender);

    let collection_data = CollectionData::new(
        Some(name.to_lowercase()),
        None,
        None,
        None,
        Vec::new(),
        None,
    );

    let nft_data = NftData::new(
        nft_type,
        MintPolicies::new(true, true),
        RequestPolicies::new(true, false, false),
    );

    Ok((Schema::new(collection_data, nft_data), project))
}
