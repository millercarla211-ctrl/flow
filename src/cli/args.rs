/// Command-line arguments
#[derive(Debug)]
pub struct Args {
    pub command: Command,
}

#[derive(Debug)]
pub enum Command {
    /// Transcribe audio file
    Transcribe { file: String },
    /// Full Wispr Flow pipeline (STT + LLM enhancement)
    Wispr { file: String },
    /// Speak text using TTS
    Speak { text: String },
    /// Live recording mode (microphone -> STT -> enhance -> TTS)
    Live,
    /// Live dictation mode (wake/hotkey -> STT -> focused input)
    Dictate,
    /// Interactive mode
    Interactive,
    /// Chat with AI (interactive CLI chat)
    Chat { model: Option<String> },
    /// Run one bounded local tool-agent prompt
    ToolAgent {
        tools: Option<String>,
        request: String,
    },
    /// OCR - Extract text from image
    Ocr {
        image: String,
        prompt: Option<String>,
    },
    /// Show the detected device profile and activation config
    Profile,
    /// Show the DX project stack and current completeness scores
    Projects,
    /// Show the Flow competitive scorecard
    Scorecard,
    /// Show Friday's competitive AI workspace feature plan
    FridayPlan,
    /// Print Friday's competitive AI workspace feature plan as JSON
    FridayPlanJson,
    /// Build a metasearch-first Friday answer-search plan
    FridaySearchPlan { query: String },
    /// Build a metasearch-first Friday deep-research plan
    FridayResearchPlan { query: String },
    /// Build a runnable Friday research workflow contract
    FridayResearchWorkflow { query: String },
    /// Print a runnable Friday research workflow contract as JSON
    FridayResearchWorkflowJson { query: String },
    /// Execute Friday answer search against the local metasearch server
    FridayMetasearch { query: String },
    /// Execute Friday answer search against the local metasearch server and print JSON
    FridayMetasearchJson { query: String },
    /// Execute Friday answer search and export a markdown research report
    FridayResearchReport { query: String },
    /// Execute Friday research and persist report, citations, source groups, and events
    FridayResearchReportSave { output_dir: String, query: String },
    /// Execute Friday research and synthesize a cited local answer
    FridayResearchSynthesize { query: String },
    /// Seed durable Friday Projects, Memory, and Connectors records
    FridayWorkspaceInit { output_dir: String },
    /// Print the durable Friday workspace state as JSON
    FridayWorkspaceJson { input_dir: Option<String> },
    /// Seed durable Friday artifact/canvas/code records
    FridayArtifactsInit { output_dir: String },
    /// Print durable Friday artifact/canvas/code state as JSON
    FridayArtifactsJson { input_dir: Option<String> },
    /// Import a multimodal artifact bundle into the durable Friday artifact store
    FridayArtifactsIndexMultimodal {
        store_dir: String,
        bundle_dir: String,
    },
    /// Import a multimodal artifact bundle and print JSON
    FridayArtifactsIndexMultimodalJson {
        store_dir: String,
        bundle_dir: String,
    },
    /// Seed durable Friday voice/multimodal/automation runtime records
    FridayRuntimeInit { output_dir: String },
    /// Print durable Friday voice/multimodal/automation runtime state as JSON
    FridayRuntimeJson { input_dir: Option<String> },
    /// Show Friday product UI integration route contracts
    FridayUiPlan,
    /// Print Friday product UI integration route contracts as JSON
    FridayUiPlanJson,
    /// Show tracked Friday live UI route file bindings
    FridayLiveUiRoutes,
    /// Print tracked Friday live UI route file bindings as JSON
    FridayLiveUiRoutesJson,
    /// Show Friday operator readiness across route, local, browser, and desktop surfaces
    FridayReadiness,
    /// Print Friday operator readiness as JSON
    FridayReadinessJson,
    /// Show Friday route screenshot verification targets
    FridayRouteVisuals,
    /// Print Friday route screenshot verification targets as JSON
    FridayRouteVisualsJson,
    /// Show Friday desktop/web/browser execution handoff contracts
    FridayExecutionHandoffs,
    /// Print Friday execution handoff contracts as JSON
    FridayExecutionHandoffsJson,
    /// Export Friday readiness bundle for Friday/DX dashboards
    FridayDashboardExport { output_dir: String },
    /// Export Friday readiness bundle and print JSON
    FridayDashboardExportJson { output_dir: String },
    /// Show dashboard UI panel data loaded from an exported Friday readiness bundle
    FridayDashboardPanel { input_dir: String },
    /// Print dashboard UI panel data as JSON
    FridayDashboardPanelJson { input_dir: String },
    /// Show product UI binding data for the visible Friday dashboard
    FridayDashboardProductUi { input_dir: String },
    /// Print product UI binding data for the visible Friday dashboard as JSON
    FridayDashboardProductUiJson { input_dir: String },
    /// Smoke-check that the visible Friday dashboard can load product UI binding data
    FridayDashboardProductUiSmoke { input_dir: String },
    /// Print the visible Friday dashboard product UI smoke check as JSON
    FridayDashboardProductUiSmokeJson { input_dir: String },
    /// Show trusted host command bridge data for dashboard actions
    FridayDashboardHostCommandBridge { input_dir: String },
    /// Print trusted host command bridge data as JSON
    FridayDashboardHostCommandBridgeJson { input_dir: String },
    /// Run or dry-run one approved trusted host command from dashboard bridge data
    FridayTrustedHostRunner {
        input_dir: String,
        action_id: Option<String>,
        approve: bool,
        execute: bool,
        cancel: bool,
        history_file: String,
        reason: Option<String>,
    },
    /// Run or dry-run one approved trusted host command and print JSON
    FridayTrustedHostRunnerJson {
        input_dir: String,
        action_id: Option<String>,
        approve: bool,
        execute: bool,
        cancel: bool,
        history_file: String,
        reason: Option<String>,
    },
    /// Show grouped trusted host runner history UX for dashboard imports
    FridayTrustedHostRunnerUx {
        history_file: String,
        release_review_file: String,
    },
    /// Print grouped trusted host runner history UX as JSON
    FridayTrustedHostRunnerUxJson {
        history_file: String,
        release_review_file: String,
    },
    /// Show trusted runner approval modal contract
    FridayTrustedHostRunnerApprovalUi {
        history_file: String,
        release_review_file: String,
    },
    /// Print trusted runner approval modal contract as JSON
    FridayTrustedHostRunnerApprovalUiJson {
        history_file: String,
        release_review_file: String,
    },
    /// Show trusted runner cancellation and recovery controls for live state
    FridayTrustedHostRunnerCancellationUx { state_file: String },
    /// Print trusted runner cancellation and recovery controls as JSON
    FridayTrustedHostRunnerCancellationUxJson { state_file: String },
    /// Show trusted runner operator review filters, release gates, and incidents
    FridayTrustedHostRunnerOperatorReview {
        history_file: String,
        status: Option<String>,
        action_id: Option<String>,
        since_ms: Option<String>,
        until_ms: Option<String>,
        limit: usize,
    },
    /// Print trusted runner operator review as JSON
    FridayTrustedHostRunnerOperatorReviewJson {
        history_file: String,
        status: Option<String>,
        action_id: Option<String>,
        since_ms: Option<String>,
        until_ms: Option<String>,
        limit: usize,
    },
    /// Generate a trusted runner release package without executing host commands
    FridayTrustedHostRunnerReleasePackage {
        export_dir: String,
        history_file: String,
        state_file: String,
        output_file: String,
    },
    /// Print a trusted runner release package as JSON without writing it
    FridayTrustedHostRunnerReleasePackageJson {
        export_dir: String,
        history_file: String,
        state_file: String,
        output_file: String,
    },
    /// Append a release package to the trusted runner release timeline
    FridayTrustedRunnerReleaseArchive {
        timeline_file: String,
        package_file: String,
    },
    /// Print trusted runner release package timeline as JSON
    FridayTrustedRunnerReleaseTimelineJson {
        timeline_file: String,
        package_files: Vec<String>,
    },
    /// Show trusted runner release package timeline
    FridayTrustedRunnerReleaseTimeline {
        timeline_file: String,
        package_files: Vec<String>,
    },
    /// Generate a Friday release operator checklist from local evidence
    FridayReleaseChecklist {
        checklist_file: String,
        package_file: String,
        timeline_file: String,
        export_dir: String,
        todo_file: String,
        changelog_file: String,
        signoff_file: String,
    },
    /// Print a Friday release operator checklist as JSON
    FridayReleaseChecklistJson {
        checklist_file: String,
        package_file: String,
        timeline_file: String,
        export_dir: String,
        todo_file: String,
        changelog_file: String,
        signoff_file: String,
    },
    /// Append a local operator signoff for a Friday release checklist
    FridayReleaseSignoff {
        checklist_file: String,
        signoff_file: String,
        operator: String,
        decision: String,
        reason: String,
    },
    /// Append a local operator signoff and print signoffs as JSON
    FridayReleaseSignoffJson {
        checklist_file: String,
        signoff_file: String,
        operator: String,
        decision: String,
        reason: String,
    },
    /// Show the Friday release QA command center
    FridayReleaseQa {
        report_file: String,
        checklist_file: String,
        package_file: String,
        timeline_file: String,
        cargo_check_result_file: String,
        extension_typecheck_result_file: String,
        dashboard_smoke_result_file: String,
    },
    /// Print the Friday release QA command center as JSON
    FridayReleaseQaJson {
        report_file: String,
        checklist_file: String,
        package_file: String,
        timeline_file: String,
        cargo_check_result_file: String,
        extension_typecheck_result_file: String,
        dashboard_smoke_result_file: String,
    },
    /// Show trusted runner live state projected from history or a live state file
    FridayTrustedHostLiveState {
        state_file: String,
        history_file: String,
    },
    /// Print trusted runner live state as JSON
    FridayTrustedHostLiveStateJson {
        state_file: String,
        history_file: String,
    },
    /// Run or dry-run a trusted host command through the desktop bridge state writer
    FridayTrustedHostBridgeRunner {
        input_dir: String,
        action_id: Option<String>,
        approve: bool,
        execute: bool,
        cancel: bool,
        history_file: String,
        state_file: String,
        reason: Option<String>,
    },
    /// Run or dry-run a trusted host bridge command and print bridge events as JSON
    FridayTrustedHostBridgeRunnerJson {
        input_dir: String,
        action_id: Option<String>,
        approve: bool,
        execute: bool,
        cancel: bool,
        history_file: String,
        state_file: String,
        reason: Option<String>,
    },
    /// Run low-resource Friday local execution readiness checks
    FridayLocalChecks,
    /// Print low-resource Friday local execution readiness checks as JSON
    FridayLocalChecksJson,
    /// Show Friday browser verification and deploy gate status
    FridayBrowserGate,
    /// Print Friday browser verification and deploy gate status as JSON
    FridayBrowserGateJson,
    /// Show packaged browser extension smoke readiness
    BrowserExtensionSmoke,
    /// Print packaged browser extension smoke readiness as JSON
    BrowserExtensionSmokeJson,
    /// Show bounded installed-browser extension launch smoke readiness
    BrowserExtensionLaunchSmoke { execute: bool },
    /// Print bounded installed-browser extension launch smoke readiness as JSON
    BrowserExtensionLaunchSmokeJson { execute: bool },
    /// Show offline browser-pack reuse smoke readiness
    BrowserPackReuseSmoke,
    /// Print offline browser-pack reuse smoke readiness as JSON
    BrowserPackReuseSmokeJson,
    /// Show browser-pack recovery smoke readiness
    BrowserPackRecoverySmoke,
    /// Print browser-pack recovery smoke readiness as JSON
    BrowserPackRecoverySmokeJson,
    /// Show Chromium WebLLM acceleration readiness
    BrowserWebLlmAcceleration,
    /// Print Chromium WebLLM acceleration readiness as JSON
    BrowserWebLlmAccelerationJson,
    /// Run a bounded Friday OCR smoke path and write artifact records
    FridayOcrSmoke {
        output_dir: String,
        image: Option<String>,
        execute_model: bool,
    },
    /// Run a bounded Friday OCR smoke path and print JSON
    FridayOcrSmokeJson {
        output_dir: String,
        image: Option<String>,
        execute_model: bool,
    },
    /// Write a Friday VLM screenshot understanding contract
    FridayVlmContract {
        output_dir: String,
        screenshot: Option<String>,
        prompt: Option<String>,
    },
    /// Write a Friday VLM screenshot understanding contract and print JSON
    FridayVlmContractJson {
        output_dir: String,
        screenshot: Option<String>,
        prompt: Option<String>,
    },
    /// Print Friday's local-first multimodal route for a request kind
    FridayMultimodalRoute {
        request_kind: String,
        remote_allowed: bool,
    },
    /// Print Friday's local-first multimodal route as JSON
    FridayMultimodalRouteJson {
        request_kind: String,
        remote_allowed: bool,
    },
    /// Show Friday's Multimodal UI diagnostics
    FridayMultimodalDiagnostics,
    /// Print Friday's Multimodal UI diagnostics as JSON
    FridayMultimodalDiagnosticsJson,
    /// Show Friday's Multimodal route visual check
    FridayMultimodalVisualCheck,
    /// Print Friday's Multimodal route visual check as JSON
    FridayMultimodalVisualCheckJson,
    /// Validate a local screenshot and create a VLM handoff bundle
    FridayScreenshotVlm {
        output_dir: String,
        screenshot: String,
        prompt: Option<String>,
    },
    /// Validate a local screenshot and create a VLM handoff bundle as JSON
    FridayScreenshotVlmJson {
        output_dir: String,
        screenshot: String,
        prompt: Option<String>,
    },
    /// Show explicit image and video install/run affordances
    FridayMediaAffordances,
    /// Print explicit image and video install/run affordances as JSON
    FridayMediaAffordancesJson,
    /// Diagnose host accessibility automation readiness
    AccessibilityDiagnostics { os: Option<String>, live: bool },
    /// Print persisted host automation audit records for operator review
    AuditLog { state_file: String, limit: usize },
    /// Show the active completion loop and next 100-point feature set
    Completion,
    /// Print the active completion loop as JSON
    CompletionJson,
    /// List the broker catalog, optionally filtered by modality
    Models { modality: Option<String> },
    /// Download a known local model artifact
    InstallModel { model: String },
    /// Show UI model candidates before downloading more models
    UiModelCandidates,
    /// Show ranked local tool-calling model candidates
    ToolModelCandidates,
    /// Show the Flow local model role policy
    ModelRoles,
    /// Run a local role-by-role model verification pass
    VerifyLocalModels,
    /// Generate a single-file UI artifact with the default local UI model
    Uigen {
        model: Option<String>,
        output: String,
        prompt: String,
    },
    /// Generate the standard Google homepage clone evaluation artifact
    UigenGoogle { model: Option<String> },
    /// Generate a UI from a screenshot with the local vision UI model
    UigenVision {
        screenshot: String,
        output: String,
        prompt: String,
    },
    /// Generate the standard Google homepage clone evaluation artifact from a screenshot
    UigenVisionGoogle,
    /// Build and print a broker execution plan
    Plan {
        modality: String,
        model: Option<String>,
    },
    /// Build and print a host embedding blueprint
    Blueprint { host: String },
    /// Show detected browser capability defaults for a browser flavor
    BrowserProfile { flavor: String },
    /// Build and print a browser execution plan
    BrowserPlan {
        flavor: String,
        task: String,
        modality: String,
        model: Option<String>,
        remote_fallback: bool,
    },
    /// Show registered browser-ready packs
    BrowserPacks,
    /// Inspect or correct text with the local grammar engine
    Grammar { text: String, fix: bool },
    /// Show local wake-word configuration
    WakeWords,
    /// Print the recommended production config for a host target
    ProductionConfig { target: String },
    /// Export all production configs and a delivery manifest into a directory
    ExportProductionBundle { output_dir: String },
    /// Print a release summary for the current repository scope
    ReleaseSummary,
    /// Export release-summary handoff files into a directory
    ExportReleaseSummary { output_dir: String },
}

