//! Semantic Scholar search via the official Academic Graph API.

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

const SEARCH_URL: &str = "https://api.semanticscholar.org/graph/v1/paper/search";
const USER_AGENT: &str = "metasearch/0.1 (+https://github.com/najmus-sakib-hossain/metasearch)";

pub struct SemanticScholar {
    metadata: EngineMetadata,
    client: Client,
    api_key: Option<String>,
}

impl SemanticScholar {
    pub fn new(client: Client) -> Self {
        Self::with_api_key(client, None)
    }

    pub fn with_api_key(client: Client, api_key: Option<String>) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "semantic_scholar".to_string().into(),
                display_name: "Semantic Scholar".to_string().into(),
                homepage: "https://www.semanticscholar.org".to_string().into(),
                categories: smallvec![SearchCategory::Science],
                enabled: true,
                timeout_ms: 7000,
                weight: 1.2,
            },
            client,
            api_key: api_key.and_then(|key| {
                let trimmed = key.trim().to_string();
                (!trimmed.is_empty()).then_some(trimmed)
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SemanticScholarResponse {
    #[serde(default)]
    data: Vec<SemanticScholarPaper>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SemanticScholarPaper {
    paper_id: Option<String>,
    title: Option<String>,
    url: Option<String>,
    #[serde(rename = "abstract")]
    abstract_text: Option<String>,
    year: Option<u32>,
    venue: Option<String>,
    citation_count: Option<u64>,
    #[serde(default)]
    authors: Vec<SemanticScholarAuthor>,
    external_ids: Option<SemanticScholarExternalIds>,
    open_access_pdf: Option<SemanticScholarPdf>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarAuthor {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarExternalIds {
    #[serde(rename = "DOI")]
    doi: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarPdf {
    url: Option<String>,
}

pub fn parse_semantic_scholar_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: SemanticScholarResponse = serde_json::from_str(body)?;
    let mut results = Vec::new();

    for paper in response.data {
        let title = paper.title.unwrap_or_default().trim().to_string();
        if title.is_empty() {
            continue;
        }

        let fallback_url = paper
            .paper_id
            .as_deref()
            .map(|paper_id| format!("https://www.semanticscholar.org/paper/{paper_id}"));
        let url = paper
            .url
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                paper
                    .open_access_pdf
                    .and_then(|pdf| pdf.url)
                    .filter(|value| !value.trim().is_empty())
            })
            .or(fallback_url)
            .unwrap_or_else(|| "https://www.semanticscholar.org".to_string());

        let authors: Vec<String> = paper
            .authors
            .into_iter()
            .filter_map(|author| {
                let name = author.name?.trim().to_string();
                (!name.is_empty()).then_some(name)
            })
            .collect();

        let mut snippet_parts = Vec::new();
        if !authors.is_empty() {
            let author_text = if authors.len() > 3 {
                format!("{} et al.", authors[..3].join(", "))
            } else {
                authors.join(", ")
            };
            snippet_parts.push(author_text);
        }
        if let Some(year) = paper.year {
            snippet_parts.push(year.to_string());
        }
        if let Some(venue) = paper.venue.filter(|value| !value.trim().is_empty()) {
            snippet_parts.push(venue);
        }
        if let Some(citations) = paper.citation_count {
            snippet_parts.push(format!("{citations} citations"));
        }
        if let Some(doi) = paper
            .external_ids
            .and_then(|ids| ids.doi)
            .filter(|value| !value.trim().is_empty())
        {
            snippet_parts.push(format!("DOI: {}", doi.trim()));
        }
        if let Some(abstract_text) = paper
            .abstract_text
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            let snippet = if abstract_text.len() > 250 {
                format!("{}...", &abstract_text[..250])
            } else {
                abstract_text
            };
            snippet_parts.push(snippet);
        }

        let mut result = SearchResult::new(
            title,
            url,
            snippet_parts.join(" | "),
            "semantic_scholar".to_string(),
        );
        result.engine_rank = (results.len() + 1) as u32;
        result.category = SearchCategory::Science.to_string();
        results.push(result);
    }

    Ok(results)
}

#[async_trait]
impl SearchEngine for SemanticScholar {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let offset = query.page.saturating_sub(1) * 10;
        let fields =
            "title,url,abstract,year,venue,citationCount,authors,externalIds,openAccessPdf";

        let mut request = self
            .client
            .get(SEARCH_URL)
            .timeout(std::time::Duration::from_millis(self.metadata.timeout_ms))
            .header("User-Agent", USER_AGENT)
            .query(&[
                ("query", query.query.as_str()),
                ("limit", "10"),
                ("offset", &offset.to_string()),
                ("fields", fields),
            ]);

        if let Some(api_key) = &self.api_key {
            request = request.header("x-api-key", api_key);
        }

        let body = request
            .send()
            .await
            .map_err(|e| MetasearchError::HttpError(e.to_string()))?
            .error_for_status()
            .map_err(|e| MetasearchError::HttpError(e.to_string()))?
            .text()
            .await
            .map_err(|e| MetasearchError::ParseError(e.to_string()))?;

        parse_semantic_scholar_results(&body)
    }
}
