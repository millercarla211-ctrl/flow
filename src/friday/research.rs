use serde::{Deserialize, Serialize};

use super::{friday_answer_search_plan, friday_research_search_plan};
use crate::search::{MetasearchServerConfig, SearchRequestPlan};

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
}
