//! Europe PMC publication search via its public REST API.
//!
//! Reference: <https://dev.europepmc.org/RestfulWebService>

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

pub struct EuropePmc {
    metadata: EngineMetadata,
    client: Client,
}

impl EuropePmc {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "europe_pmc".to_string().into(),
                display_name: "Europe PMC".to_string().into(),
                homepage: "https://europepmc.org".to_string().into(),
                categories: smallvec![SearchCategory::Science],
                enabled: true,
                timeout_ms: 7000,
                weight: 1.0,
            },
            client,
        }
    }
}

#[derive(Debug, Deserialize)]
struct EuropePmcResponse {
    #[serde(rename = "resultList")]
    result_list: EuropePmcResultList,
}

#[derive(Debug, Deserialize)]
struct EuropePmcResultList {
    #[serde(default)]
    result: Vec<EuropePmcRecord>,
}

#[derive(Debug, Deserialize)]
struct EuropePmcRecord {
    id: Option<String>,
    source: Option<String>,
    title: Option<String>,
    #[serde(rename = "authorString")]
    author_string: Option<String>,
    #[serde(rename = "journalTitle")]
    journal_title: Option<String>,
    #[serde(rename = "pubYear")]
    pub_year: Option<String>,
    doi: Option<String>,
    #[serde(rename = "abstractText")]
    abstract_text: Option<String>,
}

pub fn parse_europe_pmc_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: EuropePmcResponse = serde_json::from_str(body)?;
    let results = response
        .result_list
        .result
        .into_iter()
        .enumerate()
        .filter_map(|(i, record)| {
            let title = record.title.filter(|value| !value.trim().is_empty())?;
            let url = record
                .doi
                .filter(|value| !value.trim().is_empty())
                .map(|doi| format!("https://doi.org/{doi}"))
                .or_else(|| {
                    let source = record.source.filter(|value| !value.trim().is_empty())?;
                    let id = record.id.filter(|value| !value.trim().is_empty())?;
                    Some(format!("https://europepmc.org/article/{source}/{id}"))
                })?;

            let mut parts = Vec::new();
            if let Some(authors) = record
                .author_string
                .filter(|value| !value.trim().is_empty())
            {
                parts.push(authors);
            }
            if let Some(year) = record.pub_year.filter(|value| !value.trim().is_empty()) {
                parts.push(year);
            }
            if let Some(journal) = record
                .journal_title
                .filter(|value| !value.trim().is_empty())
            {
                parts.push(journal);
            }
            if let Some(abstract_text) = record
                .abstract_text
                .filter(|value| !value.trim().is_empty())
            {
                parts.push(abstract_text);
            }

            let mut result = SearchResult::new(title, url, parts.join(" - "), "europe_pmc");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::Science.to_string();
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for EuropePmc {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://www.ebi.ac.uk/europepmc/webservices/rest/search?query={}&format=json&pageSize=10&page={}",
            urlencoding::encode(&query.query),
            query.page.max(1),
        );

        let body = self
            .client
            .get(url)
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

        parse_europe_pmc_results(&body)
    }
}