impl Args {
    /// Parse command-line arguments
    pub fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();

        if args.len() < 2 {
            return Self {
                command: Command::Interactive,
            };
        }

        let command = match args[1].as_str() {
            "--transcribe" | "-t" => {
                let file = args
                    .get(2)
                    .cloned()
                    .unwrap_or_else(|| "tests/fixtures/audio.mp3".to_string());
                Command::Transcribe { file }
            }
            "--wispr" | "-w" => {
                let file = args
                    .get(2)
                    .cloned()
                    .unwrap_or_else(|| "tests/fixtures/audio.mp3".to_string());
                Command::Wispr { file }
            }
            "--speak" | "-s" => {
                let text = args[2..].join(" ");
                Command::Speak { text }
            }
            "--live" | "-l" => Command::Live,
            "--dictate" | "--live-type" | "--type" => Command::Dictate,
            "--chat" | "-c" => {
                let model = args.get(2).cloned();
                Command::Chat { model }
            }
            "--tool-agent" => {
                if args.len() <= 2 {
                    eprintln!("Error: prompt required");
                    eprintln!("Usage: flow --tool-agent <prompt>");
                    std::process::exit(1);
                }
                Command::ToolAgent {
                    tools: None,
                    request: args[2..].join(" "),
                }
            }
            "--tool-agent-tools" => {
                let tools = args.get(2).cloned().unwrap_or_else(|| {
                    eprintln!("Error: tools JSON path required");
                    eprintln!("Usage: flow --tool-agent-tools <tools.json> <request>");
                    std::process::exit(1);
                });
                if args.len() <= 3 {
                    eprintln!("Error: request required");
                    eprintln!("Usage: flow --tool-agent-tools <tools.json> <request>");
                    std::process::exit(1);
                }
                Command::ToolAgent {
                    tools: Some(tools),
                    request: args[3..].join(" "),
                }
            }
            "--ocr" | "-o" => {
                let image = args.get(2).cloned().unwrap_or_else(|| {
                    eprintln!("Error: Image path required for OCR");
                    eprintln!("Usage: flow --ocr <image_path> [prompt]");
                    std::process::exit(1);
                });
                let prompt = if args.len() > 3 {
                    Some(args[3..].join(" "))
                } else {
                    None
                };
                Command::Ocr { image, prompt }
            }
            "--profile" => Command::Profile,
            "--projects" => Command::Projects,
            "--scorecard" => Command::Scorecard,
            "--friday" | "--friday-plan" | "--friday-capabilities" => Command::FridayPlan,
            "--friday-json" | "--friday-plan-json" | "--friday-capabilities-json" => {
                Command::FridayPlanJson
            }
            "--friday-search" | "--friday-search-plan" => {
                if args.len() <= 2 {
                    eprintln!("Error: query required");
                    eprintln!("Usage: flow --friday-search <query>");
                    std::process::exit(1);
                }
                Command::FridaySearchPlan {
                    query: args[2..].join(" "),
                }
            }
            "--friday-research" | "--friday-research-plan" => {
                if args.len() <= 2 {
                    eprintln!("Error: query required");
                    eprintln!("Usage: flow --friday-research <query>");
                    std::process::exit(1);
                }
                Command::FridayResearchPlan {
                    query: args[2..].join(" "),
                }
            }
            "--friday-research-workflow" => {
                if args.len() <= 2 {
                    eprintln!("Error: query required");
                    eprintln!("Usage: flow --friday-research-workflow <query>");
                    std::process::exit(1);
                }
                Command::FridayResearchWorkflow {
                    query: args[2..].join(" "),
                }
            }
            "--friday-research-workflow-json" => {
                if args.len() <= 2 {
                    eprintln!("Error: query required");
                    eprintln!("Usage: flow --friday-research-workflow-json <query>");
                    std::process::exit(1);
                }
                Command::FridayResearchWorkflowJson {
                    query: args[2..].join(" "),
                }
            }
            "--friday-metasearch" | "--friday-search-local" => {
                if args.len() <= 2 {
                    eprintln!("Error: query required");
                    eprintln!("Usage: flow --friday-metasearch <query>");
                    std::process::exit(1);
                }
                Command::FridayMetasearch {
                    query: args[2..].join(" "),
                }
            }
            "--friday-metasearch-json" | "--friday-search-local-json" => {
                if args.len() <= 2 {
                    eprintln!("Error: query required");
                    eprintln!("Usage: flow --friday-metasearch-json <query>");
                    std::process::exit(1);
                }
                Command::FridayMetasearchJson {
                    query: args[2..].join(" "),
                }
            }
            "--friday-research-report" | "--friday-report" => {
                if args.len() <= 2 {
                    eprintln!("Error: query required");
                    eprintln!("Usage: flow --friday-research-report <query>");
                    std::process::exit(1);
                }
                Command::FridayResearchReport {
                    query: args[2..].join(" "),
                }
            }
            "--friday-research-report-save" | "--friday-report-save" => {
                if args.len() <= 3 {
                    eprintln!("Error: output directory and query required");
                    eprintln!("Usage: flow --friday-research-report-save <output-dir> <query>");
                    std::process::exit(1);
                }
                Command::FridayResearchReportSave {
                    output_dir: args[2].clone(),
                    query: args[3..].join(" "),
                }
            }
            "--friday-research-synthesize" | "--friday-synthesize" => {
                if args.len() <= 2 {
                    eprintln!("Error: query required");
                    eprintln!("Usage: flow --friday-research-synthesize <query>");
                    std::process::exit(1);
                }
                Command::FridayResearchSynthesize {
                    query: args[2..].join(" "),
                }
            }
            "--friday-workspace-init" => {
                let output_dir = args.get(2).cloned().unwrap_or_else(|| {
                    eprintln!("Error: output directory required");
                    eprintln!("Usage: flow --friday-workspace-init <output-dir>");
                    std::process::exit(1);
                });
                Command::FridayWorkspaceInit { output_dir }
            }
            "--friday-workspace-json" => Command::FridayWorkspaceJson {
                input_dir: args.get(2).cloned(),
            },
            "--friday-artifacts-init" | "--friday-canvas-init" => {
                let output_dir = args.get(2).cloned().unwrap_or_else(|| {
                    eprintln!("Error: output directory required");
                    eprintln!("Usage: flow --friday-artifacts-init <output-dir>");
                    std::process::exit(1);
                });
                Command::FridayArtifactsInit { output_dir }
            }
            "--friday-artifacts-json" | "--friday-canvas-json" => Command::FridayArtifactsJson {
                input_dir: args.get(2).cloned(),
            },
            "--friday-artifacts-index-multimodal" => {
                let (store_dir, bundle_dir) = parse_two_path_args(
                    &args,
                    "flow --friday-artifacts-index-multimodal <store-dir> <bundle-dir>",
                );
                Command::FridayArtifactsIndexMultimodal {
                    store_dir,
                    bundle_dir,
                }
            }
            "--friday-artifacts-index-multimodal-json" => {
                let (store_dir, bundle_dir) = parse_two_path_args(
                    &args,
                    "flow --friday-artifacts-index-multimodal-json <store-dir> <bundle-dir>",
                );
                Command::FridayArtifactsIndexMultimodalJson {
                    store_dir,
                    bundle_dir,
                }
            }
            "--friday-runtime-init" | "--friday-voice-runtime-init" => {
                let output_dir = args.get(2).cloned().unwrap_or_else(|| {
                    eprintln!("Error: output directory required");
                    eprintln!("Usage: flow --friday-runtime-init <output-dir>");
                    std::process::exit(1);
                });
                Command::FridayRuntimeInit { output_dir }
            }
            "--friday-runtime-json" | "--friday-voice-runtime-json" => Command::FridayRuntimeJson {
                input_dir: args.get(2).cloned(),
            },
            "--friday-ui" | "--friday-ui-plan" => Command::FridayUiPlan,
            "--friday-ui-json" | "--friday-ui-plan-json" => Command::FridayUiPlanJson,
            "--friday-live-ui-routes" | "--friday-route-files" => Command::FridayLiveUiRoutes,
            "--friday-live-ui-routes-json" | "--friday-route-files-json" => {
                Command::FridayLiveUiRoutesJson
            }
            "--friday-readiness" | "--friday-operator-readiness" => Command::FridayReadiness,
            "--friday-readiness-json" | "--friday-operator-readiness-json" => {
                Command::FridayReadinessJson
            }
            "--friday-route-visuals" | "--friday-screenshot-routes" => Command::FridayRouteVisuals,
            "--friday-route-visuals-json" | "--friday-screenshot-routes-json" => {
                Command::FridayRouteVisualsJson
            }
            "--friday-execution-handoffs" | "--friday-handoffs" => Command::FridayExecutionHandoffs,
            "--friday-execution-handoffs-json" | "--friday-handoffs-json" => {
                Command::FridayExecutionHandoffsJson
            }
            "--friday-dashboard-export" | "--friday-readiness-export" => {
                Command::FridayDashboardExport {
                    output_dir: args
                        .get(2)
                        .cloned()
                        .unwrap_or_else(|| "tmp/friday-dashboard".to_string()),
                }
            }
            "--friday-dashboard-export-json" | "--friday-readiness-export-json" => {
                Command::FridayDashboardExportJson {
                    output_dir: args
                        .get(2)
                        .cloned()
                        .unwrap_or_else(|| "tmp/friday-dashboard".to_string()),
                }
            }
            "--friday-dashboard-panel" | "--friday-dashboard-ui" => Command::FridayDashboardPanel {
                input_dir: args
                    .get(2)
                    .cloned()
                    .unwrap_or_else(|| "tmp/friday-dashboard".to_string()),
            },
            "--friday-dashboard-panel-json" | "--friday-dashboard-ui-json" => {
                Command::FridayDashboardPanelJson {
                    input_dir: args
                        .get(2)
                        .cloned()
                        .unwrap_or_else(|| "tmp/friday-dashboard".to_string()),
                }
            }
            "--friday-dashboard-product-ui" | "--friday-dashboard-product-ui-binding" => {
                Command::FridayDashboardProductUi {
                    input_dir: args
                        .get(2)
                        .cloned()
                        .unwrap_or_else(|| "tmp/friday-dashboard".to_string()),
                }
            }
            "--friday-dashboard-product-ui-json" | "--friday-dashboard-product-ui-binding-json" => {
                Command::FridayDashboardProductUiJson {
                    input_dir: args
                        .get(2)
                        .cloned()
                        .unwrap_or_else(|| "tmp/friday-dashboard".to_string()),
                }
            }
            "--friday-dashboard-product-ui-smoke" | "--friday-dashboard-ui-smoke" => {
                Command::FridayDashboardProductUiSmoke {
                    input_dir: args
                        .get(2)
                        .cloned()
                        .unwrap_or_else(|| "tmp/friday-dashboard".to_string()),
                }
            }
            "--friday-dashboard-product-ui-smoke-json" | "--friday-dashboard-ui-smoke-json" => {
                Command::FridayDashboardProductUiSmokeJson {
                    input_dir: args
                        .get(2)
                        .cloned()
                        .unwrap_or_else(|| "tmp/friday-dashboard".to_string()),
                }
            }
            "--friday-dashboard-host-bridge" | "--friday-dashboard-host-command-bridge" => {
                Command::FridayDashboardHostCommandBridge {
                    input_dir: args
                        .get(2)
                        .cloned()
                        .unwrap_or_else(|| "tmp/friday-dashboard".to_string()),
                }
            }
            "--friday-dashboard-host-bridge-json"
            | "--friday-dashboard-host-command-bridge-json" => {
                Command::FridayDashboardHostCommandBridgeJson {
                    input_dir: args
                        .get(2)
                        .cloned()
                        .unwrap_or_else(|| "tmp/friday-dashboard".to_string()),
                }
            }
            "--friday-trusted-host-runner" | "--friday-dashboard-trusted-runner" => {
                let (input_dir, action_id, approve, execute, cancel, history_file, reason) =
                    parse_friday_trusted_host_runner_args(&args);
                Command::FridayTrustedHostRunner {
                    input_dir,
                    action_id,
                    approve,
                    execute,
                    cancel,
                    history_file,
                    reason,
                }
            }
            "--friday-trusted-host-runner-json" | "--friday-dashboard-trusted-runner-json" => {
                let (input_dir, action_id, approve, execute, cancel, history_file, reason) =
                    parse_friday_trusted_host_runner_args(&args);
                Command::FridayTrustedHostRunnerJson {
                    input_dir,
                    action_id,
                    approve,
                    execute,
                    cancel,
                    history_file,
                    reason,
                }
            }
            "--friday-trusted-host-runner-ux" | "--friday-dashboard-trusted-runner-ux" => {
                let (history_file, release_review_file) =
                    parse_friday_trusted_host_runner_ux_args(&args);
                Command::FridayTrustedHostRunnerUx {
                    history_file,
                    release_review_file,
                }
            }
            "--friday-trusted-host-runner-ux-json"
            | "--friday-dashboard-trusted-runner-ux-json" => {
                let (history_file, release_review_file) =
                    parse_friday_trusted_host_runner_ux_args(&args);
                Command::FridayTrustedHostRunnerUxJson {
                    history_file,
                    release_review_file,
                }
            }
            "--friday-trusted-host-runner-approval-ui"
            | "--friday-dashboard-trusted-runner-approval-ui" => {
                let (history_file, release_review_file) =
                    parse_friday_trusted_host_runner_ux_args(&args);
                Command::FridayTrustedHostRunnerApprovalUi {
                    history_file,
                    release_review_file,
                }
            }
            "--friday-trusted-host-runner-approval-ui-json"
            | "--friday-dashboard-trusted-runner-approval-ui-json" => {
                let (history_file, release_review_file) =
                    parse_friday_trusted_host_runner_ux_args(&args);
                Command::FridayTrustedHostRunnerApprovalUiJson {
                    history_file,
                    release_review_file,
                }
            }
            "--friday-trusted-host-runner-cancellation-ux"
            | "--friday-dashboard-trusted-runner-cancellation-ux" => {
                let state_file = parse_friday_trusted_host_cancellation_ux_args(&args);
                Command::FridayTrustedHostRunnerCancellationUx { state_file }
            }
            "--friday-trusted-host-runner-cancellation-ux-json"
            | "--friday-dashboard-trusted-runner-cancellation-ux-json" => {
                let state_file = parse_friday_trusted_host_cancellation_ux_args(&args);
                Command::FridayTrustedHostRunnerCancellationUxJson { state_file }
            }
            "--friday-trusted-host-runner-review" | "--friday-dashboard-trusted-runner-review" => {
                let (history_file, status, action_id, since_ms, until_ms, limit) =
                    parse_friday_trusted_host_runner_review_args(&args);
                Command::FridayTrustedHostRunnerOperatorReview {
                    history_file,
                    status,
                    action_id,
                    since_ms,
                    until_ms,
                    limit,
                }
            }
            "--friday-trusted-host-runner-review-json"
            | "--friday-dashboard-trusted-runner-review-json" => {
                let (history_file, status, action_id, since_ms, until_ms, limit) =
                    parse_friday_trusted_host_runner_review_args(&args);
                Command::FridayTrustedHostRunnerOperatorReviewJson {
                    history_file,
                    status,
                    action_id,
                    since_ms,
                    until_ms,
                    limit,
                }
            }
            "--friday-trusted-host-runner-release-package"
            | "--friday-dashboard-trusted-runner-release-package" => {
                let (export_dir, history_file, state_file, output_file) =
                    parse_friday_trusted_host_runner_release_package_args(&args);
                Command::FridayTrustedHostRunnerReleasePackage {
                    export_dir,
                    history_file,
                    state_file,
                    output_file,
                }
            }
            "--friday-trusted-host-runner-release-package-json"
            | "--friday-dashboard-trusted-runner-release-package-json" => {
                let (export_dir, history_file, state_file, output_file) =
                    parse_friday_trusted_host_runner_release_package_args(&args);
                Command::FridayTrustedHostRunnerReleasePackageJson {
                    export_dir,
                    history_file,
                    state_file,
                    output_file,
                }
            }
            "--friday-trusted-runner-release-archive"
            | "--friday-dashboard-trusted-runner-release-archive" => {
                let (timeline_file, package_file) =
                    parse_friday_trusted_runner_release_archive_args(&args);
                Command::FridayTrustedRunnerReleaseArchive {
                    timeline_file,
                    package_file,
                }
            }
            "--friday-trusted-runner-release-timeline-json"
            | "--friday-dashboard-trusted-runner-release-timeline-json" => {
                let (timeline_file, package_files) =
                    parse_friday_trusted_runner_release_timeline_args(&args);
                Command::FridayTrustedRunnerReleaseTimelineJson {
                    timeline_file,
                    package_files,
                }
            }
            "--friday-trusted-runner-release-timeline"
            | "--friday-dashboard-trusted-runner-release-timeline" => {
                let (timeline_file, package_files) =
                    parse_friday_trusted_runner_release_timeline_args(&args);
                Command::FridayTrustedRunnerReleaseTimeline {
                    timeline_file,
                    package_files,
                }
            }
            "--friday-release-checklist" | "--friday-operator-release-checklist" => {
                let (
                    checklist_file,
                    package_file,
                    timeline_file,
                    export_dir,
                    todo_file,
                    changelog_file,
                    signoff_file,
                ) = parse_friday_release_checklist_args(&args);
                Command::FridayReleaseChecklist {
                    checklist_file,
                    package_file,
                    timeline_file,
                    export_dir,
                    todo_file,
                    changelog_file,
                    signoff_file,
                }
            }
            "--friday-release-checklist-json" | "--friday-operator-release-checklist-json" => {
                let (
                    checklist_file,
                    package_file,
                    timeline_file,
                    export_dir,
                    todo_file,
                    changelog_file,
                    signoff_file,
                ) = parse_friday_release_checklist_args(&args);
                Command::FridayReleaseChecklistJson {
                    checklist_file,
                    package_file,
                    timeline_file,
                    export_dir,
                    todo_file,
                    changelog_file,
                    signoff_file,
                }
            }
            "--friday-release-signoff" | "--friday-operator-release-signoff" => {
                let (checklist_file, signoff_file, operator, decision, reason) =
                    parse_friday_release_signoff_args(&args);
                Command::FridayReleaseSignoff {
                    checklist_file,
                    signoff_file,
                    operator,
                    decision,
                    reason,
                }
            }
            "--friday-release-signoff-json" | "--friday-operator-release-signoff-json" => {
                let (checklist_file, signoff_file, operator, decision, reason) =
                    parse_friday_release_signoff_args(&args);
                Command::FridayReleaseSignoffJson {
                    checklist_file,
                    signoff_file,
                    operator,
                    decision,
                    reason,
                }
            }
            "--friday-release-qa" | "--friday-release-qa-command-center" => {
                let (
                    report_file,
                    checklist_file,
                    package_file,
                    timeline_file,
                    cargo_check_result_file,
                    extension_typecheck_result_file,
                    dashboard_smoke_result_file,
                ) = parse_friday_release_qa_args(&args);
                Command::FridayReleaseQa {
                    report_file,
                    checklist_file,
                    package_file,
                    timeline_file,
                    cargo_check_result_file,
                    extension_typecheck_result_file,
                    dashboard_smoke_result_file,
                }
            }
            "--friday-release-qa-json" | "--friday-release-qa-command-center-json" => {
                let (
                    report_file,
                    checklist_file,
                    package_file,
                    timeline_file,
                    cargo_check_result_file,
                    extension_typecheck_result_file,
                    dashboard_smoke_result_file,
                ) = parse_friday_release_qa_args(&args);
                Command::FridayReleaseQaJson {
                    report_file,
                    checklist_file,
                    package_file,
                    timeline_file,
                    cargo_check_result_file,
                    extension_typecheck_result_file,
                    dashboard_smoke_result_file,
                }
            }
            "--friday-trusted-host-live-state" | "--friday-dashboard-trusted-live-state" => {
                let (state_file, history_file) = parse_friday_trusted_host_live_state_args(&args);
                Command::FridayTrustedHostLiveState {
                    state_file,
                    history_file,
                }
            }
            "--friday-trusted-host-live-state-json"
            | "--friday-dashboard-trusted-live-state-json" => {
                let (state_file, history_file) = parse_friday_trusted_host_live_state_args(&args);
                Command::FridayTrustedHostLiveStateJson {
                    state_file,
                    history_file,
                }
            }
            "--friday-trusted-host-bridge-runner" | "--friday-dashboard-trusted-bridge-runner" => {
                let (input_dir, action_id, approve, execute, cancel, history_file, reason) =
                    parse_friday_trusted_host_runner_args(&args);
                Command::FridayTrustedHostBridgeRunner {
                    state_file: trusted_host_state_file_arg(&args, &input_dir),
                    input_dir,
                    action_id,
                    approve,
                    execute,
                    cancel,
                    history_file,
                    reason,
                }
            }
            "--friday-trusted-host-bridge-runner-json"
            | "--friday-dashboard-trusted-bridge-runner-json" => {
                let (input_dir, action_id, approve, execute, cancel, history_file, reason) =
                    parse_friday_trusted_host_runner_args(&args);
                Command::FridayTrustedHostBridgeRunnerJson {
                    state_file: trusted_host_state_file_arg(&args, &input_dir),
                    input_dir,
                    action_id,
                    approve,
                    execute,
                    cancel,
                    history_file,
                    reason,
                }
            }
            "--friday-local-checks" | "--friday-execution-checks" => Command::FridayLocalChecks,
            "--friday-local-checks-json" | "--friday-execution-checks-json" => {
                Command::FridayLocalChecksJson
            }
            "--friday-browser-gate" | "--friday-verification-gate" => Command::FridayBrowserGate,
            "--friday-browser-gate-json" | "--friday-verification-gate-json" => {
                Command::FridayBrowserGateJson
            }
            "--browser-extension-smoke" | "--friday-browser-extension-smoke" => {
                Command::BrowserExtensionSmoke
            }
            "--browser-extension-smoke-json" | "--friday-browser-extension-smoke-json" => {
                Command::BrowserExtensionSmokeJson
            }
            "--browser-extension-launch-smoke" | "--friday-browser-extension-launch-smoke" => {
                Command::BrowserExtensionLaunchSmoke {
                    execute: args.iter().any(|value| value == "--execute"),
                }
            }
            "--browser-extension-launch-smoke-json"
            | "--friday-browser-extension-launch-smoke-json" => {
                Command::BrowserExtensionLaunchSmokeJson {
                    execute: args.iter().any(|value| value == "--execute"),
                }
            }
            "--browser-pack-reuse-smoke" | "--friday-browser-pack-reuse-smoke" => {
                Command::BrowserPackReuseSmoke
            }
            "--browser-pack-reuse-smoke-json" | "--friday-browser-pack-reuse-smoke-json" => {
                Command::BrowserPackReuseSmokeJson
            }
            "--browser-pack-recovery-smoke" | "--friday-browser-pack-recovery-smoke" => {
                Command::BrowserPackRecoverySmoke
            }
            "--browser-pack-recovery-smoke-json" | "--friday-browser-pack-recovery-smoke-json" => {
                Command::BrowserPackRecoverySmokeJson
            }
            "--browser-webllm-acceleration" | "--friday-browser-webllm-acceleration" => {
                Command::BrowserWebLlmAcceleration
            }
            "--browser-webllm-acceleration-json" | "--friday-browser-webllm-acceleration-json" => {
                Command::BrowserWebLlmAccelerationJson
            }
            "--friday-ocr-smoke" => {
                let (output_dir, image, execute_model) = parse_friday_ocr_smoke_args(&args);
                Command::FridayOcrSmoke {
                    output_dir,
                    image,
                    execute_model,
                }
            }
            "--friday-ocr-smoke-json" => {
                let (output_dir, image, execute_model) = parse_friday_ocr_smoke_args(&args);
                Command::FridayOcrSmokeJson {
                    output_dir,
                    image,
                    execute_model,
                }
            }
            "--friday-vlm-contract" | "--friday-vlm-smoke" => {
                let (output_dir, screenshot, prompt) = parse_friday_vlm_contract_args(&args);
                Command::FridayVlmContract {
                    output_dir,
                    screenshot,
                    prompt,
                }
            }
            "--friday-vlm-contract-json" | "--friday-vlm-smoke-json" => {
                let (output_dir, screenshot, prompt) = parse_friday_vlm_contract_args(&args);
                Command::FridayVlmContractJson {
                    output_dir,
                    screenshot,
                    prompt,
                }
            }
            "--friday-multimodal-route" | "--friday-media-route" => {
                let (request_kind, remote_allowed) = parse_friday_multimodal_route_args(&args);
                Command::FridayMultimodalRoute {
                    request_kind,
                    remote_allowed,
                }
            }
            "--friday-multimodal-route-json" | "--friday-media-route-json" => {
                let (request_kind, remote_allowed) = parse_friday_multimodal_route_args(&args);
                Command::FridayMultimodalRouteJson {
                    request_kind,
                    remote_allowed,
                }
            }
            "--friday-multimodal-diagnostics" | "--friday-ocr-diagnostics" => {
                Command::FridayMultimodalDiagnostics
            }
            "--friday-multimodal-diagnostics-json" | "--friday-ocr-diagnostics-json" => {
                Command::FridayMultimodalDiagnosticsJson
            }
            "--friday-multimodal-visual-check" | "--friday-ocr-visual-check" => {
                Command::FridayMultimodalVisualCheck
            }
            "--friday-multimodal-visual-check-json" | "--friday-ocr-visual-check-json" => {
                Command::FridayMultimodalVisualCheckJson
            }
            "--friday-screenshot-vlm" | "--friday-vlm-screenshot" => {
                let (output_dir, screenshot, prompt) = parse_friday_screenshot_vlm_args(&args);
                Command::FridayScreenshotVlm {
                    output_dir,
                    screenshot,
                    prompt,
                }
            }
            "--friday-screenshot-vlm-json" | "--friday-vlm-screenshot-json" => {
                let (output_dir, screenshot, prompt) = parse_friday_screenshot_vlm_args(&args);
                Command::FridayScreenshotVlmJson {
                    output_dir,
                    screenshot,
                    prompt,
                }
            }
            "--friday-media-affordances" | "--friday-media-actions" => {
                Command::FridayMediaAffordances
            }
            "--friday-media-affordances-json" | "--friday-media-actions-json" => {
                Command::FridayMediaAffordancesJson
            }
            "--accessibility-diagnostics" | "--accessibility" => {
                let live = !args.iter().any(|value| value == "--dry-run");
                let os = args
                    .iter()
                    .skip(2)
                    .find(|value| !value.starts_with("--"))
                    .cloned();
                Command::AccessibilityDiagnostics { os, live }
            }
            "--audit-log" | "--audit-summary" => {
                let state_file = args.get(2).cloned().unwrap_or_else(|| {
                    eprintln!("Error: state file required");
                    eprintln!("Usage: flow --audit-log <flow-state-file> [limit]");
                    std::process::exit(1);
                });
                let limit = args
                    .get(3)
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(20);
                Command::AuditLog { state_file, limit }
            }
            "--completion" | "--progress" | "--next-100" => Command::Completion,
            "--completion-json" | "--progress-json" | "--next-100-json" => Command::CompletionJson,
            "--models" => {
                let modality = args.get(2).cloned();
                Command::Models { modality }
            }
            "--install-model" => {
                let model = args.get(2).cloned().unwrap_or_else(|| {
                    eprintln!("Error: model key required");
                    eprintln!("Usage: flow --install-model webgen-4b-preview-i1-q4km");
                    std::process::exit(1);
                });
                Command::InstallModel { model }
            }
            "--ui-model-candidates" => Command::UiModelCandidates,
            "--tool-model-candidates" | "--agent-model-candidates" => Command::ToolModelCandidates,
            "--model-roles" | "--local-model-roles" => Command::ModelRoles,
            "--verify-local-models" | "--local-model-benchmark" => Command::VerifyLocalModels,
            "--uigen" => {
                let output = args.get(2).cloned().unwrap_or_else(|| {
                    eprintln!("Error: output path required");
                    eprintln!("Usage: flow --uigen <output.html> <prompt>");
                    std::process::exit(1);
                });
                if args.len() <= 3 {
                    eprintln!("Error: prompt required");
                    eprintln!("Usage: flow --uigen <output.html> <prompt>");
                    std::process::exit(1);
                }
                let prompt = args[3..].join(" ");
                Command::Uigen {
                    model: None,
                    output,
                    prompt,
                }
            }
            "--uigen-model" => {
                let model = args.get(2).cloned().unwrap_or_else(|| {
                    eprintln!("Error: model key required");
                    eprintln!("Usage: flow --uigen-model <model> <output.html> <prompt>");
                    std::process::exit(1);
                });
                let output = args.get(3).cloned().unwrap_or_else(|| {
                    eprintln!("Error: output path required");
                    eprintln!("Usage: flow --uigen-model <model> <output.html> <prompt>");
                    std::process::exit(1);
                });
                if args.len() <= 4 {
                    eprintln!("Error: prompt required");
                    eprintln!("Usage: flow --uigen-model <model> <output.html> <prompt>");
                    std::process::exit(1);
                }
                let prompt = args[4..].join(" ");
                Command::Uigen {
                    model: Some(model),
                    output,
                    prompt,
                }
            }
            "--uigen-google" => {
                let model = args.get(2).cloned();
                Command::UigenGoogle { model }
            }
            "--uigen-vision" => {
                let screenshot = args.get(2).cloned().unwrap_or_else(|| {
                    eprintln!("Error: screenshot path required");
                    eprintln!("Usage: flow --uigen-vision <screenshot.png> <output.html> <prompt>");
                    std::process::exit(1);
                });
                let output = args.get(3).cloned().unwrap_or_else(|| {
                    eprintln!("Error: output path required");
                    eprintln!("Usage: flow --uigen-vision <screenshot.png> <output.html> <prompt>");
                    std::process::exit(1);
                });
                if args.len() <= 4 {
                    eprintln!("Error: prompt required");
                    eprintln!("Usage: flow --uigen-vision <screenshot.png> <output.html> <prompt>");
                    std::process::exit(1);
                }
                let prompt = args[4..].join(" ");
                Command::UigenVision {
                    screenshot,
                    output,
                    prompt,
                }
            }
            "--uigen-vision-google" => Command::UigenVisionGoogle,
            "--plan" => {
                let modality = args.get(2).cloned().unwrap_or_else(|| "chat".to_string());
                let model = args.get(3).cloned();
                Command::Plan { modality, model }
            }
            "--blueprint" => {
                let host = args.get(2).cloned().unwrap_or_else(|| "dx".to_string());
                Command::Blueprint { host }
            }
            "--browser-profile" => {
                let flavor = args
                    .get(2)
                    .cloned()
                    .unwrap_or_else(|| "chromium".to_string());
                Command::BrowserProfile { flavor }
            }
            "--browser-plan" => {
                let flavor = args
                    .get(2)
                    .cloned()
                    .unwrap_or_else(|| "chromium".to_string());
                let task = args
                    .get(3)
                    .cloned()
                    .unwrap_or_else(|| "rewrite-selection".to_string());
                let modality = args.get(4).cloned().unwrap_or_else(|| "chat".to_string());
                let mut remote_fallback = false;
                let mut model = None;
                for arg in args.iter().skip(5) {
                    if matches!(arg.as_str(), "--remote" | "--allow-remote") {
                        remote_fallback = true;
                    } else if model.is_none() {
                        model = Some(arg.clone());
                    }
                }
                Command::BrowserPlan {
                    flavor,
                    task,
                    modality,
                    model,
                    remote_fallback,
                }
            }
            "--browser-packs" => Command::BrowserPacks,
            "--grammar" => {
                let mut index = 2;
                let mut fix = false;
                if matches!(args.get(index).map(String::as_str), Some("--fix" | "-f")) {
                    fix = true;
                    index += 1;
                }

                if args.len() <= index {
                    eprintln!("Error: Text is required for grammar analysis");
                    eprintln!("Usage: flow --grammar [--fix] <text>");
                    std::process::exit(1);
                }

                let text = args[index..].join(" ");
                Command::Grammar { text, fix }
            }
            "--wakewords" | "--wake-words" => Command::WakeWords,
            "--production-config" => {
                let target = args
                    .get(2)
                    .cloned()
                    .unwrap_or_else(|| "dx-desktop".to_string());
                Command::ProductionConfig { target }
            }
            "--export-production-bundle" => {
                let output_dir = args
                    .get(2)
                    .cloned()
                    .unwrap_or_else(|| "configs/production".to_string());
                Command::ExportProductionBundle { output_dir }
            }
            "--release-summary" => Command::ReleaseSummary,
            "--export-release-summary" => {
                let output_dir = args
                    .get(2)
                    .cloned()
                    .unwrap_or_else(|| "release".to_string());
                Command::ExportReleaseSummary { output_dir }
            }
            _ => Command::Interactive,
        };

