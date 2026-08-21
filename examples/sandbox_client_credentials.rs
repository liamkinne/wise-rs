//! Check Wise sandbox client-credentials authentication and make a read-only API call.
//!
//! Required environment variables:
//! - `WISE_CLIENT_ID`
//! - `WISE_CLIENT_SECRET`
//!
//! Run with:
//! `cargo run --example sandbox_client_credentials`

use std::env;

use reqwest::Client as HttpClient;
use wise_platform::Client;

const DEFAULT_SANDBOX_URL: &str = "https://api.wise-sandbox.com";

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_id = env::var("WISE_CLIENT_ID")?;
    let client_secret = env::var("WISE_CLIENT_SECRET")?;
    let sandbox_url = env::var("WISE_SANDBOX_URL")
        .unwrap_or_else(|_| DEFAULT_SANDBOX_URL.to_owned())
        .trim_end_matches('/')
        .to_owned();

    let http = HttpClient::new();
    let token = http
        .post(format!("{sandbox_url}/oauth/token"))
        .basic_auth(client_id, Some(client_secret))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await?
        .error_for_status()?
        .json::<TokenResponse>()
        .await?;

    println!(
        "authenticated: token_type={}, expires_in={}s",
        token.token_type, token.expires_in
    );

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", token.access_token).parse()?,
    );

    // The generated client takes the API base URL, while OAuth is rooted at the
    // sandbox host. WISE_API_BASE_URL can be used if Wise changes the API prefix.
    let api_base_url =
        env::var("WISE_API_BASE_URL").unwrap_or_else(|_| format!("{sandbox_url}/2026Q3"));
    let api = Client::new_with_client(
        &api_base_url,
        reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?,
    );

    let quote = api
        .quote_create_unauthenticated(
            None,
            &wise_platform::types::QuoteCreateUnauthenticatedBody {
                source_currency: "GBP".to_owned(),
                target_currency: "USD".to_owned(),
                source_amount: Some(100.0),
                target_amount: None,
                pricing_configuration: None,
            },
        )
        .await?;

    println!("API call succeeded; quote response: {quote:#?}");
    Ok(())
}
