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
    FridayDashboardActionKind, FridayDashboardHostApprovalState, FridayDashboardHostCommandStatus,
    FridayDashboardPanelStatus, FridayDashboardProductUiSmokeStatus,
    FridayDashboardScreenshotStatus, FridayExecutionHandoffStatus, FridayLiveUiBindingStatus,
    FridayMultimodalDiagnosticStatus, FridayMultimodalRequestKind, FridayMultimodalRouteStatus,
    FridayMultimodalSurface, FridayOperatorReadinessStatus, FridayPermissionScope,
    FridayPreviewRunner, FridayReleaseCandidateArchiveEntry, FridayReleaseChecklistSignoffDecision,
    FridayReleaseCheckpointDecision, FridayReleaseCheckpointSignoffDecision,
    FridayReleaseCheckpointSignoffRequest, FridayReleaseDeploymentGateDecision,
    FridayReleaseDeploymentTarget, FridayReleaseEscalationGateOutcome,
    FridayReleaseEscalationOwnerResponse, FridayReleaseEvidenceEscalationLevel,
    FridayReleaseEvidenceSlaState, FridayReleaseIncidentOutcome, FridayReleaseIncidentSeverity,
    FridayReleaseOwnerFollowUpCompletionState, FridayReleasePreventionActionKind,
    FridayReleasePreventionFindingKind, FridayReleasePromotionDecision,
    FridayReleasePromotionRecordRequest, FridayReleaseQaCheckStatus, FridayResearchWorkflow,
    FridayRouteVisualStatus, FridayRuntimeSurfaceStore, FridayTrustedHostCommandExecutor,
    FridayTrustedHostCommandRawOutput, FridayTrustedHostLiveRunnerRecord,
    FridayTrustedHostLiveRunnerStatus, FridayTrustedHostRunnerCancellationToken,
    FridayTrustedHostRunnerOperatorReviewFilter, FridayTrustedHostRunnerRequest,
    FridayTrustedHostRunnerStatus, FridayUiIntegrationStatus, FridayUiStateKind, FridayUiStateTone,
    FridayUiVisualCheckStatus, FridayVerificationStatus, FridayWorkspaceStore,
    append_friday_release_candidate_to_archive, append_friday_release_checkpoint_signoff_to_ledger,
    append_friday_release_escalation_to_ledger, append_friday_release_incident_to_archive,
    append_friday_release_operator_signoff, append_friday_release_promotion_to_ledger,
    append_friday_trusted_host_runner_history,
    append_friday_trusted_runner_release_package_to_timeline,
    default_friday_browser_verification_report, default_friday_local_execution_checks,
    default_friday_product_plan, default_friday_ui_integration_plan,
    export_friday_dashboard_bundle, friday_dashboard_export_history_from_export,
    friday_dashboard_host_command_bridge_from_export,
    friday_dashboard_host_command_record_from_action, friday_dashboard_panel_from_export,
    friday_dashboard_product_ui_binding_from_export, friday_dashboard_product_ui_smoke_from_export,
    friday_dashboard_release_review_from_export, friday_dashboard_screenshot_history,
    friday_execution_handoff_report, friday_live_ui_route_binding_report, friday_media_affordances,
    friday_multimodal_route, friday_multimodal_ui_diagnostics, friday_multimodal_visual_check,
    friday_operator_readiness_report, friday_release_candidate_archive_report,
    friday_release_candidate_entry_from_gate, friday_release_checkpoint_review_board_report,
    friday_release_deployment_gate_report, friday_release_evidence_export_kit_report,
    friday_release_evidence_sla_monitor_report_at, friday_release_incident_archive_report,
    friday_release_incident_entry_from_sources, friday_release_operator_checklist_report,
    friday_release_owner_followup_board_report_at, friday_release_post_promotion_monitor_report,
    friday_release_prevention_plan_report, friday_release_qa_command_center_report,
    friday_release_recovery_runbook_report, friday_release_rollback_drill_report,
    friday_release_stability_board_report, friday_route_visual_report,
    friday_route_visual_report_for_root, friday_trusted_host_live_runner_state_from_history,
    friday_trusted_host_runner_approval_ui_report,
    friday_trusted_host_runner_cancellation_ux_report,
    friday_trusted_host_runner_operator_review_report, friday_trusted_host_runner_ux_report,
    friday_trusted_runner_release_package_report, friday_trusted_runner_release_timeline_report,
    read_friday_trusted_host_live_runner_state, read_friday_trusted_host_runner_history,
    refresh_friday_trusted_host_live_runner_state, run_friday_ocr_smoke,
    run_friday_screenshot_vlm_handoff, run_friday_trusted_host_command_bridge_with_executor,
    run_friday_trusted_host_command_with_executor, run_friday_vlm_contract,
    write_friday_release_checkpoint_review_board_report, write_friday_release_deployment_gate,
    write_friday_release_evidence_export_kit, write_friday_release_evidence_sla_monitor_report,
    write_friday_release_operator_checklist, write_friday_release_owner_followup_board_report,
    write_friday_release_post_promotion_monitor_report,
    write_friday_release_prevention_plan_report, write_friday_release_qa_command_center_report,
    write_friday_release_recovery_runbook_report, write_friday_release_rollback_drill_report,
    write_friday_release_stability_board_report, write_friday_trusted_host_live_runner_state,
    write_friday_trusted_runner_release_package, write_friday_trusted_runner_release_timeline,
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
    assert!(
        multimodal
            .primary_command
            .contains("--friday-multimodal-diagnostics")
    );
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
    assert_eq!(
        report.metadata.request_kind,
        FridayMultimodalRequestKind::Ocr
    );
    assert_eq!(
        report.artifact.preview_runner,
        FridayPreviewRunner::Markdown
    );

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
    assert_eq!(
        report.metadata.request_kind,
        FridayMultimodalRequestKind::Vlm
    );
    assert_eq!(
        report.artifact.preview_runner,
        FridayPreviewRunner::Markdown
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_multimodal_routes_keep_local_first_boundaries() {
    let ocr = friday_multimodal_route(FridayMultimodalRequestKind::Ocr, false);
    assert!(ocr.local_first);
    assert!(!ocr.remote_allowed);
    assert!(ocr.selected.unwrap().command.contains("--friday-ocr-smoke"));

    let audio = friday_multimodal_route(FridayMultimodalRequestKind::Audio, false);
    assert!(
        audio
            .fallbacks
            .iter()
            .any(|route| route.model_key == "kokoro-int8")
    );

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
    assert!(
        diagnostics
            .items
            .iter()
            .any(|item| item.command.contains("--friday-ocr-smoke")
                && item.status == FridayMultimodalDiagnosticStatus::Ready)
    );
    assert!(
        diagnostics
            .items
            .iter()
            .any(|item| item.command.contains("--friday-vlm-contract"))
    );
    assert!(
        diagnostics
            .items
            .iter()
            .any(|item| item.artifact_output.contains("metadata"))
    );
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

    assert!(affordances.iter().any(
        |item| item.request_kind == FridayMultimodalRequestKind::Image
            && item.install_command.contains("--models image")
    ));
    assert!(affordances.iter().any(
        |item| item.request_kind == FridayMultimodalRequestKind::Video
            && item.run_command.contains("--plan video")
            && !item.resident
    ));
}

#[test]
fn friday_multimodal_visual_check_targets_route_and_viewports() {
    let report = friday_multimodal_visual_check();

    assert_eq!(report.route, "/multimodal");
    assert_eq!(report.status, FridayUiVisualCheckStatus::Passed);
    assert_eq!(report.score_out_of_100, 100);
    assert_eq!(report.viewports.len(), 3);
    assert!(report.blocking_count() == 0);
    assert!(
        report
            .requirements
            .iter()
            .any(|requirement| requirement.id == "multimodal-diagnostic-cards")
    );
    assert!(
        report
            .requirements
            .iter()
            .any(|requirement| requirement.id == "multimodal-artifact-metadata")
    );
    assert!(
        report
            .requirements
            .iter()
            .all(|requirement| requirement.status == FridayUiVisualCheckStatus::Passed)
    );
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

    assert!(
        report
            .targets
            .iter()
            .any(|target| target.id == "flow-webext-source")
    );
    assert!(
        report
            .targets
            .iter()
            .any(|target| target.id == "flow-webext-chromium-dist")
    );
    assert!(
        report
            .targets
            .iter()
            .any(|target| target.id == "flow-webext-firefox-artifact")
    );
    assert!(
        report
            .targets
            .iter()
            .all(|target| target.status == FridayVerificationStatus::Passed)
    );
    assert!(report.deploy_gate.deployment_allowed);
    assert!(
        report
            .deploy_gate
            .required_verification_command
            .contains("flow --friday-local-checks")
    );
}

#[test]
fn friday_live_ui_routes_bind_contracts_to_tracked_files() {
    let report = friday_live_ui_route_binding_report();

    assert_eq!(report.score_out_of_100, 100);
    assert_eq!(report.blocking_count, 0);
    assert_eq!(
        report.route_count,
        default_friday_ui_integration_plan().routes.len()
    );
    assert!(report.routes.iter().all(|route| {
        route.status == FridayLiveUiBindingStatus::Passed
            && !route.source_files.is_empty()
            && route
                .source_files
                .iter()
                .all(|file| file.exists && file.bytes > 0)
    }));
    assert!(report.routes.iter().any(|route| {
        route.route == "/voice"
            && route
                .source_files
                .iter()
                .any(|file| file.path == "src/bin/flow-dictate.rs")
    }));
    assert!(report.routes.iter().any(|route| {
        route.route == "/multimodal"
            && route
                .source_files
                .iter()
                .any(|file| file.path.contains("transformers-runtime.ts"))
    }));
}

#[test]
fn friday_operator_readiness_rolls_up_live_surfaces() {
    let report = friday_operator_readiness_report();

    assert!(report.score_out_of_100 >= 70);
    assert!(report.items.iter().any(|item| {
        item.id == "route-bindings" && item.status == FridayOperatorReadinessStatus::Passed
    }));
    assert!(report.items.iter().any(|item| {
        item.id == "local-execution" && item.command == "flow --friday-local-checks"
    }));
    assert!(
        report.items.iter().any(|item| {
            item.id == "browser-gate" && item.command == "flow --friday-browser-gate"
        })
    );
    assert!(report.items.iter().any(|item| {
        item.id == "desktop-host"
            && item
                .evidence
                .iter()
                .any(|evidence| evidence.contains("src/bin/flow-dictate.rs=present"))
    }));
    assert!(report.items.iter().all(|item| item.local_only));
}

#[test]
fn friday_route_visuals_cover_most_used_routes() {
    let report = friday_route_visual_report();

    assert_eq!(report.score_out_of_100, 100);
    assert_eq!(report.target_count, 10);
    assert_eq!(report.blocking_count, 0);
    assert!(report.targets.iter().all(|target| {
        target.status == FridayRouteVisualStatus::Passed
            && target.screenshot_path.ends_with(".png")
            && target.metadata_path.ends_with(".json")
            && target.capture_command.contains("agent-browser screenshot")
    }));
    for route in ["/ask", "/search", "/research", "/voice", "/multimodal"] {
        assert!(report.targets.iter().any(|target| target.route == route));
    }
    assert!(report.targets.iter().any(|target| {
        target.route == "/voice"
            && target.source_file == "extensions/flow-webext/src/content/index.ts"
    }));
    assert!(report.targets.iter().any(|target| {
        target.route == "/multimodal" && target.source_file.contains("transformers-runtime.ts")
    }));
}

#[test]
fn friday_execution_handoffs_bind_ui_actions_to_local_commands() {
    let report = friday_execution_handoff_report();

    assert_eq!(report.score_out_of_100, 100);
    assert_eq!(report.handoff_count, 6);
    assert_eq!(report.blocking_count, 0);
    assert!(report.handoffs.iter().all(|handoff| {
        handoff.status == FridayExecutionHandoffStatus::Passed
            && handoff.local_only
            && !handoff.permission_scopes.is_empty()
            && !handoff.recovery_command.trim().is_empty()
            && (handoff.source_file.ends_with(".rs") || handoff.source_file.ends_with(".ts"))
    }));
    assert!(report.handoffs.iter().any(|handoff| {
        handoff.id == "voice-dictation"
            && handoff.command == "flow --dictate"
            && handoff.requires_user_gesture
            && handoff
                .permission_scopes
                .contains(&"microphone".to_string())
    }));
    assert!(report.handoffs.iter().any(|handoff| {
        handoff.id == "research-report"
            && handoff
                .artifact_path
                .as_deref()
                .unwrap_or_default()
                .ends_with("manifest.json")
    }));
    assert!(
        report
            .handoffs
            .iter()
            .any(|handoff| handoff.id == "readiness-command" && !handoff.requires_user_gesture)
    );
}

#[test]
fn friday_dashboard_export_writes_dashboard_bundle() {
    let root = temp_root("friday-dashboard-export");
    let bundle = export_friday_dashboard_bundle(&root).unwrap();

    assert_eq!(
        bundle.completion.name,
        "Friday Trusted Runner Operator Review"
    );
    assert_eq!(bundle.completion.current_score_out_of_100, 100);
    assert_eq!(bundle.manifest.score_out_of_100, 100);
    assert_eq!(bundle.export_history.record_count, 1);
    assert_eq!(
        bundle.release_review.loop_name,
        "Friday Trusted Runner Operator Review"
    );
    assert!(PathBuf::from(&bundle.manifest.dashboard_history_json).exists());
    assert!(PathBuf::from(&bundle.manifest.release_review_json).exists());
    assert_eq!(bundle.readiness.blocking_count, 0);
    assert_eq!(bundle.route_bindings.blocking_count, 0);
    assert_eq!(bundle.route_visuals.blocking_count, 0);
    assert_eq!(bundle.execution_handoffs.blocking_count, 0);
    assert!(PathBuf::from(&bundle.manifest.manifest_json).exists());
    assert!(PathBuf::from(&bundle.manifest.readiness_json).exists());
    assert!(PathBuf::from(&bundle.manifest.route_bindings_json).exists());
    assert!(PathBuf::from(&bundle.manifest.route_visuals_json).exists());
    assert!(PathBuf::from(&bundle.manifest.execution_handoffs_json).exists());
    assert!(PathBuf::from(&bundle.manifest.completion_json).exists());
    assert!(PathBuf::from(&bundle.manifest.dashboard_index_json).exists());
    assert!(PathBuf::from(&bundle.manifest.summary_markdown).exists());
    assert!(bundle.manifest.files.iter().any(|file| {
        file.path.ends_with("readiness.json") && file.kind == "operator-readiness" && file.bytes > 0
    }));
    assert!(bundle.manifest.files.iter().any(|file| {
        file.path.ends_with("dashboard-history.json")
            && file.kind == "dashboard-history"
            && file.bytes > 0
    }));
    assert!(bundle.manifest.files.iter().any(|file| {
        file.path.ends_with("release-review.json")
            && file.kind == "release-review"
            && file.bytes > 0
    }));
    assert!(
        bundle
            .manifest
            .commands
            .iter()
            .any(|command| command.contains("--friday-dashboard-export"))
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_dashboard_panel_consumes_exported_bundle() {
    let root = temp_root("friday-dashboard-panel");
    export_friday_dashboard_bundle(&root).unwrap();
    let panel = friday_dashboard_panel_from_export(&root).unwrap();

    assert_eq!(panel.loop_name, "Friday Trusted Runner Operator Review");
    assert_eq!(panel.score_out_of_100, 100);
    assert_eq!(panel.status, FridayDashboardPanelStatus::Warning);
    assert_eq!(panel.cards.len(), 8);
    assert!(panel.cards.iter().any(|card| {
        card.id == "completion-loop"
            && card.source_json.ends_with("completion.json")
            && card.actions.iter().any(|action| {
                action.kind == FridayDashboardActionKind::Open
                    && action.command == "flow --completion"
                    && action.source == "completion-loop"
            })
    }));
    assert!(panel.cards.iter().any(|card| {
        card.id == "operator-readiness"
            && card.primary_metric.contains("passed")
            && card.actions.iter().any(|action| {
                action.kind == FridayDashboardActionKind::RunCheck
                    && action.command == "flow --friday-readiness"
            })
            && card.actions.iter().any(|action| {
                action.kind == FridayDashboardActionKind::Recover
                    && action.command == "flow --friday-media-affordances"
            })
    }));
    assert!(panel.cards.iter().any(|card| {
        card.id == "screenshot-history"
            && card.primary_metric.contains("captures present")
            && card.actions.iter().any(|action| {
                action.kind == FridayDashboardActionKind::Capture
                    && action.command == "flow --friday-route-visuals"
            })
    }));
    assert!(panel.cards.iter().any(|card| {
        card.id == "export-history"
            && card.source_json.ends_with("dashboard-history.json")
            && card.actions.iter().any(|action| {
                action.kind == FridayDashboardActionKind::RunCheck
                    && action.command.contains("--friday-dashboard-export")
            })
    }));
    assert!(panel.cards.iter().any(|card| {
        card.id == "release-review"
            && card.source_json.ends_with("release-review.json")
            && card.actions.iter().any(|action| {
                action.kind == FridayDashboardActionKind::Open
                    && action.command.contains("--friday-dashboard-panel")
            })
    }));
    assert!(
        panel
            .cards
            .iter()
            .flat_map(|card| &card.actions)
            .all(|action| {
                action.local_only
                    && action.enabled
                    && !action.destructive
                    && !action.requires_confirmation
            })
    );
    assert_eq!(panel.screenshot_history.total_targets, 10);
    assert_eq!(panel.export_history.record_count, 1);
    assert_eq!(panel.export_history.score_delta_from_previous, 0);
    assert!(
        panel
            .release_review
            .links
            .iter()
            .any(|link| link.path == "TODO.md")
    );
    assert!(panel.screenshot_history.missing_count > 0);
    assert!(panel.screenshot_history.records.iter().any(|record| {
        record.status == FridayDashboardScreenshotStatus::Missing
            && record.prompt.contains("Capture the")
            && record.capture_command.contains("agent-browser screenshot")
    }));
    assert!(panel.source_files.iter().any(|file| {
        file.path.ends_with("route-visuals.json") && file.kind == "route-visuals" && file.bytes > 0
    }));
    assert!(panel.warnings.iter().any(|warning| {
        warning.contains("Completion Loop") || warning.contains("Operator Readiness")
    }));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_dashboard_export_history_tracks_checkpoints() {
    let root = temp_root("friday-dashboard-export-history");
    export_friday_dashboard_bundle(&root).unwrap();
    export_friday_dashboard_bundle(&root).unwrap();

    let history = friday_dashboard_export_history_from_export(&root).unwrap();
    assert_eq!(history.record_count, 2);
    assert!(history.latest.is_some());
    assert!(history.previous.is_some());
    assert_eq!(history.score_delta_from_previous, 0);
    assert_eq!(history.readiness_delta_from_previous, 0);
    assert!(history.records.iter().all(|record| {
        record.loop_name == "Friday Trusted Runner Operator Review"
            && record.manifest_json.ends_with("manifest.json")
    }));

    let panel = friday_dashboard_panel_from_export(&root).unwrap();
    assert_eq!(panel.export_history.record_count, 2);
    assert!(panel.cards.iter().any(|card| {
        card.id == "export-history" && card.status == FridayDashboardPanelStatus::Ready
    }));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_dashboard_release_review_links_release_artifacts() {
    let root = temp_root("friday-dashboard-release-review");
    export_friday_dashboard_bundle(&root).unwrap();

    let review = friday_dashboard_release_review_from_export(&root).unwrap();
    assert_eq!(review.loop_name, "Friday Trusted Runner Operator Review");
    assert_eq!(review.score_out_of_100, 100);
    assert!(review.total_count >= 6);
    assert!(
        review
            .checklist
            .iter()
            .any(|item| item.id == "completion-loop" && item.ready)
    );
    assert!(
        review
            .links
            .iter()
            .any(|link| link.id == "changelog" && link.path == "CHANGELOG.md")
    );
    assert!(
        review
            .links
            .iter()
            .any(|link| link.id == "route-visuals" && link.path.ends_with("route-visuals.json"))
    );
    assert!(
        review
            .commands
            .iter()
            .any(|command| command.contains("--friday-dashboard-panel"))
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_dashboard_product_ui_binding_maps_panel_json_to_route() {
    let root = temp_root("friday-dashboard-product-ui-binding");
    export_friday_dashboard_bundle(&root).unwrap();

    let binding = friday_dashboard_product_ui_binding_from_export(&root).unwrap();
    assert_eq!(binding.product_name, "Friday");
    assert_eq!(binding.route, "/dashboard");
    assert_eq!(binding.score_out_of_100, 100);
    assert_eq!(binding.card_count, 8);
    assert_eq!(binding.bound_card_count, 8);
    assert!(
        binding
            .panel_json_command
            .contains("--friday-dashboard-panel-json")
    );
    assert!(binding.export_command.contains("--friday-dashboard-export"));
    assert!(binding.data_bindings.iter().any(|data_binding| {
        data_binding.id == "dashboard-panel-json"
            && data_binding.writes_to == "Friday dashboard route state"
            && data_binding.local_only
    }));
    assert!(
        binding
            .cards
            .iter()
            .any(|card| card.card_id == "release-review")
    );
    assert_eq!(binding.history.record_count, 1);
    assert_eq!(binding.history.trend_label, "not-enough-history");
    assert!(
        binding
            .screenshot_prompts
            .iter()
            .any(|prompt| prompt.capture_command.contains("agent-browser screenshot"))
    );
    assert!(binding.release_links.iter().any(|link| {
        link.id == "changelog"
            && link.section == "release-notes"
            && link.path == "CHANGELOG.md"
            && !link.button_state.disabled
            && link
                .button_state
                .aria_label
                .contains("Open dashboard action")
    }));
    assert!(binding.release_links.iter().any(|link| {
        link.id == "route-visuals"
            && link.section == "visual-review"
            && link.path.ends_with("route-visuals.json")
    }));
    assert!(binding.release_links.iter().any(|link| {
        link.id == "dashboard-history"
            && link.section == "export-artifacts"
            && link.path.ends_with("dashboard-history.json")
    }));
    assert!(binding.action_bindings.iter().any(|action| {
        action.card_id == "operator-readiness"
            && action.kind == FridayDashboardActionKind::Recover
            && action.command == "flow --friday-media-affordances"
            && !action.button_state.disabled
            && action.button_state.loading_label == "Recovering..."
            && action.button_state.error_label == "Recover failed"
    }));
    assert!(binding.data_bindings.iter().any(|data_binding| {
        data_binding.id == "dashboard-host-command-bridge"
            && data_binding
                .command
                .contains("--friday-dashboard-host-bridge-json")
            && data_binding.local_only
    }));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_dashboard_host_command_bridge_requires_approval_and_blocks_unsafe_commands() {
    let root = temp_root("friday-dashboard-host-command-bridge");
    export_friday_dashboard_bundle(&root).unwrap();

    let report = friday_dashboard_host_command_bridge_from_export(&root).unwrap();
    assert_eq!(report.product_name, "Friday");
    assert_eq!(report.route, "/dashboard");
    assert_eq!(report.command_count, 9);
    assert_eq!(report.awaiting_approval_count, 9);
    assert_eq!(report.blocked_count, 0);
    assert_eq!(report.audit_count, report.command_count);
    assert!(
        report
            .source_command
            .contains("--friday-dashboard-product-ui-json")
    );
    assert!(report.records.iter().all(|record| {
        record.status == FridayDashboardHostCommandStatus::AwaitingApproval
            && record.approval_state == FridayDashboardHostApprovalState::Required
            && !record.silent_execution_allowed
            && record.can_execute_after_approval
            && record.audit.duration_ms == 0
            && record.audit.stdout_summary.contains("not executed")
    }));

    let binding = friday_dashboard_product_ui_binding_from_export(&root).unwrap();
    let action = binding.action_bindings[0].clone();

    let mut remote = action.clone();
    remote.local_only = false;
    let remote_record = friday_dashboard_host_command_record_from_action(&remote, 1);
    assert_eq!(
        remote_record.status,
        FridayDashboardHostCommandStatus::Blocked
    );
    assert!(remote_record.blocked_reason.unwrap().contains("Remote"));

    let mut destructive = action.clone();
    destructive.button_state.destructive = true;
    let destructive_record = friday_dashboard_host_command_record_from_action(&destructive, 2);
    assert_eq!(
        destructive_record.approval_state,
        FridayDashboardHostApprovalState::Blocked
    );
    assert!(
        destructive_record
            .blocked_reason
            .unwrap()
            .contains("Destructive")
    );

    let mut disabled = action.clone();
    disabled.enabled = false;
    disabled.button_state.disabled = true;
    disabled.button_state.disabled_reason = Some("not ready".to_string());
    let disabled_record = friday_dashboard_host_command_record_from_action(&disabled, 3);
    assert_eq!(
        disabled_record.status,
        FridayDashboardHostCommandStatus::Blocked
    );
    assert_eq!(disabled_record.blocked_reason.as_deref(), Some("not ready"));

    let mut malformed = action;
    malformed.command.clear();
    let malformed_record = friday_dashboard_host_command_record_from_action(&malformed, 4);
    assert_eq!(
        malformed_record.status,
        FridayDashboardHostCommandStatus::Blocked
    );
    assert!(malformed_record.blocked_reason.unwrap().contains("empty"));

    let _ = fs::remove_dir_all(&root);
}

#[derive(Debug, Clone)]
struct StubTrustedHostExecutor {
    output: Result<FridayTrustedHostCommandRawOutput, String>,
}

impl FridayTrustedHostCommandExecutor for StubTrustedHostExecutor {
    fn execute(
        &self,
        _command: &str,
        _timeout_ms: u64,
    ) -> Result<FridayTrustedHostCommandRawOutput, String> {
        self.output.clone()
    }
}

#[test]
fn friday_dashboard_trusted_host_runner_executes_only_approved_bounded_commands() {
    let root = temp_root("friday-trusted-host-runner");
    export_friday_dashboard_bundle(&root).unwrap();
    let bridge = friday_dashboard_host_command_bridge_from_export(&root).unwrap();
    let record = bridge.records[0].clone();
    let approved = FridayTrustedHostRunnerRequest {
        approved: true,
        timeout_ms: 25,
        stdout_limit_bytes: 8,
        stderr_limit_bytes: 8,
        operator_reason: Some("operator approved dashboard readiness retry".to_string()),
        ..Default::default()
    };
    let success = run_friday_trusted_host_command_with_executor(
        &record,
        &approved,
        &StubTrustedHostExecutor {
            output: Ok(FridayTrustedHostCommandRawOutput {
                exit_code: Some(0),
                stdout: "hello from trusted runner".to_string(),
                stderr: String::new(),
                duration_ms: 4,
                timed_out: false,
            }),
        },
    );
    assert_eq!(success.status, FridayTrustedHostRunnerStatus::Succeeded);
    assert_eq!(success.exit_code, Some(0));
    assert!(success.stdout_truncated);
    assert_eq!(success.timeout_ms, 25);
    assert_eq!(
        success.operator_reason.as_deref(),
        Some("operator approved dashboard readiness retry")
    );

    let denied = run_friday_trusted_host_command_with_executor(
        &record,
        &FridayTrustedHostRunnerRequest {
            operator_reason: Some("not safe yet".to_string()),
            ..Default::default()
        },
        &StubTrustedHostExecutor {
            output: Err("executor should not run".to_string()),
        },
    );
    assert_eq!(denied.status, FridayTrustedHostRunnerStatus::Denied);
    assert!(denied.stderr_summary.contains("approval"));
    assert_eq!(denied.operator_reason.as_deref(), Some("not safe yet"));

    let cancelled = run_friday_trusted_host_command_with_executor(
        &record,
        &FridayTrustedHostRunnerRequest {
            approved: true,
            cancel_requested: true,
            ..Default::default()
        },
        &StubTrustedHostExecutor {
            output: Err("executor should not run".to_string()),
        },
    );
    assert_eq!(cancelled.status, FridayTrustedHostRunnerStatus::Cancelled);
    assert!(cancelled.cancelled);

    let timed_out = run_friday_trusted_host_command_with_executor(
        &record,
        &approved,
        &StubTrustedHostExecutor {
            output: Ok(FridayTrustedHostCommandRawOutput {
                exit_code: None,
                stdout: String::new(),
                stderr: "late".to_string(),
                duration_ms: 25,
                timed_out: true,
            }),
        },
    );
    assert_eq!(timed_out.status, FridayTrustedHostRunnerStatus::TimedOut);

    let failed = run_friday_trusted_host_command_with_executor(
        &record,
        &approved,
        &StubTrustedHostExecutor {
            output: Ok(FridayTrustedHostCommandRawOutput {
                exit_code: Some(2),
                stdout: String::new(),
                stderr: "boom".to_string(),
                duration_ms: 6,
                timed_out: false,
            }),
        },
    );
    assert_eq!(failed.status, FridayTrustedHostRunnerStatus::Failed);

    let mut remote = record.clone();
    remote.local_only = false;
    let remote_denied = run_friday_trusted_host_command_with_executor(
        &remote,
        &approved,
        &StubTrustedHostExecutor {
            output: Err("executor should not run".to_string()),
        },
    );
    assert_eq!(remote_denied.status, FridayTrustedHostRunnerStatus::Denied);
    assert!(remote_denied.stderr_summary.contains("Remote"));

    let mut malformed = record;
    malformed.command = "flow --completion; whoami".to_string();
    let roomy_approved = FridayTrustedHostRunnerRequest {
        approved: true,
        timeout_ms: 25,
        stdout_limit_bytes: 128,
        stderr_limit_bytes: 128,
        ..Default::default()
    };
    let malformed_denied = run_friday_trusted_host_command_with_executor(
        &malformed,
        &roomy_approved,
        &StubTrustedHostExecutor {
            output: Err("executor should not run".to_string()),
        },
    );
    assert_eq!(
        malformed_denied.status,
        FridayTrustedHostRunnerStatus::Denied
    );
    assert!(malformed_denied.stderr_summary.contains("metacharacters"));

    let history_path = root.join("trusted-host-runner-history.json");
    append_friday_trusted_host_runner_history(&history_path, success.clone()).unwrap();
    append_friday_trusted_host_runner_history(&history_path, denied.clone()).unwrap();
    append_friday_trusted_host_runner_history(&history_path, cancelled.clone()).unwrap();
    append_friday_trusted_host_runner_history(&history_path, timed_out.clone()).unwrap();
    let history = append_friday_trusted_host_runner_history(&history_path, failed.clone()).unwrap();
    assert_eq!(history.result_count, 5);
    assert!(history.latest.is_some());
    let loaded = read_friday_trusted_host_runner_history(&history_path).unwrap();
    assert_eq!(loaded.result_count, 5);
    let ux = friday_trusted_host_runner_ux_report(&loaded, root.join("release-review.json"));
    assert_eq!(ux.result_count, 5);
    assert!(ux.status_summaries.iter().all(|summary| summary.count == 1));
    assert!(ux.status_summaries.iter().any(|summary| {
        summary.status == FridayTrustedHostRunnerStatus::TimedOut
            && summary.description.contains("timeout")
    }));
    assert!(ux.status_summaries.iter().any(|summary| {
        summary.status == FridayTrustedHostRunnerStatus::Cancelled
            && summary.description.contains("cancelled")
    }));
    assert!(ux.affordances.iter().any(|affordance| {
        affordance.kind == "retry" && affordance.requires_approval && !affordance.command.is_empty()
    }));
    assert!(
        ux.affordances
            .iter()
            .any(|affordance| { affordance.kind == "copy-command" && !affordance.disabled })
    );
    assert!(
        ux.operator_notes
            .iter()
            .any(|note| note.release_review_path.ends_with("release-review.json"))
    );
    let operator_review = friday_trusted_host_runner_operator_review_report(
        &loaded,
        FridayTrustedHostRunnerOperatorReviewFilter::default(),
    );
    assert_eq!(operator_review.record_count, 5);
    assert_eq!(operator_review.matched_count, 5);
    assert_eq!(operator_review.ready_count, 1);
    assert_eq!(operator_review.blocked_count, 3);
    assert_eq!(operator_review.release_gate_status, "blocked");
    assert!(
        operator_review
            .release_gate_summaries
            .iter()
            .any(|summary| {
                summary.id == "failed" && summary.severity == "blocked" && summary.count == 1
            })
    );
    assert!(
        operator_review
            .release_gate_summaries
            .iter()
            .any(|summary| summary.id == "stale-live-state")
    );
    assert!(operator_review.incident_notes.iter().any(|note| {
        note.status == FridayTrustedHostRunnerStatus::Failed
            && note.export_markdown.contains("### Failed")
            && note.export_markdown.contains("Stderr")
    }));
    let failed_review = friday_trusted_host_runner_operator_review_report(
        &loaded,
        FridayTrustedHostRunnerOperatorReviewFilter {
            status: Some(FridayTrustedHostRunnerStatus::Failed),
            action_id: Some(success.action_id.clone()),
            limit: 10,
            ..Default::default()
        },
    );
    assert_eq!(failed_review.matched_count, 1);
    assert_eq!(
        failed_review.records[0].status,
        FridayTrustedHostRunnerStatus::Failed
    );
    let empty_review = friday_trusted_host_runner_operator_review_report(
        &loaded,
        FridayTrustedHostRunnerOperatorReviewFilter {
            since_unix_ms: Some(u128::MAX),
            limit: 10,
            ..Default::default()
        },
    );
    assert_eq!(empty_review.release_gate_status, "empty");
    let approval_ui =
        friday_trusted_host_runner_approval_ui_report(&loaded, root.join("release-review.json"));
    assert_eq!(approval_ui.modal_id, "trusted-runner-approval");
    assert!(approval_ui.audit_reason_required);
    assert!(approval_ui.controls.iter().any(|control| {
        control.kind == "approve"
            && control.requires_reason
            && control.requires_approval
            && control
                .keyboard_shortcut
                .as_ref()
                .is_some_and(|shortcut| shortcut.key == "Ctrl+Enter")
    }));
    assert!(approval_ui.controls.iter().any(|control| {
        control.kind == "deny"
            && control.requires_reason
            && !control.requires_approval
            && control.command.contains("--reason")
    }));
    assert!(
        approval_ui
            .controls
            .iter()
            .any(|control| control.kind == "snooze")
    );
    assert!(
        approval_ui
            .controls
            .iter()
            .any(|control| control.kind == "undo")
    );
    assert!(
        approval_ui
            .release_review_path
            .ends_with("release-review.json")
    );
    let live_state_path = root.join("trusted-host-live-state.json");
    let live_from_history =
        friday_trusted_host_live_runner_state_from_history(&loaded, &live_state_path);
    assert_eq!(live_from_history.record_count, 5);
    assert_eq!(live_from_history.finished_count, 5);
    assert_eq!(live_from_history.pending_count, 0);
    assert_eq!(live_from_history.running_count, 0);
    assert!(live_from_history.records.iter().any(|record| {
        record.status == FridayTrustedHostLiveRunnerStatus::TimedOut
            && record.recovery_command.contains("--cancel")
            && record
                .cleanup_command
                .contains("--friday-trusted-host-live-state")
    }));
    let history_cancellation_ux =
        friday_trusted_host_runner_cancellation_ux_report(&live_from_history);
    assert_eq!(history_cancellation_ux.denial_count, 1);
    assert!(history_cancellation_ux.controls.iter().any(|control| {
        control.kind == "denial-recovery"
            && control.requires_reason
            && control.command.contains("<denial recovery reason>")
    }));

    let pending = FridayTrustedHostLiveRunnerRecord {
        job_id: "pending-job".to_string(),
        action_id: "open-completion".to_string(),
        label: "Open completion".to_string(),
        command: "flow --completion".to_string(),
        status: FridayTrustedHostLiveRunnerStatus::Pending,
        message: "Waiting for approval.".to_string(),
        local_only: true,
        approved: false,
        timeout_ms: 30_000,
        stale_after_ms: u128::MAX,
        created_at_unix_ms: 1,
        updated_at_unix_ms: 1,
        finished_at_unix_ms: None,
        history_json: None,
        recovery_command: "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id open-completion --cancel".to_string(),
        cleanup_command: "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json".to_string(),
    };
    let running = FridayTrustedHostLiveRunnerRecord {
        job_id: "running-job".to_string(),
        action_id: "readiness".to_string(),
        label: "Readiness".to_string(),
        command: "flow --friday-readiness".to_string(),
        status: FridayTrustedHostLiveRunnerStatus::Running,
        message: "Running.".to_string(),
        local_only: true,
        approved: true,
        timeout_ms: 30_000,
        stale_after_ms: 1,
        created_at_unix_ms: 1,
        updated_at_unix_ms: 1,
        finished_at_unix_ms: None,
        history_json: None,
        recovery_command: "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id readiness --cancel".to_string(),
        cleanup_command: "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json".to_string(),
    };
    let live_written =
        write_friday_trusted_host_live_runner_state(&live_state_path, vec![pending, running])
            .unwrap();
    assert_eq!(live_written.record_count, 2);
    assert_eq!(live_written.pending_count, 1);
    assert_eq!(live_written.stale_count, 1);
    assert!(live_written.stale_recovery_copy.contains("Stale"));
    let cancellation_ux = friday_trusted_host_runner_cancellation_ux_report(&live_written);
    assert_eq!(cancellation_ux.active_count, 1);
    assert_eq!(cancellation_ux.stale_count, 1);
    assert!(
        cancellation_ux
            .draft
            .storage_key
            .contains("runnerCancellationDrafts")
    );
    assert!(cancellation_ux.controls.iter().any(|control| {
        control.kind == "cancel"
            && control.requires_reason
            && control.command.contains("--cancel")
            && control.command.contains("--state")
    }));
    assert!(cancellation_ux.controls.iter().any(|control| {
        control.kind == "cleanup-stale"
            && control.command.contains("--friday-trusted-host-live-state")
    }));
    assert!(cancellation_ux.controls.iter().any(|control| {
        control.kind == "retry"
            && control.requires_reason
            && control.command.contains("--approve --execute")
    }));
    let package_path = root.join("trusted-runner-release-package.json");
    let release_package = friday_trusted_runner_release_package_report(
        &root,
        &history_path,
        &live_state_path,
        &package_path,
    );
    assert_eq!(
        release_package.manifest.history_json,
        history_path.to_string_lossy().replace('\\', "/")
    );
    assert_eq!(release_package.manifest.missing_count, 0);
    assert!(!release_package.ready_to_ship);
    assert!(
        release_package
            .manifest
            .files
            .iter()
            .any(|file| file.id == "runner-history" && file.sha256.is_some())
    );
    assert!(
        release_package
            .manifest
            .files
            .iter()
            .any(|file| file.id == "runner-live-state" && file.present)
    );
    assert!(
        release_package
            .manifest
            .files
            .iter()
            .any(|file| file.id == "incident-notes" && file.sha256.is_some())
    );
    assert!(
        release_package
            .warnings
            .iter()
            .any(|warning| warning.contains("stale live runner"))
    );
    assert!(release_package.incident_markdown.contains("### Failed"));
    write_friday_trusted_runner_release_package(&package_path, &release_package).unwrap();
    assert!(package_path.exists());
    let missing_package = friday_trusted_runner_release_package_report(
        root.join("missing-dashboard"),
        root.join("missing-history.json"),
        root.join("missing-live-state.json"),
        root.join("missing-package.json"),
    );
    assert!(missing_package.manifest.missing_count >= 4);
    assert!(
        missing_package
            .warnings
            .iter()
            .any(|warning| warning.contains("missing"))
    );
    let first_package_path = root.join("trusted-runner-release-package-1.json");
    let second_package_path = root.join("trusted-runner-release-package-2.json");
    let mut first_package = release_package.clone();
    first_package.summary = "Trusted runner release package is ready.".to_string();
    first_package.ready_to_ship = true;
    first_package.warnings.clear();
    first_package.manifest.package_id = "trusted-runner-release-1".to_string();
    first_package.manifest.generated_at_unix_ms = 1;
    first_package.manifest.package_json = first_package_path.to_string_lossy().replace('\\', "/");
    first_package.manifest.warning_count = 0;
    first_package.manifest.package_signature = "sig-1".to_string();
    let mut second_package = missing_package.clone();
    second_package.manifest.package_id = "trusted-runner-release-2".to_string();
    second_package.manifest.generated_at_unix_ms = 2;
    second_package.manifest.package_json = second_package_path.to_string_lossy().replace('\\', "/");
    second_package.manifest.package_signature = "sig-2".to_string();
    write_friday_trusted_runner_release_package(&first_package_path, &first_package).unwrap();
    write_friday_trusted_runner_release_package(&second_package_path, &second_package).unwrap();
    let timeline_path = root.join("trusted-runner-release-timeline.json");
    let archived = append_friday_trusted_runner_release_package_to_timeline(
        &timeline_path,
        &first_package_path,
    )
    .unwrap();
    assert_eq!(archived.package_count, 1);
    let timeline = friday_trusted_runner_release_timeline_report(
        &timeline_path,
        &[second_package_path.clone()],
    );
    assert_eq!(timeline.package_count, 2);
    assert_eq!(
        timeline.latest_package_id.as_deref(),
        Some("trusted-runner-release-2")
    );
    assert_eq!(timeline.missing_evidence_regressions, 1);
    assert_eq!(timeline.warning_regressions, 1);
    assert_eq!(timeline.signature_changes, 1);
    assert!(timeline.diffs.iter().any(|diff| {
        diff.regression
            && diff.from_package_id == "trusted-runner-release-1"
            && diff.to_package_id == "trusted-runner-release-2"
            && diff.missing_delta > 0
    }));
    write_friday_trusted_runner_release_timeline(&timeline_path, &timeline).unwrap();
    let checklist_path = root.join("release-operator-checklist.json");
    let signoff_path = root.join("release-signoffs.json");
    let checklist = friday_release_operator_checklist_report(
        &checklist_path,
        &package_path,
        &timeline_path,
        &root,
        "TODO.md",
        "CHANGELOG.md",
        &signoff_path,
    );
    assert_eq!(checklist.product_name, "Friday");
    assert_eq!(checklist.status, FridayDashboardPanelStatus::Blocked);
    assert!(!checklist.ready_to_ship);
    assert!(checklist.blocking_count > 0);
    assert!(
        checklist
            .blockers
            .iter()
            .any(|blocker| blocker.category == "stale-live-state")
    );
    assert!(
        checklist
            .blockers
            .iter()
            .any(|blocker| blocker.category == "warning-regression")
    );
    assert!(
        checklist
            .checklist
            .iter()
            .any(|item| item.id == "operator-signoff" && !item.ready)
    );
    assert!(
        checklist
            .commands
            .iter()
            .any(|command| command.contains("--friday-release-signoff"))
    );
    write_friday_release_operator_checklist(&checklist_path, &checklist).unwrap();
    let signoffs = append_friday_release_operator_signoff(
        &checklist_path,
        &signoff_path,
        "essencefromexistence",
        FridayReleaseChecklistSignoffDecision::Approved,
        "Reviewed the local package, timeline, TODO, and changelog.",
    )
    .unwrap();
    assert_eq!(signoffs.len(), 1);
    assert_eq!(
        signoffs[0].decision,
        FridayReleaseChecklistSignoffDecision::Approved
    );
    let checklist_after_signoff = friday_release_operator_checklist_report(
        &checklist_path,
        &package_path,
        &timeline_path,
        &root,
        "TODO.md",
        "CHANGELOG.md",
        &signoff_path,
    );
    assert_eq!(checklist_after_signoff.signoff_count, 1);
    assert!(checklist_after_signoff.latest_signoff.is_some());
    let cargo_check_result_path = root.join("cargo-check.txt");
    let extension_typecheck_result_path = root.join("extension-typecheck.txt");
    let dashboard_smoke_result_path = root.join("dashboard-smoke.txt");
    fs::write(&cargo_check_result_path, "cargo check passed").unwrap();
    fs::write(&extension_typecheck_result_path, "typecheck passed").unwrap();
    fs::write(
        &dashboard_smoke_result_path,
        "Friday dashboard UI smoke: 100/100",
    )
    .unwrap();
    let qa_path = root.join("release-qa-command-center.json");
    let qa_report = friday_release_qa_command_center_report(
        &qa_path,
        &checklist_path,
        &package_path,
        &timeline_path,
        &cargo_check_result_path,
        &extension_typecheck_result_path,
        &dashboard_smoke_result_path,
    );
    write_friday_release_qa_command_center_report(&qa_path, &qa_report).unwrap();
    assert_eq!(qa_report.product_name, "Friday");
    assert!(qa_report.score_out_of_100 > 0);
    assert!(
        qa_report
            .checks
            .iter()
            .any(|check| check.id == "rust-cargo-check"
                && check.status == FridayReleaseQaCheckStatus::Passed)
    );
    assert!(
        qa_report
            .checks
            .iter()
            .any(|check| check.id == "release-checklist"
                && check.status == FridayReleaseQaCheckStatus::Failed)
    );
    assert!(
        qa_report
            .commands
            .iter()
            .any(|command| command.contains("--friday-release-qa"))
    );
    let export_kit_path = root.join("release-evidence-export-kit.json");
    let export_kit = friday_release_evidence_export_kit_report(
        &export_kit_path,
        &root,
        &checklist_path,
        &qa_path,
        &package_path,
        &timeline_path,
        &signoff_path,
        &cargo_check_result_path,
        &extension_typecheck_result_path,
        &dashboard_smoke_result_path,
    );
    assert_eq!(export_kit.manifest.product_name, "Friday");
    assert_eq!(export_kit.manifest.file_count, 8);
    assert_eq!(export_kit.manifest.missing_count, 0);
    assert!(!export_kit.ready_to_attach);
    assert_eq!(export_kit.signoff_count, 1);
    assert!(
        export_kit
            .manifest
            .files
            .iter()
            .any(|file| file.id == "release-qa" && file.sha256.is_some())
    );
    assert!(
        export_kit
            .operator_copy
            .contains("Friday release evidence kit")
    );
    assert!(
        export_kit
            .manifest
            .commands
            .iter()
            .any(|command| command.contains("--friday-release-export-kit"))
    );
    write_friday_release_evidence_export_kit(&export_kit_path, &export_kit).unwrap();
    assert!(export_kit_path.exists());
    let deployment_gate_path = root.join("release-deployment-gate.json");
    let deployment_gate = friday_release_deployment_gate_report(
        &deployment_gate_path,
        &export_kit_path,
        &qa_path,
        &checklist_path,
        &package_path,
        &timeline_path,
        &root,
        FridayReleaseDeploymentTarget::default(),
    );
    assert_eq!(deployment_gate.product_name, "Friday");
    assert_eq!(
        deployment_gate.decision,
        FridayReleaseDeploymentGateDecision::NoGo
    );
    assert!(!deployment_gate.ready_to_deploy);
    assert!(deployment_gate.no_deploy_reason_count > 0);
    assert!(deployment_gate.score_out_of_100 < 100);
    assert!(
        deployment_gate
            .reasons
            .iter()
            .any(|reason| reason.category.label() == "blocked-qa")
    );
    assert!(
        deployment_gate
            .operator_copy
            .contains("Friday deployment gate")
    );
    assert!(
        deployment_gate
            .commands
            .iter()
            .any(|command| command.contains("--friday-release-deployment-gate"))
    );
    write_friday_release_deployment_gate(&deployment_gate_path, &deployment_gate).unwrap();
    assert!(deployment_gate_path.exists());
    let candidate_archive_path = root.join("release-candidate-archive.json");
    let candidate_archive =
        append_friday_release_candidate_to_archive(&candidate_archive_path, &deployment_gate_path)
            .unwrap();
    assert_eq!(candidate_archive.candidate_count, 1);
    assert_eq!(
        candidate_archive.latest_decision,
        Some(FridayReleaseDeploymentGateDecision::NoGo)
    );
    assert_eq!(candidate_archive.no_go_count, 1);
    assert!(
        candidate_archive
            .entries
            .iter()
            .any(|entry| entry.export_kit_manifest_sha256.is_some())
    );
    let mut loaded_candidate =
        friday_release_candidate_entry_from_gate(&deployment_gate_path).unwrap();
    loaded_candidate.score_out_of_100 = 80;
    let mut regressed_candidate: FridayReleaseCandidateArchiveEntry = loaded_candidate.clone();
    regressed_candidate.candidate_id = "candidate-regressed".to_string();
    regressed_candidate.gate_json = root
        .join("release-deployment-gate-regressed.json")
        .to_string_lossy()
        .replace('\\', "/");
    regressed_candidate.generated_at_unix_ms += 1;
    regressed_candidate.score_out_of_100 = 70;
    regressed_candidate.no_deploy_reason_count += 1;
    regressed_candidate
        .reason_ids
        .push("new-deploy-blocker".to_string());
    regressed_candidate.export_kit_manifest_sha256 = Some("changed".to_string());
    let compared_archive = friday_release_candidate_archive_report(
        &candidate_archive_path,
        vec![loaded_candidate, regressed_candidate],
    );
    assert_eq!(compared_archive.candidate_count, 2);
    assert_eq!(compared_archive.regression_count, 1);
    assert!(compared_archive.diffs.iter().any(|diff| {
        diff.regression
            && diff.score_delta < 0
            && diff
                .new_blocker_ids
                .contains(&"new-deploy-blocker".to_string())
            && diff.evidence_checksum_changed
    }));
    let promotion_ledger_path = root.join("release-promotion-ledger.json");
    let promotion_ledger = append_friday_release_promotion_to_ledger(
        &promotion_ledger_path,
        &candidate_archive_path,
        FridayReleasePromotionRecordRequest {
            candidate_id: candidate_archive.latest_candidate_id.clone(),
            decision: FridayReleasePromotionDecision::Held,
            operator: "essencefromexistence".to_string(),
            reason: "No-go candidate is held until QA is green.".to_string(),
            deployment_note: "Not deployed; attach gate and candidate archive.".to_string(),
            rollback_reference: "previous-stable-friday".to_string(),
            post_check_files: vec![format!(
                "dashboard-smoke={}",
                dashboard_smoke_result_path
                    .to_string_lossy()
                    .replace('\\', "/")
            )],
        },
    )
    .unwrap();
    assert_eq!(promotion_ledger.record_count, 1);
    assert_eq!(promotion_ledger.held_count, 1);
    assert_eq!(
        promotion_ledger.latest_decision,
        Some(FridayReleasePromotionDecision::Held)
    );
    assert_eq!(
        promotion_ledger.active_rollback_reference.as_deref(),
        Some("previous-stable-friday")
    );
    assert!(promotion_ledger.records[0].post_promotion_missing_count == 0);
    assert!(
        promotion_ledger
            .commands
            .iter()
            .any(|command| command.contains("--friday-release-promotion-ledger"))
    );
    let promoted_ledger = append_friday_release_promotion_to_ledger(
        &promotion_ledger_path,
        &candidate_archive_path,
        FridayReleasePromotionRecordRequest {
            candidate_id: candidate_archive.latest_candidate_id.clone(),
            decision: FridayReleasePromotionDecision::Promoted,
            operator: "essencefromexistence".to_string(),
            reason: "Promotion is intentionally recorded for warning coverage.".to_string(),
            deployment_note: "Promoted only in the local audit ledger.".to_string(),
            rollback_reference: "previous-stable-friday".to_string(),
            post_check_files: vec![
                "post-promotion-smoke=missing-post-promotion-smoke.json".to_string(),
            ],
        },
    )
    .unwrap();
    assert_eq!(promoted_ledger.record_count, 2);
    assert_eq!(promoted_ledger.promoted_count, 1);
    assert!(promoted_ledger.post_promotion_missing_count > 0);
    assert!(
        promoted_ledger
            .warnings
            .iter()
            .any(|warning| { warning.contains("missing") || warning.contains("not marked ready") })
    );
    let incident_note_path = root.join("post-promotion-incident.md");
    fs::write(
        &incident_note_path,
        "No customer-facing incident. Post-promotion smoke still needs evidence.",
    )
    .unwrap();
    let post_promotion_monitor_path = root.join("release-post-promotion-monitor.json");
    let post_promotion_monitor = friday_release_post_promotion_monitor_report(
        &post_promotion_monitor_path,
        &promotion_ledger_path,
        &qa_path,
        &dashboard_smoke_result_path,
        vec![incident_note_path.to_string_lossy().replace('\\', "/")],
    );
    write_friday_release_post_promotion_monitor_report(
        &post_promotion_monitor_path,
        &post_promotion_monitor,
    )
    .unwrap();
    assert_eq!(post_promotion_monitor.promoted_count, 1);
    assert_eq!(
        post_promotion_monitor.active_rollback_reference.as_deref(),
        Some("previous-stable-friday")
    );
    assert!(post_promotion_monitor.blocking_count > 0);
    assert!(!post_promotion_monitor.ready_for_stable);
    assert!(post_promotion_monitor.incident_note_count == 1);
    assert!(post_promotion_monitor.checks.iter().any(|check| {
        check.id.contains("post-promotion-smoke") && check.status.label() == "missing"
    }));
    assert!(
        post_promotion_monitor
            .commands
            .iter()
            .any(|command| { command.contains("--friday-release-post-promotion-monitor") })
    );
    let rollback_drill_path = root.join("release-rollback-drill.json");
    let rollback_drill = friday_release_rollback_drill_report(
        &rollback_drill_path,
        &post_promotion_monitor_path,
        &promotion_ledger_path,
        &candidate_archive_path,
        &deployment_gate_path,
        "flow rollback previous-stable-friday --dry-run",
        "essencefromexistence",
        "Verify rollback before stable promotion.",
    );
    write_friday_release_rollback_drill_report(&rollback_drill_path, &rollback_drill).unwrap();
    assert_eq!(
        rollback_drill.active_rollback_reference.as_deref(),
        Some("previous-stable-friday")
    );
    assert!(rollback_drill.blocking_count > 0);
    assert!(!rollback_drill.ready_to_rollback);
    assert!(!rollback_drill.ready_for_stable);
    assert!(
        rollback_drill
            .blocked_reasons
            .iter()
            .any(|reason| reason.contains("Post-promotion monitor"))
    );
    assert!(
        rollback_drill.checks.iter().any(|check| {
            check.id == "post-promotion-monitor" && check.status.label() == "failed"
        })
    );
    assert!(
        rollback_drill
            .commands
            .iter()
            .any(|command| { command.contains("--friday-release-rollback-drill") })
    );
    let stability_board_path = root.join("release-stability-board.json");
    let stability_board = friday_release_stability_board_report(
        &stability_board_path,
        &qa_path,
        &candidate_archive_path,
        &promotion_ledger_path,
        &post_promotion_monitor_path,
        &rollback_drill_path,
        &deployment_gate_path,
    );
    write_friday_release_stability_board_report(&stability_board_path, &stability_board).unwrap();
    assert_eq!(
        stability_board.active_rollback_reference.as_deref(),
        Some("previous-stable-friday")
    );
    assert!(stability_board.score_out_of_100 < 100);
    assert!(stability_board.blocking_count > 0);
    assert!(!stability_board.ready_for_checkpoint);
    assert!(!stability_board.recoverable);
    assert!(
        stability_board
            .checks
            .iter()
            .any(|check| check.category.label() == "rollback-recovery")
    );
    assert!(
        stability_board
            .active_risks
            .iter()
            .any(|risk| risk.contains("Post-promotion freshness"))
    );
    assert!(
        stability_board
            .commands
            .iter()
            .any(|command| command.contains("--friday-release-stability-board"))
    );
    let recovery_runbook_path = root.join("release-recovery-runbook.json");
    let recovery_runbook = friday_release_recovery_runbook_report(
        &recovery_runbook_path,
        &stability_board_path,
        &rollback_drill_path,
        &promotion_ledger_path,
        &post_promotion_monitor_path,
    );
    write_friday_release_recovery_runbook_report(&recovery_runbook_path, &recovery_runbook)
        .unwrap();
    assert_eq!(recovery_runbook.phase_count, 6);
    assert_eq!(
        recovery_runbook
            .phases
            .iter()
            .map(|phase| phase.kind.label())
            .collect::<Vec<_>>(),
        vec![
            "pause",
            "diagnose",
            "rollback",
            "verify",
            "resume",
            "follow-up"
        ]
    );
    assert!(recovery_runbook.blocked_phase_count > 0);
    assert!(!recovery_runbook.ready_to_execute_recovery);
    assert!(recovery_runbook.approval_gate_count >= 3);
    assert!(
        recovery_runbook.phases.iter().any(|phase| {
            phase.kind.label() == "rollback" && phase.command.contains("--dry-run")
        })
    );
    assert!(
        recovery_runbook
            .commands
            .iter()
            .any(|command| command.contains("--friday-release-recovery-runbook"))
    );
    let incident_archive_path = root.join("release-incident-archive.json");
    let incident_entry = friday_release_incident_entry_from_sources(
        &recovery_runbook_path,
        &stability_board_path,
        &rollback_drill_path,
        &post_promotion_monitor_path,
        vec![incident_note_path.to_string_lossy().replace('\\', "/")],
        FridayReleaseIncidentOutcome::Open,
    );
    assert_eq!(
        incident_entry.severity,
        FridayReleaseIncidentSeverity::Critical
    );
    assert_eq!(incident_entry.outcome, FridayReleaseIncidentOutcome::Open);
    assert_eq!(
        incident_entry.active_rollback_reference.as_deref(),
        Some("previous-stable-friday")
    );
    assert!(incident_entry.blocked_phase_count > 0);
    assert!(
        incident_entry
            .follow_up_actions
            .iter()
            .any(|action| action.contains("Rollback"))
    );
    assert!(
        incident_entry
            .prevention_items
            .iter()
            .any(|item| item.contains("Prevent recurrence"))
    );
    let incident_archive =
        friday_release_incident_archive_report(&incident_archive_path, vec![incident_entry]);
    assert_eq!(incident_archive.incident_count, 1);
    assert_eq!(incident_archive.open_count, 1);
    assert_eq!(incident_archive.critical_count, 1);
    assert!(incident_archive.follow_up_count > 0);
    assert!(
        incident_archive
            .commands
            .iter()
            .any(|command| command.contains("--friday-release-incident-archive"))
    );
    let appended_incident_archive = append_friday_release_incident_to_archive(
        &incident_archive_path,
        &recovery_runbook_path,
        &stability_board_path,
        &rollback_drill_path,
        &post_promotion_monitor_path,
        vec![incident_note_path.to_string_lossy().replace('\\', "/")],
        FridayReleaseIncidentOutcome::Monitoring,
    )
    .unwrap();
    assert_eq!(appended_incident_archive.incident_count, 1);
    assert_eq!(appended_incident_archive.monitoring_count, 1);
    assert!(
        appended_incident_archive
            .entries
            .iter()
            .all(|entry| !entry.evidence_paths.is_empty())
    );
    let prevention_plan_path = root.join("release-prevention-plan.json");
    let prevention_plan = friday_release_prevention_plan_report(
        &prevention_plan_path,
        &incident_archive_path,
        &stability_board_path,
    );
    write_friday_release_prevention_plan_report(&prevention_plan_path, &prevention_plan).unwrap();
    assert_eq!(prevention_plan.incident_count, 1);
    assert!(!prevention_plan.ready_for_next_checkpoint);
    assert!(prevention_plan.blocker_count > 0);
    assert!(prevention_plan.gate_blocking_count > 0);
    assert!(prevention_plan.findings.iter().any(|finding| {
        finding.kind == FridayReleasePreventionFindingKind::RollbackGap
            && finding.release_gate_blocking
    }));
    assert!(prevention_plan.findings.iter().any(|finding| {
        finding.kind == FridayReleasePreventionFindingKind::RepeatedFailureClass
            && finding.id.contains("rollback")
    }));
    assert!(prevention_plan.actions.iter().any(|action| {
        action.kind == FridayReleasePreventionActionKind::HardenRollback
            && action.command.contains("--dry-run")
    }));
    assert!(
        prevention_plan
            .owner_ready_copy
            .contains("Friday release prevention plan")
    );
    assert!(
        prevention_plan
            .commands
            .iter()
            .any(|command| command.contains("--friday-release-prevention-plan"))
    );
    let owner_followup_board_path = root.join("release-owner-followup-board.json");
    let owner_followup_board = friday_release_owner_followup_board_report_at(
        &owner_followup_board_path,
        &prevention_plan_path,
        1,
    );
    write_friday_release_owner_followup_board_report(
        &owner_followup_board_path,
        &owner_followup_board,
    )
    .unwrap();
    assert_eq!(
        owner_followup_board.record_count,
        prevention_plan.action_count
    );
    assert!(owner_followup_board.owner_count > 0);
    assert!(owner_followup_board.evidence_missing_count > 0);
    assert!(owner_followup_board.overdue_count > 0);
    assert!(owner_followup_board.gate_blocking_count > 0);
    assert!(
        owner_followup_board
            .owner_groups
            .iter()
            .any(|group| { group.owner == "release-operator" && group.evidence_missing_count > 0 })
    );
    assert!(owner_followup_board.records.iter().any(|record| {
        record.completion_state == FridayReleaseOwnerFollowUpCompletionState::Overdue
            && record.assignment_copy.contains("@release-operator")
            && record.command.contains("--dry-run")
    }));
    assert!(
        owner_followup_board
            .assignment_copy
            .contains("Friday release owner follow-up board")
    );
    assert!(owner_followup_board.commands.iter().any(|command| {
        command.contains("--friday-release-owner-followup-board")
            && command.contains("--prevention-plan")
    }));
    let evidence_sla_monitor_path = root.join("release-evidence-sla-monitor.json");
    let evidence_sla_monitor = friday_release_evidence_sla_monitor_report_at(
        &evidence_sla_monitor_path,
        &owner_followup_board_path,
        &prevention_plan_path,
        &stability_board_path,
        2,
    );
    write_friday_release_evidence_sla_monitor_report(
        &evidence_sla_monitor_path,
        &evidence_sla_monitor,
    )
    .unwrap();
    assert!(evidence_sla_monitor.requirement_count >= owner_followup_board.record_count);
    assert!(evidence_sla_monitor.owner_count > 0);
    assert!(evidence_sla_monitor.overdue_count > 0);
    assert!(evidence_sla_monitor.escalation_count > 0);
    assert!(evidence_sla_monitor.gate_blocking_count > 0);
    assert!(evidence_sla_monitor.owner_groups.iter().any(|group| {
        group.owner == "release-operator" && group.release_gate_blocking_count > 0
    }));
    assert!(evidence_sla_monitor.requirements.iter().any(|requirement| {
        requirement.state == FridayReleaseEvidenceSlaState::Overdue
            && requirement.escalation_level >= FridayReleaseEvidenceEscalationLevel::ReleaseGate
            && requirement.escalation_copy.contains("@release-operator")
    }));
    assert!(
        evidence_sla_monitor
            .escalation_copy
            .contains("Friday release evidence SLA monitor")
    );
    assert!(evidence_sla_monitor.commands.iter().any(|command| {
        command.contains("--friday-release-evidence-sla-monitor")
            && command.contains("--owner-followup-board")
    }));
    let escalation_ledger_path = root.join("release-escalation-ledger.json");
    let escalation_ledger = append_friday_release_escalation_to_ledger(
        &escalation_ledger_path,
        &evidence_sla_monitor_path,
        FridayReleaseEscalationOwnerResponse::Pending,
        FridayReleaseEscalationGateOutcome::CarryOver,
    )
    .unwrap();
    assert!(escalation_ledger.entry_count > 0);
    assert!(escalation_ledger.active_count > 0);
    assert!(escalation_ledger.carryover_count > 0);
    assert!(escalation_ledger.acknowledgement_blocker_count > 0);
    assert!(escalation_ledger.release_gate_blocking_count > 0);
    assert!(
        escalation_ledger
            .owner_groups
            .iter()
            .any(|group| { group.owner == "release-operator" && group.active_count > 0 })
    );
    assert!(escalation_ledger.entries.iter().any(|entry| {
        entry.owner_response == FridayReleaseEscalationOwnerResponse::Pending
            && entry.gate_outcome == FridayReleaseEscalationGateOutcome::CarryOver
            && entry.owner_response_copy.contains("@release-operator")
    }));
    assert!(
        escalation_ledger
            .owner_response_copy
            .contains("Friday release escalation ledger")
    );
    assert!(escalation_ledger.commands.iter().any(|command| {
        command.contains("--friday-release-escalation-ledger") && command.contains("--monitor")
    }));
    let checkpoint_review_path = root.join("release-checkpoint-review.json");
    let checkpoint_review = friday_release_checkpoint_review_board_report(
        &checkpoint_review_path,
        &escalation_ledger_path,
        &evidence_sla_monitor_path,
        &owner_followup_board_path,
        &prevention_plan_path,
        &stability_board_path,
    );
    write_friday_release_checkpoint_review_board_report(
        &checkpoint_review_path,
        &checkpoint_review,
    )
    .unwrap();
    assert_eq!(
        checkpoint_review.decision,
        FridayReleaseCheckpointDecision::Hold
    );
    assert!(!checkpoint_review.ready_for_checkpoint);
    assert!(checkpoint_review.item_count > 0);
    assert!(checkpoint_review.hold_count > 0);
    assert!(checkpoint_review.acknowledgement_blocker_count > 0);
    assert!(checkpoint_review.release_gate_blocking_count > 0);
    assert!(checkpoint_review.owner_groups.iter().any(|group| {
        group.owner == "release-operator" && group.release_gate_blocking_count > 0
    }));
    assert!(checkpoint_review.items.iter().any(|item| {
        item.active_carryover
            && item.release_gate_blocking
            && item.next_action.contains("@release-operator")
    }));
    assert!(
        checkpoint_review
            .review_notes_copy
            .contains("Friday release checkpoint review")
    );
    assert!(checkpoint_review.commands.iter().any(|command| {
        command.contains("--friday-release-checkpoint-review") && command.contains("--ledger")
    }));
    let checkpoint_signoff_ledger_path = root.join("release-checkpoint-signoff-ledger.json");
    let checkpoint_signoff_ledger = append_friday_release_checkpoint_signoff_to_ledger(
        &checkpoint_signoff_ledger_path,
        &checkpoint_review_path,
        FridayReleaseCheckpointSignoffRequest {
            decision: FridayReleaseCheckpointSignoffDecision::Held,
            operator: "release-operator".to_string(),
            reason: "Hold checkpoint until acknowledgement blockers are cleared.".to_string(),
            acknowledgement_evidence_path: String::new(),
            carryover_commitment:
                "Carry rollback and prevention acknowledgement evidence into the next loop."
                    .to_string(),
        },
    )
    .unwrap();
    assert_eq!(checkpoint_signoff_ledger.record_count, 1);
    assert_eq!(checkpoint_signoff_ledger.held_count, 1);
    assert_eq!(
        checkpoint_signoff_ledger.active_decision,
        Some(FridayReleaseCheckpointSignoffDecision::Held)
    );
    assert!(checkpoint_signoff_ledger.active_hold_count > 0);
    assert!(checkpoint_signoff_ledger.active_carryover_count > 0);
    assert!(checkpoint_signoff_ledger.release_gate_blocking_count > 0);
    assert!(checkpoint_signoff_ledger.records.iter().any(|record| {
        record.operator == "release-operator"
            && record.active_hold
            && record.active_carryover
            && record.reason.contains("acknowledgement blockers")
    }));
    assert!(
        checkpoint_signoff_ledger
            .release_notes_copy
            .contains("Friday checkpoint signoff ledger")
    );
    assert!(checkpoint_signoff_ledger.commands.iter().any(|command| {
        command.contains("--friday-release-checkpoint-signoff") && command.contains("--review")
    }));
    let live_loaded = read_friday_trusted_host_live_runner_state(&live_state_path).unwrap();
    assert_eq!(live_loaded.record_count, 2);
    let live_refreshed = refresh_friday_trusted_host_live_runner_state(&live_loaded);
    assert_eq!(live_refreshed.stale_count, 1);
    assert!(
        live_refreshed
            .records
            .iter()
            .any(|record| record.status == FridayTrustedHostLiveRunnerStatus::Stale)
    );
    let bridge_state_path = root.join("trusted-host-bridge-live-state.json");
    let bridge_history_path = root.join("trusted-host-bridge-history.json");
    let bridge_report = run_friday_trusted_host_command_bridge_with_executor(
        &bridge.records[0],
        &approved,
        &StubTrustedHostExecutor {
            output: Ok(FridayTrustedHostCommandRawOutput {
                exit_code: Some(0),
                stdout: "bridge ok".to_string(),
                stderr: String::new(),
                duration_ms: 7,
                timed_out: false,
            }),
        },
        &bridge_state_path,
        &bridge_history_path,
        &FridayTrustedHostRunnerCancellationToken::none(),
    )
    .unwrap();
    assert_eq!(
        bridge_report.result.status,
        FridayTrustedHostRunnerStatus::Succeeded
    );
    assert_eq!(bridge_report.event_count, 3);
    assert_eq!(
        bridge_report.events[0].status,
        FridayTrustedHostLiveRunnerStatus::Pending
    );
    assert_eq!(
        bridge_report.events[1].status,
        FridayTrustedHostLiveRunnerStatus::Running
    );
    assert_eq!(
        bridge_report.events[2].status,
        FridayTrustedHostLiveRunnerStatus::Succeeded
    );
    assert_eq!(bridge_report.live_state.finished_count, 1);
    assert_eq!(bridge_report.history.result_count, 1);
    assert!(
        bridge_report
            .dashboard_import_guidance
            .contains("live-state JSON")
    );

    let cancelled_bridge = run_friday_trusted_host_command_bridge_with_executor(
        &bridge.records[0],
        &FridayTrustedHostRunnerRequest {
            approved: true,
            operator_reason: Some("cancel from bridge test".to_string()),
            ..Default::default()
        },
        &StubTrustedHostExecutor {
            output: Err("executor should not run".to_string()),
        },
        root.join("cancel-live-state.json"),
        root.join("cancel-history.json"),
        &FridayTrustedHostRunnerCancellationToken::requested("cancel from bridge test"),
    )
    .unwrap();
    assert_eq!(
        cancelled_bridge.result.status,
        FridayTrustedHostRunnerStatus::Cancelled
    );
    assert_eq!(cancelled_bridge.event_count, 2);
    assert!(
        cancelled_bridge
            .events
            .iter()
            .all(|event| event.status != FridayTrustedHostLiveRunnerStatus::Running)
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_dashboard_product_ui_smoke_proves_visible_dashboard_contract() {
    let root = temp_root("friday-dashboard-product-ui-smoke");
    export_friday_dashboard_bundle(&root).unwrap();

    let report = friday_dashboard_product_ui_smoke_from_export(&root).unwrap();
    assert_eq!(report.product_name, "Friday");
    assert_eq!(report.route, "/dashboard");
    assert_eq!(report.status, FridayDashboardProductUiSmokeStatus::Passed);
    assert_eq!(report.score_out_of_100, 100);
    assert_eq!(report.blocking_count, 0);
    assert_eq!(report.warning_count, 0);
    assert_eq!(report.check_count, 6);
    assert!(report.checks.iter().any(|check| {
        check.id == "dashboard-source-file"
            && check.status == FridayDashboardProductUiSmokeStatus::Passed
    }));
    assert!(report.checks.iter().any(|check| {
        check.id == "panel-json-binding"
            && check.status == FridayDashboardProductUiSmokeStatus::Passed
    }));
    assert!(report.checks.iter().any(|check| {
        check.id == "safe-actions" && check.status == FridayDashboardProductUiSmokeStatus::Passed
    }));
    assert!(report.checks.iter().any(|check| {
        check.id == "history-and-release-links"
            && check.status == FridayDashboardProductUiSmokeStatus::Passed
    }));
    assert!(report.checks.iter().any(|check| {
        check.id == "screenshot-prompts"
            && check.status == FridayDashboardProductUiSmokeStatus::Passed
    }));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn friday_dashboard_screenshot_history_tracks_captures_and_prompts() {
    let root = temp_root("friday-dashboard-screenshot-history");
    fs::write(root.join("ask-desktop.png"), b"png-fixture").unwrap();
    fs::write(root.join("ask-desktop.json"), b"{\"ok\":true}").unwrap();

    let report = friday_route_visual_report_for_root(&root);
    let history = friday_dashboard_screenshot_history(&report);

    assert_eq!(history.total_targets, 10);
    assert_eq!(history.captured_count, 1);
    assert_eq!(history.missing_count, 9);
    assert_eq!(history.metadata_missing_count, 0);
    assert!(history.records.iter().any(|record| {
        record.route == "/ask"
            && record.viewport_id == "desktop"
            && record.status == FridayDashboardScreenshotStatus::Captured
            && record.screenshot_bytes > 0
            && record.metadata_bytes > 0
            && record.captured_at_unix_ms.is_some()
    }));
    assert!(history.records.iter().any(|record| {
        record.status == FridayDashboardScreenshotStatus::Missing
            && record.prompt.contains("agent-browser screenshot")
    }));

    let _ = fs::remove_dir_all(&root);
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
    assert!(
        report
            .targets
            .iter()
            .all(|target| target.status == BrowserExtensionSmokeStatus::Passed)
    );

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
    assert!(
        report
            .targets
            .iter()
            .filter(|target| target.extension_target == "chromium")
            .all(|target| target.command_preview.contains("--user-data-dir"))
    );
    assert!(
        report
            .targets
            .iter()
            .filter(|target| target.extension_target == "chromium")
            .all(|target| target.profile_dir.is_some())
    );
    assert!(
        report
            .targets
            .iter()
            .any(|target| target.status == BrowserExtensionSmokeStatus::Warning)
    );

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
            && target.files.iter().all(|file| {
                file.local_url
                    .starts_with("https://flow.browserpack.local/")
            })
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
