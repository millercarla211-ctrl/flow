//! Hacker News search via the Algolia HN Search API.

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

pub struct HackerNews {
    metadata: EngineMetadata,
    client: Client,
}

impl HackerNews {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "hackernews".to_string().into(),
                display_name: "Hacker News".to_string().into(),
                homepage: "https://news.ycombinator.com".to_string().into(),
                categories: smallvec![SearchCategory::IT],
                enabled: true,
                timeout_ms: 5000,
                weight: 1.0,
            },
            client,
        }
    }
}

pub fn parse_hackernews_results(body: &str) -> Result<Vec<SearchResult>, MetasearchError> {
    let data: serde_json::Value =
        serde_json::from_str(body).map_err(|e| MetasearchError::ParseError(e.to_string()))?;
    let mut results = Vec::new();

    if let Some(hits) = data["hits"].as_array() {
        for (i, hit) in hits.iter().enumerate() {
            let object_id = hit["objectID"].as_str().unwrap_or_default().trim();
            let title = hit["title"].as_str().unwrap_or("Untitled").trim();
            if object_id.is_empty() || title.is_empty() {
                continue;
            }

            let points = hit["points"].as_u64().unwrap_or(0);
            let num_comments = hit["num_comments"].as_u64().unwrap_or(0);
            let author = hit["author"].as_str().unwrap_or("").trim();
            let source_url = hit["url"].as_str().unwrap_or("").trim();
            let hn_url = format!("https://news.ycombinator.com/item?id={object_id}");

            let snippet = format!(
                "{} - points: {} | comments: {} | by {}",
                if source_url.is_empty() {
                    hn_url.as_str()
                } else {
                    source_url
                },
                points,
                num_comments,
                author,
            );

            let mut result =
                SearchResult::new(title.to_string(), hn_url, snippet, "hackernews".to_string());
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::IT.to_string();
            results.push(result);
        }
    }

    Ok(results)
}

#[async_trait]
impl SearchEngine for HackerNews {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, MetasearchError> {
        let page = query.page.saturating_sub(1);

        let url = format!(
            "https://hn.algolia.com/api/v1/search?query={}&page={}&hitsPerPage=30&tags=story",
            urlencoding::encode(&query.query),
            page,
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

        parse_hackernews_results(&body)
    }
}