        Self { command }
    }
}

fn parse_friday_ocr_smoke_args(args: &[String]) -> (String, Option<String>, bool) {
    let output_dir = args.get(2).cloned().unwrap_or_else(|| {
        eprintln!("Error: output directory required");
        eprintln!("Usage: flow --friday-ocr-smoke <output-dir> [image-path] [--execute]");
        std::process::exit(1);
    });
    let execute_model = args.iter().skip(3).any(|value| value == "--execute");
    let image = args
        .iter()
        .skip(3)
        .find(|value| !value.starts_with("--"))
        .cloned();
    (output_dir, image, execute_model)
}

fn parse_friday_vlm_contract_args(args: &[String]) -> (String, Option<String>, Option<String>) {
    let output_dir = args.get(2).cloned().unwrap_or_else(|| {
        eprintln!("Error: output directory required");
        eprintln!("Usage: flow --friday-vlm-contract <output-dir> [screenshot-path] [prompt]");
        std::process::exit(1);
    });
    let screenshot = args.get(3).cloned();
    let prompt = if args.len() > 4 {
        Some(args[4..].join(" "))
    } else {
        None
    };
    (output_dir, screenshot, prompt)
}

fn parse_friday_multimodal_route_args(args: &[String]) -> (String, bool) {
    let request_kind = args.get(2).cloned().unwrap_or_else(|| {
        eprintln!("Error: request kind required");
        eprintln!("Usage: flow --friday-multimodal-route <ocr|vlm|audio|image|video> [--remote]");
        std::process::exit(1);
    });
    let remote_allowed = args.iter().any(|value| value == "--remote");
    (request_kind, remote_allowed)
}

