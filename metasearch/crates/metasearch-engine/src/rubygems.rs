//! RubyGems package search via the documented RubyGems.org JSON API.
//!
//! Reference: <https://guides.rubygems.org/rubygems-org-api/>

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

pub struct RubyGems {
    metadata: EngineMetadata,
    client: Client,
}

impl RubyGems {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "rubygems".to_string().into(),
                display_name: "RubyGems".to_string().into(),
                homepage: "https://rubygems.org".to_string().into(),
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
struct RubyGemItem {
    name: Option<String>,
    info: Option<String>,
    version: Option<String>,
    project_uri: Option<String>,
    gem_uri: Option<String>,
    downloads: Option<u64>,
}

pub fn parse_rubygems_results(body: &str) -> Result<Vec<SearchResult>> {
    let items: Vec<RubyGemItem> = serde_json::from_str(body)?;
    let results = items
        .into_iter()
        .enumerate()
        .filter_map(|(i, item)| {
            let name = item.name?;
            let url = item
                .project_uri
                .or(item.gem_uri)
                .unwrap_or_else(|| format!("https://rubygems.org/gems/{name}"));

            let mut parts = Vec::new();
            if let Some(version) = item.version.filter(|value| !value.is_empty()) {
                parts.push(format!("v{version}"));
            }
            if let Some(info) = item.info.filter(|value| !value.is_empty()) {
                parts.push(info);
            }
            if let Some(downloads) = item.downloads {
                parts.push(format!("{downloads} downloads"));
            }

            let mut result = SearchResult::new(name, url, parts.join(" - "), "rubygems");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::IT.to_string();
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for RubyGems {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://rubygems.org/api/v1/search.json?query={}&page={}",
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

        parse_rubygems_results(&body)
    }
}
