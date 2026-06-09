//! SourceHut project search.
//!
//! This adapter parses SourceHut HTML and is disabled by default until a
//! documented no-key project search API is available.

use async_trait::async_trait;
use metasearch_core::{
    category::SearchCategory,
    engine::{EngineMetadata, SearchEngine},
    error::{MetasearchError, Result},
    query::SearchQuery,
    result::SearchResult,
};
use reqwest::Client;
use scraper::{Html, Selector};
use smallvec::smallvec;

const USER_AGENT: &str = "metasearch/0.1 (+https://github.com/najmus-sakib-hossain/metasearch)";

pub struct Sourcehut {
    metadata: EngineMetadata,
    client: Client,
}

impl Sourcehut {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "sourcehut".to_string().into(),
                display_name: "SourceHut".to_string().into(),
                homepage: "https://sr.ht".to_string().into(),
                categories: smallvec![SearchCategory::IT],
                enabled: false,
                timeout_ms: 8000,
                weight: 0.6,
            },
            client,
        }
    }
}

pub fn parse_sourcehut_results(body: &str) -> Result<Vec<SearchResult>> {
    let document = Html::parse_document(body);
    let event_sel = Selector::parse("div.event-list div.event")
        .map_err(|e| MetasearchError::ParseError(e.to_string()))?;
    let link_sel =
        Selector::parse("h4 a").map_err(|e| MetasearchError::ParseError(e.to_string()))?;
    let desc_sel = Selector::parse("p").map_err(|e| MetasearchError::ParseError(e.to_string()))?;

    let mut results = Vec::new();

    for event in document.select(&event_sel) {
        let links: Vec<_> = event.select(&link_sel).collect();
        let project_link = if links.len() >= 2 {
            links[1]
        } else if !links.is_empty() {
            links[0]
        } else {
            continue;
        };

        let href = project_link.value().attr("href").unwrap_or_default().trim();
        if href.is_empty() {
            continue;
        }

        let title = project_link.text().collect::<String>().trim().to_string();
        if title.is_empty() {
            continue;
        }

        let content = event
            .select(&desc_sel)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        let mut result = SearchResult::new(
            title,
            format!("https://sr.ht{href}"),
            content,
            "sourcehut".to_string(),
        );
        result.engine_rank = (results.len() + 1) as u32;
        result.category = SearchCategory::IT.to_string();
        results.push(result);
    }

    Ok(results)
}

#[async_trait]
impl SearchEngine for Sourcehut {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://sr.ht/projects?search={}&page={}&sort=recently-updated",
            urlencoding::encode(&query.query),
            query.page,
        );

        let body = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_millis(self.metadata.timeout_ms))
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| MetasearchError::HttpError(e.to_string()))?
            .error_for_status()
            .map_err(|e| MetasearchError::HttpError(e.to_string()))?
            .text()
            .await
            .map_err(|e| MetasearchError::ParseError(e.to_string()))?;

        parse_sourcehut_results(&body)
    }
}