fn parse_friday_screenshot_vlm_args(args: &[String]) -> (String, String, Option<String>) {
    if args.len() <= 3 {
        eprintln!("Error: output directory and screenshot path required");
        eprintln!("Usage: flow --friday-screenshot-vlm <output-dir> <screenshot-path> [prompt]");
        std::process::exit(1);
    }
    let output_dir = args[2].clone();
    let screenshot = args[3].clone();
    let prompt = if args.len() > 4 {
        Some(args[4..].join(" "))
    } else {
        None
    };
    (output_dir, screenshot, prompt)
}

fn parse_friday_trusted_host_runner_args(
    args: &[String],
) -> (
    String,
    Option<String>,
    bool,
    bool,
    bool,
    String,
    Option<String>,
) {
    let input_dir = args
        .get(2)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| "tmp/friday-dashboard".to_string());
    let action_id = flag_value(args, "--action-id").or_else(|| flag_value(args, "--action"));
    let approve = args.iter().any(|value| value == "--approve");
    let execute = args.iter().any(|value| value == "--execute");
    let cancel = args.iter().any(|value| value == "--cancel");
    let history_file = flag_value(args, "--history")
        .unwrap_or_else(|| format!("{input_dir}/trusted-host-runner-history.json"));
    let reason = flag_value(args, "--reason");
    (
        input_dir,
        action_id,
        approve,
        execute,
        cancel,
        history_file,
        reason,
    )
}

