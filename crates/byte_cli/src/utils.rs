use anyhow::{anyhow, Result};
use gutenberg::{models::settings::SupplyPolicy, Schema};

pub fn assert_no_unstable_features(schema: &Schema) -> Result<()> {
    if schema.settings.composability.is_some() {
        return Err(anyhow!("Composability feature is currently unstable and therefore not supported by the CLI."));
    }
    if schema.settings.loose {
        return Err(anyhow!("NFT Looseness feature is currently unstable and therefore not supported by the CLI."));
    }
    if let SupplyPolicy::Limited { .. } = schema.settings.supply_policy {
        return Err(anyhow!("Limited supply feature is currently unstable and therefore not supported by the CLI."));
    }
    if schema.settings.mint_policies.direct {
        return Err(anyhow!("Direct minting feature is currently unstable and therefore not supported by the CLI."));
    }

    Ok(())
}
