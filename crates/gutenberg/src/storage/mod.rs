use std::{
    fs::File,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub mod aws;
pub mod nft_storage;
pub mod pinata;
pub mod uploader;

pub use aws::*;
pub use nft_storage::*;
pub use pinata::*;
pub use uploader::*;

use anyhow::{anyhow, Result};

use rust_sdk::mint::NftData as NftState;

#[derive(Debug, Deserialize, Serialize)]
pub enum Storage {
    Aws(AWSConfig),
    Pinata(PinataConfig),
    NftStorage(NftStorageConfig),
    // Bundlr(BundlrConfig),
    // Shdw(ShdwConfig),
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Item {
    #[serde(default = "String::default")]
    pub hash: String,
    pub link: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct StorageState {
    pub batch_pointer: u64,
    pub uploaded_items: StorageItems,
    pub missed_items: IndexMap<String, u64>,
}

// impl StorageState {
//     pub fn sync_state(&mut self, )
// }

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct StorageItems(pub IndexMap<String, Item>);

pub async fn upload_data(
    assets: &mut Vec<Asset>,
    state: PathBuf,
    lazy: bool,
    uploader: &dyn Uploader,
) -> Result<()> {
    uploader.upload(assets, state, lazy).await?;

    Ok(())
}

pub async fn write_state(
    mut state_path: PathBuf,
    mut file_name: String,
    url: String,
) -> Result<()> {
    file_name.push_str(".json");
    state_path.push(file_name);

    let mut state = try_read_state(&state_path)?;

    state.url = Some(url);

    write_state_file(&state, &state_path.as_path()).await?;

    Ok(())
}

pub fn try_read_state(path_buf: &PathBuf) -> Result<NftState> {
    let f = File::open(path_buf);

    let data = match f {
        Ok(file) => match serde_json::from_reader(file) {
            Ok(data) => Ok(data),
            Err(err) => Err(anyhow!("The following error has occurred while reading an NFT metadata object: {},", err)),
        },
        Err(err) => Err(anyhow!("The following error has occurred while reading an NFT metadata object: {},", err)),
    }?;

    Ok(data)
}

pub async fn write_state_file(
    state: &NftState,
    output_file: &Path,
) -> Result<(), anyhow::Error> {
    let file = File::create(output_file).map_err(|err| {
        anyhow!(
            r#"Could not create configuration file "{}": {err}"#,
            output_file.display()
        )
    })?;

    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let ser = &mut serde_json::Serializer::with_formatter(file, formatter);
    state.serialize(ser).map_err(|err| {
        anyhow!(
            r#"Could not write configuration file "{}": {err}"#,
            output_file.display()
        )
    })
}