fn parse_friday_trusted_host_runner_ux_args(args: &[String]) -> (String, String) {
    let history_file = flag_value(args, "--history")
        .or_else(|| {
            args.get(2)
                .filter(|value| !value.starts_with("--"))
                .cloned()
        })
        .unwrap_or_else(|| "tmp/friday-dashboard/trusted-host-runner-history.json".to_string());
    let release_review_file = flag_value(args, "--release-review")
        .unwrap_or_else(|| "tmp/friday-dashboard/release-review.json".to_string());
    (history_file, release_review_file)
}

fn parse_friday_trusted_host_live_state_args(args: &[String]) -> (String, String) {
    let state_file = args
        .get(2)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .or_else(|| flag_value(args, "--state"))
        .unwrap_or_else(|| "tmp/friday-dashboard/trusted-host-live-state.json".to_string());
    let history_file = flag_value(args, "--history")
        .unwrap_or_else(|| "tmp/friday-dashboard/trusted-host-runner-history.json".to_string());
    (state_file, history_file)
}

fn parse_friday_trusted_host_cancellation_ux_args(args: &[String]) -> String {
    args.get(2)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .or_else(|| flag_value(args, "--state"))
        .unwrap_or_else(|| "tmp/friday-dashboard/trusted-host-live-state.json".to_string())
}

