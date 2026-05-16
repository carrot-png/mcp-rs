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

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

pub async fn query(web_search: WebSearch) -> CallToolResult {
    let query = web_search.query;
    let category = web_search.category;
    let result = send_query(query, category).await;

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

async fn send_query(query: String, category: Category) -> anyhow::Result<String> {
    println!("Query: {query} -- {category}");
    let api = std::env::var("SEARXNG_URL").expect("Missing SEARXNG_URL environment variable.");
    let url = format!("{api}/search?q={query}&format=json&categories={category}");
    let res: ResponseSet = reqwest::get(url).await?.json().await?;

    let result = res
        .results
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(result)
}

#[tokio::test]
async fn try_it() {
    let a = send_query(String::from("test"), Category::IT).await;
    println!("a:{a:#?}");
}
