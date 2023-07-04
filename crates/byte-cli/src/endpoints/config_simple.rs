use crate::models::project::Project;
use convert_case::{Case, Casing};
use gutenberg::models::{
    collection::Supply,
    nft::{Burn, Dynamic, MintCap, Orderbook},
};
use gutenberg::{
    models::{
        collection::CollectionData,
        nft::{FieldType, MintPolicies, NftData, RequestPolicies},
    },
    Schema,
};
use package_manager::Address;

pub async fn init_schema(
    name: &String,
) -> Result<(Schema, Project), anyhow::Error> {
    use crate::models::FromPrompt;

    let keystore = rust_sdk::utils::get_keystore().await?;
    let sender = rust_sdk::utils::get_active_address(&keystore)?;
    let sender_string = Address::new(&sender.to_string())?;

    let nft_type = name.as_str().to_case(Case::Pascal);

    let project = Project::new(name.clone(), sender);

    let mut royalties = crate::models::royalties::get_policy_type()?;
    royalties.add_beneficiary_vecs(&[sender_string], &[10000]);

    let collection_data = CollectionData::new(
        Some(name.clone()),
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
        Some(Burn::Permissionless),
        Dynamic::new(false),
        MintCap::from_prompt(())?,
        MintPolicies::new(true, true),
        RequestPolicies::new(true, false, false),
        Some(Orderbook::Unprotected),
        vec![
            ("name", FieldType::String),
            ("description", FieldType::String),
            ("url", FieldType::Url),
            ("attributes", FieldType::Attributes),
        ]
        .into(),
    );

    Ok((
        Schema::new(name.clone(), collection_data, nft_data),
        project,
    ))
}