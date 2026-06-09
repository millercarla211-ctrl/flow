//! NuGet package search via the documented V3 SearchQueryService.
//!
//! Reference: <https://learn.microsoft.com/en-us/nuget/api/search-query-service-resource>

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

pub struct Nuget {
    metadata: EngineMetadata,
    client: Client,
}

impl Nuget {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "nuget".to_string().into(),
                display_name: "NuGet".to_string().into(),
                homepage: "https://www.nuget.org".to_string().into(),
                categories: smallvec![SearchCategory::IT],
                enabled: true,
                timeout_ms: 6000,
                weight: 1.0,
            },
            client,
        }
    }
}

#[derive(Debug, Deserialize)]
struct NugetResponse {
    #[serde(default)]
    data: Vec<NugetPackage>,
}

#[derive(Debug, Deserialize)]
struct NugetPackage {
    id: Option<String>,
    version: Option<String>,
    description: Option<String>,
    authors: Option<Vec<String>>,
    #[serde(rename = "totalDownloads")]
    total_downloads: Option<u64>,
}

pub fn parse_nuget_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: NugetResponse = serde_json::from_str(body)?;
    let results = response
        .data
        .into_iter()
        .enumerate()
        .filter_map(|(i, package)| {
            let id = package.id.filter(|value| !value.trim().is_empty())?;
            let url = format!("https://www.nuget.org/packages/{id}");

            let mut parts = Vec::new();
            if let Some(version) = package.version.filter(|value| !value.trim().is_empty()) {
                parts.push(format!("v{version}"));
            }
            if let Some(authors) = package.authors {
                let authors = authors
                    .into_iter()
                    .filter(|author| !author.trim().is_empty())
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(", ");
                if !authors.is_empty() {
                    parts.push(authors);
                }
            }
            if let Some(description) = package.description.filter(|value| !value.trim().is_empty())
            {
                parts.push(description);
            }
            if let Some(downloads) = package.total_downloads {
                parts.push(format!("{downloads} downloads"));
            }

            let mut result = SearchResult::new(id, url, parts.join(" - "), "nuget");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::IT.to_string();
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for Nuget {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let skip = (query.page.max(1) - 1) * 10;
        let url = format!(
            "https://azuresearch-usnc.nuget.org/query?q={}&skip={}&take=10&prerelease=false",
            urlencoding::encode(&query.query),
            skip,
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

        parse_nuget_results(&body)
    }
}
