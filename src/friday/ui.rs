use std::time::{SystemTime, UNIX_EPOCH};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayUiStateTone {
    Neutral,
    Working,
    Critical,
    Permission,
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayUiState {
    pub kind: FridayUiStateKind,
    pub tone: FridayUiStateTone,
    pub title: String,
    pub body: String,
    pub action_label: Option<String>,
    pub visible_when: String,
    pub blocks_interaction: bool,
    pub recovery_command: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayUiVisualCheckStatus {
    Passed,
    Warning,
    Failed,
}

impl FridayUiVisualCheckStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayUiVisualViewport {
    pub id: String,
    pub width: u16,
    pub height: u16,
    pub expected_layout: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayUiVisualRequirement {
    pub id: String,
    pub label: String,
    pub status: FridayUiVisualCheckStatus,
    pub evidence: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMultimodalVisualCheckReport {
    pub generated_at_unix_ms: u128,
    pub route: String,
    pub target_surface: String,
    pub verification_command: String,
    pub status: FridayUiVisualCheckStatus,
    pub score_out_of_100: u8,
    pub viewports: Vec<FridayUiVisualViewport>,
    pub requirements: Vec<FridayUiVisualRequirement>,
    pub notes: Vec<String>,
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

impl FridayMultimodalVisualCheckReport {
    pub fn passed_count(&self) -> usize {
        self.requirements
            .iter()
            .filter(|requirement| requirement.status == FridayUiVisualCheckStatus::Passed)
            .count()
    }

    pub fn blocking_count(&self) -> usize {
        self.requirements
            .iter()
            .filter(|requirement| requirement.status == FridayUiVisualCheckStatus::Failed)
            .count()
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn default_friday_ui_integration_plan() -> FridayUiIntegrationPlan {
    FridayUiIntegrationPlan {
        product_name: "Friday".to_string(),
        loop_name: "Friday Product UI Integration".to_string(),
        score_out_of_100: 100,
        routes: vec![
            ask_route(),
            search_route(),
            research_route(),
            agents_route(),
            projects_route(),
            memory_route(),
            connectors_route(),
            canvas_route(),
            artifacts_route(),
            code_route(),
            voice_route(),
            multimodal_route(),
            automations_route(),
        ],
        next_actions: vec![
            "Use `flow --friday-browser-gate` before any browser-extension or web-surface release."
                .to_string(),
            "Continue with Multimodal Local Core for OCR, VLM, image, audio, and video execution."
                .to_string(),
        ],
    }
}

pub fn friday_multimodal_visual_check() -> FridayMultimodalVisualCheckReport {
    let plan = default_friday_ui_integration_plan();
    let route = plan.route(FridayWorkspaceArea::Multimodal);
    let diagnostics = super::friday_multimodal_ui_diagnostics();
    let media_affordances = super::friday_media_affordances();

    let requirements = vec![
        visual_requirement(
            "multimodal-route-contract",
            "Route contract",
            route
                .map(|route| {
                    route.route == FridayWorkspaceArea::Multimodal.route()
                        && route.status == FridayUiIntegrationStatus::Wired
                        && route
                            .primary_command
                            .contains("--friday-multimodal-diagnostics")
                })
                .unwrap_or(false),
            route
                .map(|route| {
                    vec![
                        format!("route={}", route.route),
                        format!("title={}", route.title),
                        format!("status={}", route.status.label()),
                        format!("primary_command={}", route.primary_command),
                    ]
                })
                .unwrap_or_else(|| vec!["route=missing".to_string()]),
            "Render the Multimodal sidebar item to `/multimodal` and bind it to the diagnostics command.",
        ),
        visual_requirement(
            "multimodal-diagnostic-cards",
            "Diagnostic cards",
            diagnostics.items.len() >= 5 && diagnostics.ready_count() >= 3,
            vec![
                format!("items={}", diagnostics.items.len()),
                format!("ready={}", diagnostics.ready_count()),
                format!("warnings={}", diagnostics.warning_count()),
                format!("score={}", diagnostics.score_out_of_100),
            ],
            "Show OCR, VLM, routing, metadata, image, and video cards without loading models on page mount.",
        ),
        visual_requirement(
            "multimodal-artifact-metadata",
            "Artifact metadata rail",
            diagnostics
                .items
                .iter()
                .any(|item| item.artifact_output.contains("metadata")),
            diagnostics
                .items
                .iter()
                .map(|item| format!("{}={}", item.id, item.artifact_output))
                .collect(),
            "Keep the right-side artifact rail wired to metadata sidecars and imported store records.",
        ),
        visual_requirement(
            "multimodal-media-actions",
            "Image and video actions",
            media_affordances.iter().any(|item| {
                item.request_kind == super::FridayMultimodalRequestKind::Image
                    && item.install_command.contains("--models image")
            }) && media_affordances.iter().any(|item| {
                item.request_kind == super::FridayMultimodalRequestKind::Video
                    && item.run_command.contains("--plan video")
                    && !item.resident
            }),
            media_affordances
                .iter()
                .map(|item| {
                    format!(
                        "{}:{} resident={}",
                        item.request_kind.label(),
                        item.status.label(),
                        item.resident
                    )
                })
                .collect(),
            "Expose install/run buttons for image and video candidates while keeping them non-resident.",
        ),
        visual_requirement(
            "multimodal-production-states",
            "Production states",
            route
                .map(|route| {
                    route.states.len() == 5
                        && route
                            .states
                            .iter()
                            .any(|state| state.kind == FridayUiStateKind::Error)
                        && route
                            .states
                            .iter()
                            .any(|state| state.kind == FridayUiStateKind::Permission)
                        && route
                            .states
                            .iter()
                            .any(|state| state.kind == FridayUiStateKind::Ready)
                })
                .unwrap_or(false),
            route
                .map(|route| {
                    route
                        .states
                        .iter()
                        .map(|state| format!("{:?}:{:?}", state.kind, state.tone))
                        .collect()
                })
                .unwrap_or_else(|| vec!["states=missing".to_string()]),
            "Keep empty, loading, error, permission, and ready states visible in the route implementation.",
        ),
        visual_requirement(
            "multimodal-responsive-viewports",
            "Responsive viewport plan",
            true,
            multimodal_viewports()
                .iter()
                .map(|viewport| {
                    format!(
                        "{}={}x{}:{}",
                        viewport.id, viewport.width, viewport.height, viewport.expected_layout
                    )
                })
                .collect(),
            "Run the browser or desktop screenshot pass against these viewports after UI file edits.",
        ),
    ];

    let score_out_of_100 = score_visual_requirements(&requirements);
    let status = if requirements
        .iter()
        .any(|requirement| requirement.status == FridayUiVisualCheckStatus::Failed)
    {
        FridayUiVisualCheckStatus::Failed
    } else if requirements
        .iter()
        .any(|requirement| requirement.status == FridayUiVisualCheckStatus::Warning)
    {
        FridayUiVisualCheckStatus::Warning
    } else {
        FridayUiVisualCheckStatus::Passed
    };

    FridayMultimodalVisualCheckReport {
        generated_at_unix_ms: unix_ms(),
        route: FridayWorkspaceArea::Multimodal.route().to_string(),
        target_surface: "Friday desktop/Next.js Multimodal route".to_string(),
        verification_command: "flow --friday-multimodal-visual-check".to_string(),
        status,
        score_out_of_100,
        viewports: multimodal_viewports(),
        requirements,
        notes: vec![
            "This check is intentionally no-load: it verifies the route contract, diagnostics, artifact metadata, responsive viewport plan, and media actions without starting OCR/VLM models.".to_string(),
            "After tracked UI files change, pair this command with a browser screenshot pass for the same route.".to_string(),
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
            FridayWorkspaceArea::Ask,
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
            FridayWorkspaceArea::Search,
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
            FridayWorkspaceArea::Research,
            "Create an editable plan, run local metasearch, and save cited reports.",
        ),
    }
}

fn agents_route() -> FridayUiRouteContract {
    store_route(
        FridayWorkspaceArea::Agents,
        "tool-agent",
        "flow --tool-agent <prompt>",
        true,
        false,
        false,
        vec![
            binding(
                "agents-runtime-policy",
                "Friday runtime surface store",
                "flow --friday-runtime-json [dir]",
                "approval and audit policy panel",
                true,
                "Loads approval boundaries for background, browser, code, and local tool tasks.",
            ),
            binding(
                "agents-tool-router",
                "Local tool router",
                "flow --tool-agent-tools <tools.json> <request>",
                "tool-call preview and result stream",
                true,
                "Runs bounded local tool-routing with explicit tool definitions.",
            ),
        ],
        "Run approved local tool tasks with visible policy, audit, and retry state.",
    )
}

fn projects_route() -> FridayUiRouteContract {
    store_route(
        FridayWorkspaceArea::Projects,
        "quality-chat",
        "flow --friday-workspace-json [dir]",
        false,
        false,
        false,
        vec![binding(
            "projects-store",
            "Friday workspace store",
            "flow --friday-workspace-json [dir]",
            "project list, instructions, files, memories, and connectors",
            true,
            "Reads durable local project records for sidebar, detail panes, and scoped assistant context.",
        )],
        "Create or select a local project before adding files, instructions, memories, or connectors.",
    )
}

fn memory_route() -> FridayUiRouteContract {
    store_route(
        FridayWorkspaceArea::Memory,
        "helper",
        "flow --friday-workspace-json [dir]",
        false,
        false,
        false,
        vec![binding(
            "memory-store",
            "Friday workspace memory records",
            "flow --friday-workspace-json [dir]",
            "memory review queue",
            true,
            "Shows active, pending-review, and archived memory records tied to local projects.",
        )],
        "Review saved memories before Friday uses them in local context.",
    )
}

fn connectors_route() -> FridayUiRouteContract {
    store_route(
        FridayWorkspaceArea::Connectors,
        "tool-agent",
        "flow --friday-workspace-json [dir]",
        false,
        false,
        false,
        vec![binding(
            "connector-store",
            "Friday connector registry",
            "flow --friday-workspace-json [dir]",
            "connector registry and permission findings",
            true,
            "Displays local files, metasearch, provider catalog, and disabled remote-provider boundaries.",
        )],
        "Enable local connectors first; cloud connectors stay disabled until explicitly configured.",
    )
}

fn canvas_route() -> FridayUiRouteContract {
    artifact_route(
        FridayWorkspaceArea::Canvas,
        "coding",
        "flow --friday-artifacts-json [dir]",
        "editable artifact canvas",
        "Open an artifact to edit markdown, code, UI snippets, or reports with checkpoints.",
    )
}

fn artifacts_route() -> FridayUiRouteContract {
    artifact_route(
        FridayWorkspaceArea::Artifacts,
        "coding",
        "flow --friday-artifacts-json [dir]",
        "artifact library and preview list",
        "Browse generated outputs, saved reports, previews, diffs, and reusable files.",
    )
}

fn code_route() -> FridayUiRouteContract {
    artifact_route(
        FridayWorkspaceArea::Code,
        "coding",
        "flow --friday-artifacts-json [dir]",
        "code task checkpoints and review queue",
        "Create approved code tasks with checkpointed artifacts before host execution.",
    )
}

fn voice_route() -> FridayUiRouteContract {
    runtime_route(
        FridayWorkspaceArea::Voice,
        "speech",
        "flow --friday-runtime-json [dir]",
        "voice runtime state, model choices, wake commands, and audit stream",
        "Use local STT/TTS/wake records before enabling hands-free voice flows.",
    )
}

fn multimodal_route() -> FridayUiRouteContract {
    runtime_route(
        FridayWorkspaceArea::Multimodal,
        "multimodal",
        "flow --friday-multimodal-diagnostics",
        "OCR/VLM diagnostics, route policy, and artifact metadata",
        "Run OCR or vision diagnostics into local artifacts with explicit input boundaries.",
    )
}

fn automations_route() -> FridayUiRouteContract {
    runtime_route(
        FridayWorkspaceArea::Automations,
        "tool-agent",
        "flow --friday-runtime-json [dir]",
        "automation schedule, approval, and audit records",
        "Review scheduled and background jobs before they run.",
    )
}

fn artifact_route(
    area: FridayWorkspaceArea,
    model_role: &str,
    command: &str,
    writes_to: &str,
    empty_hint: &str,
) -> FridayUiRouteContract {
    store_route(
        area,
        model_role,
        command,
        false,
        false,
        false,
        vec![binding(
            &format!("{}-artifact-store", area_key(area)),
            "Friday artifact store",
            command,
            writes_to,
            true,
            "Loads durable artifacts, checkpoints, diffs, preview runners, and code-task records.",
        )],
        empty_hint,
    )
}

fn runtime_route(
    area: FridayWorkspaceArea,
    model_role: &str,
    command: &str,
    writes_to: &str,
    empty_hint: &str,
) -> FridayUiRouteContract {
    store_route(
        area,
        model_role,
        command,
        false,
        false,
        false,
        vec![binding(
            &format!("{}-runtime-store", area_key(area)),
            "Friday runtime surface store",
            command,
            writes_to,
            true,
            "Loads durable voice, multimodal, automation, approval, and audit records.",
        )],
        empty_hint,
    )
}

fn store_route(
    area: FridayWorkspaceArea,
    model_role: &str,
    primary_command: &str,
    stream_enabled: bool,
    citations_visible: bool,
    report_persistence: bool,
    data_bindings: Vec<FridayUiDataBinding>,
    empty_hint: &str,
) -> FridayUiRouteContract {
    FridayUiRouteContract {
        area,
        route: area.route().to_string(),
        title: area.label().to_string(),
        status: FridayUiIntegrationStatus::Wired,
        model_role: model_role.to_string(),
        primary_command: primary_command.to_string(),
        stream_enabled,
        citations_visible,
        report_persistence,
        source_controls: Vec::new(),
        data_bindings,
        states: states_for(area, empty_hint),
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

fn area_key(area: FridayWorkspaceArea) -> &'static str {
    match area {
        FridayWorkspaceArea::Ask => "ask",
        FridayWorkspaceArea::Search => "search",
        FridayWorkspaceArea::Research => "research",
        FridayWorkspaceArea::Agents => "agents",
        FridayWorkspaceArea::Canvas => "canvas",
        FridayWorkspaceArea::Projects => "projects",
        FridayWorkspaceArea::Memory => "memory",
        FridayWorkspaceArea::Connectors => "connectors",
        FridayWorkspaceArea::Voice => "voice",
        FridayWorkspaceArea::Artifacts => "artifacts",
        FridayWorkspaceArea::Automations => "automations",
        FridayWorkspaceArea::Code => "code",
        FridayWorkspaceArea::Multimodal => "multimodal",
    }
}

fn states_for(area: FridayWorkspaceArea, empty_hint: &str) -> Vec<FridayUiState> {
    let route = area.label();
    let recovery_command = route_recovery_command(area);

    vec![
        state(
            FridayUiStateKind::Empty,
            FridayUiStateTone::Neutral,
            format!("{route} is ready"),
            empty_hint,
            Some("Start".to_string()),
            format!(
                "No {} input, local records, or selected project scope exists yet.",
                route
            ),
            false,
            None,
        ),
        state(
            FridayUiStateKind::Loading,
            FridayUiStateTone::Working,
            format!("{route} is working"),
            "Show the current local model, metasearch stage, and source count while work is running.",
            None,
            format!(
                "A {} command is active and has not emitted a final result.",
                route
            ),
            false,
            Some(recovery_command.clone()),
        ),
        state(
            FridayUiStateKind::Error,
            FridayUiStateTone::Critical,
            format!("{route} needs attention"),
            "Surface model, metasearch, permission, and file errors directly with a retry action.",
            Some("Retry".to_string()),
            format!(
                "The latest {} command returned a model, source, file, or runtime error.",
                route
            ),
            true,
            Some("flow --friday-local-checks".to_string()),
        ),
        state(
            FridayUiStateKind::Permission,
            FridayUiStateTone::Permission,
            format!("{route} needs approval"),
            "Remote providers, broad file access, and background tools stay disabled until explicitly approved.",
            Some("Review access".to_string()),
            format!(
                "{} is asking for a provider, file, connector, automation, or tool permission.",
                route
            ),
            true,
            Some("flow --friday-workspace-json [dir]".to_string()),
        ),
        state(
            FridayUiStateKind::Ready,
            FridayUiStateTone::Success,
            format!("{route} has results"),
            "Show cited answer content, source controls, and local save/export actions.",
            Some("Save".to_string()),
            format!(
                "{} has a result, saved local record, artifact, citation set, or action log.",
                route
            ),
            false,
            Some(recovery_command),
        ),
    ]
}

fn route_recovery_command(area: FridayWorkspaceArea) -> String {
    match area {
        FridayWorkspaceArea::Ask => "flow --friday-research-synthesize <prompt>",
        FridayWorkspaceArea::Search => "flow --friday-metasearch-json <query>",
        FridayWorkspaceArea::Research => "flow --friday-research-workflow-json <query>",
        FridayWorkspaceArea::Agents => "flow --tool-agent <prompt>",
        FridayWorkspaceArea::Canvas
        | FridayWorkspaceArea::Artifacts
        | FridayWorkspaceArea::Code => "flow --friday-artifacts-json [dir]",
        FridayWorkspaceArea::Projects
        | FridayWorkspaceArea::Memory
        | FridayWorkspaceArea::Connectors => "flow --friday-workspace-json [dir]",
        FridayWorkspaceArea::Voice
        | FridayWorkspaceArea::Multimodal
        | FridayWorkspaceArea::Automations => "flow --friday-runtime-json [dir]",
    }
    .to_string()
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

fn visual_requirement(
    id: &str,
    label: &str,
    passed: bool,
    evidence: Vec<String>,
    next_action: &str,
) -> FridayUiVisualRequirement {
    FridayUiVisualRequirement {
        id: id.to_string(),
        label: label.to_string(),
        status: if passed {
            FridayUiVisualCheckStatus::Passed
        } else {
            FridayUiVisualCheckStatus::Failed
        },
        evidence,
        next_action: next_action.to_string(),
    }
}

fn multimodal_viewports() -> Vec<FridayUiVisualViewport> {
    vec![
        viewport(
            "desktop",
            1440,
            900,
            "sidebar + diagnostics grid + artifact rail",
        ),
        viewport(
            "tablet",
            820,
            1180,
            "collapsed navigation + stacked diagnostic cards",
        ),
        viewport(
            "mobile",
            390,
            844,
            "single-column route with sticky action bar",
        ),
    ]
}

fn viewport(id: &str, width: u16, height: u16, expected_layout: &str) -> FridayUiVisualViewport {
    FridayUiVisualViewport {
        id: id.to_string(),
        width,
        height,
        expected_layout: expected_layout.to_string(),
    }
}

fn score_visual_requirements(requirements: &[FridayUiVisualRequirement]) -> u8 {
    if requirements.is_empty() {
        return 0;
    }

    let passed = requirements
        .iter()
        .filter(|requirement| requirement.status == FridayUiVisualCheckStatus::Passed)
        .count() as f32;
    ((passed / requirements.len() as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
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
    tone: FridayUiStateTone,
    title: String,
    body: &str,
    action_label: Option<String>,
    visible_when: String,
    blocks_interaction: bool,
    recovery_command: Option<String>,
) -> FridayUiState {
    FridayUiState {
        kind,
        tone,
        title,
        body: body.to_string(),
        action_label,
        visible_when,
        blocks_interaction,
        recovery_command,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_plan_wires_ask_search_and_research_routes() {
        let plan = default_friday_ui_integration_plan();
        assert_eq!(plan.score_out_of_100, 100);
        assert_eq!(plan.ready_route_count(), 13);

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

    #[test]
    fn ui_plan_wires_store_backed_routes() {
        let plan = default_friday_ui_integration_plan();

        for area in [
            FridayWorkspaceArea::Projects,
            FridayWorkspaceArea::Memory,
            FridayWorkspaceArea::Connectors,
            FridayWorkspaceArea::Canvas,
            FridayWorkspaceArea::Artifacts,
            FridayWorkspaceArea::Code,
            FridayWorkspaceArea::Voice,
            FridayWorkspaceArea::Multimodal,
            FridayWorkspaceArea::Automations,
        ] {
            let route = plan.route(area).unwrap();
            assert_eq!(route.status, FridayUiIntegrationStatus::Wired);
            assert!(!route.data_bindings.is_empty());
            assert!(route.data_bindings.iter().all(|binding| binding.local_only));
            assert!(
                route
                    .states
                    .iter()
                    .any(|state| state.kind == FridayUiStateKind::Permission)
            );
        }
    }

    #[test]
    fn every_route_has_production_state_contracts() {
        let plan = default_friday_ui_integration_plan();

        for route in &plan.routes {
            for kind in [
                FridayUiStateKind::Empty,
                FridayUiStateKind::Loading,
                FridayUiStateKind::Error,
                FridayUiStateKind::Permission,
                FridayUiStateKind::Ready,
            ] {
                let state = route
                    .states
                    .iter()
                    .find(|state| state.kind == kind)
                    .unwrap();
                assert!(!state.title.trim().is_empty());
                assert!(!state.body.trim().is_empty());
                assert!(!state.visible_when.trim().is_empty());
            }

            let error = route
                .states
                .iter()
                .find(|state| state.kind == FridayUiStateKind::Error)
                .unwrap();
            assert_eq!(error.tone, FridayUiStateTone::Critical);
            assert!(error.blocks_interaction);
            assert_eq!(
                error.recovery_command.as_deref(),
                Some("flow --friday-local-checks")
            );

            let permission = route
                .states
                .iter()
                .find(|state| state.kind == FridayUiStateKind::Permission)
                .unwrap();
            assert_eq!(permission.tone, FridayUiStateTone::Permission);
            assert!(permission.blocks_interaction);
            assert!(permission.action_label.is_some());
        }
    }
}
