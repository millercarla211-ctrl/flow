pub mod artifacts;
pub mod checks;
pub mod dashboard_export;
pub mod dashboard_release_checklist;
pub mod dashboard_host_bridge;
pub mod dashboard_host_runner;
pub mod dashboard_product_ui;
pub mod dashboard_runner_release;
pub mod execution_handoff;
pub mod live_ui;
pub mod multimodal;
pub mod readiness;
pub mod research;
pub mod release_qa;
pub mod route_visuals;
pub mod runtime;
pub mod ui;
pub mod verification;
pub mod workspace;

pub use artifacts::*;
pub use checks::*;
pub use dashboard_export::*;
pub use dashboard_release_checklist::*;
pub use dashboard_host_bridge::*;
pub use dashboard_host_runner::*;
pub use dashboard_product_ui::*;
pub use dashboard_runner_release::*;
pub use execution_handoff::*;
pub use live_ui::*;
pub use multimodal::*;
pub use readiness::*;
pub use research::*;
pub use release_qa::*;
pub use route_visuals::*;
pub use runtime::*;
pub use ui::*;
pub use verification::*;
pub use workspace::*;

use serde::{Deserialize, Serialize};

use crate::search::{MetasearchBridge, SearchRequestPlan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayCompetitor {
    ChatGpt,
    Gemini,
    Perplexity,
    Grok,
    Claude,
}

impl FridayCompetitor {
    pub fn label(self) -> &'static str {
        match self {
            Self::ChatGpt => "ChatGPT",
            Self::Gemini => "Gemini",
            Self::Perplexity => "Perplexity",
            Self::Grok => "Grok",
            Self::Claude => "Claude",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayWorkspaceArea {
    Ask,
    Search,
    Research,
    Agents,
    Canvas,
    Projects,
    Memory,
    Connectors,
    Voice,
    Artifacts,
    Automations,
    Code,
    Multimodal,
}

impl FridayWorkspaceArea {
    pub fn route(self) -> &'static str {
        match self {
            Self::Ask => "/ask",
            Self::Search => "/search",
            Self::Research => "/research",
            Self::Agents => "/agents",
            Self::Canvas => "/canvas",
            Self::Projects => "/projects",
            Self::Memory => "/memory",
            Self::Connectors => "/connectors",
            Self::Voice => "/voice",
            Self::Artifacts => "/artifacts",
            Self::Automations => "/automations",
            Self::Code => "/code",
            Self::Multimodal => "/multimodal",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Ask => "Ask",
            Self::Search => "Search",
            Self::Research => "Research",
            Self::Agents => "Agents",
            Self::Canvas => "Canvas",
            Self::Projects => "Projects",
            Self::Memory => "Memory",
            Self::Connectors => "Connectors",
            Self::Voice => "Voice",
            Self::Artifacts => "Artifacts",
            Self::Automations => "Automations",
            Self::Code => "Code",
            Self::Multimodal => "Multimodal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayFeatureStatus {
    Shipped,
    Partial,
    Planned,
    Blocked,
}

impl FridayFeatureStatus {
    fn multiplier(self) -> f32 {
        match self {
            Self::Shipped => 1.0,
            Self::Partial => 0.55,
            Self::Planned => 0.15,
            Self::Blocked => 0.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Shipped => "shipped",
            Self::Partial => "partial",
            Self::Planned => "planned",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayCapability {
    pub competitor: FridayCompetitor,
    pub area: FridayWorkspaceArea,
    pub feature: String,
    pub weight: u8,
    pub friday_status: FridayFeatureStatus,
    pub friday_route: String,
    pub local_first: bool,
    pub uses_metasearch: bool,
    pub implementation_note: String,
    pub source_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayWorkspaceView {
    pub area: FridayWorkspaceArea,
    pub route: String,
    pub title: String,
    pub objective: String,
    pub primary_model_role: String,
    pub required_tools: Vec<String>,
    pub local_boundary: String,
    pub empty_state: String,
    pub error_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridaySearchPolicy {
    pub engine: String,
    pub forbids_perplexity_computer: bool,
    pub requires_citations: bool,
    pub source_controls: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FridayProductPlan {
    pub product_name: String,
    pub measured_on: String,
    pub score_out_of_100: u8,
    pub search_policy: FridaySearchPolicy,
    pub workspace_views: Vec<FridayWorkspaceView>,
    pub capabilities: Vec<FridayCapability>,
    pub top_priorities: Vec<String>,
}

impl FridayProductPlan {
    pub fn capability_gaps(&self) -> Vec<&FridayCapability> {
        self.capabilities
            .iter()
            .filter(|capability| capability.friday_status != FridayFeatureStatus::Shipped)
            .collect()
    }

    pub fn metasearch_capabilities(&self) -> Vec<&FridayCapability> {
        self.capabilities
            .iter()
            .filter(|capability| capability.uses_metasearch)
            .collect()
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn default_friday_product_plan() -> FridayProductPlan {
    let capabilities = friday_capabilities();

    FridayProductPlan {
        product_name: "Friday".to_string(),
        measured_on: "2026-05-14".to_string(),
        score_out_of_100: score_capabilities(&capabilities),
        search_policy: FridaySearchPolicy {
            engine: "adjacent-metasearch-rust-crate".to_string(),
            forbids_perplexity_computer: true,
            requires_citations: true,
            source_controls: vec![
                "web".to_string(),
                "news".to_string(),
                "academic".to_string(),
                "code".to_string(),
                "models".to_string(),
                "files".to_string(),
            ],
            notes: vec![
                "Friday search must route through the local metasearch crate or a host-provided metasearch service."
                    .to_string(),
                "Do not depend on Perplexity Computer or a browser-control provider for answer search."
                    .to_string(),
                "Every research answer should preserve source provenance and exportable citation records."
                    .to_string(),
            ],
        },
        workspace_views: friday_workspace_views(),
        capabilities,
        top_priorities: vec![
            "Wire Ask and Research to a streaming local-first model route with metasearch citations."
                .to_string(),
            "Connect Projects, Memory, and Connectors stores to the Friday UI before cloud sync."
                .to_string(),
            "Promote WhisperFlow Beater into Friday Voice without weakening the low-resource STT path."
                .to_string(),
            "Add Canvas and Artifacts as editable output surfaces for docs, code, UI, and reports."
                .to_string(),
            "Add an Agents page that can run browser/code/tool tasks behind explicit approval."
                .to_string(),
        ],
    }
}

pub fn friday_answer_search_plan(query: impl Into<String>) -> SearchRequestPlan {
    MetasearchBridge::for_friday_answer_search(query)
}

pub fn friday_research_search_plan(query: impl Into<String>) -> SearchRequestPlan {
    MetasearchBridge::for_friday_research(query)
}

fn friday_workspace_views() -> Vec<FridayWorkspaceView> {
    vec![
        view(
            FridayWorkspaceArea::Ask,
            "Local-first streaming chat with model picker, source toggles, image/file inputs, and tool visibility.",
            "quality-chat",
            &["local-llm", "remote-router", "tool-router"],
        ),
        view(
            FridayWorkspaceArea::Search,
            "Answer-first cited search powered by the adjacent metasearch crate.",
            "quality-chat",
            &["metasearch", "citation-store", "source-controls"],
        ),
        view(
            FridayWorkspaceArea::Research,
            "Deep research plans, progress logs, source selection, cited reports, and exports.",
            "daily-smart",
            &["metasearch", "rlm", "serializer", "report-export"],
        ),
        view(
            FridayWorkspaceArea::Agents,
            "Approved multi-step tasks for browser, code, files, terminal, connectors, and local tools.",
            "tool-agent",
            &[
                "xlam-router",
                "host-approvals",
                "audit-log",
                "tool-registry",
            ],
        ),
        view(
            FridayWorkspaceArea::Canvas,
            "Editable workspace for long answers, markdown, code, generated UI, and side-by-side revisions.",
            "coding",
            &["artifact-store", "diff-engine", "preview-runner"],
        ),
        view(
            FridayWorkspaceArea::Projects,
            "Persistent workspaces with files, instructions, memory, threads, and scoped tools.",
            "quality-chat",
            &["local-store", "file-index", "project-memory"],
        ),
        view(
            FridayWorkspaceArea::Memory,
            "User-controlled memories, facts, preferences, and imported history with explicit review.",
            "helper",
            &["memory-store", "review-queue", "privacy-policy"],
        ),
        view(
            FridayWorkspaceArea::Connectors,
            "Local files, MCP servers, app connectors, provider accounts, and future sync integrations.",
            "tool-agent",
            &["mcp-registry", "provider-router", "permission-gate"],
        ),
        view(
            FridayWorkspaceArea::Voice,
            "WhisperFlow Beater voice input, local STT/TTS, wake control, dictation overlay, and transcript actions.",
            "speech",
            &["stt", "tts", "wake", "overlay", "host-dictation"],
        ),
        view(
            FridayWorkspaceArea::Artifacts,
            "Generated files, apps, charts, receipts, research reports, snippets, and reusable cards.",
            "coding",
            &["artifact-store", "exporters", "preview-runner"],
        ),
        view(
            FridayWorkspaceArea::Automations,
            "Scheduled jobs, reminders, background research, recurring checks, and local agent runs.",
            "tool-agent",
            &["scheduler", "audit-log", "approval-policy"],
        ),
        view(
            FridayWorkspaceArea::Code,
            "Codex/Claude-Code style coding workspace with diffs, terminals, plans, reviews, and checkpoints.",
            "coding",
            &["codex-adapter", "forge", "terminal-gate", "review-engine"],
        ),
        view(
            FridayWorkspaceArea::Multimodal,
            "Image, OCR, vision, audio, video, music, and generated media surfaces.",
            "multimodal",
            &["ocr", "vlm", "tts", "stt", "media-store"],
        ),
    ]
}

fn view(
    area: FridayWorkspaceArea,
    objective: &str,
    primary_model_role: &str,
    required_tools: &[&str],
) -> FridayWorkspaceView {
    FridayWorkspaceView {
        area,
        route: area.route().to_string(),
        title: area.label().to_string(),
        objective: objective.to_string(),
        primary_model_role: primary_model_role.to_string(),
        required_tools: required_tools.iter().map(|tool| (*tool).to_string()).collect(),
        local_boundary: "Local-first by default; remote providers require explicit configuration and policy allowance."
            .to_string(),
        empty_state: format!(
            "{} is ready when the first local project, prompt, file, source, or task is added.",
            area.label()
        ),
        error_state: format!(
            "{} must show provider, model, permission, and source errors directly instead of silently falling back.",
            area.label()
        ),
    }
}

fn friday_capabilities() -> Vec<FridayCapability> {
    vec![
        capability(
            FridayCompetitor::ChatGpt,
            FridayWorkspaceArea::Ask,
            "Streaming multimodal chat with model picker and tool/source controls",
            8,
            FridayFeatureStatus::Partial,
            true,
            false,
            "Local chat and model roles exist; Friday still needs the final UI and streaming message store.",
            "ChatGPT capabilities overview: chat, image input, file uploads, data analysis, and tools.",
        ),
        capability(
            FridayCompetitor::ChatGpt,
            FridayWorkspaceArea::Research,
            "Deep research with editable plans, cited reports, and exportable sources",
            9,
            FridayFeatureStatus::Partial,
            true,
            true,
            "Metasearch workflow stages and export contracts exist; report runner and citation persistence are the next build target.",
            "ChatGPT deep research and Perplexity research patterns.",
        ),
        capability(
            FridayCompetitor::ChatGpt,
            FridayWorkspaceArea::Canvas,
            "Canvas for co-writing, editing, debugging, and generated UI/code snippets",
            7,
            FridayFeatureStatus::Partial,
            true,
            false,
            "Friday now has artifact, preview, diff, and checkpoint records; rich editor UI and live preview execution are next.",
            "ChatGPT Canvas and Gemini Canvas.",
        ),
        capability(
            FridayCompetitor::ChatGpt,
            FridayWorkspaceArea::Automations,
            "Scheduled tasks, async follow-ups, and background research runs",
            6,
            FridayFeatureStatus::Partial,
            true,
            true,
            "Host pause/audit and automation foundations exist; Friday needs durable scheduled task records.",
            "ChatGPT scheduled tasks and Pulse-style async work.",
        ),
        capability(
            FridayCompetitor::Gemini,
            FridayWorkspaceArea::Connectors,
            "Google-style app connections and personal context with explicit opt-in",
            7,
            FridayFeatureStatus::Partial,
            false,
            false,
            "Friday now has a durable local connector registry; concrete Gmail/Calendar/Drive-style auth and sync remain optional future work.",
            "Gemini app connections and Personal Intelligence.",
        ),
        capability(
            FridayCompetitor::Gemini,
            FridayWorkspaceArea::Multimodal,
            "Image, video, audio, music, OCR, and long-file understanding",
            8,
            FridayFeatureStatus::Partial,
            true,
            false,
            "OCR/STT/TTS/model planning exists; local VLM/video/music execution remains incomplete.",
            "Gemini multimodal generation, Live, Audio Overview, long context, and media features.",
        ),
        capability(
            FridayCompetitor::Gemini,
            FridayWorkspaceArea::Projects,
            "Gems/custom experts with files, detailed instructions, and project memory",
            6,
            FridayFeatureStatus::Partial,
            true,
            false,
            "Friday now has durable project-scoped records for instructions, files, memories, and connector boundaries; threaded UI wiring is next.",
            "Gemini Gems and ChatGPT Projects.",
        ),
        capability(
            FridayCompetitor::Perplexity,
            FridayWorkspaceArea::Search,
            "Answer-first cited search with source controls for web, academic, code, files, and premium/provider sources",
            10,
            FridayFeatureStatus::Partial,
            true,
            true,
            "Friday now has a metasearch-first policy and search plans; execution and citation UI are next.",
            "Perplexity source-led answers, Pro Search, Research, and Spaces source controls.",
        ),
        capability(
            FridayCompetitor::Perplexity,
            FridayWorkspaceArea::Projects,
            "Research spaces with files, pinned threads, custom instructions, and collaborator boundaries",
            7,
            FridayFeatureStatus::Partial,
            true,
            true,
            "The local project registry now preserves files, memories, connectors, and permission boundaries; collaboration and pinned thread UI remain.",
            "Perplexity Spaces.",
        ),
        capability(
            FridayCompetitor::Perplexity,
            FridayWorkspaceArea::Agents,
            "Web task execution without relying on Perplexity Computer",
            6,
            FridayFeatureStatus::Partial,
            true,
            true,
            "Friday should use host-approved browser/code tools plus metasearch, not Perplexity Computer.",
            "Perplexity Comet/Computer is a competitor reference, not a runtime dependency.",
        ),
        capability(
            FridayCompetitor::Grok,
            FridayWorkspaceArea::Voice,
            "Realtime voice agent with interruptions, tool use, STT, TTS, and custom voices",
            8,
            FridayFeatureStatus::Partial,
            true,
            false,
            "Local STT/TTS/wake work is strong; real duplex voice orchestration and custom voice management remain.",
            "Grok Voice Think Fast, xAI Voice API, STT/TTS, and custom voices.",
        ),
        capability(
            FridayCompetitor::Grok,
            FridayWorkspaceArea::Search,
            "Realtime web/X-style search and RAG collections",
            7,
            FridayFeatureStatus::Partial,
            true,
            true,
            "Metasearch covers web/research search; social/X and durable RAG collections still need adapters.",
            "xAI web_search, x_search, file_search, collections search, and tool calling.",
        ),
        capability(
            FridayCompetitor::Grok,
            FridayWorkspaceArea::Agents,
            "Strong tool calling, structured outputs, cost tracking, and multi-agent orchestration",
            7,
            FridayFeatureStatus::Partial,
            true,
            true,
            "xLAM router and local model roles exist; Friday needs per-run cost/latency/capability ledgers.",
            "xAI tool calling and Claude Code agent-team patterns.",
        ),
        capability(
            FridayCompetitor::Claude,
            FridayWorkspaceArea::Artifacts,
            "Artifacts for live apps, documents, visual work, and reusable generated outputs",
            9,
            FridayFeatureStatus::Partial,
            true,
            false,
            "Friday now has a durable artifact store plus preview-runner and checkpoint contracts for docs, code, UI, and reports.",
            "Claude Artifacts and Claude Design.",
        ),
        capability(
            FridayCompetitor::Claude,
            FridayWorkspaceArea::Code,
            "Coding agent workspace with terminal/IDE tools, subagents, hooks, checkpoints, and reviews",
            9,
            FridayFeatureStatus::Partial,
            true,
            true,
            "Codex, Zed, and ZeroClaw adapters exist, and Friday now has code-task checkpoint records; terminal/review UI wiring is next.",
            "Claude Code, subagents, hooks, MCP, and cloud coding tasks.",
        ),
        capability(
            FridayCompetitor::Claude,
            FridayWorkspaceArea::Connectors,
            "MCP and app connector directory with permission-scoped tools",
            8,
            FridayFeatureStatus::Partial,
            true,
            false,
            "Provider catalog and connector planning exist; Friday needs install/auth/test flows.",
            "Claude connectors directory and MCP tool ecosystem.",
        ),
        capability(
            FridayCompetitor::Claude,
            FridayWorkspaceArea::Memory,
            "Long-context project memory and editable context boundaries",
            6,
            FridayFeatureStatus::Partial,
            true,
            false,
            "Friday now has reviewed local memory records tied to projects; user-visible memory editing and retention controls are next.",
            "Claude Projects/context and long-context workflows.",
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn capability(
    competitor: FridayCompetitor,
    area: FridayWorkspaceArea,
    feature: &str,
    weight: u8,
    friday_status: FridayFeatureStatus,
    local_first: bool,
    uses_metasearch: bool,
    implementation_note: &str,
    source_note: &str,
) -> FridayCapability {
    FridayCapability {
        competitor,
        area,
        feature: feature.to_string(),
        weight,
        friday_status,
        friday_route: area.route().to_string(),
        local_first,
        uses_metasearch,
        implementation_note: implementation_note.to_string(),
        source_note: source_note.to_string(),
    }
}

fn score_capabilities(capabilities: &[FridayCapability]) -> u8 {
    let earned = capabilities
        .iter()
        .map(|capability| capability.weight as f32 * capability.friday_status.multiplier())
        .sum::<f32>();
    let possible = capabilities
        .iter()
        .map(|capability| capability.weight as f32)
        .sum::<f32>();

    if possible <= f32::EPSILON {
        0
    } else {
        ((earned / possible) * 100.0).round().clamp(0.0, 100.0) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friday_plan_tracks_all_target_competitors() {
        let plan = default_friday_product_plan();
        let competitors = plan
            .capabilities
            .iter()
            .map(|capability| capability.competitor)
            .collect::<std::collections::HashSet<_>>();

        assert!(competitors.contains(&FridayCompetitor::ChatGpt));
        assert!(competitors.contains(&FridayCompetitor::Gemini));
        assert!(competitors.contains(&FridayCompetitor::Perplexity));
        assert!(competitors.contains(&FridayCompetitor::Grok));
        assert!(competitors.contains(&FridayCompetitor::Claude));
    }

    #[test]
    fn friday_search_policy_forbids_perplexity_computer() {
        let plan = default_friday_product_plan();
        assert!(plan.search_policy.forbids_perplexity_computer);
        assert_eq!(plan.search_policy.engine, "adjacent-metasearch-rust-crate");
        assert!(!plan.metasearch_capabilities().is_empty());
    }

    #[test]
    fn friday_score_remains_honest_until_planned_items_ship() {
        let plan = default_friday_product_plan();
        assert!(plan.score_out_of_100 < 60);
        assert!(
            plan.capabilities
                .iter()
                .any(|capability| capability.friday_status != FridayFeatureStatus::Shipped)
        );
    }
}
