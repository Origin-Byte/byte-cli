use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Write},
    path::Path,
};

use package_manager::{
    package::{GitPath, Package},
    toml::MoveToml,
    version::Version,
    Address,
};

pub fn generate_manifest(package_name: String) -> MoveToml {
    MoveToml::new(
        Package::new(package_name.clone(), Version::new(0, 0, 0), None),
        BTreeMap::from([
            (
                "Launchpad".to_string(),
                GitPath::new(
                    "https://github.com/Origin-Byte/nft-protocol.git"
                        .to_string(),
                    Some("contracts/launchpad".to_string()),
                    "v1.3.0-mainnet".to_string(),
                ),
            ),
            (
                "NftProtocol".to_string(),
                GitPath::new(
                    "https://github.com/Origin-Byte/nft-protocol.git"
                        .to_string(),
                    Some("contracts/nft_protocol".to_string()),
                    "v1.3.0-mainnet".to_string(),
                ),
            ),
            (
                "LiquidityLayerV1".to_string(),
                GitPath::new(
                    "https://github.com/Origin-Byte/nft-protocol.git"
                        .to_string(),
                    Some("contracts/liquidity_layer_v1".to_string()),
                    "v1.3.0-mainnet".to_string(),
                ),
            ),
        ]),
        BTreeMap::from([(package_name, Address::zero())]),
    )
}

pub fn write_manifest(
    package_name: String,
    contract_dir: &Path,
) -> Result<(), io::Error> {
    let manifest = generate_manifest(package_name);

    let path = contract_dir.join("Move.toml");
    let mut file = File::create(path)?;
    file.write_all(manifest.to_string().unwrap().as_bytes())?;

    Ok(())
}
