//! Searchcode Code engine — search source code via Searchcode API.

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
use std::time::Duration;

const USER_AGENT: &str = "metasearch/0.1 (+https://github.com/najmus-sakib-hossain/metasearch)";

pub struct SearchcodeCode {
    metadata: EngineMetadata,
    client: Client,
}

impl SearchcodeCode {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "searchcode_code".to_string().into(),
                display_name: "Searchcode".to_string().into(),
                homepage: "https://searchcode.com".to_string().into(),
                categories: smallvec![SearchCategory::IT],
                enabled: false,
                timeout_ms: 5000,
                weight: 1.0,
            },
            client,
        }
    }
}

#[async_trait]
impl SearchEngine for SearchcodeCode {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://searchcode.com/api/codesearch_I/?q={}",
            urlencoding::encode(&query.query),
        );

        let resp = match self
            .client
            .get(&url)
            .timeout(Duration::from_millis(self.metadata.timeout_ms))
            .header("User-Agent", USER_AGENT)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return Err(MetasearchError::HttpError(format!("Searchcode: {e}"))),
        };

        let resp = resp
            .error_for_status()
            .map_err(|e| MetasearchError::HttpError(format!("Searchcode status: {e}")))?;

        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => return Err(MetasearchError::HttpError(format!("Searchcode body: {e}"))),
        };

        if text.trim_start().starts_with('<') {
            return Err(MetasearchError::ParseError(
                "Searchcode returned HTML instead of JSON".into(),
            ));
        }

        let data: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => return Err(MetasearchError::ParseError(e.to_string())),
        };

        let mut results = Vec::new();

        if let Some(items) = data["results"].as_array() {
            for item in items {
                let item_url = item["url"].as_str().unwrap_or_default();
                let name = item["name"].as_str().unwrap_or_default();
                let filename = item["filename"].as_str().unwrap_or("");
                let repo = item["repo"].as_str().unwrap_or("");

                if item_url.is_empty() || (name.is_empty() && filename.is_empty()) {
                    continue;
                }

                let title = if filename.is_empty() {
                    name.to_string()
                } else {
                    format!("{} - {}", name, filename)
                };

                let mut result = SearchResult::new(
                    title,
                    item_url.to_string(),
                    repo.to_string(),
                    "searchcode_code".to_string(),
                );
                result.engine_rank = (results.len() + 1) as u32;
                result.category = SearchCategory::IT.to_string();
                results.push(result);
            }
        }

        Ok(results)
    }
}
