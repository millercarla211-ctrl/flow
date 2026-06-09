//! Maven Central package search via Sonatype's documented Solr search API.
//!
//! Reference: <https://central.sonatype.org/search/rest-api-guide/>

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

pub struct MavenCentral {
    metadata: EngineMetadata,
    client: Client,
}

impl MavenCentral {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "maven_central".to_string().into(),
                display_name: "Maven Central".to_string().into(),
                homepage: "https://central.sonatype.com".to_string().into(),
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
struct MavenResponse {
    response: MavenDocs,
}

#[derive(Debug, Deserialize)]
struct MavenDocs {
    #[serde(default)]
    docs: Vec<MavenDoc>,
}

#[derive(Debug, Deserialize)]
struct MavenDoc {
    id: Option<String>,
    g: Option<String>,
    a: Option<String>,
    #[serde(rename = "latestVersion")]
    latest_version: Option<String>,
    p: Option<String>,
    #[serde(rename = "versionCount")]
    version_count: Option<u64>,
}

pub fn parse_maven_central_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: MavenResponse = serde_json::from_str(body)?;
    let results = response
        .response
        .docs
        .into_iter()
        .enumerate()
        .filter_map(|(i, doc)| {
            let group = doc.g.filter(|value| !value.trim().is_empty())?;
            let artifact = doc.a.filter(|value| !value.trim().is_empty())?;
            let title = doc
                .id
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| format!("{group}:{artifact}"));
            let url = format!("https://central.sonatype.com/artifact/{group}/{artifact}");

            let mut parts = Vec::new();
            if let Some(version) = doc.latest_version.filter(|value| !value.trim().is_empty()) {
                parts.push(format!("latest {version}"));
            }
            if let Some(packaging) = doc.p.filter(|value| !value.trim().is_empty()) {
                parts.push(format!("packaging {packaging}"));
            }
            if let Some(count) = doc.version_count {
                parts.push(format!("{count} versions"));
            }

            let mut result = SearchResult::new(title, url, parts.join(" - "), "maven_central");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::IT.to_string();
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for MavenCentral {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let start = (query.page.max(1) - 1) * 10;
        let url = format!(
            "https://search.maven.org/solrsearch/select?q={}&rows=10&start={}&wt=json",
            urlencoding::encode(&query.query),
            start,
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

        parse_maven_central_results(&body)
    }
}
