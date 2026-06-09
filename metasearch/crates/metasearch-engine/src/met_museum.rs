//! The Metropolitan Museum of Art collection search via the public API.
//!
//! Reference: <https://metmuseum.github.io/>

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
const MAX_OBJECT_FETCHES: usize = 5;

pub struct MetMuseum {
    metadata: EngineMetadata,
    client: Client,
}

impl MetMuseum {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "met_museum".to_string().into(),
                display_name: "The Met Collection".to_string().into(),
                homepage: "https://www.metmuseum.org/art/collection"
                    .to_string()
                    .into(),
                categories: smallvec![SearchCategory::Images],
                enabled: true,
                timeout_ms: 8000,
                weight: 0.7,
            },
            client,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MetSearchResponse {
    #[serde(rename = "objectIDs")]
    object_ids: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
struct MetObject {
    #[serde(rename = "objectID")]
    object_id: Option<u64>,
    title: Option<String>,
    #[serde(rename = "objectURL")]
    object_url: Option<String>,
    #[serde(rename = "artistDisplayName")]
    artist_display_name: Option<String>,
    #[serde(rename = "objectDate")]
    object_date: Option<String>,
    medium: Option<String>,
    #[serde(rename = "primaryImageSmall")]
    primary_image_small: Option<String>,
}

pub fn parse_met_search_ids(body: &str) -> Result<Vec<u64>> {
    let response: MetSearchResponse = serde_json::from_str(body)?;
    Ok(response
        .object_ids
        .unwrap_or_default()
        .into_iter()
        .take(MAX_OBJECT_FETCHES)
        .collect())
}

pub fn parse_met_object_results(bodies: &[&str]) -> Result<Vec<SearchResult>> {
    bodies
        .iter()
        .enumerate()
        .filter_map(|(i, body)| {
            let object: MetObject = match serde_json::from_str(body) {
                Ok(object) => object,
                Err(error) => return Some(Err(error.into())),
            };

            let title = object.title.filter(|value| !value.trim().is_empty())?;
            let object_id = object.object_id?;
            let url = object
                .object_url
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| {
                    format!("https://www.metmuseum.org/art/collection/search/{object_id}")
                });

            let mut parts = Vec::new();
            if let Some(artist) = object
                .artist_display_name
                .filter(|value| !value.trim().is_empty())
            {
                parts.push(artist);
            }
            if let Some(date) = object.object_date.filter(|value| !value.trim().is_empty()) {
                parts.push(date);
            }
            if let Some(medium) = object.medium.filter(|value| !value.trim().is_empty()) {
                parts.push(medium);
            }

            let mut result = SearchResult::new(title, url, parts.join(" - "), "met_museum");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::Images.to_string();
            result.thumbnail = object
                .primary_image_small
                .filter(|value| !value.trim().is_empty());
            Some(Ok(result))
        })
        .collect()
}

#[async_trait]
impl SearchEngine for MetMuseum {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let search_url = format!(
            "https://collectionapi.metmuseum.org/public/collection/v1/search?hasImages=true&q={}",
            urlencoding::encode(&query.query),
        );

        let search_body = self
            .client
            .get(search_url)
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

        let object_ids = parse_met_search_ids(&search_body)?;
        let mut object_bodies = Vec::new();
        for object_id in object_ids {
            let object_url = format!(
                "https://collectionapi.metmuseum.org/public/collection/v1/objects/{object_id}"
            );
            let body = self
                .client
                .get(object_url)
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
            object_bodies.push(body);
        }

        let borrowed = object_bodies.iter().map(String::as_str).collect::<Vec<_>>();
        parse_met_object_results(&borrowed)
    }
}
