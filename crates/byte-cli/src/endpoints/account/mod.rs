use anyhow::{anyhow, Result};
use reqwest::{Client, Response};
use serde_json::{json, Value};

pub mod create;
pub mod link;
pub mod list;
pub mod switch;
pub mod unlink;

pub async fn login(email: &str, password: &str) -> Result<Response> {
    let client = Client::new();

    let req_body = json!({
        "email": email,
        "password": password,
    });

    let res = client
        .post("https://suiplay-api.originbyte.io/v1/admin/accounts/login")
        // .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await?;

    let status = res.status();

    // Check if the status is a success.
    if status.is_success() {
        println!("Successfully verified account.");
    } else if status.is_client_error() {
        // Get the body of the response.
        let body = res.text().await?;
        return Err(anyhow!(
            "Failed with status: {} and the following message: {}",
            status,
            body
        ));
    } else {
        // For other errors (like server errors)
        println!("An unexpected error occurred.");
    }

    Ok(res)
}

pub async fn get_jwt(response: Response) -> Result<String> {
    let body = response.text().await?;
    let json: Value =
        serde_json::from_str(&body).expect("Failed to parse JSON");
    let id_token = json["idToken"].as_str().expect("idToken is not a string");

    Ok(id_token.to_string())
}
