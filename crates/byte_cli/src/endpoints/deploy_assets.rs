use std::env;
use std::path::Path;

use crate::aws;
use anyhow::Result;
use dotenv::dotenv;
use glob::glob;

pub async fn deploy_assets(assets_dir: &str) -> Result<()> {
    dotenv().ok();

    let region = env::var("REGION")?;
    let bucket_name = env::var("BUCKET_NAME")?;

    let client = aws::get_aws_client(region.as_str())?;

    let mut files: Vec<String> = vec![];

    for e in glob("suimarines/images/*").expect("Failed to read glob pattern") {
        let file_string = e?.file_name().unwrap().to_str().unwrap().to_string();

        files.push(file_string);
    }

    for file in files {
        // TODO: Make this multi-threaded
        let path_string = format!(
            "{assets_dir}/images/{file}",
            assets_dir = assets_dir,
            file = file
        );

        let path = Path::new(path_string.as_str());

        aws::upload_file(&client, bucket_name.as_str(), path).await?;
    }

    Ok(())
}
