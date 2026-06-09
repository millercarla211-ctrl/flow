//! Stack Exchange Q&A search via the public API v2.3.

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

pub struct StackExchange {
    metadata: EngineMetadata,
    client: Client,
    api_site: String,
}

impl StackExchange {
    pub fn new(client: Client) -> Self {
        Self::with_site(client, "stackoverflow")
    }

    pub fn with_site(client: Client, site: &str) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "stackexchange".to_string().into(),
                display_name: "StackExchange".to_string().into(),
                homepage: format!("https://{}.com", site).into(),
                categories: smallvec![SearchCategory::IT],
                enabled: true,
                timeout_ms: 5000,
                weight: 1.2,
            },
            client,
            api_site: site.to_string(),
        }
    }
}

pub fn parse_stackexchange_results(
    body: &str,
    api_site: &str,
) -> Result<Vec<SearchResult>, MetasearchError> {
    let data: serde_json::Value =
        serde_json::from_str(body).map_err(|e| MetasearchError::ParseError(e.to_string()))?;
    let mut results = Vec::new();

    if let Some(items) = data["items"].as_array() {
        for item in items {
            let question_id = item["question_id"].as_u64().unwrap_or(0);
            if question_id == 0 {
                continue;
            }

            let raw_title = item["title"].as_str().unwrap_or("Untitled").trim();
            if raw_title.is_empty() {
                continue;
            }
            let title = html_escape::decode_html_entities(raw_title).to_string();
            let question_url = format!("https://{}.com/q/{}", api_site, question_id);

            let tags: Vec<String> = item["tags"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|tag| tag.as_str().map(ToOwned::to_owned))
                .collect();

            let owner = item["owner"]["display_name"].as_str().unwrap_or("");
            let owner_clean = html_escape::decode_html_entities(owner).to_string();
            let is_answered = item["is_answered"].as_bool().unwrap_or(false);
            let score = item["score"].as_i64().unwrap_or(0);
            let answer_count = item["answer_count"].as_u64().unwrap_or(0);
            let view_count = item["view_count"].as_u64().unwrap_or(0);

            let mut content_parts = Vec::new();
            if !tags.is_empty() {
                content_parts.push(format!("[{}]", tags.join(", ")));
            }
            if !owner_clean.is_empty() {
                content_parts.push(owner_clean);
            }
            if is_answered {
                content_parts.push("answered".to_string());
            }
            content_parts.push(format!("score: {}", score));
            content_parts.push(format!("{} answers", answer_count));
            content_parts.push(format!("{} views", view_count));

            let mut result = SearchResult::new(
                title,
                question_url,
                content_parts.join(" // "),
                "stackexchange".to_string(),
            );
            result.engine_rank = (results.len() + 1) as u32;
            result.category = SearchCategory::IT.to_string();
            results.push(result);
        }
    }

    Ok(results)
}

#[async_trait]
impl SearchEngine for StackExchange {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, MetasearchError> {
        let pagesize: u32 = 10;
        let url = format!(
            "https://api.stackexchange.com/2.3/search/advanced?q={}&page={}&pagesize={}&site={}&sort=activity&order=desc",
            urlencoding::encode(&query.query),
            query.page,
            pagesize,
            self.api_site,
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

        parse_stackexchange_results(&body, &self.api_site)
    }
}
