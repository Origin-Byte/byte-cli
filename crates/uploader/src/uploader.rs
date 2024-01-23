use anyhow::{anyhow, Result};
use async_trait::async_trait;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{self, AsyncReadExt, BufReader},
    task::{JoinHandle, JoinSet},
};

use indexmap::IndexMap;
use rust_sdk::metadata::{GlobalMetadata, Metadata};
use serde::{Deserialize, Serialize};

/// Maximum number of concurrent tasks (this is important for tasks that handle
/// files and network connections).
pub const PARALLEL_LIMIT: usize = 45;

/// Represents the outcome of an asset upload process.
pub enum UploadEffects {
    Success(UploadedAsset),
    Failure(u32),
}

/// Structure representing an item with a hash and a link.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Item {
    pub hash: String,
    pub link: String,
}

/// Holds the state of storage including the batch pointer and items.
#[derive(Debug, Default)]
pub struct StorageState {
    pub batch_pointer: u64,
    pub uploaded_items: StorageItems,
    pub missed_items: IndexMap<String, u64>,
}

/// Represents a collection of storage items.
#[derive(Debug, Default)]
pub struct StorageItems(pub IndexMap<String, Item>);

/// Asynchronously uploads data.
///
/// # Arguments
/// * `assets` - The assets to upload.
/// * `metadata` - The global metadata.
/// * `uploader` - The uploader instance.
///
/// # Returns
/// A result indicating success or error.
pub async fn upload_data(
    assets: &mut Vec<Asset>,
    metadata: Arc<GlobalMetadata>,
    uploader: &dyn Uploader,
) -> Result<()> {
    uploader.upload(assets, metadata).await?;

    Ok(())
}

