use crate::models::FromPrompt;
use byte_cli::SchemaBuilder;
use console::style;
use gutenberg_types::models::{collection::CollectionData, nft::NftData};
use rust_sdk::models::project::Project;

/// Asynchronously initializes the configuration for an NFT collection.
///
/// # Arguments
/// * `builder` - A mutable SchemaBuilder object to configure the collection and NFT metadata.
///
/// # Returns
/// A Result tuple containing the updated SchemaBuilder and Project, or an error.
///
/// # Functionality
/// - Welcomes the user and guides them through setting up collection level metadata.
/// - Obtains collection data from the user using interactive prompts.
/// - Sets up the project with the collection name and sender address.
/// - Guides the user to configure NFT level metadata.
/// - Finalizes the configuration and congratulates the user.
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
