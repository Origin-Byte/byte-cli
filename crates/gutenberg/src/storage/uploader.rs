use std::{
    cmp,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use futures::future::select_all;
// use futures::stream::futures_unordered::FuturesUnordered;
use futures::StreamExt;
use std::sync::mpsc::{channel, Sender};
use tokio::task::{AbortHandle, JoinHandle, JoinSet};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::{thread, time};

use crate::err::GutenError;

use super::StorageState;

/// Maximum number of concurrent tasks (this is important for tasks that handle files
/// and network connections).
pub const PARALLEL_LIMIT: usize = 45;

#[derive(Debug)]
pub struct Asset {
    /// Id of the asset
    pub id: String,
    /// File path of the asset
    pub path: PathBuf,
    /// MIME content type.
    pub content_type: String,
}

impl Asset {
    pub fn new(id: String, path: PathBuf, content_type: String) -> Self {
        Asset {
            id,
            path,
            content_type,
        }
    }
}

pub struct UploadedAsset {
    /// Id of the asset
    pub id: String,
    /// Link of the asset
    pub link: String,
}

impl UploadedAsset {
    pub fn new(id: String, link: String) -> Self {
        UploadedAsset { id, link }
    }
}

#[async_trait]
pub trait Prepare {
    async fn prepare(&self, assets: &Vec<Asset>) -> Result<()>;
}

#[async_trait]
pub trait Uploader {
    async fn upload(
        &self,
        assets: &mut Vec<Asset>,
        state: &mut StorageState,
        lazy: bool,
    ) -> Result<()>;
}

#[async_trait]
pub trait ParallelUploader: Uploader + Send + Sync {
    fn upload_asset(
        &self,
        // set: &mut JoinSet<()>,
        // tx: Sender<UploadedAsset>,
        asset: Asset,
    ) -> JoinHandle<Result<UploadedAsset>>;
}

/// Default implementation of the trait ['Uploader'](Uploader) for all ['ParallelUploader'](ParallelUploader).
#[async_trait]
impl<T: ParallelUploader> Uploader for T {
    /// Uploads assets in parallel. It creates [`self::parallel_limit()`] tasks at a time to avoid
    /// reaching the limit of concurrent files open and it syncs the cache file at every `self.parallel_limit() / 2`
    /// step.
    async fn upload(
        &self,
        assets: &mut Vec<Asset>,
        state: &mut StorageState,
        lazy: bool,
    ) -> Result<()> {
        println!("I am here...");
        // let mut handles = Vec::new();
        let mut set = JoinSet::new();

        let mut current_batch = 0;
        let batch_size = 50;

        let num_jobs = assets.len();

        for asset in assets.drain(..) {
            set.spawn(self.upload_asset(asset));
        }

        while let Some(res) = set.join_next().await {
            res.unwrap().unwrap().unwrap();
        }

        // let handles = assets
        //     .drain(..)
        //     .map(|asset| self.upload_asset(asset))
        //     .collect::<Vec<JoinHandle<Result<UploadedAsset>>>>();

        // let mut uploaded_assets = Vec::new();
        // for handle in handles {
        //     uploaded_assets.push(handle.await.unwrap());
        // }

        // let one_sec = time::Duration::from_millis(10_000);
        // thread::sleep(one_sec);

        // while !assets.is_empty() {
        //     for asset in assets.drain(0..cmp::min(batch_size, assets.len())) {
        //         handles.push(self.upload_asset(asset));

        //         while !interrupt.load(Ordering::SeqCst) && !handles.is_empty() {
        //             // The returned future will wait for any future within iter to be
        //             // ready. Upon completion the item resolved will be returned,
        //             // along with the index of the future that was ready and the
        //             // list of all the remaining futures.
        //             match select_all(handles).await {
        //                 (Ok(res), _index, remaining) => {
        //                     handles = remaining;
        //                     if res.is_ok() {
        //                         let uploaded_asset = res?;
        //                         let item = state
        //                             .uploaded_items
        //                             .0
        //                             .get_mut(&uploaded_asset.id)
        //                             .unwrap();

        //                         item.link = uploaded_asset.link.clone();

        //                         current_batch += 1;
        //                     } else {
        //                         let err = GutenError::UploadError(format!(
        //                             "Upload error: {:?}",
        //                             res.err().unwrap()
        //                         ));

        //                         if !lazy {
        //                             return Err(anyhow!(format!(
        //                                 "Error: {}",
        //                                 err
        //                             )));
        //                         } else {
        //                             // user will need to retry the upload
        //                             errors.push(err);
        //                         }
        //                     }
        //                 }
        //                 (Err(err), _index, remaining) => {
        //                     handles = remaining;
        //                     let error = GutenError::UploadError(format!(
        //                         "Upload error: {:?}",
        //                         err
        //                     ));

        //                     if !lazy {
        //                         return Err(anyhow!(format!(
        //                             "Error: {}",
        //                             error
        //                         )));
        //                     } else {
        //                         // user will need to retry the upload
        //                         errors.push(error);
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }

        println!("SERVICE DONE");

        Ok(())
    }
}
