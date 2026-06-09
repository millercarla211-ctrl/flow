//! Packagist package search via the documented anonymous JSON API.
//!
//! Reference: <https://packagist.org/apidoc>

use async_trait::async_trait;
use metasearch_core::{
    category::SearchCategory,
    engine::{EngineMetadata, SearchEngine},
    error::{MetasearchError, Result},
    query::SearchQuery,
    result::SearchResult,
};
use reqwest::Client;
use serde::Deserialize;
use smallvec::smallvec;

pub struct Packagist {
    metadata: EngineMetadata,
    client: Client,
}

impl Packagist {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "packagist".to_string().into(),
                display_name: "Packagist".to_string().into(),
                homepage: "https://packagist.org".to_string().into(),
                categories: smallvec![SearchCategory::IT],
                enabled: true,
                timeout_ms: 5000,
                weight: 1.0,
            },
            client,
        }
    }
}

#[derive(Debug, Deserialize)]
struct PackagistResponse {
    #[serde(default)]
    results: Vec<PackagistPackage>,
}

#[derive(Debug, Deserialize)]
struct PackagistPackage {
    name: Option<String>,
    description: Option<String>,
    url: Option<String>,
    downloads: Option<u64>,
    favers: Option<u64>,
}

pub fn parse_packagist_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: PackagistResponse = serde_json::from_str(body)?;
    let results = response
        .results
        .into_iter()
        .enumerate()
        .filter_map(|(i, package)| {
            let name = package.name?;
            let url = package
                .url
                .unwrap_or_else(|| format!("https://packagist.org/packages/{name}"));

            let mut parts = Vec::new();
            if let Some(description) = package.description.filter(|value| !value.is_empty()) {
                parts.push(description);
            }
            if let Some(downloads) = package.downloads {
                parts.push(format!("{downloads} downloads"));
            }
            if let Some(favers) = package.favers {
                parts.push(format!("{favers} favorites"));
            }

            let mut result = SearchResult::new(name, url, parts.join(" - "), "packagist");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::IT.to_string();
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for Packagist {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://packagist.org/search.json?q={}&page={}",
            urlencoding::encode(&query.query),
            query.page.max(1),
        );

        let body = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?
            .error_for_status()
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?
            .text()
            .await
            .map_err(|error| MetasearchError::ParseError(error.to_string()))?;

        parse_packagist_results(&body)
    }
}
