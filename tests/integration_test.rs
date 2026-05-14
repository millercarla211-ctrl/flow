use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use flow::FlowLocalRuntime;
use flow::audio::{AudioLoader, MelSpectrogramConfig, compute_mel_spectrogram};
use flow::browser::{
    BrowserExtensionInstallProbe, BrowserExtensionSmokeStatus, BrowserPackRecoveryScenarioKind,
    BrowserPackRecoveryStatus, BrowserPackReuseStatus, BrowserWebLlmAccelerationStatus,
    browser_extension_launch_smoke_report_for_root, browser_extension_smoke_report_for_root,
    browser_pack_recovery_smoke_report, browser_pack_reuse_smoke_report,
    browser_webllm_acceleration_report,
};
use flow::competitive::default_competitive_scorecard;
use flow::embed::{FlowEmbeddingRegistry, HostSurface, IntegrationMode};
use flow::experience::{
    AppContext, DictationAssistRequest, DictionaryEntry, FlowDictationEngine, FlowTypingAssistant,
    SnippetEntry, StylePreset, ToneStyle, TypingAssistRequest, WritingDomain,
};
use flow::forge_bridge::{ForgeBridge, ForgeRemoteKind};
use flow::friday::{
    FridayArtifactStore, FridayAutomationTrigger, FridayCompetitor, FridayConnectorAuthState,
    FridayMultimodalDiagnosticStatus, FridayMultimodalRequestKind, FridayMultimodalRouteStatus,
    FridayMultimodalSurface, FridayPermissionScope, FridayPreviewRunner, FridayResearchWorkflow,
    FridayRuntimeSurfaceStore, FridayUiIntegrationStatus, FridayUiStateKind, FridayUiStateTone,
    FridayUiVisualCheckStatus, FridayVerificationStatus, FridayWorkspaceStore,
    default_friday_browser_verification_report, default_friday_local_execution_checks,
    default_friday_product_plan,
    default_friday_ui_integration_plan, friday_media_affordances, friday_multimodal_route,
    friday_multimodal_ui_diagnostics, friday_multimodal_visual_check, run_friday_ocr_smoke,
    run_friday_screenshot_vlm_handoff, run_friday_vlm_contract,
};
use flow::long_context::RlmBridge;
use flow::prompt::DxSerializer;
use flow::provider_catalog::{CatalogSource, ProviderCatalogBridge};
use flow::remote::{AccessTier, RemoteCapability, RemoteModelEndpoint, RemoteProviderRouter};
use flow::runtime::{
    ArtifactBundle, ArtifactFile, ArtifactFormat, BrokerRequest, ComputeBackend, DeviceProfile,
    DeviceTier, GraphicsDevice, Modality, RuntimeBroker, RuntimeKind, RuntimeLaunch,
    benchmark_record,
};
use flow::search::{MetasearchServerConfig, metasearch_categories};
use flow::storage::{FlowPackStore, PromptCacheEntry, PromptCacheIndex};
use flow::workspace::dx_project_statuses;
use flow::writing::HarperGrammarChecker;

fn make_device_profile(total_memory_bytes: u64, available_memory_bytes: u64) -> DeviceProfile {
    DeviceProfile {
        os: "windows".to_string(),
        arch: "x86_64".to_string(),
        cpu_model: "Test CPU".to_string(),
        physical_cores: 4,
        logical_cores: 8,
        total_memory_bytes,
        available_memory_bytes,
        battery_powered: None,
        thermal_class: None,
        graphics: vec![GraphicsDevice {
            name: "Integrated GPU".to_string(),
            vendor: Some("intel".to_string()),
            vram_bytes: None,
            integrated: true,
            backends: vec![ComputeBackend::Cpu, ComputeBackend::DirectMl],
        }],
        tier: if total_memory_bytes < 8 * 1024 * 1024 * 1024 {
            DeviceTier::Low
        } else {
            DeviceTier::Balanced
        },
    }
}

fn app_context(domain: WritingDomain) -> AppContext {
    AppContext {
        app_name: "Test App".to_string(),
        window_title: None,
        url: None,
        language: Some("en".to_string()),
        domain,
        workspace_files: vec!["src/main.rs".to_string(), "README.md".to_string()],
        team_terms: vec!["Supabase".to_string()],
    }
}