fn parse_friday_trusted_host_runner_review_args(
    args: &[String],
) -> (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    usize,
) {
    let history_file = args
        .get(2)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .or_else(|| flag_value(args, "--history"))
        .unwrap_or_else(|| "tmp/friday-dashboard/trusted-host-runner-history.json".to_string());
    let status = flag_value(args, "--status");
    let action_id = flag_value(args, "--action-id");
    let since_ms = flag_value(args, "--since-ms");
    let until_ms = flag_value(args, "--until-ms");
    let limit = flag_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(50);
    (history_file, status, action_id, since_ms, until_ms, limit)
}

fn parse_friday_trusted_host_runner_release_package_args(
    args: &[String],
) -> (String, String, String, String) {
    let export_dir = args
        .get(2)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .or_else(|| flag_value(args, "--export-dir"))
        .unwrap_or_else(|| "tmp/friday-dashboard".to_string());
    let history_file = flag_value(args, "--history")
        .unwrap_or_else(|| format!("{export_dir}/trusted-host-runner-history.json"));
    let state_file = flag_value(args, "--state")
        .unwrap_or_else(|| format!("{export_dir}/trusted-host-live-state.json"));
    let output_file = flag_value(args, "--output")
        .unwrap_or_else(|| format!("{export_dir}/trusted-runner-release-package.json"));
    (export_dir, history_file, state_file, output_file)
}

