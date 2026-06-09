//! DBpedia entity lookup via the public Lookup API.
//!
//! Reference: <https://www.dbpedia.org/resources/lookup/>

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

pub struct Dbpedia {
    metadata: EngineMetadata,
    client: Client,
}

impl Dbpedia {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "dbpedia".to_string().into(),
                display_name: "DBpedia".to_string().into(),
                homepage: "https://www.dbpedia.org".to_string().into(),
                categories: smallvec![SearchCategory::General],
                enabled: true,
                timeout_ms: 6000,
                weight: 0.8,
            },
            client,
        }
    }
}

#[derive(Debug, Deserialize)]
struct DbpediaResponse {
    #[serde(default)]
    docs: Vec<DbpediaDoc>,
}

#[derive(Debug, Deserialize)]
struct DbpediaDoc {
    label: Option<Vec<String>>,
    resource: Option<Vec<String>>,
    comment: Option<Vec<String>>,
}

fn first_text(values: Option<Vec<String>>) -> Option<String> {
    values?.into_iter().find(|value| !value.trim().is_empty())
}

pub fn parse_dbpedia_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: DbpediaResponse = serde_json::from_str(body)?;
    let results = response
        .docs
        .into_iter()
        .enumerate()
        .filter_map(|(i, doc)| {
            let title = first_text(doc.label)?;
            let url = first_text(doc.resource)?;
            let content = first_text(doc.comment).unwrap_or_default();

            let mut result = SearchResult::new(title, url, content, "dbpedia");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::General.to_string();
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for Dbpedia {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://lookup.dbpedia.org/api/search?query={}&format=json&maxResults=10",
            urlencoding::encode(&query.query),
        );

        let body = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .header("User-Agent", USER_AGENT)
            .timeout(std::time::Duration::from_millis(self.metadata.timeout_ms))
            .send()
            .await
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?
            .error_for_status()
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?
            .text()
            .await
            .map_err(|error| MetasearchError::ParseError(error.to_string()))?;

        parse_dbpedia_results(&body)
    }
}
