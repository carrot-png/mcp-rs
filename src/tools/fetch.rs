#![cfg(feature = "fetch")]
use axum::http::{HeaderMap, HeaderValue};
use reqwest::{Url, header::USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::util::{CallToolResult, error, success};

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
pub struct WebFetch {
    url: String,
}

pub async fn run(client: reqwest::Client, web_search: WebFetch) -> CallToolResult {
    let mut url = web_search.url;
    if !url.contains("://") {
        url = format!("https://{url}");
    }

    let url = match Url::parse(&url) {
        Ok(url) => url,
        Err(err) => return error(format!("Failed to parse URL: {err}")),
    };

    match fetch(client, url).await {
        Ok(text) => success(text),
        Err(err) => error(format!("Failed to fetch URL: {err}")),
    }
}

async fn fetch(client: reqwest::Client, url: Url) -> anyhow::Result<String> {
    let raw_html = client.get(url).send().await?.text().await?;
    let text = html2text::from_read(raw_html.as_bytes(), 80)?;
    Ok(text)
}

pub fn get_client() -> reqwest::Client {
    let mut headers = HeaderMap::new();
    let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| "mcp-rs/1.0".into());

    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&user_agent).expect("Invalid USER_AGENT string."),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build fetch client")
}
