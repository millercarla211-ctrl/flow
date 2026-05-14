use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{friday_answer_search_plan, friday_research_search_plan};
use crate::models::GenerationMetrics;
use crate::runtime::FlowLocalRuntime;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayResearchEventKind {
    WorkflowReady,
    MetasearchResponseReceived,
    SourceGroupsPrepared,
    CitationLedgerPrepared,
    ReportExported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayResearchRunEvent {
    pub sequence: u16,
    pub kind: FridayResearchEventKind,
    pub stage: FridayResearchStageKind,
    pub progress_percent: u8,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayResearchExportManifest {
    pub query: String,
    pub root_dir: PathBuf,
    pub report_markdown: PathBuf,
    pub citations_json: PathBuf,
    pub source_groups_json: PathBuf,
    pub events_json: PathBuf,
    pub manifest_json: PathBuf,
    pub citation_count: usize,
    pub source_group_count: usize,
    pub search_time_ms: u64,
    pub cached: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayAnswerDeltaKind {
    Text,
    CitationReference,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayAnswerDelta {
    pub sequence: u16,
    pub kind: FridayAnswerDeltaKind,
    pub text: String,
    pub citation_ids: Vec<String>,
    pub progress_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FridayGenerationSummary {
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub total_time_ms: u128,
    pub tokens_per_second: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FridaySynthesizedAnswer {
    pub query: String,
    pub answer: String,
    pub citation_ids: Vec<String>,
    pub deltas: Vec<FridayAnswerDelta>,
    pub generation: Option<FridayGenerationSummary>,
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
                    "- [{}] [{}]({}) - {}",
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

    pub fn progress_events(&self) -> Vec<FridayResearchRunEvent> {
        vec![
            research_event(
                1,
                FridayResearchEventKind::WorkflowReady,
                FridayResearchStageKind::Clarify,
                10,
                "Research workflow prepared with local-first metasearch boundaries.",
            ),
            research_event(
                2,
                FridayResearchEventKind::MetasearchResponseReceived,
                FridayResearchStageKind::MetasearchDiscovery,
                40,
                &format!(
                    "Received {} source candidate(s) from {} engine(s).",
                    self.citations.len(),
                    self.engines_used.len()
                ),
            ),
            research_event(
                3,
                FridayResearchEventKind::SourceGroupsPrepared,
                FridayResearchStageKind::SourceScoring,
                60,
                &format!("Prepared {} source group(s).", self.source_groups.len()),
            ),
            research_event(
                4,
                FridayResearchEventKind::CitationLedgerPrepared,
                FridayResearchStageKind::CitationExtraction,
                80,
                &format!("Prepared {} citation record(s).", self.citations.len()),
            ),
            research_event(
                5,
                FridayResearchEventKind::ReportExported,
                FridayResearchStageKind::Export,
                100,
                "Markdown report, citation ledger, source groups, and event log are ready.",
            ),
        ]
    }

    pub fn write_bundle(
        &self,
        output_dir: impl AsRef<Path>,
    ) -> Result<FridayResearchExportManifest> {
        let root_dir = output_dir.as_ref().to_path_buf();
        fs::create_dir_all(&root_dir)
            .with_context(|| format!("Could not create {}", root_dir.display()))?;

        let report_markdown = root_dir.join("report.md");
        let citations_json = root_dir.join("citations.json");
        let source_groups_json = root_dir.join("source-groups.json");
        let events_json = root_dir.join("events.json");
        let manifest_json = root_dir.join("manifest.json");

        fs::write(&report_markdown, self.to_markdown())
            .with_context(|| format!("Could not write {}", report_markdown.display()))?;
        fs::write(
            &citations_json,
            serde_json::to_string_pretty(&self.citations)?,
        )
        .with_context(|| format!("Could not write {}", citations_json.display()))?;
        fs::write(
            &source_groups_json,
            serde_json::to_string_pretty(&self.source_groups)?,
        )
        .with_context(|| format!("Could not write {}", source_groups_json.display()))?;
        fs::write(
            &events_json,
            serde_json::to_string_pretty(&self.progress_events())?,
        )
        .with_context(|| format!("Could not write {}", events_json.display()))?;

        let manifest = FridayResearchExportManifest {
            query: self.query.clone(),
            root_dir,
            report_markdown,
            citations_json,
            source_groups_json,
            events_json,
            manifest_json,
            citation_count: self.citations.len(),
            source_group_count: self.source_groups.len(),
            search_time_ms: self.search_time_ms,
            cached: self.cached,
        };

        fs::write(
            &manifest.manifest_json,
            serde_json::to_string_pretty(&manifest)?,
        )
        .with_context(|| format!("Could not write {}", manifest.manifest_json.display()))?;

        Ok(manifest)
    }

    pub fn synthesis_prompt(&self) -> String {
        let sources = self
            .citations
            .iter()
            .take(10)
            .map(|citation| {
                format!(
                    "[{}] {} | {} | {}",
                    citation.id, citation.title, citation.url, citation.snippet
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "\
You are Friday's local research synthesizer.
Answer the query using only the provided source notes.
Use compact paragraphs.
Cite claims with bracketed source IDs like [S1].
If the sources are not enough, say what is missing.

Query:
{}

Sources:
{}

Answer:",
            self.query, sources
        )
    }

    pub async fn synthesize_with_runtime(
        &self,
        runtime: &FlowLocalRuntime,
    ) -> Result<FridaySynthesizedAnswer> {
        let prompt = self.synthesis_prompt();
        let (answer, metrics) = runtime.generate_quality_chat_with_metrics(&prompt).await?;
        Ok(self.synthesized_answer_from_text(answer, Some(metrics.into())))
    }

    pub fn synthesized_answer_from_text(
        &self,
        answer: impl Into<String>,
        generation: Option<FridayGenerationSummary>,
    ) -> FridaySynthesizedAnswer {
        let answer = answer.into();
        let citation_ids = extract_citation_ids(&answer, &self.citations);
        let mut deltas = chunk_answer_deltas(&answer, &citation_ids);
        deltas.push(FridayAnswerDelta {
            sequence: deltas.len() as u16 + 1,
            kind: FridayAnswerDeltaKind::Done,
            text: "done".to_string(),
            citation_ids: citation_ids.clone(),
            progress_percent: 100,
        });

        FridaySynthesizedAnswer {
            query: self.query.clone(),
            answer,
            citation_ids,
            deltas,
            generation,
        }
    }
}

impl From<GenerationMetrics> for FridayGenerationSummary {
    fn from(metrics: GenerationMetrics) -> Self {
        Self {
            prompt_tokens: metrics.prompt_tokens,
            generated_tokens: metrics.generated_tokens,
            total_time_ms: metrics.total_time_ms,
            tokens_per_second: metrics.tokens_per_second,
        }
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

fn research_event(
    sequence: u16,
    kind: FridayResearchEventKind,
    stage: FridayResearchStageKind,
    progress_percent: u8,
    message: &str,
) -> FridayResearchRunEvent {
    FridayResearchRunEvent {
        sequence,
        kind,
        stage,
        progress_percent,
        message: message.to_string(),
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

fn extract_citation_ids(answer: &str, citations: &[FridayCitationRecord]) -> Vec<String> {
    citations
        .iter()
        .filter(|citation| answer.contains(&format!("[{}]", citation.id)))
        .map(|citation| citation.id.clone())
        .collect()
}

fn chunk_answer_deltas(answer: &str, citation_ids: &[String]) -> Vec<FridayAnswerDelta> {
    let chunks = sentence_like_chunks(answer);
    let total = chunks.len().max(1);
    chunks
        .into_iter()
        .enumerate()
        .map(|(index, chunk)| {
            let sequence = index as u16 + 1;
            let citation_ids = citation_ids
                .iter()
                .filter(|id| chunk.contains(&format!("[{id}]")))
                .cloned()
                .collect::<Vec<_>>();
            FridayAnswerDelta {
                sequence,
                kind: if citation_ids.is_empty() {
                    FridayAnswerDeltaKind::Text
                } else {
                    FridayAnswerDeltaKind::CitationReference
                },
                text: chunk,
                citation_ids,
                progress_percent: (((index + 1) as f32 / total as f32) * 95.0)
                    .round()
                    .clamp(1.0, 95.0) as u8,
            }
        })
        .collect()
}

fn sentence_like_chunks(answer: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for ch in answer.chars() {
        current.push(ch);
        if matches!(ch, '.' | '?' | '!' | '\n') {
            let chunk = current.trim();
            if !chunk.is_empty() {
                chunks.push(chunk.to_string());
            }
            current.clear();
        }
    }

    let trailing = current.trim();
    if !trailing.is_empty() {
        chunks.push(trailing.to_string());
    }

    if chunks.is_empty() && !answer.trim().is_empty() {
        chunks.push(answer.trim().to_string());
    }

    chunks
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

    #[test]
    fn research_report_persists_bundle_files() {
        let response = MetasearchApiResponse {
            query: "bundle test".to_string(),
            results: vec![MetasearchApiResult {
                title: "Bundle source".to_string(),
                url: "https://example.com/bundle".to_string(),
                content: "Bundle evidence.".to_string(),
                engine: "duckduckgo".to_string(),
                engine_rank: 1,
                score: 0.7,
                thumbnail: None,
                published_date: None,
                category: "general".to_string(),
                metadata: serde_json::Value::Null,
            }],
            number_of_results: 1,
            engines_used: vec!["duckduckgo".to_string()],
            engines_failed: Vec::new(),
            search_time_ms: 12,
            cached: false,
            category: None,
            categories: vec!["general".to_string()],
            page: 1,
            language: Some("en".to_string()),
        };
        let report = FridayResearchReport::from_metasearch_response(&response);
        let root =
            std::env::temp_dir().join(format!("friday-research-bundle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);

        let manifest = report.write_bundle(&root).unwrap();

        assert!(manifest.report_markdown.exists());
        assert!(manifest.citations_json.exists());
        assert!(manifest.source_groups_json.exists());
        assert!(manifest.events_json.exists());
        assert!(manifest.manifest_json.exists());
        assert_eq!(manifest.citation_count, 1);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn research_report_builds_citation_aware_answer_deltas() {
        let response = MetasearchApiResponse {
            query: "citation synthesis".to_string(),
            results: vec![MetasearchApiResult {
                title: "Citation source".to_string(),
                url: "https://example.com/citation".to_string(),
                content: "Citation evidence.".to_string(),
                engine: "duckduckgo".to_string(),
                engine_rank: 1,
                score: 0.7,
                thumbnail: None,
                published_date: None,
                category: "general".to_string(),
                metadata: serde_json::Value::Null,
            }],
            number_of_results: 1,
            engines_used: vec!["duckduckgo".to_string()],
            engines_failed: Vec::new(),
            search_time_ms: 12,
            cached: false,
            category: None,
            categories: vec!["general".to_string()],
            page: 1,
            language: Some("en".to_string()),
        };
        let report = FridayResearchReport::from_metasearch_response(&response);
        let answer = report.synthesized_answer_from_text(
            "Friday can cite local metasearch sources [S1]. This answer is chunked.",
            None,
        );

        assert_eq!(answer.citation_ids, vec!["S1".to_string()]);
        assert!(
            answer
                .deltas
                .iter()
                .any(|delta| delta.kind == FridayAnswerDeltaKind::CitationReference)
        );
        assert_eq!(
            answer.deltas.last().map(|delta| delta.kind),
            Some(FridayAnswerDeltaKind::Done)
        );
    }
}
