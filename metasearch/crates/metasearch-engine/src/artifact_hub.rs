//! Artifact Hub cloud-native package search via its documented API.
//!
//! Reference: <https://artifacthub.github.io/hub/api/>

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

pub struct ArtifactHub {
    metadata: EngineMetadata,
    client: Client,
}

impl ArtifactHub {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "artifact_hub".to_string().into(),
                display_name: "Artifact Hub".to_string().into(),
                homepage: "https://artifacthub.io".to_string().into(),
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
struct ArtifactHubResponse {
    #[serde(default)]
    packages: Vec<ArtifactHubPackage>,
}

#[derive(Debug, Deserialize)]
struct ArtifactHubPackage {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    app_version: Option<String>,
    stars: Option<u64>,
    deprecated: Option<bool>,
    repository: Option<ArtifactHubRepository>,
}

#[derive(Debug, Deserialize)]
struct ArtifactHubRepository {
    name: Option<String>,
    display_name: Option<String>,
    verified_publisher: Option<bool>,
}

pub fn parse_artifact_hub_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: ArtifactHubResponse = serde_json::from_str(body)?;
    let results = response
        .packages
        .into_iter()
        .enumerate()
        .filter_map(|(i, package)| {
            let name = non_empty(package.name)?;
            let url = format!(
                "https://artifacthub.io/packages/search?ts_query_web={}",
                urlencoding::encode(&name),
            );

            let mut parts = Vec::new();
            if let Some(repository) = package.repository {
                let repo_name =
                    non_empty(repository.display_name).or_else(|| non_empty(repository.name));
                if let Some(repo_name) = repo_name {
                    parts.push(repo_name);
                }
                if repository.verified_publisher.unwrap_or(false) {
                    parts.push("verified publisher".to_string());
                }
            }
            if let Some(version) = non_empty(package.version) {
                parts.push(format!("v{version}"));
            }
            if let Some(app_version) = non_empty(package.app_version) {
                parts.push(format!("app {app_version}"));
            }
            if package.deprecated.unwrap_or(false) {
                parts.push("deprecated".to_string());
            }
            if let Some(stars) = package.stars {
                parts.push(format!("stars: {stars}"));
            }
            if let Some(description) = non_empty(package.description) {
                parts.push(description);
            }

            let mut result = SearchResult::new(name, url, parts.join(" - "), "artifact_hub");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::IT.to_string();
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for ArtifactHub {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let offset = (query.page.max(1) - 1) * 10;
        let url = format!(
            "https://artifacthub.io/api/v1/packages/search?ts_query_web={}&limit=10&offset={}",
            urlencoding::encode(&query.query),
            offset,
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

        parse_artifact_hub_results(&body)
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}