fn parse_friday_trusted_runner_release_archive_args(args: &[String]) -> (String, String) {
    let positional = positional_values(args, &["--timeline", "--package"]);
    let timeline_file = flag_value(args, "--timeline")
        .or_else(|| positional.first().cloned())
        .unwrap_or_else(|| "tmp/friday-dashboard/trusted-runner-release-timeline.json".to_string());
    let package_file = flag_value(args, "--package")
        .or_else(|| positional.get(1).cloned())
        .unwrap_or_else(|| "tmp/friday-dashboard/trusted-runner-release-package.json".to_string());
    (timeline_file, package_file)
}

fn parse_friday_trusted_runner_release_timeline_args(args: &[String]) -> (String, Vec<String>) {
    let positional = positional_values(args, &["--timeline", "--package"]);
    let timeline_from_flag = flag_value(args, "--timeline");
    let timeline_file = timeline_from_flag
        .clone()
        .or_else(|| positional.first().cloned())
        .unwrap_or_else(|| "tmp/friday-dashboard/trusted-runner-release-timeline.json".to_string());
    let positional_packages = if timeline_from_flag.is_some() {
        positional
    } else {
        positional.into_iter().skip(1).collect::<Vec<_>>()
    };
    let package_files = repeated_flag_values(args, "--package")
        .into_iter()
        .chain(positional_packages)
        .collect::<Vec<_>>();
    (timeline_file, package_files)
}

