//! Internet Archive advanced search via the official JSON endpoint.
//!
//! Reference: <https://archive.org/advancedsearch.php>

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

pub struct InternetArchive {
    metadata: EngineMetadata,
    client: Client,
}

impl InternetArchive {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "internet_archive".to_string().into(),
                display_name: "Internet Archive".to_string().into(),
                homepage: "https://archive.org".to_string().into(),
                categories: smallvec![
                    SearchCategory::General,
                    SearchCategory::Files,
                    SearchCategory::Videos,
                    SearchCategory::Music,
                    SearchCategory::Science,
                ],
                enabled: true,
                timeout_ms: 7000,
                weight: 1.0,
            },
            client,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ArchiveResponse {
    response: Option<ArchiveDocs>,
}

#[derive(Debug, Deserialize)]
struct ArchiveDocs {
    #[serde(default)]
    docs: Vec<ArchiveItem>,
}

#[derive(Debug, Deserialize)]
struct ArchiveItem {
    identifier: Option<String>,
    title: Option<JsonText>,
    description: Option<JsonText>,
    mediatype: Option<String>,
    creator: Option<JsonText>,
    year: Option<JsonText>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JsonText {
    String(String),
    Strings(Vec<String>),
    Number(u64),
}

impl JsonText {
    fn into_text(self) -> String {
        match self {
            Self::String(value) => value,
            Self::Strings(values) => values.join(", "),
            Self::Number(value) => value.to_string(),
        }
    }
}

pub fn parse_internet_archive_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: ArchiveResponse = serde_json::from_str(body)?;
    let results = response
        .response
        .map(|response| response.docs)
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .filter_map(|(i, item)| {
            let identifier = item.identifier?;
            let title = item
                .title
                .map(JsonText::into_text)
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| identifier.clone());

            let mut parts = Vec::new();
            if let Some(creator) = item
                .creator
                .map(JsonText::into_text)
                .filter(|v| !v.is_empty())
            {
                parts.push(creator);
            }
            if let Some(year) = item.year.map(JsonText::into_text).filter(|v| !v.is_empty()) {
                parts.push(year);
            }
            if let Some(mediatype) = item.mediatype.filter(|v| !v.is_empty()) {
                parts.push(mediatype);
            }
            if let Some(description) = item
                .description
                .map(JsonText::into_text)
                .filter(|v| !v.is_empty())
            {
                parts.push(description);
            }

            let mut result = SearchResult::new(
                title,
                format!("https://archive.org/details/{identifier}"),
                parts.join(" - "),
                "internet_archive",
            );
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::General.to_string();
            result.thumbnail = Some(format!("https://archive.org/services/img/{identifier}"));
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for InternetArchive {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://archive.org/advancedsearch.php?q={}&fl[]=identifier&fl[]=title&fl[]=description&fl[]=mediatype&fl[]=creator&fl[]=year&rows=10&page={}&output=json",
            urlencoding::encode(&query.query),
            query.page.max(1),
        );

        let body = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?
            .error_for_status()
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?
            .text()
            .await
            .map_err(|error| MetasearchError::ParseError(error.to_string()))?;

        parse_internet_archive_results(&body)
    }
}
