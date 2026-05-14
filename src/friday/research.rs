use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{friday_answer_search_plan, friday_research_search_plan};
use crate::search::{
    MetasearchApiResponse, MetasearchApiResult, MetasearchServerConfig, SearchRequestPlan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayResearchStageKind {
    Clarify,
    MetasearchDiscovery,
    SourceScoring,
    CitationExtraction,
    Synthesis,
    Export,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayResearchStageStatus {
    Planned,
    Ready,
    Running,
    Complete,
    Failed,
}

impl FridayResearchStageStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Ready => "ready",
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayResearchStage {
    pub kind: FridayResearchStageKind,
    pub status: FridayResearchStageStatus,
    pub label: String,
    pub output_contract: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MetasearchExecutionMode {
    AdjacentServer,
    AdjacentCli,
    EmbeddedCrate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetasearchExecutionTarget {
    pub mode: MetasearchExecutionMode,
    pub available_by_default: bool,
    pub endpoint_or_command: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayResearchWorkflow {
    pub query: String,
    pub answer_plan: SearchRequestPlan,
    pub deep_research_plan: SearchRequestPlan,
    pub local_metasearch_api_path: String,
    pub metasearch_targets: Vec<MetasearchExecutionTarget>,
    pub stages: Vec<FridayResearchStage>,
    pub export_formats: Vec<String>,
    pub local_first: bool,
    pub forbids_perplexity_computer: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FridayCitationRecord {
    pub id: String,
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub engine: String,
    pub category: String,
    pub score: f64,
    pub published_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FridaySourceGroup {
    pub key: String,
    pub label: String,
    pub citations: Vec<FridayCitationRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FridayResearchReport {
    pub query: String,
    pub summary: String,
    pub citations: Vec<FridayCitationRecord>,
    pub source_groups: Vec<FridaySourceGroup>,
    pub engines_used: Vec<String>,
    pub engines_failed: Vec<String>,
    pub search_time_ms: u64,
    pub cached: bool,
}

impl FridayResearchWorkflow {
    pub fn for_query(query: impl Into<String>) -> Self {
        let query = query.into();
        let answer_plan = friday_answer_search_plan(query.clone());
        let deep_research_plan = friday_research_search_plan(query.clone());
        let local_metasearch_api_path =
            MetasearchServerConfig::default().api_path_for_plan(&deep_research_plan);
        Self {
            query,
            answer_plan,
            deep_research_plan,
            local_metasearch_api_path,
            metasearch_targets: default_metasearch_targets(),
            stages: default_research_stages(),
            export_formats: vec![
                "markdown-report".to_string(),
                "json-citations".to_string(),
                "source-ledger".to_string(),
            ],
            local_first: true,
            forbids_perplexity_computer: true,
        }
    }

    pub fn ready_stage_count(&self) -> usize {
        self.stages
            .iter()
            .filter(|stage| stage.status == FridayResearchStageStatus::Ready)
            .count()
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

impl FridayResearchReport {
    pub fn from_metasearch_response(response: &MetasearchApiResponse) -> Self {
        let citations = response
            .results
            .iter()
            .enumerate()
            .map(|(index, result)| citation_from_result(index + 1, result))
            .collect::<Vec<_>>();
        let source_groups = group_citations_by_category(&citations);

        Self {
            query: response.query.clone(),
            summary: format!(
                "Metasearch returned {} cited source candidates across {} engine(s).",
                citations.len(),
                response.engines_used.len()
            ),
            citations,
            source_groups,
            engines_used: response.engines_used.clone(),
            engines_failed: response.engines_failed.clone(),
            search_time_ms: response.search_time_ms,
            cached: response.cached,
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            format!("# Friday Research: {}", self.query),
            String::new(),
            self.summary.clone(),
            String::new(),
            format!(
                "- Engines used: {}",
                if self.engines_used.is_empty() {
                    "none".to_string()
                } else {
                    self.engines_used.join(", ")
                }
            ),
            format!("- Search time: {} ms", self.search_time_ms),
            format!("- Cached: {}", if self.cached { "yes" } else { "no" }),
        ];

        if !self.engines_failed.is_empty() {
            lines.push(format!(
                "- Engines failed: {}",
                self.engines_failed.join(", ")
            ));
        }

        lines.push(String::new());
        lines.push("## Source Groups".to_string());
        for group in &self.source_groups {
            lines.push(String::new());
            lines.push(format!("### {}", group.label));
            for citation in &group.citations {
                lines.push(format!(
                    "- [{}] [{}]({}) — {}",
                    citation.id, citation.title, citation.url, citation.snippet
                ));
            }
        }

        lines.push(String::new());
        lines.push("## Citation Ledger".to_string());
        for citation in &self.citations {
            lines.push(format!(
                "- [{}] engine=`{}` category=`{}` score=`{:.3}` url={}",
                citation.id, citation.engine, citation.category, citation.score, citation.url
            ));
        }

        lines.join("\n")
    }
}

pub fn default_metasearch_targets() -> Vec<MetasearchExecutionTarget> {
    vec![
        MetasearchExecutionTarget {
            mode: MetasearchExecutionMode::AdjacentServer,
            available_by_default: true,
            endpoint_or_command: "http://127.0.0.1:8888/api/v1/search?format=json&q=<query>"
                .to_string(),
            notes: vec![
                "Preferred path once the adjacent metasearch server is running.".to_string(),
                "Keeps source discovery outside proprietary search products.".to_string(),
            ],
        },
        MetasearchExecutionTarget {
            mode: MetasearchExecutionMode::AdjacentCli,
            available_by_default: false,
            endpoint_or_command: "cargo run -p metasearch-cli -- serve".to_string(),
            notes: vec![
                "Use from the metasearch workspace to start the local server.".to_string(),
                "The current CLI exposes server/config/engine commands; JSON search is served by the HTTP API."
                    .to_string(),
            ],
        },
        MetasearchExecutionTarget {
            mode: MetasearchExecutionMode::EmbeddedCrate,
            available_by_default: false,
            endpoint_or_command: "future: metasearch-engine path dependency or workspace member"
                .to_string(),
            notes: vec![
                "Reserved for a deeper Rust integration after the server API contract is stable."
                    .to_string(),
            ],
        },
    ]
}

fn default_research_stages() -> Vec<FridayResearchStage> {
    vec![
        stage(
            FridayResearchStageKind::Clarify,
            FridayResearchStageStatus::Ready,
            "Clarify scope",
            "One compact research objective, source requirements, and exclusion rules.",
        ),
        stage(
            FridayResearchStageKind::MetasearchDiscovery,
            FridayResearchStageStatus::Ready,
            "Discover sources through metasearch",
            "Grouped search results with title, URL, engine, snippet, timestamp, and vertical.",
        ),
        stage(
            FridayResearchStageKind::SourceScoring,
            FridayResearchStageStatus::Planned,
            "Score source quality",
            "Authority, recency, directness, duplication, and conflict scores per source.",
        ),
        stage(
            FridayResearchStageKind::CitationExtraction,
            FridayResearchStageStatus::Planned,
            "Extract citation ledger",
            "Claim-to-source ledger with exact source URLs and short quote-safe evidence notes.",
        ),
        stage(
            FridayResearchStageKind::Synthesis,
            FridayResearchStageStatus::Planned,
            "Synthesize answer or report",
            "Answer-first summary plus detailed report sections, caveats, and unresolved questions.",
        ),
        stage(
            FridayResearchStageKind::Export,
            FridayResearchStageStatus::Planned,
            "Export report",
            "Markdown report, citation JSON, and source ledger artifacts.",
        ),
    ]
}

fn stage(
    kind: FridayResearchStageKind,
    status: FridayResearchStageStatus,
    label: &str,
    output_contract: &str,
) -> FridayResearchStage {
    FridayResearchStage {
        kind,
        status,
        label: label.to_string(),
        output_contract: output_contract.to_string(),
    }
}

fn citation_from_result(index: usize, result: &MetasearchApiResult) -> FridayCitationRecord {
    FridayCitationRecord {
        id: format!("S{index}"),
        title: result.title.clone(),
        url: result.url.clone(),
        snippet: compact_snippet(&result.content),
        engine: result.engine.clone(),
        category: if result.category.trim().is_empty() {
            "general".to_string()
        } else {
            result.category.clone()
        },
        score: result.score,
        published_date: result.published_date.clone(),
    }
}

fn group_citations_by_category(citations: &[FridayCitationRecord]) -> Vec<FridaySourceGroup> {
    let mut grouped: BTreeMap<String, Vec<FridayCitationRecord>> = BTreeMap::new();
    for citation in citations {
        grouped
            .entry(citation.category.clone())
            .or_default()
            .push(citation.clone());
    }

    grouped
        .into_iter()
        .map(|(key, citations)| FridaySourceGroup {
            label: humanize_source_group(&key),
            key,
            citations,
        })
        .collect()
}

fn humanize_source_group(category: &str) -> String {
    category
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn compact_snippet(content: &str) -> String {
    let snippet = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if snippet.chars().count() <= 220 {
        return snippet;
    }

    let truncated = snippet.chars().take(217).collect::<String>();
    format!("{truncated}...")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_workflow_uses_metasearch_and_forbids_perplexity_computer() {
        let workflow = FridayResearchWorkflow::for_query("compare AI assistants");
        assert!(workflow.local_first);
        assert!(workflow.forbids_perplexity_computer);
        assert!(workflow.answer_plan.use_adjacent_metasearch);
        assert!(workflow.deep_research_plan.use_adjacent_metasearch);
        assert!(
            workflow
                .metasearch_targets
                .iter()
                .any(|target| target.mode == MetasearchExecutionMode::AdjacentServer)
        );
    }

    #[test]
    fn research_workflow_has_exportable_citation_contracts() {
        let workflow = FridayResearchWorkflow::for_query("best local OCR");
        assert!(workflow.ready_stage_count() >= 2);
        assert!(
            workflow
                .export_formats
                .contains(&"json-citations".to_string())
        );
        assert!(
            workflow
                .stages
                .iter()
                .any(|stage| stage.kind == FridayResearchStageKind::CitationExtraction)
        );
    }

    #[test]
    fn research_report_groups_sources_and_exports_markdown() {
        let response = MetasearchApiResponse {
            query: "local ai search".to_string(),
            results: vec![
                MetasearchApiResult {
                    title: "General source".to_string(),
                    url: "https://example.com/general".to_string(),
                    content: "General web evidence.".to_string(),
                    engine: "duckduckgo".to_string(),
                    engine_rank: 1,
                    score: 0.9,
                    thumbnail: None,
                    published_date: None,
                    category: "general".to_string(),
                    metadata: serde_json::Value::Null,
                },
                MetasearchApiResult {
                    title: "Science source".to_string(),
                    url: "https://example.com/science".to_string(),
                    content: "Academic evidence.".to_string(),
                    engine: "openalex".to_string(),
                    engine_rank: 1,
                    score: 0.8,
                    thumbnail: None,
                    published_date: None,
                    category: "science".to_string(),
                    metadata: serde_json::Value::Null,
                },
            ],
            number_of_results: 2,
            engines_used: vec!["duckduckgo".to_string(), "openalex".to_string()],
            engines_failed: Vec::new(),
            search_time_ms: 31,
            cached: false,
            category: None,
            categories: vec!["general".to_string(), "science".to_string()],
            page: 1,
            language: Some("en".to_string()),
        };

        let report = FridayResearchReport::from_metasearch_response(&response);
        let markdown = report.to_markdown();

        assert_eq!(report.citations.len(), 2);
        assert_eq!(report.source_groups.len(), 2);
        assert!(markdown.contains("[S1]"));
        assert!(markdown.contains("## Citation Ledger"));
    }
}
