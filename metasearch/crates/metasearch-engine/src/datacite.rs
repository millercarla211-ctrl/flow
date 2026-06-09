//! DataCite DOI metadata search via the public REST API.
//!
//! Reference: <https://support.datacite.org/docs/api>

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

pub struct DataCite {
    metadata: EngineMetadata,
    client: Client,
}

impl DataCite {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "datacite".to_string().into(),
                display_name: "DataCite".to_string().into(),
                homepage: "https://datacite.org".to_string().into(),
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
struct DataCiteResponse {
    #[serde(default)]
    data: Vec<DataCiteRecord>,
}

#[derive(Debug, Deserialize)]
struct DataCiteRecord {
    id: Option<String>,
    attributes: DataCiteAttributes,
}

#[derive(Debug, Deserialize)]
struct DataCiteAttributes {
    doi: Option<String>,
    #[serde(default)]
    titles: Vec<DataCiteTitle>,
    #[serde(default)]
    creators: Vec<DataCiteCreator>,
    publisher: Option<String>,
    #[serde(rename = "publicationYear")]
    publication_year: Option<u16>,
    url: Option<String>,
    #[serde(default)]
    descriptions: Vec<DataCiteDescription>,
    types: Option<DataCiteTypes>,
}

#[derive(Debug, Deserialize)]
struct DataCiteTitle {
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DataCiteCreator {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DataCiteDescription {
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DataCiteTypes {
    #[serde(rename = "resourceTypeGeneral")]
    resource_type_general: Option<String>,
}

pub fn parse_datacite_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: DataCiteResponse = serde_json::from_str(body)?;
    let results = response
        .data
        .into_iter()
        .enumerate()
        .filter_map(|(i, record)| {
            let title = record
                .attributes
                .titles
                .into_iter()
                .find_map(|title| non_empty(title.title))?;
            let url = non_empty(record.attributes.url)
                .or_else(|| non_empty(record.attributes.doi.clone()).map(|doi| doi_url(&doi)))
                .or_else(|| non_empty(record.id).map(|doi| doi_url(&doi)))?;

            let mut parts = Vec::new();
            let creators = record
                .attributes
                .creators
                .into_iter()
                .filter_map(|creator| non_empty(creator.name))
                .take(3)
                .collect::<Vec<_>>()
                .join(", ");
            if !creators.is_empty() {
                parts.push(creators);
            }
            if let Some(year) = record.attributes.publication_year {
                parts.push(year.to_string());
            }
            if let Some(publisher) = non_empty(record.attributes.publisher) {
                parts.push(publisher);
            }
            if let Some(resource_type) = record
                .attributes
                .types
                .and_then(|types| non_empty(types.resource_type_general))
            {
                parts.push(resource_type);
            }
            if let Some(description) = record
                .attributes
                .descriptions
                .into_iter()
                .find_map(|description| non_empty(description.description))
            {
                parts.push(description);
            }

            let mut result = SearchResult::new(title, url, parts.join(" - "), "datacite");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::Science.to_string();
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for DataCite {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://api.datacite.org/dois?query={}&page%5Bsize%5D=10&page%5Bnumber%5D={}",
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

        parse_datacite_results(&body)
    }
}

fn doi_url(doi: &str) -> String {
    format!("https://doi.org/{doi}")
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}
