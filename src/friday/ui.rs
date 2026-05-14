use serde::{Deserialize, Serialize};

use super::FridayWorkspaceArea;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayUiIntegrationStatus {
    Wired,
    Planned,
    Blocked,
}

impl FridayUiIntegrationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Wired => "wired",
            Self::Planned => "planned",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayUiStateKind {
    Empty,
    Loading,
    Error,
    Permission,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayUiState {
    pub kind: FridayUiStateKind,
    pub title: String,
    pub body: String,
    pub action_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayUiDataBinding {
    pub id: String,
    pub source: String,
    pub command: String,
    pub writes_to: String,
    pub local_only: bool,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayUiSourceControl {
    pub key: String,
    pub label: String,
    pub default_enabled: bool,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayUiRouteContract {
    pub area: FridayWorkspaceArea,
    pub route: String,
    pub title: String,
    pub status: FridayUiIntegrationStatus,
    pub model_role: String,
    pub primary_command: String,
    pub stream_enabled: bool,
    pub citations_visible: bool,
    pub report_persistence: bool,
    pub source_controls: Vec<FridayUiSourceControl>,
    pub data_bindings: Vec<FridayUiDataBinding>,
    pub states: Vec<FridayUiState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayUiIntegrationPlan {
    pub product_name: String,
    pub loop_name: String,
    pub score_out_of_100: u8,
    pub routes: Vec<FridayUiRouteContract>,
    pub next_actions: Vec<String>,
}

impl FridayUiIntegrationPlan {
    pub fn ready_route_count(&self) -> usize {
        self.routes
            .iter()
            .filter(|route| route.status == FridayUiIntegrationStatus::Wired)
            .count()
    }

    pub fn route(&self, area: FridayWorkspaceArea) -> Option<&FridayUiRouteContract> {
        self.routes.iter().find(|route| route.area == area)
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn default_friday_ui_integration_plan() -> FridayUiIntegrationPlan {
    FridayUiIntegrationPlan {
        product_name: "Friday".to_string(),
        loop_name: "Friday Product UI Integration".to_string(),
        score_out_of_100: 20,
        routes: vec![ask_route(), search_route(), research_route()],
        next_actions: vec![
            "Wire Projects, Memory, Connectors, Canvas, Artifacts, Code, Voice, Multimodal, and Automations pages to the durable stores.".to_string(),
            "Add end-to-end local checks for STT/TTS/OCR/metasearch/artifact preview flows.".to_string(),
            "Add production-ready permission and error states across every Friday route.".to_string(),
        ],
    }
}

fn ask_route() -> FridayUiRouteContract {
    FridayUiRouteContract {
        area: FridayWorkspaceArea::Ask,
        route: FridayWorkspaceArea::Ask.route().to_string(),
        title: "Ask".to_string(),
        status: FridayUiIntegrationStatus::Wired,
        model_role: "quality-chat".to_string(),
        primary_command: "flow --friday-research-synthesize <prompt>".to_string(),
        stream_enabled: true,
        citations_visible: true,
        report_persistence: false,
        source_controls: default_source_controls(),
        data_bindings: vec![
            binding(
                "ask-synthesis",
                "Friday synthesized answer deltas",
                "flow --friday-research-synthesize <prompt>",
                "chat message stream",
                true,
                "Streams answer deltas with citation references from local metasearch-backed synthesis.",
            ),
            binding(
                "ask-workspace-context",
                "Friday workspace store",
                "flow --friday-workspace-json [dir]",
                "project context panel",
                true,
                "Reads local project instructions, memories, and connector boundaries before remote providers are considered.",
            ),
        ],
        states: states_for(
            "Ask",
            "Start with a prompt, file, voice transcript, or source question.",
        ),
    }
}

fn search_route() -> FridayUiRouteContract {
    FridayUiRouteContract {
        area: FridayWorkspaceArea::Search,
        route: FridayWorkspaceArea::Search.route().to_string(),
        title: "Search".to_string(),
        status: FridayUiIntegrationStatus::Wired,
        model_role: "quality-chat".to_string(),
        primary_command: "flow --friday-metasearch <query>".to_string(),
        stream_enabled: false,
        citations_visible: true,
        report_persistence: false,
        source_controls: default_source_controls(),
        data_bindings: vec![
            binding(
                "search-metasearch",
                "Local metasearch response",
                "flow --friday-metasearch-json <query>",
                "answer-first cited result list",
                true,
                "Uses the adjacent Rust metasearch server for result provenance and source controls.",
            ),
            binding(
                "search-plan",
                "Metasearch request plan",
                "flow --friday-search <query>",
                "source control preview",
                true,
                "Shows the exact local-first search plan before execution.",
            ),
        ],
        states: states_for(
            "Search",
            "Search locally with web, news, academic, code, model, and file source controls.",
        ),
    }
}

fn research_route() -> FridayUiRouteContract {
    FridayUiRouteContract {
        area: FridayWorkspaceArea::Research,
        route: FridayWorkspaceArea::Research.route().to_string(),
        title: "Research".to_string(),
        status: FridayUiIntegrationStatus::Wired,
        model_role: "daily-smart".to_string(),
        primary_command: "flow --friday-research-report-save <dir> <query>".to_string(),
        stream_enabled: true,
        citations_visible: true,
        report_persistence: true,
        source_controls: default_source_controls(),
        data_bindings: vec![
            binding(
                "research-workflow",
                "Friday research workflow",
                "flow --friday-research-workflow-json <query>",
                "plan preview and progress timeline",
                true,
                "Displays clarify, discovery, source scoring, citation extraction, synthesis, and export stages.",
            ),
            binding(
                "research-report",
                "Persisted research bundle",
                "flow --friday-research-report-save <dir> <query>",
                "saved report, citations, source groups, events, and manifest",
                true,
                "Writes exportable research records that the UI can reload without cloud storage.",
            ),
        ],
        states: states_for(
            "Research",
            "Create an editable plan, run local metasearch, and save cited reports.",
        ),
    }
}

fn default_source_controls() -> Vec<FridayUiSourceControl> {
    vec![
        source(
            "web",
            "Web",
            true,
            "General web results through local metasearch.",
        ),
        source(
            "news",
            "News",
            true,
            "Recent coverage through metasearch news categories.",
        ),
        source(
            "academic",
            "Academic",
            true,
            "Scholar and paper-oriented source categories when available.",
        ),
        source(
            "code",
            "Code",
            true,
            "Repository, package, and docs-oriented sources.",
        ),
        source(
            "models",
            "Models",
            true,
            "Model cards, benchmark notes, and local model references.",
        ),
        source(
            "files",
            "Files",
            false,
            "Local project files only after project scope is selected.",
        ),
    ]
}

fn states_for(route: &str, empty_hint: &str) -> Vec<FridayUiState> {
    vec![
        state(
            FridayUiStateKind::Empty,
            format!("{route} is ready"),
            empty_hint,
            Some("Start".to_string()),
        ),
        state(
            FridayUiStateKind::Loading,
            format!("{route} is working"),
            "Show the current local model, metasearch stage, and source count while work is running.",
            None,
        ),
        state(
            FridayUiStateKind::Error,
            format!("{route} needs attention"),
            "Surface model, metasearch, permission, and file errors directly with a retry action.",
            Some("Retry".to_string()),
        ),
        state(
            FridayUiStateKind::Permission,
            format!("{route} needs approval"),
            "Remote providers, broad file access, and background tools stay disabled until explicitly approved.",
            Some("Review access".to_string()),
        ),
        state(
            FridayUiStateKind::Ready,
            format!("{route} has results"),
            "Show cited answer content, source controls, and local save/export actions.",
            Some("Save".to_string()),
        ),
    ]
}

fn binding(
    id: &str,
    source: &str,
    command: &str,
    writes_to: &str,
    local_only: bool,
    description: &str,
) -> FridayUiDataBinding {
    FridayUiDataBinding {
        id: id.to_string(),
        source: source.to_string(),
        command: command.to_string(),
        writes_to: writes_to.to_string(),
        local_only,
        description: description.to_string(),
    }
}

fn source(
    key: &str,
    label: &str,
    default_enabled: bool,
    description: &str,
) -> FridayUiSourceControl {
    FridayUiSourceControl {
        key: key.to_string(),
        label: label.to_string(),
        default_enabled,
        description: description.to_string(),
    }
}

fn state(
    kind: FridayUiStateKind,
    title: String,
    body: &str,
    action_label: Option<String>,
) -> FridayUiState {
    FridayUiState {
        kind,
        title,
        body: body.to_string(),
        action_label,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_plan_wires_ask_search_and_research_routes() {
        let plan = default_friday_ui_integration_plan();
        assert_eq!(plan.score_out_of_100, 20);
        assert_eq!(plan.ready_route_count(), 3);

        for area in [
            FridayWorkspaceArea::Ask,
            FridayWorkspaceArea::Search,
            FridayWorkspaceArea::Research,
        ] {
            let route = plan.route(area).unwrap();
            assert_eq!(route.status, FridayUiIntegrationStatus::Wired);
            assert!(route.citations_visible);
            assert!(!route.source_controls.is_empty());
            assert!(route.data_bindings.iter().all(|binding| binding.local_only));
            assert!(
                route
                    .states
                    .iter()
                    .any(|state| state.kind == FridayUiStateKind::Error)
            );
        }
    }

    #[test]
    fn research_route_persists_exportable_reports() {
        let plan = default_friday_ui_integration_plan();
        let route = plan.route(FridayWorkspaceArea::Research).unwrap();
        assert!(route.report_persistence);
        assert!(
            route
                .data_bindings
                .iter()
                .any(|binding| binding.command.contains("--friday-research-report-save"))
        );
    }
}