/// Attempts to read the state from a given path.
///
/// # Arguments
/// * `path_buf` - The file path to read from.
///
/// # Returns
/// A result containing metadata or an error.
pub fn try_read_state(path_buf: &PathBuf) -> Result<Metadata> {
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

/// Asynchronously writes the state to a file.
///
/// # Arguments
/// * `state` - The metadata to write.
/// * `output_file` - The file path to write to.
///
/// # Returns
/// A result indicating success or an error.
pub async fn write_state_file(
    state: &Metadata,
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

/// Represents an asset with its details.
#[derive(Debug)]
pub struct Asset {
    /// Id of the asset
    pub index: u32,
    /// Id of the asset
    pub name: String,
    /// File path of the asset
    pub path: PathBuf,
    /// MIME content type.
    pub content_type: String,
}

impl Asset {
    /// Constructs a new `Asset`.
    ///
    /// # Arguments
    /// * `index` - The index of the asset.
    /// * `name` - The name of the asset.
    /// * `path` - The file path of the asset.
    /// * `content type` - The MIME type of the asset.
    ///
    /// # Returns
    /// A new instance of `Asset`.
    pub fn new(
        index: u32,
        name: String,
        path: PathBuf,
        content_type: String,
    ) -> Self {
        Asset {
            index,
            name,
            path,
            content_type,
        }
    }
}

#[derive(Debug)]
pub struct UploadedAsset {
    /// Id of the asset
    pub index: u32,
    /// Link of the asset
    pub link: String,
}

impl UploadedAsset {
    /// Constructs a new `UploadedAsset`.
    ///
    /// # Arguments
    /// * `index` - The index of the asset.
    /// * `link` - The URL link to the uploaded asset.
    ///
    /// # Returns
    /// A new instance of `UploadedAsset`.
    pub fn new(index: u32, link: String) -> Self {
        UploadedAsset { index, link }
    }
}

/// Trait for preparing assets for upload.
#[async_trait]
pub trait Prepare {
    /// Asynchronously prepares a set of assets for upload.
    ///
    /// # Arguments
    /// * `assets` - A slice of assets to be prepared.
    ///
    /// # Returns
    /// A result indicating the success or failure of the operation.
    async fn prepare(&self, assets: &[Asset]) -> Result<()>;
}

/// Trait defining functionality for uploading assets.
#[async_trait]
pub trait Uploader: Prepare {
    /// Asynchronously uploads a set of assets.
    ///
    /// # Arguments
    /// * `assets` - A mutable reference to a vector of assets to be uploaded.
    /// * `nft_data` - An `Arc` pointing to `GlobalMetadata`.
    ///
    /// # Returns
    /// A result containing a vector of strings or an error.
    async fn upload(
        &self,
        assets: &mut Vec<Asset>,
        nft_data: Arc<GlobalMetadata>,
    ) -> Result<Vec<String>>;
}

/// Trait for uploading assets in parallel.
#[async_trait]
pub trait ParallelUploader: Uploader + Send + Sync {
    /// Spawns a task for uploading a single asset.
    ///
    /// # Arguments
    /// * `asset` - The asset to be uploaded.
    /// * `metadata` - An `Arc` pointing to `GlobalMetadata`.
    /// * `terminate_flag` - An `Arc` pointing to an `AtomicBool` to signal
    ///   termination.
    ///
    /// # Returns
    /// A `JoinHandle` that resolves to a result of `UploadEffects`.
    fn upload_asset(
        &self,
        asset: Asset,
        metadata: Arc<GlobalMetadata>,
        terminate_flag: Arc<AtomicBool>,
    ) -> JoinHandle<Result<UploadEffects>>;
}

/// Default implementation of the trait ['Uploader'](Uploader) for all
/// ['ParallelUploader'](ParallelUploader).
#[async_trait]
impl<T: ParallelUploader> Uploader for T {
    /// Uploads assets in parallel. It creates [`self::parallel_limit()`] tasks
    /// at a time to avoid reaching the limit of concurrent files open and
    /// it syncs the cache file at every `self.parallel_limit() / 2` step.
    ///
    /// It manages the execution of upload tasks in parallel, handling
    /// termination flags and errors.
    ///
    /// # Arguments
    /// * `assets` - A mutable reference to a vector of assets to be uploaded.
    /// * `metadata` - An `Arc` pointing to `GlobalMetadata`.
    ///
    /// # Returns
    /// A result containing a vector of error logs, if any.
    async fn upload(
        &self,
        assets: &mut Vec<Asset>,
        metadata: Arc<GlobalMetadata>,
    ) -> Result<Vec<String>> {
        let mut set = JoinSet::new();
        let mut error_logs = vec![];
        let terminate_flag = Arc::new(AtomicBool::new(false));

        // Create a new progress bar
        let progress_bar = ProgressBar::new(assets.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .progress_chars("#>-"),
        );

        progress_bar.inc(0);

        let flag = terminate_flag.clone();

        // Start a task to wait for keyboard input
        let keyboard_handle = tokio::spawn(async move {
            let mut stdin = BufReader::new(io::stdin());
            let mut buffer = [0u8; 1];

            while !flag.load(Ordering::SeqCst) {
                let bytes_read = stdin.read_exact(&mut buffer).await;

                match bytes_read {
                    Ok(_) => {
                        // User entered 'q' or 'Q', exit the loop gracefully
                        if buffer[0] == b'q' || buffer[0] == b'Q' {
                            // Switch the value to true
                            flag.store(true, Ordering::SeqCst);

                            // Need to wait a second before prompting this
                            // message to the user
                            tokio::time::sleep(Duration::from_millis(1000))
                                .await;
                            println!("Exiting upload process. This might take a minute..");
                            break;
                        }
                    }
                    Err(err) => {
                        eprintln!("Error reading input: {}", err);
                        // Handle or report the error as needed
                        break;
                    }
                }
            }
        });

        for asset in assets.drain(..) {
            if terminate_flag.load(Ordering::SeqCst) {
                break; // Terminate the loop if terminate_flag is true
            }

            set.spawn(self.upload_asset(
                asset,
                metadata.clone(),
                terminate_flag.clone(),
            ));
        }

        while let Some(res) = set.join_next().await {
            match res.unwrap().unwrap() {
                Ok(result) => match result {
                    UploadEffects::Success(_) => progress_bar.inc(1),
                    UploadEffects::Failure(index) => error_logs
                        .push(format!("Skipped upload of image #{}", index)),
                },
                Err(error) => {
                    error_logs.push(format!("{:?}", error));
                }
            }
        }

        // Set the terminate_flag to true after all processes have completed
        terminate_flag.store(true, Ordering::SeqCst);
        keyboard_handle.abort();

        // Finish the progress bar
        progress_bar.finish_at_current_pos();

        Ok(error_logs)
    }
}