fn parse_friday_release_checklist_args(
    args: &[String],
) -> (String, String, String, String, String, String, String) {
    let export_dir = flag_value(args, "--export-dir").unwrap_or_else(|| {
        args.get(2)
            .filter(|value| !value.starts_with("--"))
            .cloned()
            .unwrap_or_else(|| "tmp/friday-dashboard".to_string())
    });
    let checklist_file = flag_value(args, "--output")
        .or_else(|| flag_value(args, "--checklist"))
        .unwrap_or_else(|| format!("{export_dir}/release-operator-checklist.json"));
    let package_file = flag_value(args, "--package")
        .unwrap_or_else(|| format!("{export_dir}/trusted-runner-release-package.json"));
    let timeline_file = flag_value(args, "--timeline")
        .unwrap_or_else(|| format!("{export_dir}/trusted-runner-release-timeline.json"));
    let todo_file = flag_value(args, "--todo").unwrap_or_else(|| "TODO.md".to_string());
    let changelog_file =
        flag_value(args, "--changelog").unwrap_or_else(|| "CHANGELOG.md".to_string());
    let signoff_file = flag_value(args, "--signoffs")
        .unwrap_or_else(|| format!("{export_dir}/release-signoffs.json"));
    (
        checklist_file,
        package_file,
        timeline_file,
        export_dir,
        todo_file,
        changelog_file,
        signoff_file,
    )
}

fn parse_friday_release_signoff_args(args: &[String]) -> (String, String, String, String, String) {
    let checklist_file = flag_value(args, "--checklist")
        .or_else(|| args.get(2).filter(|value| !value.starts_with("--")).cloned())
        .unwrap_or_else(|| "tmp/friday-dashboard/release-operator-checklist.json".to_string());
    let signoff_file = flag_value(args, "--signoffs")
        .unwrap_or_else(|| "tmp/friday-dashboard/release-signoffs.json".to_string());
    let operator = flag_value(args, "--operator").unwrap_or_else(|| "operator".to_string());
    let decision = flag_value(args, "--decision").unwrap_or_else(|| "approved".to_string());
    let reason = flag_value(args, "--reason").unwrap_or_else(|| {
        "Operator reviewed the local release checklist evidence.".to_string()
    });
    (checklist_file, signoff_file, operator, decision, reason)
}

fn parse_friday_release_qa_args(
    args: &[String],
) -> (String, String, String, String, String, String, String) {
    let export_dir = flag_value(args, "--export-dir").unwrap_or_else(|| {
        args.get(2)
            .filter(|value| !value.starts_with("--"))
            .cloned()
            .unwrap_or_else(|| "tmp/friday-dashboard".to_string())
    });
    let report_file = flag_value(args, "--output")
        .or_else(|| flag_value(args, "--report"))
        .unwrap_or_else(|| format!("{export_dir}/release-qa-command-center.json"));
    let checklist_file = flag_value(args, "--checklist")
        .unwrap_or_else(|| format!("{export_dir}/release-operator-checklist.json"));
    let package_file = flag_value(args, "--package")
        .unwrap_or_else(|| format!("{export_dir}/trusted-runner-release-package.json"));
    let timeline_file = flag_value(args, "--timeline")
        .unwrap_or_else(|| format!("{export_dir}/trusted-runner-release-timeline.json"));
    let cargo_check_result_file = flag_value(args, "--cargo-check-result")
        .unwrap_or_else(|| format!("{export_dir}/cargo-check.txt"));
    let extension_typecheck_result_file = flag_value(args, "--extension-typecheck-result")
        .unwrap_or_else(|| format!("{export_dir}/extension-typecheck.txt"));
    let dashboard_smoke_result_file = flag_value(args, "--dashboard-smoke-result")
        .unwrap_or_else(|| format!("{export_dir}/dashboard-smoke.txt"));
    (
        report_file,
        checklist_file,
        package_file,
        timeline_file,
        cargo_check_result_file,
        extension_typecheck_result_file,
        dashboard_smoke_result_file,
    )
}

fn trusted_host_state_file_arg(args: &[String], input_dir: &str) -> String {
    flag_value(args, "--state")
        .unwrap_or_else(|| format!("{input_dir}/trusted-host-live-state.json"))
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].clone())
}

fn repeated_flag_values(args: &[String], flag: &str) -> Vec<String> {
    args.windows(2)
        .filter(|window| window[0] == flag)
        .map(|window| window[1].clone())
        .collect()
}

fn positional_values(args: &[String], value_flags: &[&str]) -> Vec<String> {
    let mut values = Vec::new();
    let mut index = 2;
    while index < args.len() {
        let value = &args[index];
        if value_flags.contains(&value.as_str()) {
            index += 2;
            continue;
        }
        if value.starts_with("--") {
            index += 1;
            continue;
        }
        values.push(value.clone());
        index += 1;
    }
    values
}

fn parse_two_path_args(args: &[String], usage: &str) -> (String, String) {
    if args.len() <= 3 {
        eprintln!("Error: two paths required");
        eprintln!("Usage: {usage}");
        std::process::exit(1);
    }
    (args[2].clone(), args[3].clone())
}
