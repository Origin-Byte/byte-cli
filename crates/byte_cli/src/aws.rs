use anyhow::{anyhow, bail, Result};
use aws_sdk_s3::{config, types::ByteStream, Client, Credentials, Region};
use std::{env, path::Path};

pub fn get_aws_client(region: &str) -> Result<Client> {
    let access_key_id = env::var("ACCESS_KEY_ID")?;
    let secret_key_id = env::var("SECRET_ACCESS_KEY")?;

    let cred = Credentials::new(
        access_key_id,
        secret_key_id,
        None,
        None,
        "loaded-from-custom-env",
    );

    let region = Region::new(region.to_string());

    let conf_builder = config::Builder::new()
        .region(region)
        .credentials_provider(cred);

    let conf = conf_builder.build();

    let client = Client::from_conf(conf);

    Ok(client)
}

pub async fn upload_file(
    client: &Client,
    bucket_name: &str,
    path: &Path,
) -> Result<()> {
    // Validate
    if !path.exists() {
        bail!("Path {} does not exist", path.display());
    }

    let key = path
        .to_str()
        .ok_or_else(|| anyhow!("Invalid path {path:?}"))?;

    // Prepare
    let body = ByteStream::from_path(&path).await?;
    let content_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    // Build request
    let request = client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body)
        .content_type(content_type);

    // Execute query
    request.send().await?;

    Ok(())
}
