//! National Vulnerability Database search via the official CVE API 2.0.
//!
//! Reference: <https://nvd.nist.gov/developers/vulnerabilities>

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
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

const PAGE_SIZE: u32 = 10;
const USER_AGENT: &str =
    "metasearch-engine/1.0 (+https://github.com/najmus-sakib-hossain/metasearch)";

pub struct Nvd {
    metadata: EngineMetadata,
    client: Client,
}

impl Nvd {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "nvd".to_string().into(),
                display_name: "NVD".to_string().into(),
                homepage: "https://nvd.nist.gov".to_string().into(),
                categories: smallvec![SearchCategory::IT],
                enabled: true,
                timeout_ms: 12000,
                weight: 1.0,
            },
            client,
        }
    }
}

#[derive(Debug, Deserialize)]
struct NvdResponse {
    #[serde(default)]
    vulnerabilities: Vec<NvdVulnerability>,
}

#[derive(Debug, Deserialize)]
struct NvdVulnerability {
    cve: NvdCve,
}

#[derive(Debug, Deserialize)]
struct NvdCve {
    id: Option<String>,
    published: Option<String>,
    #[serde(rename = "vulnStatus")]
    vuln_status: Option<String>,
    #[serde(default)]
    descriptions: Vec<NvdDescription>,
    metrics: Option<serde_json::Value>,
    #[serde(default)]
    weaknesses: Vec<NvdWeakness>,
}

#[derive(Debug, Deserialize)]
struct NvdDescription {
    lang: Option<String>,
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NvdWeakness {
    #[serde(default)]
    description: Vec<NvdDescription>,
}

pub fn parse_nvd_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: NvdResponse = serde_json::from_str(body)?;
    let results = response
        .vulnerabilities
        .into_iter()
        .enumerate()
        .filter_map(|(i, vulnerability)| {
            let cve = vulnerability.cve;
            let cve_id = non_empty(cve.id)?;
            let mut parts = Vec::new();

            if let Some(status) = non_empty(cve.vuln_status) {
                parts.push(status);
            }
            if let Some(metric) = cve.metrics.as_ref().and_then(metric_summary) {
                parts.push(metric);
            }
            let weaknesses = cve
                .weaknesses
                .into_iter()
                .flat_map(|weakness| weakness.description)
                .filter(|description| description.lang.as_deref() == Some("en"))
                .filter_map(|description| non_empty(description.value))
                .take(3)
                .collect::<Vec<_>>()
                .join(", ");
            if !weaknesses.is_empty() {
                parts.push(weaknesses);
            }
            if let Some(description) = cve
                .descriptions
                .into_iter()
                .find(|description| description.lang.as_deref() == Some("en"))
                .and_then(|description| non_empty(description.value))
            {
                parts.push(description);
            }

            let mut result = SearchResult::new(
                cve_id.clone(),
                format!("https://nvd.nist.gov/vuln/detail/{cve_id}"),
                parts.join(" - "),
                "nvd",
            );
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::IT.to_string();
            result.published_date = cve.published.as_deref().and_then(parse_nvd_timestamp);
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for Nvd {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let start_index = (query.page.max(1) - 1) * PAGE_SIZE;
        let url = format!(
            "https://services.nvd.nist.gov/rest/json/cves/2.0?keywordSearch={}&resultsPerPage={}&startIndex={}&noRejected",
            urlencoding::encode(&query.query),
            PAGE_SIZE,
            start_index,
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

        parse_nvd_results(&body)
    }
}

fn metric_summary(metrics: &serde_json::Value) -> Option<String> {
    for key in [
        "cvssMetricV40",
        "cvssMetricV31",
        "cvssMetricV30",
        "cvssMetricV2",
    ] {
        let Some(metric) = metrics
            .get(key)
            .and_then(|value| value.as_array())
            .and_then(|values| values.first())
        else {
            continue;
        };
        let cvss = metric.get("cvssData")?;
        let score = cvss.get("baseScore").and_then(|value| value.as_f64());
        let severity = cvss
            .get("baseSeverity")
            .or_else(|| metric.get("baseSeverity"))
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty());

        return match (severity, score) {
            (Some(severity), Some(score)) => Some(format!("{severity} CVSS {score:.1}")),
            (Some(severity), None) => Some(severity.to_string()),
            (None, Some(score)) => Some(format!("CVSS {score:.1}")),
            (None, None) => None,
        };
    }

    None
}

fn parse_nvd_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(DateTime::<Utc>::from)
        .ok()
        .or_else(|| {
            DateTime::parse_from_rfc3339(&format!("{value}Z"))
                .ok()
                .map(DateTime::<Utc>::from)
        })
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|value| DateTime::<Utc>::from_naive_utc_and_offset(value, Utc))
        })
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}
