//! DOAJ article search via the public article search API.
//!
//! Reference: <https://doaj.org/docs/faq/>

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

pub struct Doaj {
    metadata: EngineMetadata,
    client: Client,
}

impl Doaj {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "doaj".to_string().into(),
                display_name: "DOAJ".to_string().into(),
                homepage: "https://doaj.org".to_string().into(),
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
struct DoajResponse {
    #[serde(default)]
    results: Vec<DoajHit>,
}

#[derive(Debug, Deserialize)]
struct DoajHit {
    id: Option<String>,
    bibjson: Option<BibJson>,
}

#[derive(Debug, Deserialize)]
struct BibJson {
    title: Option<String>,
    #[serde(rename = "abstract")]
    abstract_text: Option<String>,
    year: Option<JsonText>,
    journal: Option<DoajJournal>,
    link: Option<Vec<DoajLink>>,
    identifier: Option<Vec<DoajIdentifier>>,
    author: Option<Vec<DoajAuthor>>,
}

#[derive(Debug, Deserialize)]
struct DoajJournal {
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DoajLink {
    url: Option<String>,
    #[serde(rename = "type")]
    link_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DoajIdentifier {
    #[serde(rename = "type")]
    id_type: Option<String>,
    id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DoajAuthor {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JsonText {
    String(String),
    Number(u64),
}

impl JsonText {
    fn into_text(self) -> String {
        match self {
            Self::String(value) => value,
            Self::Number(value) => value.to_string(),
        }
    }
}

pub fn parse_doaj_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: DoajResponse = serde_json::from_str(body)?;
    let results = response
        .results
        .into_iter()
        .enumerate()
        .filter_map(|(i, hit)| {
            let bibjson = hit.bibjson?;
            let title = bibjson.title?;
            let fallback_url = hit.id.map(|id| format!("https://doaj.org/article/{id}"));
            let doi_url = bibjson.identifier.as_ref().and_then(|identifiers| {
                identifiers.iter().find_map(|identifier| {
                    let id_type = identifier.id_type.as_deref().unwrap_or_default();
                    let id = identifier.id.as_deref()?;
                    id_type
                        .eq_ignore_ascii_case("doi")
                        .then(|| format!("https://doi.org/{id}"))
                })
            });
            let link_url = bibjson.link.as_ref().and_then(|links| {
                links
                    .iter()
                    .find(|link| link.link_type.as_deref() == Some("fulltext"))
                    .or_else(|| links.first())
                    .and_then(|link| link.url.clone())
            });
            let url = link_url.or(doi_url).or(fallback_url)?;

            let mut parts = Vec::new();
            if let Some(authors) = bibjson.author {
                let names = authors
                    .into_iter()
                    .filter_map(|author| author.name)
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(", ");
                if !names.is_empty() {
                    parts.push(names);
                }
            }
            if let Some(year) = bibjson.year {
                parts.push(year.into_text());
            }
            if let Some(journal) = bibjson.journal.and_then(|journal| journal.title) {
                parts.push(journal);
            }
            if let Some(abstract_text) = bibjson.abstract_text {
                parts.push(abstract_text);
            }

            let mut result = SearchResult::new(title, url, parts.join(" - "), "doaj");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::Science.to_string();
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for Doaj {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://doaj.org/api/v4/search/articles/{}?page={}&pageSize=10",
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

        parse_doaj_results(&body)
    }
}
