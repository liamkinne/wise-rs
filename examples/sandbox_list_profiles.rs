//! Make a read-only request with an existing Wise sandbox user access token.
//!
//! Required environment variable:
//! - `WISE_ACCESS_TOKEN`
//!
//! Run with:
//! `cargo run --example sandbox_list_profiles`

use std::env;

use wise_platform::Client;

const DEFAULT_API_BASE_URL: &str = "https://api.wise-sandbox.com/2026Q3";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_token = env::var("WISE_ACCESS_TOKEN")?;
    let api_base_url =
        env::var("WISE_API_BASE_URL").unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_owned());

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {access_token}").parse()?,
    );

    let client = Client::new_with_client(
        &api_base_url,
        reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?,
    );

    let profiles = client.profile_list(None).await?;
    println!("API call succeeded; profiles: {profiles:#?}");
    Ok(())
}
