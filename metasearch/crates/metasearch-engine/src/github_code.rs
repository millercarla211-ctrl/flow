//! GitHub Code search engine implementation.
//!
//! JSON API: <https://api.github.com/search/code>
//! Website: https://github.com
//! Features: Paging, optional auth token

use async_trait::async_trait;
use metasearch_core::{
    category::SearchCategory,
    engine::{EngineMetadata, SearchEngine},
    error::{MetasearchError, Result},
    query::SearchQuery,
    result::SearchResult,
};
use reqwest::Client;
use smallvec::smallvec;
use tracing::info;

pub struct GithubCode {
    metadata: EngineMetadata,
    client: Client,
    token: Option<String>,
}

impl GithubCode {
    pub fn new(client: Client, token: Option<String>) -> Self {
        let enabled = token.as_ref().is_some_and(|token| !token.trim().is_empty());
        Self {
            metadata: EngineMetadata {
                name: "github_code".to_string().into(),
                display_name: "GitHub Code".to_string().into(),
                homepage: "https://github.com".to_string().into(),
                categories: smallvec![SearchCategory::IT],
                enabled,
                timeout_ms: 5000,
                weight: 1.0,
            },
            client,
            token,
        }
    }
}

#[async_trait]
impl SearchEngine for GithubCode {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let token = self
            .token
            .as_deref()
            .filter(|token| !token.trim().is_empty())
            .ok_or_else(|| MetasearchError::EngineError {
                engine: "github_code".to_string(),
                message: "GitHub code search requires an API token".to_string(),
            })?;
        let encoded = urlencoding::encode(&query.query);
        let page = query.page.max(1);

        let url = format!(
            "https://api.github.com/search/code?sort=indexed&order=desc&q={}&page={}&per_page=10",
            encoded, page
        );

        let mut req = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github.preview.text-match+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "metasearch-engine/1.0");

        req = req.header("Authorization", format!("Bearer {}", token));

        let resp = req
            .send()
            .await
            .map_err(|e| MetasearchError::HttpError(e.to_string()))?;

        let status = resp.status().as_u16();
        if status == 403 || status == 429 {
            return Err(MetasearchError::RateLimited {
                retry_after_secs: 60,
            });
        }

        let resp = resp
            .error_for_status()
            .map_err(|e| MetasearchError::HttpError(e.to_string()))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| MetasearchError::ParseError(format!("JSON error: {}", e)))?;

        let mut results = Vec::new();

        let items = match data.get("items").and_then(|i| i.as_array()) {
            Some(items) => items,
            None => return Ok(results),
        };

        for (i, item) in items.iter().enumerate() {
            let html_url = item
                .get("html_url")
                .and_then(|u| u.as_str())
                .unwrap_or_default();

            let repository = item
                .get("repository")
                .and_then(|repo| repo.get("full_name"))
                .and_then(|n| n.as_str())
                .unwrap_or_default();
            let path = item
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or_default();
            let name = item.get("name").and_then(|n| n.as_str()).unwrap_or(path);

            if html_url.is_empty() || repository.is_empty() || name.is_empty() {
                continue;
            }

            let content = if path.is_empty() {
                repository.to_string()
            } else {
                format!("{repository} / {path}")
            };

            let mut r = SearchResult::new(name, html_url, &content, "github_code");
            r.engine_rank = (i + 1) as u32;
            r.category = SearchCategory::IT.to_string();
            results.push(r);
        }

        info!(
            engine = "github_code",
            count = results.len(),
            "Search complete"
        );
        Ok(results)
    }
}
