use crate::models::{project::Project, FromPrompt};
use byte_cli::SchemaBuilder;
use console::style;
use gutenberg_types::models::{collection::CollectionData, nft::NftData};

pub async fn init_collection_config(
    mut builder: SchemaBuilder,
) -> Result<(SchemaBuilder, Project), anyhow::Error> {
    println!("{}",
        style("Welcome to Byte CLI! We're ready to begin setting up your NFT collection.").blue().bold().dim()
    );

    println!(
        "{}",
        style("To begin, let's configure some collection level metadata.")
            .blue()
            .bold()
            .dim()
    );

    builder.collection = Some(CollectionData::from_prompt(())?);

    let keystore = rust_sdk::utils::get_keystore().await?;
    let sender = rust_sdk::utils::get_active_address(&keystore)?;
    let project = Project::new(
        builder.collection.as_ref().unwrap().name().unwrap(),
        sender,
    );

    println!(
        "{}",
        style("Let us now configure some NFT level metadata.")
            .blue()
            .bold()
            .dim()
    );

    builder.nft = Some(NftData::from_prompt(())?);

    println!(
        "{}",
        style("Congrats! The collection has been configured.")
            .blue()
            .bold()
            .dim()
    );

    Ok((builder, project))
}
