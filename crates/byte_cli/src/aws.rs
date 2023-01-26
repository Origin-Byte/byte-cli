use anyhow::anyhow;
use aws_sdk_s3::{config, types::ByteStream, Client, Credentials, Region};
use std::{env, path::Path};

use crate::err::{self, CliError};

pub fn get_aws_client(region: &str) -> Result<Client, CliError> {
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
) -> Result<(), CliError> {
    // Validate

    if !path.exists() {
        err::no_path();
    }

    let key = path.to_str().ok_or_else(|| err::invalid_path(path))?;

    // Prepare
    let body = ByteStream::from_path(&path).await.map_err(|err| {
        anyhow!(
            r#"Could not generate bytestream from path "{}": {err}"#,
            path.display()
        )
    })?;

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
