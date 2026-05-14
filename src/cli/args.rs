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

fn parse_two_path_args(args: &[String], usage: &str) -> (String, String) {
    if args.len() <= 3 {
        eprintln!("Error: two paths required");
        eprintln!("Usage: {usage}");
        std::process::exit(1);
    }
    (args[2].clone(), args[3].clone())
}
