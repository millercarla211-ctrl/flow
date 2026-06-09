//! npm engine - search JavaScript packages via the official registry API.

use async_trait::async_trait;
use metasearch_core::{
    category::SearchCategory,
    engine::{EngineMetadata, SearchEngine},
    error::MetasearchError,
    query::SearchQuery,
    result::SearchResult,
};
use reqwest::Client;
use serde_json::Value;
use smallvec::smallvec;
use std::time::Duration;

const PAGE_SIZE: u32 = 25;
const USER_AGENT: &str = "metasearch/0.1 (+https://github.com/najmus-sakib-hossain/metasearch)";

pub struct Npm {
    metadata: EngineMetadata,
    client: Client,
}

impl Npm {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "npm".to_string().into(),
                display_name: "npm".to_string().into(),
                homepage: "https://www.npmjs.com".to_string().into(),
                categories: smallvec![SearchCategory::IT],
                enabled: true,
                timeout_ms: 5000,
                weight: 1.0,
            },
            client,
        }
    }
}

#[async_trait]
impl SearchEngine for Npm {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, MetasearchError> {
        let page = query.page;
        let from = (page - 1) * PAGE_SIZE;

        let url = format!(
            "https://registry.npmjs.org/-/v1/search?text={}&from={}&size={}",
            urlencoding::encode(&query.query),
            from,
            PAGE_SIZE,
        );

        let resp = self
            .client
            .get(&url)
            .timeout(Duration::from_millis(self.metadata.timeout_ms))
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(|e| MetasearchError::HttpError(format!("npm registry search: {e}")))?;

        let body = resp
            .text()
            .await
            .map_err(|e| MetasearchError::HttpError(format!("npm response body: {e}")))?;

        parse_npm_registry_results(&body)
    }
}

pub fn parse_npm_registry_results(body: &str) -> Result<Vec<SearchResult>, MetasearchError> {
    let data: Value = serde_json::from_str(body)?;
    let entries = data["objects"].as_array().ok_or_else(|| {
        MetasearchError::ParseError("npm registry response missing objects".into())
    })?;

    let mut results = Vec::new();

    for entry in entries {
        let package = &entry["package"];
        let name = package["name"].as_str().unwrap_or_default().trim();
        if name.is_empty() {
            continue;
        }

        let npm_url = package["links"]["npm"]
            .as_str()
            .filter(|url| !url.trim().is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("https://www.npmjs.com/package/{name}"));

        let version = package["version"].as_str().unwrap_or_default().trim();
        let description = package["description"].as_str().unwrap_or_default().trim();
        let publisher = package["publisher"]["username"]
            .as_str()
            .or_else(|| package["author"]["name"].as_str())
            .unwrap_or_default()
            .trim();

        let mut snippet_parts = Vec::new();
        if !version.is_empty() {
            snippet_parts.push(format!("v{version}"));
        }
        if !description.is_empty() {
            snippet_parts.push(description.to_string());
        }
        if !publisher.is_empty() {
            snippet_parts.push(format!("by {publisher}"));
        }

        let mut result = SearchResult::new(
            name.to_string(),
            npm_url,
            snippet_parts.join(" - "),
            "npm".to_string(),
        );
        result.engine_rank = (results.len() + 1) as u32;
        result.category = SearchCategory::IT.to_string();
        if let Some(score) = entry["score"]["final"].as_f64() {
            result.score = score;
        }
        results.push(result);
    }

    Ok(results)
}
