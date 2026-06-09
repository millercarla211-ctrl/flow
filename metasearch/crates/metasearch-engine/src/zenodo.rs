//! Zenodo published record search via the documented REST API.
//!
//! Reference: <https://developers.zenodo.org/>

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

const USER_AGENT: &str =
    "metasearch-engine/1.0 (+https://github.com/najmus-sakib-hossain/metasearch)";

pub struct Zenodo {
    metadata: EngineMetadata,
    client: Client,
}

impl Zenodo {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "zenodo".to_string().into(),
                display_name: "Zenodo".to_string().into(),
                homepage: "https://zenodo.org".to_string().into(),
                categories: smallvec![SearchCategory::Science, SearchCategory::Files],
                enabled: true,
                timeout_ms: 7000,
                weight: 1.0,
            },
            client,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ZenodoResponse {
    hits: ZenodoHits,
}

#[derive(Debug, Deserialize)]
struct ZenodoHits {
    #[serde(default)]
    hits: Vec<ZenodoRecord>,
}

#[derive(Debug, Deserialize)]
struct ZenodoRecord {
    id: Option<u64>,
    doi_url: Option<String>,
    links: Option<ZenodoLinks>,
    metadata: ZenodoMetadata,
    stats: Option<ZenodoStats>,
}

#[derive(Debug, Deserialize)]
struct ZenodoLinks {
    self_html: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZenodoMetadata {
    title: Option<String>,
    publication_date: Option<String>,
    description: Option<String>,
    resource_type: Option<ZenodoResourceType>,
    #[serde(default)]
    creators: Vec<ZenodoCreator>,
    #[serde(default)]
    keywords: Vec<String>,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZenodoResourceType {
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZenodoCreator {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZenodoStats {
    views: Option<u64>,
    downloads: Option<u64>,
}

pub fn parse_zenodo_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: ZenodoResponse = serde_json::from_str(body)?;
    let results = response
        .hits
        .hits
        .into_iter()
        .enumerate()
        .filter_map(|(i, record)| {
            let title = non_empty(record.metadata.title)?;
            let url = record
                .links
                .and_then(|links| non_empty(links.self_html))
                .or_else(|| non_empty(record.doi_url))
                .or_else(|| {
                    record
                        .id
                        .map(|id| format!("https://zenodo.org/records/{id}"))
                })?;

            let mut parts = Vec::new();
            let creators = record
                .metadata
                .creators
                .into_iter()
                .filter_map(|creator| non_empty(creator.name))
                .take(3)
                .collect::<Vec<_>>()
                .join(", ");
            if !creators.is_empty() {
                parts.push(creators);
            }
            if let Some(date) = non_empty(record.metadata.publication_date) {
                parts.push(date);
            }
            if let Some(resource_type) = record
                .metadata
                .resource_type
                .and_then(|resource_type| non_empty(resource_type.title))
            {
                parts.push(resource_type);
            }
            if let Some(version) = non_empty(record.metadata.version) {
                parts.push(format!("v{version}"));
            }
            let keywords = record
                .metadata
                .keywords
                .into_iter()
                .filter_map(|keyword| non_empty(Some(keyword)))
                .take(5)
                .collect::<Vec<_>>()
                .join(", ");
            if !keywords.is_empty() {
                parts.push(keywords);
            }
            if let Some(description) = non_empty(record.metadata.description) {
                parts.push(strip_basic_html(&description));
            }
            if let Some(stats) = record.stats {
                if let Some(views) = stats.views {
                    parts.push(format!("views: {views}"));
                }
                if let Some(downloads) = stats.downloads {
                    parts.push(format!("downloads: {downloads}"));
                }
            }

            let mut result = SearchResult::new(title, url, parts.join(" - "), "zenodo");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::Science.to_string();
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for Zenodo {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://zenodo.org/api/records?q={}&size=10&page={}",
            urlencoding::encode(&query.query),
            query.page.max(1),
        );

        let body = self
            .client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Accept", "application/json")
            .timeout(std::time::Duration::from_millis(self.metadata.timeout_ms))
            .send()
            .await
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?
            .error_for_status()
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?
            .text()
            .await
            .map_err(|error| MetasearchError::ParseError(error.to_string()))?;

        parse_zenodo_results(&body)
    }
}

fn strip_basic_html(value: &str) -> String {
    html_escape::decode_html_entities(
        &value
            .replace("<p>", "")
            .replace("</p>", "")
            .replace("<br>", " ")
            .replace("<br/>", " ")
            .replace("<br />", " "),
    )
    .trim()
    .to_string()
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}