fn temp_root(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("flow-{prefix}-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn broker_prefers_small_chat_model_on_low_tier_device() {
    let profile = make_device_profile(6 * 1024 * 1024 * 1024, 4 * 1024 * 1024 * 1024);
    let broker = RuntimeBroker::from_parts(
        profile,
        flow::runtime::default_model_catalog(),
        flow::runtime::default_activation_config(),
    );

    let mut request = BrokerRequest::new(Modality::Chat);
    request.allow_conversion = false;
    request.allow_publish = false;

    let plan = broker.build_plan(request);
    assert_eq!(plan.selected_model.as_deref(), Some("qwen3-0.6b"));
    assert_eq!(plan.launch, RuntimeLaunch::Embedded);
}

#[test]
fn embeddable_local_runtime_picks_qwen3_for_low_end_devices() {
    let runtime = FlowLocalRuntime::for_device_profile(make_device_profile(
        6 * 1024 * 1024 * 1024,
        4 * 1024 * 1024 * 1024,
    ))
    .unwrap();

    assert_eq!(runtime.default_text_model_key(), Some("qwen3-0.6b"));
    assert_eq!(
        runtime.summary().speech_to_text.model_key.as_deref(),
        Some("moonshine-tiny")
    );
    assert_eq!(
        runtime.summary().text_to_speech.model_key.as_deref(),
        Some("kokoro-int8")
    );
}

#[test]
fn flowpack_round_trip_and_prompt_cache_round_trip() {
    let root = temp_root("flowpack");
    let model_file = root.join("artifact.gguf");
    fs::write(&model_file, b"flow-artifact").unwrap();

    let artifact = ArtifactBundle {
        model_key: "qwen3-0.6b".to_string(),
        upstream_repo: "Qwen/Qwen3-0.6B".to_string(),
        upstream_revision: Some("main".to_string()),
        root_dir: root.to_string_lossy().into_owned(),
        artifact_format: ArtifactFormat::Gguf,
        quantization: Some("Q4_K_M".to_string()),
        license: Some("apache-2.0".to_string()),
        runtime: RuntimeKind::LlamaCppEmbedded,
        files: vec![ArtifactFile {
            path: model_file.to_string_lossy().into_owned(),
            bytes: Some(fs::metadata(&model_file).unwrap().len()),
            sha256: Some(FlowPackStore::sha256_file(&model_file).unwrap()),
        }],
        redistributable: true,
        gated: false,
        local_only: false,
    };

    let device = make_device_profile(8 * 1024 * 1024 * 1024, 6 * 1024 * 1024 * 1024);
    let benchmarks = vec![benchmark_record(
        "qwen3-0.6b",
        RuntimeKind::LlamaCppEmbedded,
        Modality::Chat,
        1200,
        Some(28),
        None,
        DeviceTier::Balanced,
    )];

    FlowPackStore::write_flowpack(&root, &device, &artifact, &benchmarks).unwrap();
    let manifest = FlowPackStore::read_flowpack(&root).unwrap();
    assert_eq!(manifest.artifact.model_key, "qwen3-0.6b");
    assert_eq!(manifest.benchmarks.len(), 1);

    let prompt_cache = PromptCacheIndex {
        entries: vec![PromptCacheEntry {
            key: FlowPackStore::prompt_cache_key("qwen3-0.6b", "tok", "sys", "tools"),
            prompt_hash: "hash".to_string(),
            token_count: 3,
            tokens: vec![1, 2, 3],
            updated_at_unix_ms: 1,
        }],
    };
    FlowPackStore::write_prompt_cache(&root, &prompt_cache).unwrap();
    let restored = FlowPackStore::read_prompt_cache(&root).unwrap();
    assert_eq!(restored.entries.len(), 1);
    assert_eq!(restored.entries[0].tokens, vec![1, 2, 3]);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn grammar_checker_reports_and_corrects_basic_text() {
    let checker = HarperGrammarChecker::new();
    let issues = checker.analyze("This is an test.").unwrap();
    assert!(!issues.is_empty());

    let corrected = checker.correct("This is an test.").unwrap();
    assert!(corrected.contains("a test"));
}

#[test]
fn wake_command_definitions_are_canonical() {
    let definitions = flow::runtime::wake_command_definitions();
    let commands = definitions
        .iter()
        .map(|definition| definition.command_key)
        .collect::<Vec<_>>();
    assert_eq!(commands, vec!["dx", "friday", "hello", "aladdin", "arise"]);
    assert!(
        definitions
            .iter()
            .all(|definition| definition.threshold == 68)
    );
    assert!(
        definitions
            .iter()
            .all(|definition| definition.aliases.is_empty())
    );
}

#[test]
fn mel_spectrogram_has_expected_shape() {
    let config = MelSpectrogramConfig::default();
    let samples = (0..3200)
        .map(|index| (index as f32 * 0.05).sin())
        .collect::<Vec<_>>();
    let mel = compute_mel_spectrogram(&samples, &config);

    assert_eq!(mel.nrows(), config.n_mels);
    assert!(mel.ncols() >= 1);
}

#[test]
fn audio_loader_reads_fixture_when_present() {
    let fixture = PathBuf::from("tests/fixtures/audio.mp3");
    if !fixture.exists() {
        return;
    }

    let samples = AudioLoader::load(&fixture).unwrap();
    assert!(!samples.is_empty());
}

#[test]
fn embedding_registry_builds_dx_blueprint() {
    let registry = FlowEmbeddingRegistry::from_root(".");
    let blueprint = registry.blueprint(HostSurface::Dx);

    assert_eq!(blueprint.integration_mode, IntegrationMode::FullRuntime);
    assert!(
        blueprint
            .adjacent_projects
            .iter()
            .any(|item| item.key == "providers")
    );
    assert!(
        blueprint
            .adjacent_projects
            .iter()
            .any(|item| item.key == "forge")
    );
    assert!(
        blueprint
            .provider_catalog_plan
            .sources
            .contains(&CatalogSource::ModelsDev)
    );
}

#[test]
fn dx_serializer_round_trips_json_payload() {
    let payload = serde_json::json!({
        "system": "You are dx",
        "tools": [{"name": "search"}, {"name": "provider-route"}]
    });

    let envelope = DxSerializer::encode_json("prompt", &payload).unwrap();
    let restored = DxSerializer::decode_json(&envelope).unwrap();
    assert_eq!(restored, payload);
}

#[test]
fn remote_router_prefers_free_remote_before_premium() {
    let premium = RemoteModelEndpoint {
        provider_id: "premium".to_string(),
        model_id: "premium-model".to_string(),
        label: "Premium".to_string(),
        access_tier: AccessTier::PremiumRemote,
        auth_kind: flow::embed::ProviderAuthKind::ApiKey,
        capabilities: vec![RemoteCapability::Chat],
    };
    let free = RemoteModelEndpoint {
        provider_id: "free".to_string(),
        model_id: "free-model".to_string(),
        label: "Free".to_string(),
        access_tier: AccessTier::FreeRemote,
        auth_kind: flow::embed::ProviderAuthKind::OAuth,
        capabilities: vec![RemoteCapability::Chat],
    };

    let plan = RemoteProviderRouter::plan(
        Modality::Chat,
        Some("qwen3-0.6b".to_string()),
        vec![premium, free],
    );
    assert_eq!(plan.remote_candidates[0].provider_id, "free");
}

#[test]
fn forge_bridge_covers_code_and_media_targets() {
    let plan = ForgeBridge::for_dx_media_pipeline();
    assert!(plan.remotes.contains(&ForgeRemoteKind::Github));
    assert!(plan.remotes.contains(&ForgeRemoteKind::Youtube));
}

#[test]
fn rlm_bridge_prefers_serializer_and_prompt_cache() {
    let plan = RlmBridge::for_codebase_analysis();
    assert!(plan.use_serializer);
    assert!(plan.use_prompt_cache);
}

#[test]
fn dx_workspace_registry_contains_forge() {
    let projects = dx_project_statuses();
    assert!(projects.iter().any(|project| project.key == "forge"));
}

#[test]
fn provider_catalog_plan_includes_models_dev_and_litellm() {
    let plan = ProviderCatalogBridge::default_plan();
    assert!(plan.sources.contains(&CatalogSource::ModelsDev));
    assert!(plan.sources.contains(&CatalogSource::LiteLlm));
}

#[test]
fn competitive_scorecard_has_expected_baseline() {
    let scorecard = default_competitive_scorecard();
    assert_eq!(scorecard.overall_score_out_of_100, 51);
    assert_eq!(scorecard.wispr_replacement_score_out_of_100, 52);
    assert_eq!(scorecard.grammarly_replacement_score_out_of_100, 40);
    assert_eq!(scorecard.flow_native_advantage_score_out_of_100, 57);
}

#[test]
fn friday_plan_covers_the_target_ai_assistants() {
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
    assert!(plan.search_policy.forbids_perplexity_computer);
    assert!(plan.score_out_of_100 < 60);
}

#[test]
fn friday_runtime_search_plans_use_metasearch() {
    let runtime = flow::DxFlowRuntime::detect();
    let answer_plan = runtime.friday_answer_search_plan("best current local stt");
    let research_plan = runtime.friday_research_search_plan("compare ChatGPT Claude Gemini");

    assert!(answer_plan.use_adjacent_metasearch);
    assert!(research_plan.use_adjacent_metasearch);
    assert!(
        answer_plan
            .notes
            .iter()
            .any(|note| note.contains("Perplexity Computer"))
    );
}

#[test]
fn friday_research_workflow_is_metasearch_backed_and_exportable() {
    let workflow = FridayResearchWorkflow::for_query("compare current AI search features");

    assert!(workflow.local_first);
    assert!(workflow.forbids_perplexity_computer);
    assert!(workflow.ready_stage_count() >= 2);
    assert!(workflow.answer_plan.use_adjacent_metasearch);
    assert!(workflow.deep_research_plan.use_adjacent_metasearch);
    assert!(
        workflow
            .local_metasearch_api_path
            .contains("/api/v1/search")
    );
    assert!(
        workflow
            .export_formats
            .contains(&"markdown-report".to_string())
    );
}

#[test]
fn friday_metasearch_client_builds_local_api_paths() {
    let runtime = flow::DxFlowRuntime::detect();
    let answer_plan = runtime.friday_answer_search_plan("local search citations");
    let path = MetasearchServerConfig::default().api_path_for_plan(&answer_plan);
    let categories = metasearch_categories(&answer_plan.verticals);

    assert!(path.starts_with("/api/v1/search?format=json"));
    assert!(path.contains("q=local%20search%20citations"));
    assert!(categories.contains(&"general".to_string()));
    assert!(categories.contains(&"news".to_string()));
}

#[test]
fn friday_workspace_store_persists_projects_memory_and_connectors() {
    let root = temp_root("friday-workspace");
    let store = FridayWorkspaceStore::seed_local_first();
    let snapshot = store.write_to_dir(&root).unwrap();

    assert!(snapshot.projects_json.exists());
    assert!(snapshot.memories_json.exists());
    assert!(snapshot.connectors_json.exists());
    assert_eq!(snapshot.manifest.project_count, 1);
    assert_eq!(snapshot.manifest.connector_count, 3);
    assert!(snapshot.manifest.findings.is_empty());

    let restored = FridayWorkspaceStore::read_from_dir(&root).unwrap();
    let project = &restored.projects[0];
    assert_eq!(project.id, "friday-local");
    assert_eq!(restored.project_memory(&project.id).len(), 1);
    assert_eq!(restored.connectors_for_project(&project.id).len(), 3);

    let metasearch = restored.connector("metasearch").unwrap();
    assert_eq!(metasearch.auth_state, FridayConnectorAuthState::LocalOnly);
    assert!(
        metasearch
            .permission_scopes
            .contains(&FridayPermissionScope::Metasearch)
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_artifact_store_persists_canvas_code_and_artifact_records() {
    let root = temp_root("friday-artifacts");
    let store = FridayArtifactStore::seed_for_local_workspace();
    let snapshot = store.write_to_dir(&root).unwrap();

    assert!(snapshot.artifacts_json.exists());
    assert!(snapshot.checkpoints_json.exists());
    assert!(snapshot.diffs_json.exists());
    assert!(snapshot.code_tasks_json.exists());
    assert!(snapshot.multimodal_metadata_json.exists());
    assert_eq!(snapshot.manifest.artifact_count, 2);
    assert_eq!(snapshot.manifest.code_task_count, 1);
    assert!(snapshot.manifest.findings.is_empty());

    let restored = FridayArtifactStore::read_from_dir(&root).unwrap();
    let ui = restored.artifact("ui-prototype").unwrap();
    assert_eq!(ui.preview_runner, FridayPreviewRunner::Html);
    assert_eq!(restored.checkpoints_for_artifact(&ui.id).len(), 1);
    assert_eq!(restored.diffs_for_artifact(&ui.id).len(), 1);
    assert_eq!(
        restored.task_artifacts("artifact-canvas-checkpoint").len(),
        2
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_artifact_store_indexes_multimodal_bundle_metadata() {
    let root = temp_root("friday-multimodal-index");
    let bundle_dir = root.join("bundle");
    let store_dir = root.join("store");
    let smoke = run_friday_ocr_smoke(&bundle_dir, None, false).unwrap();
    let mut store = FridayArtifactStore::read_or_seed_from_dir(&store_dir).unwrap();
    let import = store.import_multimodal_bundle(&bundle_dir).unwrap();
    let snapshot = store.write_to_dir(&store_dir).unwrap();

    assert_eq!(import.imported_metadata_count, 1);
    assert!(import.imported_artifact_ids.contains(&smoke.artifact.id));
    assert!(snapshot.multimodal_metadata_json.exists());
    assert_eq!(snapshot.manifest.multimodal_metadata_count, 1);
    assert!(snapshot.manifest.findings.is_empty());

    let restored = FridayArtifactStore::read_from_dir(&store_dir).unwrap();
    assert_eq!(
        restored
            .multimodal_metadata_for_artifact(&smoke.artifact.id)
            .len(),
        1
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_runtime_store_persists_voice_multimodal_and_automations() {
    let root = temp_root("friday-runtime");
    let store = FridayRuntimeSurfaceStore::seed_local_first();
    let snapshot = store.write_to_dir(&root).unwrap();

    assert!(snapshot.voice_json.exists());
    assert!(snapshot.multimodal_json.exists());
    assert!(snapshot.automations_json.exists());
    assert_eq!(snapshot.manifest.multimodal_count, 2);
    assert_eq!(snapshot.manifest.automation_count, 2);
    assert!(snapshot.manifest.findings.is_empty());

    let restored = FridayRuntimeSurfaceStore::read_from_dir(&root).unwrap();
    assert_eq!(restored.voice.stt_model_key, "parakeet-tdt-0.6b-v3-int8");
    assert_eq!(restored.voice.tts_model_key, "kokoro-int8");
    assert!(restored.voice.wake_commands.contains(&"hello".to_string()));
    assert!(
        restored
            .multimodal
            .iter()
            .any(|item| item.surface == FridayMultimodalSurface::Ocr)
    );
    assert!(restored.automations.iter().any(|item| item.trigger
        == FridayAutomationTrigger::BackgroundResearch
        && item.approval_required));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_ui_plan_wires_ask_search_and_research_routes() {
    let plan = default_friday_ui_integration_plan();
    assert_eq!(plan.score_out_of_100, 100);
    assert_eq!(plan.ready_route_count(), 13);

    let ask = plan.route(flow::FridayWorkspaceArea::Ask).unwrap();
    assert_eq!(ask.status, FridayUiIntegrationStatus::Wired);
    assert!(ask.stream_enabled);
    assert!(ask.citations_visible);

    let search = plan.route(flow::FridayWorkspaceArea::Search).unwrap();
    assert!(search.primary_command.contains("--friday-metasearch"));
    assert!(!search.source_controls.is_empty());

    let research = plan.route(flow::FridayWorkspaceArea::Research).unwrap();
    assert!(research.report_persistence);
    assert!(
        research
            .data_bindings
            .iter()
            .any(|binding| binding.command.contains("--friday-research-report-save"))
    );
}

#[test]
fn friday_ui_plan_wires_remaining_store_backed_routes() {
    let plan = default_friday_ui_integration_plan();

    for area in [
        flow::FridayWorkspaceArea::Projects,
        flow::FridayWorkspaceArea::Memory,
        flow::FridayWorkspaceArea::Connectors,
        flow::FridayWorkspaceArea::Canvas,
        flow::FridayWorkspaceArea::Artifacts,
        flow::FridayWorkspaceArea::Code,
        flow::FridayWorkspaceArea::Voice,
        flow::FridayWorkspaceArea::Multimodal,
        flow::FridayWorkspaceArea::Automations,
    ] {
        let route = plan.route(area).unwrap();
        assert_eq!(route.status, FridayUiIntegrationStatus::Wired);
        assert!(!route.data_bindings.is_empty());
        assert!(route.data_bindings.iter().all(|binding| binding.local_only));
    }

    let multimodal = plan.route(flow::FridayWorkspaceArea::Multimodal).unwrap();
    assert!(multimodal
        .primary_command
        .contains("--friday-multimodal-diagnostics"));
}

#[test]
fn friday_local_execution_checks_cover_low_resource_runtime_paths() {
    let report = default_friday_local_execution_checks();
    let ids = report
        .checks
        .iter()
        .map(|check| check.id.as_str())
        .collect::<std::collections::HashSet<_>>();

    assert!(ids.contains("stt-parakeet-artifacts"));
    assert!(ids.contains("tts-kokoro-artifacts"));
    assert!(ids.contains("ocr-glm-artifacts"));
    assert!(ids.contains("metasearch-request-path"));
    assert!(ids.contains("artifact-preview-records"));
    assert!(ids.contains("runtime-surface-records"));
    assert!(report.checks.iter().all(|check| check.local_only));
    assert!(report.checks.iter().all(|check| !check.loads_model));
    assert!(report.checks.iter().all(|check| !check.touches_network));
}

#[test]
fn friday_ocr_smoke_writes_artifact_record() {
    let root = temp_root("friday-ocr-smoke");
    let report = run_friday_ocr_smoke(&root, None, false).unwrap();

    assert_eq!(report.status, flow::FridayOcrSmokeStatus::Passed);
    assert!(!report.model_execution);
    assert!(PathBuf::from(&report.output_markdown).exists());
    assert!(PathBuf::from(&report.artifact_json).exists());
    assert!(PathBuf::from(&report.checkpoint_json).exists());
    assert!(PathBuf::from(&report.metadata_json).exists());
    assert!(PathBuf::from(&report.report_json).exists());
    assert_eq!(report.artifact.kind, flow::FridayArtifactKind::Markdown);
    assert_eq!(report.artifact.current_checkpoint_id, report.checkpoint.id);
    assert_eq!(report.checkpoint.artifact_id, report.artifact.id);
    assert_eq!(report.metadata.artifact_id, report.artifact.id);
    assert_eq!(report.metadata.request_kind, FridayMultimodalRequestKind::Ocr);
    assert_eq!(report.artifact.preview_runner, FridayPreviewRunner::Markdown);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_vlm_contract_writes_model_boundary_artifact() {
    let root = temp_root("friday-vlm-contract");
    let report = run_friday_vlm_contract(&root, None, None).unwrap();

    assert!(!report.model_execution);
    assert_eq!(report.model_key, "gemma4-e4b-frontend-q4km");
    assert_eq!(report.model_files.len(), 2);
    assert!(PathBuf::from(&report.output_markdown).exists());
    assert!(PathBuf::from(&report.artifact_json).exists());
    assert!(PathBuf::from(&report.checkpoint_json).exists());
    assert!(PathBuf::from(&report.metadata_json).exists());
    assert!(PathBuf::from(&report.report_json).exists());
    assert_eq!(report.artifact.kind, flow::FridayArtifactKind::Markdown);
    assert_eq!(report.artifact.current_checkpoint_id, report.checkpoint.id);
    assert_eq!(report.checkpoint.artifact_id, report.artifact.id);
    assert_eq!(report.metadata.artifact_id, report.artifact.id);
    assert_eq!(report.metadata.request_kind, FridayMultimodalRequestKind::Vlm);
    assert_eq!(report.artifact.preview_runner, FridayPreviewRunner::Markdown);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_multimodal_routes_keep_local_first_boundaries() {
    let ocr = friday_multimodal_route(FridayMultimodalRequestKind::Ocr, false);
    assert!(ocr.local_first);
    assert!(!ocr.remote_allowed);
    assert!(ocr.selected.unwrap().command.contains("--friday-ocr-smoke"));

    let audio = friday_multimodal_route(FridayMultimodalRequestKind::Audio, false);
    assert!(audio
        .fallbacks
        .iter()
        .any(|route| route.model_key == "kokoro-int8"));

    let image = friday_multimodal_route(FridayMultimodalRequestKind::Image, true);
    assert_eq!(image.status, FridayMultimodalRouteStatus::Planned);
    assert!(image.local_first);
    assert!(image.remote_allowed);
    assert!(image.selected.is_none());
}

#[test]
fn friday_multimodal_diagnostics_connect_ocr_and_vlm_outputs() {
    let diagnostics = friday_multimodal_ui_diagnostics();

    assert_eq!(diagnostics.area, flow::FridayWorkspaceArea::Multimodal);
    assert!(diagnostics.score_out_of_100 >= 60);
    assert!(diagnostics
        .items
        .iter()
        .any(|item| item.command.contains("--friday-ocr-smoke")
            && item.status == FridayMultimodalDiagnosticStatus::Ready));
    assert!(diagnostics
        .items
        .iter()
        .any(|item| item.command.contains("--friday-vlm-contract")));
    assert!(diagnostics
        .items
        .iter()
        .any(|item| item.artifact_output.contains("metadata")));
}

#[test]
fn friday_screenshot_vlm_handoff_accepts_local_image_file() {
    let root = temp_root("friday-screenshot-vlm");
    let screenshot = root.join("screen.png");
    fs::write(&screenshot, b"fixture").unwrap();
    let out = root.join("out");

    let report = run_friday_screenshot_vlm_handoff(&out, &screenshot, None).unwrap();

    assert_eq!(report.source.mime, "image/png");
    assert!(report.source.accepted);
    assert!(PathBuf::from(&report.source_json).exists());
    assert!(PathBuf::from(&report.vlm_report.report_json).exists());
    assert_eq!(
        report.vlm_report.metadata.request_kind,
        FridayMultimodalRequestKind::Vlm
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_media_affordances_include_image_and_video_paths() {
    let affordances = friday_media_affordances();

    assert!(affordances
        .iter()
        .any(|item| item.request_kind == FridayMultimodalRequestKind::Image
            && item.install_command.contains("--models image")));
    assert!(affordances
        .iter()
        .any(|item| item.request_kind == FridayMultimodalRequestKind::Video
            && item.run_command.contains("--plan video")
            && !item.resident));
}

#[test]
fn friday_multimodal_visual_check_targets_route_and_viewports() {
    let report = friday_multimodal_visual_check();

    assert_eq!(report.route, "/multimodal");
    assert_eq!(report.status, FridayUiVisualCheckStatus::Passed);
    assert_eq!(report.score_out_of_100, 100);
    assert_eq!(report.viewports.len(), 3);
    assert!(report.blocking_count() == 0);
    assert!(report
        .requirements
        .iter()
        .any(|requirement| requirement.id == "multimodal-diagnostic-cards"));
    assert!(report
        .requirements
        .iter()
        .any(|requirement| requirement.id == "multimodal-artifact-metadata"));
    assert!(report
        .requirements
        .iter()
        .all(|requirement| requirement.status == FridayUiVisualCheckStatus::Passed));
}

#[test]
fn friday_ui_routes_have_production_state_contracts() {
    let plan = default_friday_ui_integration_plan();

    for route in &plan.routes {
        assert_eq!(route.states.len(), 5);
        assert!(
            route
                .states
                .iter()
                .any(|state| state.kind == FridayUiStateKind::Ready
                    && state.tone == FridayUiStateTone::Success)
        );
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

#[test]
fn friday_browser_gate_verifies_tracked_extension_surface() {
    let report = default_friday_browser_verification_report();

    assert!(report
        .targets
        .iter()
        .any(|target| target.id == "flow-webext-source"));
    assert!(report
        .targets
        .iter()
        .any(|target| target.id == "flow-webext-chromium-dist"));
    assert!(report
        .targets
        .iter()
        .any(|target| target.id == "flow-webext-firefox-artifact"));
    assert!(report
        .targets
        .iter()
        .all(|target| target.status == FridayVerificationStatus::Passed));
    assert!(report.deploy_gate.deployment_allowed);
    assert!(report
        .deploy_gate
        .required_verification_command
        .contains("flow --friday-local-checks"));
}

#[test]
fn browser_extension_smoke_report_scores_packaged_targets() {
    let root = temp_root("browser-extension-smoke");
    for target in ["chromium", "firefox", "safari"] {
        write_extension_smoke_fixture(&root, target);
    }
    let probes = ["chrome", "edge", "firefox", "safari"]
        .into_iter()
        .map(|target_id| BrowserExtensionInstallProbe {
            target_id: target_id.to_string(),
            platform_supported: true,
            detected_executable: Some(format!("{target_id}-fixture")),
        })
        .collect::<Vec<_>>();

    let report = browser_extension_smoke_report_for_root(&root, &probes);

    assert_eq!(report.targets.len(), 4);
    assert_eq!(report.score_out_of_100, 100);
    assert_eq!(report.blocking_count(), 0);
    assert!(report.local_only);
    assert!(!report.touches_network);
    assert!(report
        .targets
        .iter()
        .all(|target| target.status == BrowserExtensionSmokeStatus::Passed));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn browser_extension_launch_smoke_plans_temporary_profiles() {
    let root = temp_root("browser-extension-launch-smoke");
    for target in ["chromium", "firefox", "safari"] {
        write_extension_smoke_fixture(&root, target);
    }
    let probes = ["chrome", "edge", "firefox", "safari"]
        .into_iter()
        .map(|target_id| BrowserExtensionInstallProbe {
            target_id: target_id.to_string(),
            platform_supported: true,
            detected_executable: Some(format!("{target_id}-fixture")),
        })
        .collect::<Vec<_>>();

    let report = browser_extension_launch_smoke_report_for_root(&root, &probes, false, 100);

    assert_eq!(report.targets.len(), 4);
    assert!(report.local_only);
    assert!(!report.touches_network);
    assert!(report
        .targets
        .iter()
        .filter(|target| target.extension_target == "chromium")
        .all(|target| target
            .command_preview
            .contains("--user-data-dir")));
    assert!(report
        .targets
        .iter()
        .filter(|target| target.extension_target == "chromium")
        .all(|target| target.profile_dir.is_some()));
    assert!(report
        .targets
        .iter()
        .any(|target| target.status == BrowserExtensionSmokeStatus::Warning));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn browser_pack_reuse_smoke_proves_offline_cached_routing() {
    let report = browser_pack_reuse_smoke_report();

    assert_eq!(report.score_out_of_100, 100);
    assert_eq!(report.blocking_count(), 0);
    assert!(report.local_only);
    assert!(!report.touches_network);
    assert!(report.targets.len() >= 3);
    assert!(report.targets.iter().all(|target| {
        target.status == BrowserPackReuseStatus::Passed
            && target.local_only
            && !target.remote_allowed
            && target.files_cached == target.files_total
            && target
                .files
                .iter()
                .all(|file| file.local_url.starts_with("https://flow.browserpack.local/"))
    }));
}

#[test]
fn browser_pack_recovery_smoke_covers_resume_hash_and_quota_paths() {
    let report = browser_pack_recovery_smoke_report();

    assert_eq!(report.score_out_of_100, 100);
    assert_eq!(report.blocking_count(), 0);
    assert!(report.local_only);
    assert!(!report.touches_network);
    assert!(report.targets.len() >= 3);
    assert!(report.targets.iter().all(|target| {
        target.status() == BrowserPackRecoveryStatus::Passed
            && target.available_bytes_before < target.required_bytes
            && target.available_bytes_after >= target.required_bytes
            && target.files.iter().all(|file| {
                file.valid_relative_path
                    && file.partial_bytes > 0
                    && file.partial_bytes < file.expected_bytes
                    && file.resumed_bytes + file.partial_bytes == file.expected_bytes
                    && file.observed_hash != file.expected_hash
            })
            && target.scenarios.iter().any(|scenario| {
                scenario.kind == BrowserPackRecoveryScenarioKind::PartialDownloadResume
                    && scenario.status == BrowserPackRecoveryStatus::Passed
            })
            && target.scenarios.iter().any(|scenario| {
                scenario.kind == BrowserPackRecoveryScenarioKind::HashMismatchRejection
                    && scenario.status == BrowserPackRecoveryStatus::Passed
            })
            && target.scenarios.iter().any(|scenario| {
                scenario.kind == BrowserPackRecoveryScenarioKind::QuotaPressureRecovery
                    && scenario.status == BrowserPackRecoveryStatus::Passed
            })
    }));
}

#[test]
fn browser_webllm_acceleration_gates_chromium_and_preserves_fallbacks() {
    let report = browser_webllm_acceleration_report();

    assert_eq!(report.score_out_of_100, 100);
    assert_eq!(report.blocking_count(), 0);
    assert!(report.local_only);
    assert!(!report.touches_network);
    assert!(!report.targets.is_empty());
    assert!(report.targets.iter().all(|target| {
        target.status == BrowserWebLlmAccelerationStatus::Passed
            && target.model_key == "qwen3-0.6b"
            && target.acceleration_backend == flow::BrowserExecutionBackend::WebLlmWorker
            && target.fallback_backend == flow::BrowserExecutionBackend::TransformersJsOnnx
            && target.host_flavor == flow::BrowserHostFlavor::ChromiumExtension
            && target.device_target == flow::BrowserDeviceTarget::WebGpu
            && target.worker_kind == flow::BrowserWorkerKind::DedicatedWorker
            && target.local_only
            && !target.remote_allowed
            && target
                .requirements
                .contains(&"explicit-user-opt-in".to_string())
    }));
    assert!(report.guardrails.iter().all(|guardrail| {
        !guardrail.acceleration_allowed
            && guardrail.fallback_backend == flow::BrowserExecutionBackend::TransformersJsOnnx
    }));
}

#[test]
fn typing_assistant_handles_snippets_dictionary_and_styles() {
    let assistant = FlowTypingAssistant::new();
    let result = assistant
        .process(TypingAssistRequest {
            text: "addr and supabase".to_string(),
            app_context: app_context(WritingDomain::Email),
            dictionary: vec![DictionaryEntry {
                surface: "supabase".to_string(),
                canonical: "Supabase".to_string(),
                case_sensitive: false,
                shared: true,
            }],
            snippets: vec![SnippetEntry {
                trigger: "addr".to_string(),
                expansion: "221B Baker Street".to_string(),
                shared: false,
                description: None,
            }],
            styles: vec![StylePreset {
                name: "email-professional".to_string(),
                domain: WritingDomain::Email,
                tone: ToneStyle::Professional,
                rules: Vec::new(),
            }],
            auto_correct: false,
            expand_snippets: true,
        })
        .unwrap();

    assert!(result.final_text.contains("221B Baker Street"));
    assert!(result.final_text.contains("Supabase"));
}

fn write_extension_smoke_fixture(root: &std::path::Path, target: &str) {
    let dist = root
        .join("extensions")
        .join("flow-webext")
        .join("dist")
        .join(target);
    for relative in [
        "manifest.json",
        "popup.html",
        "sidepanel.html",
        "sidebar.html",
        "options.html",
        "flow.css",
        "background/index.js",
        "content/index.js",
        "ui/popup.js",
        "ui/options.js",
    ] {
        let path = dist.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "fixture").unwrap();
    }
    if target == "chromium" {
        for relative in ["offscreen.html", "ui/offscreen.js"] {
            let path = dist.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "fixture").unwrap();
        }
    }

    let artifact = root
        .join("extensions")
        .join("flow-webext")
        .join("artifacts")
        .join(format!("flow-webext-{target}-v0.1.0.zip"));
    fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    fs::write(&artifact, "zip-fixture").unwrap();
    fs::write(artifact.with_extension("zip.sha256"), "sha256-fixture").unwrap();
}

#[test]
fn dictation_engine_cleans_fillers_and_tags_files() {
    let engine = FlowDictationEngine::new();
    let result = engine
        .process(DictationAssistRequest {
            transcript: "um please update main.rs and actually readme.md".to_string(),
            app_context: app_context(WritingDomain::Code),
            dictionary: Vec::new(),
            snippets: Vec::new(),
            styles: Vec::new(),
            remove_fillers: true,
            auto_punctuate: false,
            format_lists: false,
            tag_workspace_files: true,
        })
        .unwrap();

    assert!(result.cleaned_text.contains("readme.md"));
    assert!(!result.cleaned_text.contains("um"));
    assert!(!result.file_tags.is_empty());
}
