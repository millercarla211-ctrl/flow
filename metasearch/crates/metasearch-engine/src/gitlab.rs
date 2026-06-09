//! GitLab project search via the public Projects API.

use async_trait::async_trait;
use metasearch_core::{
    category::SearchCategory,
    engine::{EngineMetadata, SearchEngine},
    error::MetasearchError,
    query::SearchQuery,
    result::SearchResult,
};
use reqwest::Client;
use smallvec::smallvec;

const USER_AGENT: &str = "metasearch/0.1 (+https://github.com/najmus-sakib-hossain/metasearch)";

pub struct GitLab {
    metadata: EngineMetadata,
    client: Client,
    base_url: String,
}

impl GitLab {
    pub fn new(client: Client) -> Self {
        Self::with_base_url(client, "https://gitlab.com")
    }

    pub fn with_base_url(client: Client, base_url: &str) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "gitlab".to_string().into(),
                display_name: "GitLab".to_string().into(),
                homepage: base_url.to_string().into(),
                categories: smallvec![SearchCategory::IT],
                enabled: true,
                timeout_ms: 5000,
                weight: 0.8,
            },
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

pub fn parse_gitlab_results(body: &str) -> Result<Vec<SearchResult>, MetasearchError> {
    let data: serde_json::Value =
        serde_json::from_str(body).map_err(|e| MetasearchError::ParseError(e.to_string()))?;
    let projects = data
        .as_array()
        .ok_or_else(|| MetasearchError::ParseError("GitLab returned non-array JSON".to_string()))?;
    let mut results = Vec::new();

    for item in projects {
        let title = item["name"].as_str().unwrap_or_default().trim();
        let web_url = item["web_url"].as_str().unwrap_or_default().trim();
        if title.is_empty() || web_url.is_empty() {
            continue;
        }

        let description = item["description"].as_str().unwrap_or("").trim();
        let stars = item["star_count"].as_u64().unwrap_or(0);
        let forks = item["forks_count"].as_u64().unwrap_or(0);
        let namespace = item["namespace"]["name"].as_str().unwrap_or("").trim();

        let snippet = format!(
            "{} - stars: {} | forks: {} | namespace: {}",
            description, stars, forks, namespace,
        );

        let mut result = SearchResult::new(
            title.to_string(),
            web_url.to_string(),
            snippet,
            "gitlab".to_string(),
        );
        result.engine_rank = (results.len() + 1) as u32;
        result.category = SearchCategory::IT.to_string();
        result.thumbnail = item["avatar_url"].as_str().map(|s| s.to_string());
        results.push(result);
    }

    Ok(results)
}

#[async_trait]
impl SearchEngine for GitLab {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, MetasearchError> {
        let url = format!(
            "{}/api/v4/projects?search={}&page={}",
            self.base_url,
            urlencoding::encode(&query.query),
            query.page,
        );

        let resp = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_millis(self.metadata.timeout_ms))
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| MetasearchError::HttpError(e.to_string()))?
            .error_for_status()
            .map_err(|e| MetasearchError::HttpError(e.to_string()))?;

        let body = resp
            .text()
            .await
            .map_err(|e| MetasearchError::ParseError(e.to_string()))?;

        parse_gitlab_results(&body)
    }
}
