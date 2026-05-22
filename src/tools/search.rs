#![cfg(feature = "searxng")]
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::util::{CallToolResult, error, success};

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
pub struct WebSearch {
    query: String,
    category: Category,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    General,
    News,
    IT,
    Science,
}

#[derive(Clone)]
pub(crate) struct SearchConfig {
    searxng_api: reqwest::Url,
}

impl SearchConfig {
    pub(crate) fn new() -> Self {
        let searxng_api =
            std::env::var("SEARXNG_URL").expect("Missing SEARXNG_URL environment variable.");

        let searxng_api = reqwest::Url::parse(&searxng_api).expect("Invalid SEARXNG_URL.");

        Self { searxng_api }
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

pub async fn query(search_config: &SearchConfig, web_search: WebSearch) -> CallToolResult {
    let query = web_search.query;
    let category = web_search.category;
    let result = send_query(query, category, &search_config.searxng_api).await;

    match result {
        Ok(response) => success(response),
        Err(err) => error(format!("Error: {err:#?}")),
    }
}

#[derive(Deserialize, Debug)]
struct QueryResult {
    url: String,
    title: String,
    content: String,
}

impl Display for QueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "# {}\n{}\n{}\n", self.title, self.url, self.content)
    }
}

#[derive(Deserialize)]
struct ResponseSet {
    results: Vec<QueryResult>,
}

async fn send_query(query: String, category: Category, api: &reqwest::Url) -> anyhow::Result<String> {
    println!("Query: {query} -- {category}");

    let url = reqwest::Url::parse_with_params(
        api.join("search")?.as_str(),
        &[
            ("q", query.as_str()),
            ("format", "json"),
            ("categories", &category.to_string()),
        ],
    )?;

    let res: ResponseSet = reqwest::get(url).await?.json().await?;

    let result = res
        .results
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(result)
}
