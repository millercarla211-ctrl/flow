use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hf_hub::api::sync::Api;
use serde_json::Value;

use crate::audio::{NoiseGateVAD, WakeWordDetector};
use crate::browser::{
    BrowserExtensionSmokeStatus, BrowserHostFlavor, BrowserTask, FlowBrowserEngine,
    browser_extension_launch_smoke_report, browser_extension_smoke_report,
    browser_pack_recovery_smoke_report, browser_pack_reuse_smoke_report,
    browser_webllm_acceleration_report, default_browser_pack_catalog,
};
use crate::cli::Command;
use crate::competitive::active_completion_set;
use crate::competitive::default_competitive_scorecard;
use crate::config::FlowIntegrationTarget;
use crate::embed::{FlowEmbeddingRegistry, HostSurface};
use crate::experience::{
    FlowAccessibilityDiagnostic, FlowAccessibilityRuntime, FlowAutomationBridge,
    FlowFileStateStore, FlowStateStore, NativeSelectionBridge, OperatingSystemFamily,
};
use crate::friday::{
    FridayArtifactStore, FridayFeatureStatus, FridayReleaseCandidateArchive,
    FridayReleaseChecklistSignoff, FridayReleaseChecklistSignoffDecision,
    FridayReleaseCheckpointEvidenceVault, FridayReleaseCheckpointReviewBoardReport,
    FridayReleaseCheckpointSignoffDecision, FridayReleaseCheckpointSignoffLedger,
    FridayReleaseCheckpointSignoffRequest, FridayReleaseClosureLedger, FridayReleaseClosureRequest,
    FridayReleaseClosureState, FridayReleaseContinuityEntryKind, FridayReleaseContinuityJournal,
    FridayReleaseContinuityRequest, FridayReleaseDeploymentGateReport,
    FridayReleaseDeploymentTarget, FridayReleaseEscalationGateOutcome,
    FridayReleaseEscalationLedger, FridayReleaseEscalationOwnerResponse,
    FridayReleaseEvidenceAttachmentReview, FridayReleaseEvidenceExportKitReport,
    FridayReleaseEvidenceSlaMonitorReport, FridayReleaseExternalReceiptArchive,
    FridayReleaseExternalReceiptKind, FridayReleaseExternalReceiptRequest,
    FridayReleaseExternalReceiptState, FridayReleaseHandoffAuditRequest,
    FridayReleaseHandoffAuditState, FridayReleaseHandoffAuditTrail,
    FridayReleaseHandoffCompletionLedger, FridayReleaseHandoffCompletionRequest,
    FridayReleaseHandoffCompletionState, FridayReleaseHandoffDispatchAuditRequest,
    FridayReleaseHandoffDispatchAuditState, FridayReleaseHandoffDispatchAuditTrail,
    FridayReleaseHandoffDispatchChecklist, FridayReleaseHandoffDispatchChecklistRequest,
    FridayReleaseHandoffDispatchGovernanceReview, FridayReleaseHandoffGovernanceReview,
    FridayReleaseHandoffPacket, FridayReleaseIncidentArchive, FridayReleaseIncidentOutcome,
    FridayReleaseLearningCategory, FridayReleaseLearningRegister, FridayReleaseLearningRequest,
    FridayReleaseOperatorChecklistReport, FridayReleaseOutboundReviewLedger,
    FridayReleaseOutboundReviewRequest, FridayReleaseOutboundReviewState,
    FridayReleaseOwnerFollowUpBoardReport, FridayReleasePostPromotionMonitorReport,
    FridayReleasePreventionPlanReport, FridayReleasePromotionDecision,
    FridayReleasePromotionLedger, FridayReleasePromotionRecordRequest,
    FridayReleasePublicationControl, FridayReleasePublicationRequest,
    FridayReleasePublicationState, FridayReleaseQaCommandCenterReport,
    FridayReleaseReceiptReviewBoardReport, FridayReleaseRecoveryRunbookReport,
    FridayReleaseRollbackDrillReport, FridayReleaseStabilityBoardReport, FridayResearchReport,
    FridayResearchWorkflow, FridayRuntimeSurfaceStore, FridayTrustedHostLiveRunnerState,
    FridayTrustedHostRunnerApprovalUiReport, FridayTrustedHostRunnerBridgeReport,
    FridayTrustedHostRunnerCancellationToken, FridayTrustedHostRunnerCancellationUxReport,
    FridayTrustedHostRunnerOperatorReviewFilter, FridayTrustedHostRunnerOperatorReviewReport,
    FridayTrustedHostRunnerRequest, FridayTrustedHostRunnerResult, FridayTrustedHostRunnerStatus,
    FridayTrustedHostRunnerUxReport, FridayTrustedRunnerReleasePackageReport,
    FridayTrustedRunnerReleaseTimeline, FridayUiIntegrationStatus, FridayWorkspaceStore,
    append_friday_release_candidate_to_archive, append_friday_release_checkpoint_signoff_to_ledger,
    append_friday_release_closure_to_ledger, append_friday_release_continuity_to_journal,
    append_friday_release_escalation_to_ledger, append_friday_release_external_receipt_to_archive,
    append_friday_release_handoff_audit_to_trail,
    append_friday_release_handoff_completion_to_ledger,
    append_friday_release_handoff_dispatch_audit_to_trail,
    append_friday_release_incident_to_archive, append_friday_release_learning_to_register,
    append_friday_release_operator_signoff, append_friday_release_outbound_review_to_ledger,
    append_friday_release_promotion_to_ledger, append_friday_trusted_host_runner_history,
    append_friday_trusted_runner_release_package_to_timeline,
    default_friday_browser_verification_report, default_friday_local_execution_checks,
    default_friday_product_plan, default_friday_ui_integration_plan,
    export_friday_dashboard_bundle, friday_answer_search_plan,
    friday_dashboard_host_command_bridge_from_export, friday_dashboard_panel_from_export,
    friday_dashboard_product_ui_binding_from_export, friday_dashboard_product_ui_smoke_from_export,
    friday_execution_handoff_report, friday_live_ui_route_binding_report, friday_media_affordances,
    friday_multimodal_route, friday_multimodal_ui_diagnostics, friday_multimodal_visual_check,
    friday_operator_readiness_report, friday_release_candidate_archive_report,
    friday_release_candidate_entry_from_gate, friday_release_checkpoint_evidence_vault_report,
    friday_release_checkpoint_review_board_report, friday_release_checkpoint_signoff_ledger_report,
    friday_release_checkpoint_signoff_record_from_review, friday_release_closure_ledger_report,
    friday_release_closure_record_from_receipt_review,
    friday_release_continuity_entry_from_closure_ledger, friday_release_continuity_journal_report,
    friday_release_deployment_gate_report, friday_release_escalation_entries_from_monitor,
    friday_release_escalation_ledger_report, friday_release_evidence_attachment_review_report,
    friday_release_evidence_export_kit_report, friday_release_evidence_sla_monitor_report,
    friday_release_external_receipt_archive_report,
    friday_release_external_receipt_record_from_outbound_review,
    friday_release_handoff_audit_record_from_packet, friday_release_handoff_audit_trail_report,
    friday_release_handoff_completion_ledger_report,
    friday_release_handoff_completion_record_from_governance_review,
    friday_release_handoff_dispatch_audit_record_from_checklist,
    friday_release_handoff_dispatch_audit_trail_report,
    friday_release_handoff_dispatch_checklist_report,
    friday_release_handoff_dispatch_governance_review_report,
    friday_release_handoff_governance_review_report, friday_release_handoff_packet_report,
    friday_release_incident_archive_report, friday_release_incident_entry_from_sources,
    friday_release_learning_record_from_continuity_journal,
    friday_release_learning_register_report, friday_release_operator_checklist_report,
    friday_release_outbound_review_ledger_report,
    friday_release_outbound_review_record_from_publication_control,
    friday_release_owner_followup_board_report, friday_release_post_promotion_monitor_report,
    friday_release_prevention_plan_report, friday_release_promotion_ledger_report,
    friday_release_publication_control_report, friday_release_qa_command_center_report,
    friday_release_receipt_review_board_report, friday_release_recovery_runbook_report,
    friday_release_rollback_drill_report, friday_release_stability_board_report,
    friday_research_search_plan, friday_route_visual_report,
    friday_trusted_host_live_runner_state_from_history_file,
    friday_trusted_host_runner_approval_ui_report_from_history_file,
    friday_trusted_host_runner_cancellation_ux_report_from_state_file,
    friday_trusted_host_runner_operator_review_report_from_history_file,
    friday_trusted_host_runner_ux_report_from_history_file,
    friday_trusted_runner_release_package_report, friday_trusted_runner_release_timeline_report,
    read_friday_release_candidate_archive, read_friday_release_checkpoint_signoff_ledger,
    read_friday_release_closure_ledger, read_friday_release_continuity_journal,
    read_friday_release_escalation_ledger, read_friday_release_external_receipt_archive,
    read_friday_release_handoff_audit_trail, read_friday_release_handoff_completion_ledger,
    read_friday_release_handoff_dispatch_audit_trail, read_friday_release_incident_archive,
    read_friday_release_learning_register, read_friday_release_outbound_review_ledger,
    read_friday_release_promotion_ledger, run_friday_ocr_smoke, run_friday_screenshot_vlm_handoff,
    run_friday_trusted_host_command, run_friday_trusted_host_command_bridge,
    run_friday_vlm_contract, write_friday_release_checkpoint_evidence_vault,
    write_friday_release_checkpoint_review_board_report,
    write_friday_release_checkpoint_signoff_ledger, write_friday_release_closure_ledger,
    write_friday_release_continuity_journal, write_friday_release_deployment_gate,
    write_friday_release_escalation_ledger, write_friday_release_evidence_attachment_review,
    write_friday_release_evidence_export_kit, write_friday_release_evidence_sla_monitor_report,
    write_friday_release_external_receipt_archive, write_friday_release_handoff_audit_trail,
    write_friday_release_handoff_completion_ledger,
    write_friday_release_handoff_dispatch_audit_trail,
    write_friday_release_handoff_dispatch_checklist,
    write_friday_release_handoff_dispatch_governance_review,
    write_friday_release_handoff_governance_review, write_friday_release_handoff_packet,
    write_friday_release_incident_archive, write_friday_release_learning_register,
    write_friday_release_operator_checklist, write_friday_release_outbound_review_ledger,
    write_friday_release_owner_followup_board_report,
    write_friday_release_post_promotion_monitor_report,
    write_friday_release_prevention_plan_report, write_friday_release_publication_control,
    write_friday_release_qa_command_center_report,
    write_friday_release_receipt_review_board_report, write_friday_release_recovery_runbook_report,
    write_friday_release_rollback_drill_report, write_friday_release_stability_board_report,
    write_friday_trusted_host_live_runner_state, write_friday_trusted_runner_release_package,
    write_friday_trusted_runner_release_timeline,
};
use crate::models::{
    FLOW_CODING_MODEL_KEY, FLOW_HELPER_MODEL_KEY, FLOW_QUALITY_CHAT_MODEL_KEY, FLOW_TOOL_MODEL_KEY,
    GenerationMetrics, GlmOcr, KokoroTTS, LocalLlm, LocalSttEngine,
};
use crate::runtime::{
    BrokerRequest, ExecutionPlan, Modality, RuntimeBroker, wake_command_definitions,
};
use crate::search::{MetasearchApiResponse, MetasearchServerConfig};
use crate::workspace::dx_project_statuses;
use crate::writing::HarperGrammarChecker;

const DEFAULT_UI_MODEL_KEY: &str = "qwendean-4b-q4km";
const QWEN3_06B_MODEL_KEY: &str = "qwen3-0.6b";
const QWEN3_06B_MODEL_REPO: &str = "jc-builds/Qwen3-0.6B-Q4_K_M-GGUF";
const QWEN3_06B_MODEL_FILE: &str = "Qwen3-0.6B-Q4_K_M.gguf";
const QWEN3_06B_MODEL_PATH: &str = "models/llm/Qwen3-0.6B-Q4_K_M.gguf";
const WEBGEN_MODEL_KEY: &str = "webgen-4b-preview-i1-q4km";
const WEBGEN_MODEL_REPO: &str = "mradermacher/WEBGEN-4B-Preview-i1-GGUF";
const WEBGEN_MODEL_FILE: &str = "WEBGEN-4B-Preview.i1-Q4_K_M.gguf";
const WEBGEN_MODEL_PATH: &str = "models/llm/WEBGEN-4B-Preview.i1-Q4_K_M.gguf";
const QWENDEAN_MODEL_KEY: &str = "qwendean-4b-q4km";
const QWENDEAN_MODEL_REPO: &str = "iamdyeus/qwendean-4b-GGUF";
const QWENDEAN_MODEL_FILE: &str = "Qwendean-4B.Q4_K_M.gguf";
const QWENDEAN_MODEL_PATH: &str = "models/llm/Qwendean-4B.Q4_K_M.gguf";
const QWEN35_9B_MODEL_KEY: &str = "qwen35-9b-q4km";
const QWEN35_9B_MODEL_REPO: &str = "jc-builds/Qwen3.5-9B-Q4_K_M-GGUF";
const QWEN35_9B_MODEL_FILE: &str = "Qwen3.5-9B-Q4_K_M.gguf";
const QWEN35_9B_MODEL_PATH: &str = "models/llm/Qwen3.5-9B-Q4_K_M.gguf";
const QWEN35_4B_REVISED_MODEL_KEY: &str = "qwen35-4b-revised-q4km";
const QWEN35_4B_REVISED_MODEL_REPO: &str = "Smoffyy/Qwen3.5-4B-Instruct-Revised-GGUF";
const QWEN35_4B_REVISED_MODEL_FILE: &str = "Qwen3.5-4B-q4_k_m.gguf";
const QWEN35_4B_REVISED_MODEL_PATH: &str = "models/llm/Qwen3.5-4B-q4_k_m.gguf";
const XLAM2_3B_TOOL_MODEL_KEY: &str = "xlam2-3b-fc-r-q4km";
const XLAM2_3B_TOOL_MODEL_REPO: &str = "Salesforce/xLAM-2-3b-fc-r-gguf";
const XLAM2_3B_TOOL_MODEL_FILE: &str = "xLAM-2-3B-fc-r-Q4_K_M.gguf";
const XLAM2_3B_TOOL_MODEL_PATH: &str = "models/llm/xLAM-2-3B-fc-r-Q4_K_M.gguf";
const MINISTRAL3_3B_MODEL_KEY: &str = "ministral3-3b-instruct-q4km";
const MINISTRAL3_3B_MODEL_REPO: &str = "unsloth/Ministral-3-3B-Instruct-2512-GGUF";
const MINISTRAL3_3B_MODEL_FILE: &str = "Ministral-3-3B-Instruct-2512-Q4_K_M.gguf";
const MINISTRAL3_3B_MODEL_PATH: &str = "models/llm/Ministral-3-3B-Instruct-2512-Q4_K_M.gguf";
const GRANITE4_H_MICRO_MODEL_KEY: &str = "granite4-h-micro-q4km";
const GRANITE4_H_MICRO_MODEL_REPO: &str = "ibm-granite/granite-4.0-h-micro-GGUF";
const GRANITE4_H_MICRO_MODEL_FILE: &str = "granite-4.0-h-micro-Q4_K_M.gguf";
const GRANITE4_H_MICRO_MODEL_PATH: &str = "models/llm/granite-4.0-h-micro-Q4_K_M.gguf";
const PHI4_MINI_MODEL_KEY: &str = "phi4-mini-instruct-q4km";
const PHI4_MINI_MODEL_REPO: &str = "DuoNeural/Phi-4-mini-instruct-GGUF";
const PHI4_MINI_MODEL_FILE: &str = "Phi-4-mini-instruct-Q4_K_M.gguf";
const PHI4_MINI_MODEL_PATH: &str = "models/llm/Phi-4-mini-instruct-Q4_K_M.gguf";
const SMOLLM3_3B_MODEL_KEY: &str = "smollm3-3b-q4km";
const SMOLLM3_3B_MODEL_REPO: &str = "ggml-org/SmolLM3-3B-GGUF";
const SMOLLM3_3B_MODEL_FILE: &str = "SmolLM3-Q4_K_M.gguf";
const SMOLLM3_3B_MODEL_PATH: &str = "models/llm/SmolLM3-Q4_K_M.gguf";
const GEMMA4_FRONTEND_MODEL_KEY: &str = "gemma4-e4b-frontend-q4km";
const GEMMA4_FRONTEND_MODEL_REPO: &str = "DuoNeural/Gemma-4-E4B-Frontend-GGUF";
const GEMMA4_FRONTEND_MODEL_FILE: &str = "gemma-4-E4B-it.Q4_K_M.gguf";
const GEMMA4_FRONTEND_MODEL_PATH: &str = "models/llm/gemma-4-E4B-it.Q4_K_M.gguf";
const GEMMA4_FRONTEND_MMPROJ_FILE: &str = "gemma-4-E4B-it.BF16-mmproj.gguf";
const GEMMA4_FRONTEND_MMPROJ_PATH: &str = "models/llm/gemma-4-E4B-it.BF16-mmproj.gguf";
const UIGEN_FX_MODEL_KEY: &str = "uigen-fx-4b-preview-q4km";
const UIGEN_FX_MODEL_REPO: &str = "QuantFactory/UIGEN-FX-4B-Preview-GGUF";
const UIGEN_FX_MODEL_FILE: &str = "UIGEN-FX-4B-Preview.Q4_K_M.gguf";
const UIGEN_FX_MODEL_PATH: &str = "models/llm/UIGEN-FX-4B-Preview.Q4_K_M.gguf";
const UIMODEL_GOOGLE_OUTPUT_ROOT: &str = "tmp/uigen-google";
const UIVISION_GOOGLE_OUTPUT_ROOT: &str = "tmp/uigen-vision-google";
const PARAKEET_STT_KEY: &str = "parakeet-tdt-0.6b-v3-int8";
const PARAKEET_STT_PATH: &str = "models/stt/parakeet-tdt-0.6b-v3-int8/encoder.int8.onnx";
const GOOGLE_URL: &str = "https://www.google.com/?hl=en&gl=US&pws=0";
const GOOGLE_HOMEPAGE_PROMPT: &str = "\
Make a responsive Google.com homepage look-alike for screenshot evaluation only.
Do not use Google image assets, fonts, scripts, CDNs, or external assets.
Use a plain text multicolor wordmark spelling Google centered in the first viewport, a 582px rounded search box with a subtle border and shadow, two light gray buttons labeled Google Search and I'm Feeling Lucky, top-right text navigation with Gmail, Images, an apps icon placeholder, and an avatar circle, and a two-row bottom footer with realistic links.
Desktop layout: large whitespace, centered search composition around the vertical middle, footer pinned to bottom, top nav at the top-right.
Mobile layout: same centered search composition, narrower search field, footer links wrapping cleanly.
Keep it simple, accurate, complete, and screenshot-stable.
";
const GOOGLE_VISION_PROMPT: &str = "\
Use the screenshot as the source of truth and recreate the visible search homepage as a complete standalone HTML/CSS file for evaluation only.
Return only HTML code.
Do not use external scripts, external fonts, CDNs, images, or brand assets.
Use semantic HTML and inline CSS in one <style> tag.
Keep CSS compact and do not repeat selectors.
Keep the whole file under 180 lines.
Match the visible layout: top navigation, centered logo/search composition, search controls, bottom footer, spacing, and mobile responsiveness.
Include </body> and </html>.
";

/// Execute CLI command
pub async fn execute(command: Command) -> Result<()> {
    match command {
        Command::Transcribe { file } => {
            let broker = RuntimeBroker::detect();
            let mut stt = LocalSttEngine::from_broker(&broker)?;
            let text = stt.transcribe(&file)?;
            println!("[stt] {}", text);
        }

        Command::Wispr { file } => {
            println!("Flow Wispr Mode");
            println!("================");
            println!("[input] {}", file);

            let broker = RuntimeBroker::detect();
            let mut stt = LocalSttEngine::from_broker(&broker)?;
            let raw_text = stt.transcribe(&file)?;
            println!("[stt/raw] {}", raw_text);

            let llm = load_chat_llm(&broker).await?;
            let cleaned = llm.clean_speech(&raw_text).await?;

            println!("\n[cleaned]");
            println!("{}", cleaned);
        }

        Command::Speak { text } => {
            let mut tts = KokoroTTS::new_async().await?;
            tts.speak(&text)?;
            println!("[tts] saved to output.wav");
        }

        Command::Live => {
            run_live_mode().await?;
        }

        Command::Dictate => {
            run_dictation_mode().await?;
        }

        Command::Interactive => {
            print_interactive_help();
        }

        Command::Chat { model } => {
            crate::cli::chat::run_chat(model).await?;
        }

        Command::ToolAgent { tools, request } => {
            run_tool_agent(tools.as_deref(), &request).await?;
        }

        Command::VerifyLocalModels => {
            verify_local_models().await?;
        }

        Command::Ocr { image, prompt } => {
            println!("Flow OCR");
            println!("========");

            let ocr = GlmOcr::new()?;
            let result = if let Some(custom_prompt) = prompt {
                ocr.ocr_with_prompt(&image, &custom_prompt)?
            } else {
                ocr.ocr_image(&image)?
            };

            println!("{}", result);
        }

        Command::Profile => {
            print_profile(&RuntimeBroker::detect());
        }

        Command::Projects => {
            print_projects();
        }

        Command::Scorecard => {
            print_scorecard();
        }

        Command::FridayPlan => {
            print_friday_plan();
        }

        Command::FridayPlanJson => {
            print_friday_plan_json()?;
        }

        Command::FridaySearchPlan { query } => {
            print_search_request_plan(
                "Friday Answer Search Plan",
                &friday_answer_search_plan(query),
            )?;
        }

        Command::FridayResearchPlan { query } => {
            print_search_request_plan(
                "Friday Deep Research Plan",
                &friday_research_search_plan(query),
            )?;
        }

        Command::FridayResearchWorkflow { query } => {
            print_friday_research_workflow(&FridayResearchWorkflow::for_query(query));
        }

        Command::FridayResearchWorkflowJson { query } => {
            println!(
                "{}",
                FridayResearchWorkflow::for_query(query).to_pretty_json()?
            );
        }

        Command::FridayMetasearch { query } => {
            let response = MetasearchServerConfig::default()
                .search_blocking(&friday_answer_search_plan(query))?;
            print_friday_metasearch_response(&response);
        }

        Command::FridayMetasearchJson { query } => {
            let response = MetasearchServerConfig::default()
                .search_blocking(&friday_answer_search_plan(query))?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }

        Command::FridayResearchReport { query } => {
            let response = MetasearchServerConfig::default()
                .search_blocking(&friday_research_search_plan(query))?;
            let report = FridayResearchReport::from_metasearch_response(&response);
            println!("{}", report.to_markdown());
        }

        Command::FridayResearchReportSave { output_dir, query } => {
            let response = MetasearchServerConfig::default()
                .search_blocking(&friday_research_search_plan(query))?;
            let report = FridayResearchReport::from_metasearch_response(&response);
            let manifest = report.write_bundle(resolve_repo_relative_path(&output_dir))?;
            println!("Friday Research Bundle");
            println!("======================");
            println!("Report: {}", manifest.report_markdown.display());
            println!("Citations: {}", manifest.citations_json.display());
            println!("Source groups: {}", manifest.source_groups_json.display());
            println!("Events: {}", manifest.events_json.display());
            println!("Manifest: {}", manifest.manifest_json.display());
        }

        Command::FridayResearchSynthesize { query } => {
            let response = MetasearchServerConfig::default()
                .search_blocking(&friday_research_search_plan(query))?;
            let report = FridayResearchReport::from_metasearch_response(&response);
            let runtime = crate::FlowLocalRuntime::detect()?;
            let synthesized = report.synthesize_with_runtime(&runtime).await?;
            println!("Friday Research Answer");
            println!("======================");
            println!("{}", synthesized.answer);
            println!();
            println!("Citations: {}", synthesized.citation_ids.join(", "));
            println!("Deltas: {}", synthesized.deltas.len());
            if let Some(generation) = synthesized.generation {
                println!(
                    "Generation: {} tokens at {:.2} tok/s in {} ms",
                    generation.generated_tokens,
                    generation.tokens_per_second,
                    generation.total_time_ms
                );
            }
        }

        Command::FridayWorkspaceInit { output_dir } => {
            let store = FridayWorkspaceStore::seed_local_first();
            let snapshot = store.write_to_dir(resolve_repo_relative_path(&output_dir))?;
            println!("Friday Workspace Store");
            println!("======================");
            println!("Root: {}", snapshot.root_dir.display());
            println!("Projects: {}", snapshot.projects_json.display());
            println!("Memories: {}", snapshot.memories_json.display());
            println!("Connectors: {}", snapshot.connectors_json.display());
            println!("Manifest: {}", snapshot.manifest_json.display());
            println!(
                "Counts: {} projects, {} memories, {} connectors",
                snapshot.manifest.project_count,
                snapshot.manifest.memory_count,
                snapshot.manifest.connector_count
            );
            if snapshot.manifest.findings.is_empty() {
                println!("Permission findings: none");
            } else {
                println!("Permission findings:");
                for finding in &snapshot.manifest.findings {
                    println!("  - {:?}: {}", finding.severity, finding.message);
                }
            }
        }

        Command::FridayWorkspaceJson { input_dir } => {
            let store = if let Some(input_dir) = input_dir {
                FridayWorkspaceStore::read_from_dir(resolve_repo_relative_path(&input_dir))?
            } else {
                FridayWorkspaceStore::seed_local_first()
            };
            println!("{}", store.to_pretty_json()?);
        }

        Command::FridayArtifactsInit { output_dir } => {
            let store = FridayArtifactStore::seed_for_local_workspace();
            let snapshot = store.write_to_dir(resolve_repo_relative_path(&output_dir))?;
            println!("Friday Artifact Store");
            println!("=====================");
            println!("Root: {}", snapshot.root_dir.display());
            println!("Artifacts: {}", snapshot.artifacts_json.display());
            println!("Checkpoints: {}", snapshot.checkpoints_json.display());
            println!("Diffs: {}", snapshot.diffs_json.display());
            println!("Code tasks: {}", snapshot.code_tasks_json.display());
            println!("Manifest: {}", snapshot.manifest_json.display());
            println!(
                "Counts: {} artifacts, {} checkpoints, {} diffs, {} code tasks",
                snapshot.manifest.artifact_count,
                snapshot.manifest.checkpoint_count,
                snapshot.manifest.diff_count,
                snapshot.manifest.code_task_count
            );
            if snapshot.manifest.findings.is_empty() {
                println!("Artifact findings: none");
            } else {
                println!("Artifact findings:");
                for finding in &snapshot.manifest.findings {
                    println!("  - {}", finding.message);
                }
            }
        }

        Command::FridayArtifactsJson { input_dir } => {
            let store = if let Some(input_dir) = input_dir {
                FridayArtifactStore::read_from_dir(resolve_repo_relative_path(&input_dir))?
            } else {
                FridayArtifactStore::seed_for_local_workspace()
            };
            println!("{}", store.to_pretty_json()?);
        }

        Command::FridayArtifactsIndexMultimodal {
            store_dir,
            bundle_dir,
        } => {
            let report = index_friday_multimodal_artifact_bundle(&store_dir, &bundle_dir)?;
            print_friday_multimodal_artifact_import(&report);
        }

        Command::FridayArtifactsIndexMultimodalJson {
            store_dir,
            bundle_dir,
        } => {
            let report = index_friday_multimodal_artifact_bundle(&store_dir, &bundle_dir)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }

        Command::FridayRuntimeInit { output_dir } => {
            let store = FridayRuntimeSurfaceStore::seed_local_first();
            let snapshot = store.write_to_dir(resolve_repo_relative_path(&output_dir))?;
            println!("Friday Runtime Surface Store");
            println!("============================");
            println!("Root: {}", snapshot.root_dir.display());
            println!("Voice: {}", snapshot.voice_json.display());
            println!("Multimodal: {}", snapshot.multimodal_json.display());
            println!("Automations: {}", snapshot.automations_json.display());
            println!("Manifest: {}", snapshot.manifest_json.display());
            println!(
                "Counts: {} multimodal surfaces, {} automations",
                snapshot.manifest.multimodal_count, snapshot.manifest.automation_count
            );
            if snapshot.manifest.findings.is_empty() {
                println!("Runtime findings: none");
            } else {
                println!("Runtime findings:");
                for finding in &snapshot.manifest.findings {
                    println!("  - {}", finding.message);
                }
            }
        }

        Command::FridayRuntimeJson { input_dir } => {
            let store = if let Some(input_dir) = input_dir {
                FridayRuntimeSurfaceStore::read_from_dir(resolve_repo_relative_path(&input_dir))?
            } else {
                FridayRuntimeSurfaceStore::seed_local_first()
            };
            println!("{}", store.to_pretty_json()?);
        }

        Command::FridayUiPlan => {
            print_friday_ui_plan();
        }

        Command::FridayUiPlanJson => {
            println!("{}", default_friday_ui_integration_plan().to_pretty_json()?);
        }

        Command::FridayLiveUiRoutes => {
            print_friday_live_ui_routes();
        }

        Command::FridayLiveUiRoutesJson => {
            println!(
                "{}",
                friday_live_ui_route_binding_report().to_pretty_json()?
            );
        }

        Command::FridayReadiness => {
            print_friday_readiness();
        }

        Command::FridayReadinessJson => {
            println!("{}", friday_operator_readiness_report().to_pretty_json()?);
        }

        Command::FridayRouteVisuals => {
            print_friday_route_visuals();
        }

        Command::FridayRouteVisualsJson => {
            println!("{}", friday_route_visual_report().to_pretty_json()?);
        }

        Command::FridayExecutionHandoffs => {
            print_friday_execution_handoffs();
        }

        Command::FridayExecutionHandoffsJson => {
            println!("{}", friday_execution_handoff_report().to_pretty_json()?);
        }

        Command::FridayDashboardExport { output_dir } => {
            print_friday_dashboard_export(&output_dir)?;
        }

        Command::FridayDashboardExportJson { output_dir } => {
            let bundle = export_friday_dashboard_bundle(resolve_repo_relative_path(&output_dir))?;
            println!("{}", bundle.to_pretty_json()?);
        }

        Command::FridayDashboardPanel { input_dir } => {
            print_friday_dashboard_panel(&input_dir)?;
        }

        Command::FridayDashboardPanelJson { input_dir } => {
            let panel = friday_dashboard_panel_from_export(resolve_repo_relative_path(&input_dir))?;
            println!("{}", panel.to_pretty_json()?);
        }

        Command::FridayDashboardProductUi { input_dir } => {
            print_friday_dashboard_product_ui(&input_dir)?;
        }

        Command::FridayDashboardProductUiJson { input_dir } => {
            let binding = friday_dashboard_product_ui_binding_from_export(
                resolve_repo_relative_path(&input_dir),
            )?;
            println!("{}", binding.to_pretty_json()?);
        }

        Command::FridayDashboardProductUiSmoke { input_dir } => {
            print_friday_dashboard_product_ui_smoke(&input_dir)?;
        }

        Command::FridayDashboardProductUiSmokeJson { input_dir } => {
            let report = friday_dashboard_product_ui_smoke_from_export(
                resolve_repo_relative_path(&input_dir),
            )?;
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayDashboardHostCommandBridge { input_dir } => {
            print_friday_dashboard_host_command_bridge(&input_dir)?;
        }

        Command::FridayDashboardHostCommandBridgeJson { input_dir } => {
            let report = friday_dashboard_host_command_bridge_from_export(
                resolve_repo_relative_path(&input_dir),
            )?;
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayTrustedHostRunner {
            input_dir,
            action_id,
            approve,
            execute,
            cancel,
            history_file,
            reason,
        } => {
            let result = run_friday_trusted_host_runner_command(
                &input_dir,
                action_id.as_deref(),
                approve,
                execute,
                cancel,
                &history_file,
                reason.as_deref(),
            )?;
            print_friday_trusted_host_runner_result(&result);
        }

        Command::FridayTrustedHostRunnerJson {
            input_dir,
            action_id,
            approve,
            execute,
            cancel,
            history_file,
            reason,
        } => {
            let result = run_friday_trusted_host_runner_command(
                &input_dir,
                action_id.as_deref(),
                approve,
                execute,
                cancel,
                &history_file,
                reason.as_deref(),
            )?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Command::FridayTrustedHostRunnerUx {
            history_file,
            release_review_file,
        } => {
            let report = friday_trusted_host_runner_ux_report_from_history_file(
                resolve_repo_relative_path(&history_file),
                resolve_repo_relative_path(&release_review_file),
            )?;
            print_friday_trusted_host_runner_ux_report(&report);
        }

        Command::FridayTrustedHostRunnerUxJson {
            history_file,
            release_review_file,
        } => {
            let report = friday_trusted_host_runner_ux_report_from_history_file(
                resolve_repo_relative_path(&history_file),
                resolve_repo_relative_path(&release_review_file),
            )?;
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayTrustedHostRunnerApprovalUi {
            history_file,
            release_review_file,
        } => {
            let report = friday_trusted_host_runner_approval_ui_report_from_history_file(
                resolve_repo_relative_path(&history_file),
                resolve_repo_relative_path(&release_review_file),
            )?;
            print_friday_trusted_host_runner_approval_ui_report(&report);
        }

        Command::FridayTrustedHostRunnerApprovalUiJson {
            history_file,
            release_review_file,
        } => {
            let report = friday_trusted_host_runner_approval_ui_report_from_history_file(
                resolve_repo_relative_path(&history_file),
                resolve_repo_relative_path(&release_review_file),
            )?;
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayTrustedHostRunnerCancellationUx { state_file } => {
            let report = friday_trusted_host_runner_cancellation_ux_report_from_state_file(
                resolve_repo_relative_path(&state_file),
            )?;
            print_friday_trusted_host_runner_cancellation_ux_report(&report);
        }

        Command::FridayTrustedHostRunnerCancellationUxJson { state_file } => {
            let report = friday_trusted_host_runner_cancellation_ux_report_from_state_file(
                resolve_repo_relative_path(&state_file),
            )?;
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayTrustedHostRunnerOperatorReview {
            history_file,
            status,
            action_id,
            since_ms,
            until_ms,
            limit,
        } => {
            let filter = trusted_host_runner_review_filter(
                &status, &action_id, &since_ms, &until_ms, limit,
            )?;
            let report = friday_trusted_host_runner_operator_review_report_from_history_file(
                resolve_repo_relative_path(&history_file),
                filter,
            )?;
            print_friday_trusted_host_runner_operator_review_report(&report);
        }

        Command::FridayTrustedHostRunnerOperatorReviewJson {
            history_file,
            status,
            action_id,
            since_ms,
            until_ms,
            limit,
        } => {
            let filter = trusted_host_runner_review_filter(
                &status, &action_id, &since_ms, &until_ms, limit,
            )?;
            let report = friday_trusted_host_runner_operator_review_report_from_history_file(
                resolve_repo_relative_path(&history_file),
                filter,
            )?;
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayTrustedHostRunnerReleasePackage {
            export_dir,
            history_file,
            state_file,
            output_file,
        } => {
            let report = friday_trusted_runner_release_package_report(
                resolve_repo_relative_path(&export_dir),
                resolve_repo_relative_path(&history_file),
                resolve_repo_relative_path(&state_file),
                resolve_repo_relative_path(&output_file),
            );
            write_friday_trusted_runner_release_package(
                resolve_repo_relative_path(&output_file),
                &report,
            )?;
            print_friday_trusted_runner_release_package_report(&report);
        }

        Command::FridayTrustedHostRunnerReleasePackageJson {
            export_dir,
            history_file,
            state_file,
            output_file,
        } => {
            let report = friday_trusted_runner_release_package_report(
                resolve_repo_relative_path(&export_dir),
                resolve_repo_relative_path(&history_file),
                resolve_repo_relative_path(&state_file),
                resolve_repo_relative_path(&output_file),
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayTrustedRunnerReleaseArchive {
            timeline_file,
            package_file,
        } => {
            let timeline = append_friday_trusted_runner_release_package_to_timeline(
                resolve_repo_relative_path(&timeline_file),
                resolve_repo_relative_path(&package_file),
            )?;
            print_friday_trusted_runner_release_timeline(&timeline);
        }

        Command::FridayTrustedRunnerReleaseTimelineJson {
            timeline_file,
            package_files,
        } => {
            let package_paths = package_files
                .iter()
                .map(|path| resolve_repo_relative_path(path))
                .collect::<Vec<_>>();
            let timeline = friday_trusted_runner_release_timeline_report(
                resolve_repo_relative_path(&timeline_file),
                &package_paths,
            );
            println!("{}", timeline.to_pretty_json()?);
        }

        Command::FridayTrustedRunnerReleaseTimeline {
            timeline_file,
            package_files,
        } => {
            let package_paths = package_files
                .iter()
                .map(|path| resolve_repo_relative_path(path))
                .collect::<Vec<_>>();
            let timeline = friday_trusted_runner_release_timeline_report(
                resolve_repo_relative_path(&timeline_file),
                &package_paths,
            );
            write_friday_trusted_runner_release_timeline(
                resolve_repo_relative_path(&timeline_file),
                &timeline,
            )?;
            print_friday_trusted_runner_release_timeline(&timeline);
        }

        Command::FridayReleaseChecklist {
            checklist_file,
            package_file,
            timeline_file,
            export_dir,
            todo_file,
            changelog_file,
            signoff_file,
        } => {
            let report = friday_release_operator_checklist_report(
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&package_file),
                resolve_repo_relative_path(&timeline_file),
                resolve_repo_relative_path(&export_dir),
                resolve_repo_relative_path(&todo_file),
                resolve_repo_relative_path(&changelog_file),
                resolve_repo_relative_path(&signoff_file),
            );
            write_friday_release_operator_checklist(
                resolve_repo_relative_path(&checklist_file),
                &report,
            )?;
            print_friday_release_operator_checklist(&report);
        }

        Command::FridayReleaseChecklistJson {
            checklist_file,
            package_file,
            timeline_file,
            export_dir,
            todo_file,
            changelog_file,
            signoff_file,
        } => {
            let report = friday_release_operator_checklist_report(
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&package_file),
                resolve_repo_relative_path(&timeline_file),
                resolve_repo_relative_path(&export_dir),
                resolve_repo_relative_path(&todo_file),
                resolve_repo_relative_path(&changelog_file),
                resolve_repo_relative_path(&signoff_file),
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseSignoff {
            checklist_file,
            signoff_file,
            operator,
            decision,
            reason,
        } => {
            let decision = friday_release_signoff_decision(&decision)?;
            let signoffs = append_friday_release_operator_signoff(
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&signoff_file),
                &operator,
                decision,
                &reason,
            )?;
            print_friday_release_signoffs(&signoffs);
        }

        Command::FridayReleaseSignoffJson {
            checklist_file,
            signoff_file,
            operator,
            decision,
            reason,
        } => {
            let decision = friday_release_signoff_decision(&decision)?;
            let signoffs = append_friday_release_operator_signoff(
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&signoff_file),
                &operator,
                decision,
                &reason,
            )?;
            println!("{}", serde_json::to_string_pretty(&signoffs)?);
        }

        Command::FridayReleaseQa {
            report_file,
            checklist_file,
            package_file,
            timeline_file,
            cargo_check_result_file,
            extension_typecheck_result_file,
            dashboard_smoke_result_file,
        } => {
            let report = friday_release_qa_command_center_report(
                resolve_repo_relative_path(&report_file),
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&package_file),
                resolve_repo_relative_path(&timeline_file),
                resolve_repo_relative_path(&cargo_check_result_file),
                resolve_repo_relative_path(&extension_typecheck_result_file),
                resolve_repo_relative_path(&dashboard_smoke_result_file),
            );
            write_friday_release_qa_command_center_report(
                resolve_repo_relative_path(&report_file),
                &report,
            )?;
            print_friday_release_qa_command_center(&report);
        }

        Command::FridayReleaseQaJson {
            report_file,
            checklist_file,
            package_file,
            timeline_file,
            cargo_check_result_file,
            extension_typecheck_result_file,
            dashboard_smoke_result_file,
        } => {
            let report = friday_release_qa_command_center_report(
                resolve_repo_relative_path(&report_file),
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&package_file),
                resolve_repo_relative_path(&timeline_file),
                resolve_repo_relative_path(&cargo_check_result_file),
                resolve_repo_relative_path(&extension_typecheck_result_file),
                resolve_repo_relative_path(&dashboard_smoke_result_file),
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseExportKit {
            kit_file,
            export_dir,
            checklist_file,
            qa_file,
            package_file,
            timeline_file,
            signoff_file,
            cargo_check_result_file,
            extension_typecheck_result_file,
            dashboard_smoke_result_file,
        } => {
            let report = friday_release_evidence_export_kit_report(
                resolve_repo_relative_path(&kit_file),
                resolve_repo_relative_path(&export_dir),
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&qa_file),
                resolve_repo_relative_path(&package_file),
                resolve_repo_relative_path(&timeline_file),
                resolve_repo_relative_path(&signoff_file),
                resolve_repo_relative_path(&cargo_check_result_file),
                resolve_repo_relative_path(&extension_typecheck_result_file),
                resolve_repo_relative_path(&dashboard_smoke_result_file),
            );
            write_friday_release_evidence_export_kit(
                resolve_repo_relative_path(&kit_file),
                &report,
            )?;
            print_friday_release_evidence_export_kit(&report);
        }

        Command::FridayReleaseExportKitJson {
            kit_file,
            export_dir,
            checklist_file,
            qa_file,
            package_file,
            timeline_file,
            signoff_file,
            cargo_check_result_file,
            extension_typecheck_result_file,
            dashboard_smoke_result_file,
        } => {
            let report = friday_release_evidence_export_kit_report(
                resolve_repo_relative_path(&kit_file),
                resolve_repo_relative_path(&export_dir),
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&qa_file),
                resolve_repo_relative_path(&package_file),
                resolve_repo_relative_path(&timeline_file),
                resolve_repo_relative_path(&signoff_file),
                resolve_repo_relative_path(&cargo_check_result_file),
                resolve_repo_relative_path(&extension_typecheck_result_file),
                resolve_repo_relative_path(&dashboard_smoke_result_file),
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseDeploymentGate {
            gate_file,
            export_dir,
            export_kit_file,
            qa_file,
            checklist_file,
            package_file,
            timeline_file,
            target_id,
            target_label,
            environment,
            provider,
            target_url,
            local_only_required,
            requires_vercel,
            expected_product_name,
            rollback_note,
        } => {
            let target = FridayReleaseDeploymentTarget {
                id: target_id,
                label: target_label,
                environment,
                provider,
                url: target_url,
                local_only_required,
                requires_vercel,
                expected_product_name,
                rollback_note,
            };
            let report = friday_release_deployment_gate_report(
                resolve_repo_relative_path(&gate_file),
                resolve_repo_relative_path(&export_kit_file),
                resolve_repo_relative_path(&qa_file),
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&package_file),
                resolve_repo_relative_path(&timeline_file),
                resolve_repo_relative_path(&export_dir),
                target,
            );
            write_friday_release_deployment_gate(resolve_repo_relative_path(&gate_file), &report)?;
            print_friday_release_deployment_gate(&report);
        }

        Command::FridayReleaseDeploymentGateJson {
            gate_file,
            export_dir,
            export_kit_file,
            qa_file,
            checklist_file,
            package_file,
            timeline_file,
            target_id,
            target_label,
            environment,
            provider,
            target_url,
            local_only_required,
            requires_vercel,
            expected_product_name,
            rollback_note,
        } => {
            let target = FridayReleaseDeploymentTarget {
                id: target_id,
                label: target_label,
                environment,
                provider,
                url: target_url,
                local_only_required,
                requires_vercel,
                expected_product_name,
                rollback_note,
            };
            let report = friday_release_deployment_gate_report(
                resolve_repo_relative_path(&gate_file),
                resolve_repo_relative_path(&export_kit_file),
                resolve_repo_relative_path(&qa_file),
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&package_file),
                resolve_repo_relative_path(&timeline_file),
                resolve_repo_relative_path(&export_dir),
                target,
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseCandidateArchive {
            archive_file,
            gate_files,
        } => {
            let archive = run_friday_release_candidate_archive(&archive_file, &gate_files, true)?;
            print_friday_release_candidate_archive(&archive);
        }

        Command::FridayReleaseCandidateArchiveJson {
            archive_file,
            gate_files,
        } => {
            let archive = run_friday_release_candidate_archive(&archive_file, &gate_files, false)?;
            println!("{}", archive.to_pretty_json()?);
        }

        Command::FridayReleasePromotionLedger {
            ledger_file,
            archive_file,
            candidate_id,
            decision,
            operator,
            reason,
            deployment_note,
            rollback_reference,
            post_check_files,
        } => {
            let ledger = run_friday_release_promotion_ledger(
                &ledger_file,
                &archive_file,
                candidate_id,
                &decision,
                &operator,
                &reason,
                &deployment_note,
                &rollback_reference,
                post_check_files,
            )?;
            print_friday_release_promotion_ledger(&ledger);
        }

        Command::FridayReleasePromotionLedgerJson {
            ledger_file,
            archive_file,
            candidate_id,
            decision,
            operator,
            reason,
            deployment_note,
            rollback_reference,
            post_check_files,
        } => {
            let ledger = run_friday_release_promotion_ledger(
                &ledger_file,
                &archive_file,
                candidate_id,
                &decision,
                &operator,
                &reason,
                &deployment_note,
                &rollback_reference,
                post_check_files,
            )?;
            println!("{}", ledger.to_pretty_json()?);
        }

        Command::FridayReleasePostPromotionMonitor {
            monitor_file,
            promotion_ledger_file,
            qa_file,
            dashboard_smoke_result_file,
            incident_note_files,
        } => {
            let report = friday_release_post_promotion_monitor_report(
                resolve_repo_relative_path(&monitor_file),
                resolve_repo_relative_path(&promotion_ledger_file),
                resolve_repo_relative_path(&qa_file),
                resolve_repo_relative_path(&dashboard_smoke_result_file),
                incident_note_files,
            );
            write_friday_release_post_promotion_monitor_report(
                resolve_repo_relative_path(&monitor_file),
                &report,
            )?;
            print_friday_release_post_promotion_monitor(&report);
        }

        Command::FridayReleasePostPromotionMonitorJson {
            monitor_file,
            promotion_ledger_file,
            qa_file,
            dashboard_smoke_result_file,
            incident_note_files,
        } => {
            let report = friday_release_post_promotion_monitor_report(
                resolve_repo_relative_path(&monitor_file),
                resolve_repo_relative_path(&promotion_ledger_file),
                resolve_repo_relative_path(&qa_file),
                resolve_repo_relative_path(&dashboard_smoke_result_file),
                incident_note_files,
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseRollbackDrill {
            drill_file,
            post_promotion_monitor_file,
            promotion_ledger_file,
            candidate_archive_file,
            deployment_gate_file,
            rollback_command,
            operator,
            reason,
        } => {
            let report = friday_release_rollback_drill_report(
                resolve_repo_relative_path(&drill_file),
                resolve_repo_relative_path(&post_promotion_monitor_file),
                resolve_repo_relative_path(&promotion_ledger_file),
                resolve_repo_relative_path(&candidate_archive_file),
                resolve_repo_relative_path(&deployment_gate_file),
                rollback_command,
                operator,
                reason,
            );
            write_friday_release_rollback_drill_report(
                resolve_repo_relative_path(&drill_file),
                &report,
            )?;
            print_friday_release_rollback_drill(&report);
        }

        Command::FridayReleaseRollbackDrillJson {
            drill_file,
            post_promotion_monitor_file,
            promotion_ledger_file,
            candidate_archive_file,
            deployment_gate_file,
            rollback_command,
            operator,
            reason,
        } => {
            let report = friday_release_rollback_drill_report(
                resolve_repo_relative_path(&drill_file),
                resolve_repo_relative_path(&post_promotion_monitor_file),
                resolve_repo_relative_path(&promotion_ledger_file),
                resolve_repo_relative_path(&candidate_archive_file),
                resolve_repo_relative_path(&deployment_gate_file),
                rollback_command,
                operator,
                reason,
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseStabilityBoard {
            board_file,
            qa_file,
            candidate_archive_file,
            promotion_ledger_file,
            post_promotion_monitor_file,
            rollback_drill_file,
            deployment_gate_file,
        } => {
            let report = friday_release_stability_board_report(
                resolve_repo_relative_path(&board_file),
                resolve_repo_relative_path(&qa_file),
                resolve_repo_relative_path(&candidate_archive_file),
                resolve_repo_relative_path(&promotion_ledger_file),
                resolve_repo_relative_path(&post_promotion_monitor_file),
                resolve_repo_relative_path(&rollback_drill_file),
                resolve_repo_relative_path(&deployment_gate_file),
            );
            write_friday_release_stability_board_report(
                resolve_repo_relative_path(&board_file),
                &report,
            )?;
            print_friday_release_stability_board(&report);
        }

        Command::FridayReleaseStabilityBoardJson {
            board_file,
            qa_file,
            candidate_archive_file,
            promotion_ledger_file,
            post_promotion_monitor_file,
            rollback_drill_file,
            deployment_gate_file,
        } => {
            let report = friday_release_stability_board_report(
                resolve_repo_relative_path(&board_file),
                resolve_repo_relative_path(&qa_file),
                resolve_repo_relative_path(&candidate_archive_file),
                resolve_repo_relative_path(&promotion_ledger_file),
                resolve_repo_relative_path(&post_promotion_monitor_file),
                resolve_repo_relative_path(&rollback_drill_file),
                resolve_repo_relative_path(&deployment_gate_file),
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseRecoveryRunbook {
            runbook_file,
            stability_board_file,
            rollback_drill_file,
            promotion_ledger_file,
            post_promotion_monitor_file,
        } => {
            let report = friday_release_recovery_runbook_report(
                resolve_repo_relative_path(&runbook_file),
                resolve_repo_relative_path(&stability_board_file),
                resolve_repo_relative_path(&rollback_drill_file),
                resolve_repo_relative_path(&promotion_ledger_file),
                resolve_repo_relative_path(&post_promotion_monitor_file),
            );
            write_friday_release_recovery_runbook_report(
                resolve_repo_relative_path(&runbook_file),
                &report,
            )?;
            print_friday_release_recovery_runbook(&report);
        }

        Command::FridayReleaseRecoveryRunbookJson {
            runbook_file,
            stability_board_file,
            rollback_drill_file,
            promotion_ledger_file,
            post_promotion_monitor_file,
        } => {
            let report = friday_release_recovery_runbook_report(
                resolve_repo_relative_path(&runbook_file),
                resolve_repo_relative_path(&stability_board_file),
                resolve_repo_relative_path(&rollback_drill_file),
                resolve_repo_relative_path(&promotion_ledger_file),
                resolve_repo_relative_path(&post_promotion_monitor_file),
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseIncidentArchive {
            archive_file,
            runbook_file,
            stability_board_file,
            rollback_drill_file,
            post_promotion_monitor_file,
            incident_note_files,
            outcome,
        } => {
            let archive = append_friday_release_incident_to_archive(
                resolve_repo_relative_path(&archive_file),
                resolve_repo_relative_path(&runbook_file),
                resolve_repo_relative_path(&stability_board_file),
                resolve_repo_relative_path(&rollback_drill_file),
                resolve_repo_relative_path(&post_promotion_monitor_file),
                incident_note_files,
                FridayReleaseIncidentOutcome::parse(&outcome)?,
            )?;
            print_friday_release_incident_archive(&archive);
        }

        Command::FridayReleaseIncidentArchiveJson {
            archive_file,
            runbook_file,
            stability_board_file,
            rollback_drill_file,
            post_promotion_monitor_file,
            incident_note_files,
            outcome,
        } => {
            let mut entries =
                read_friday_release_incident_archive(resolve_repo_relative_path(&archive_file))
                    .map(|archive| archive.entries)
                    .unwrap_or_default();
            entries.push(friday_release_incident_entry_from_sources(
                resolve_repo_relative_path(&runbook_file),
                resolve_repo_relative_path(&stability_board_file),
                resolve_repo_relative_path(&rollback_drill_file),
                resolve_repo_relative_path(&post_promotion_monitor_file),
                incident_note_files,
                FridayReleaseIncidentOutcome::parse(&outcome)?,
            ));
            let archive = friday_release_incident_archive_report(
                resolve_repo_relative_path(&archive_file),
                entries,
            );
            println!("{}", archive.to_pretty_json()?);
        }

        Command::FridayReleaseIncidentArchiveList { archive_file } => {
            let archive =
                read_friday_release_incident_archive(resolve_repo_relative_path(&archive_file))?;
            print_friday_release_incident_archive(&archive);
        }

        Command::FridayReleaseIncidentArchiveExport {
            archive_file,
            output_file,
        } => {
            let archive =
                read_friday_release_incident_archive(resolve_repo_relative_path(&archive_file))?;
            write_friday_release_incident_archive(
                resolve_repo_relative_path(&output_file),
                &archive,
            )?;
            print_friday_release_incident_archive(&archive);
        }

        Command::FridayReleasePreventionPlan {
            plan_file,
            incident_archive_file,
            stability_board_file,
        } => {
            let report = friday_release_prevention_plan_report(
                resolve_repo_relative_path(&plan_file),
                resolve_repo_relative_path(&incident_archive_file),
                resolve_repo_relative_path(&stability_board_file),
            );
            write_friday_release_prevention_plan_report(
                resolve_repo_relative_path(&plan_file),
                &report,
            )?;
            print_friday_release_prevention_plan(&report);
        }

        Command::FridayReleasePreventionPlanJson {
            plan_file,
            incident_archive_file,
            stability_board_file,
        } => {
            let report = friday_release_prevention_plan_report(
                resolve_repo_relative_path(&plan_file),
                resolve_repo_relative_path(&incident_archive_file),
                resolve_repo_relative_path(&stability_board_file),
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseOwnerFollowUpBoard {
            board_file,
            prevention_plan_file,
        } => {
            let report = friday_release_owner_followup_board_report(
                resolve_repo_relative_path(&board_file),
                resolve_repo_relative_path(&prevention_plan_file),
            );
            write_friday_release_owner_followup_board_report(
                resolve_repo_relative_path(&board_file),
                &report,
            )?;
            print_friday_release_owner_followup_board(&report);
        }

        Command::FridayReleaseOwnerFollowUpBoardJson {
            board_file,
            prevention_plan_file,
        } => {
            let report = friday_release_owner_followup_board_report(
                resolve_repo_relative_path(&board_file),
                resolve_repo_relative_path(&prevention_plan_file),
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseEvidenceSlaMonitor {
            monitor_file,
            owner_followup_board_file,
            prevention_plan_file,
            stability_board_file,
        } => {
            let report = friday_release_evidence_sla_monitor_report(
                resolve_repo_relative_path(&monitor_file),
                resolve_repo_relative_path(&owner_followup_board_file),
                resolve_repo_relative_path(&prevention_plan_file),
                resolve_repo_relative_path(&stability_board_file),
            );
            write_friday_release_evidence_sla_monitor_report(
                resolve_repo_relative_path(&monitor_file),
                &report,
            )?;
            print_friday_release_evidence_sla_monitor(&report);
        }

        Command::FridayReleaseEvidenceSlaMonitorJson {
            monitor_file,
            owner_followup_board_file,
            prevention_plan_file,
            stability_board_file,
        } => {
            let report = friday_release_evidence_sla_monitor_report(
                resolve_repo_relative_path(&monitor_file),
                resolve_repo_relative_path(&owner_followup_board_file),
                resolve_repo_relative_path(&prevention_plan_file),
                resolve_repo_relative_path(&stability_board_file),
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseEscalationLedger {
            ledger_file,
            monitor_file,
            owner_response,
            gate_outcome,
        } => {
            let ledger = append_friday_release_escalation_to_ledger(
                resolve_repo_relative_path(&ledger_file),
                resolve_repo_relative_path(&monitor_file),
                FridayReleaseEscalationOwnerResponse::parse(&owner_response)?,
                FridayReleaseEscalationGateOutcome::parse(&gate_outcome)?,
            )?;
            print_friday_release_escalation_ledger(&ledger);
        }

        Command::FridayReleaseEscalationLedgerJson {
            ledger_file,
            monitor_file,
            owner_response,
            gate_outcome,
        } => {
            let ledger_path = resolve_repo_relative_path(&ledger_file);
            let monitor_path = resolve_repo_relative_path(&monitor_file);
            let mut entries = read_friday_release_escalation_ledger(&ledger_path)
                .map(|ledger| ledger.entries)
                .unwrap_or_default();
            entries.extend(friday_release_escalation_entries_from_monitor(
                &monitor_path,
                FridayReleaseEscalationOwnerResponse::parse(&owner_response)?,
                FridayReleaseEscalationGateOutcome::parse(&gate_outcome)?,
            )?);
            let ledger = friday_release_escalation_ledger_report(&ledger_path, entries);
            println!("{}", ledger.to_pretty_json()?);
        }

        Command::FridayReleaseEscalationLedgerList { ledger_file } => {
            let ledger =
                read_friday_release_escalation_ledger(resolve_repo_relative_path(&ledger_file))?;
            print_friday_release_escalation_ledger(&ledger);
        }

        Command::FridayReleaseEscalationLedgerExport {
            ledger_file,
            output_file,
        } => {
            let ledger =
                read_friday_release_escalation_ledger(resolve_repo_relative_path(&ledger_file))?;
            write_friday_release_escalation_ledger(
                resolve_repo_relative_path(&output_file),
                &ledger,
            )?;
            print_friday_release_escalation_ledger(&ledger);
        }

        Command::FridayReleaseCheckpointReview {
            review_file,
            escalation_ledger_file,
            sla_monitor_file,
            owner_followup_board_file,
            prevention_plan_file,
            stability_board_file,
        } => {
            let report = friday_release_checkpoint_review_board_report(
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&escalation_ledger_file),
                resolve_repo_relative_path(&sla_monitor_file),
                resolve_repo_relative_path(&owner_followup_board_file),
                resolve_repo_relative_path(&prevention_plan_file),
                resolve_repo_relative_path(&stability_board_file),
            );
            write_friday_release_checkpoint_review_board_report(
                resolve_repo_relative_path(&review_file),
                &report,
            )?;
            print_friday_release_checkpoint_review_board(&report);
        }

        Command::FridayReleaseCheckpointReviewJson {
            review_file,
            escalation_ledger_file,
            sla_monitor_file,
            owner_followup_board_file,
            prevention_plan_file,
            stability_board_file,
        } => {
            let report = friday_release_checkpoint_review_board_report(
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&escalation_ledger_file),
                resolve_repo_relative_path(&sla_monitor_file),
                resolve_repo_relative_path(&owner_followup_board_file),
                resolve_repo_relative_path(&prevention_plan_file),
                resolve_repo_relative_path(&stability_board_file),
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseCheckpointSignoff {
            ledger_file,
            review_file,
            decision,
            operator,
            reason,
            acknowledgement_evidence_file,
            carryover_commitment,
        } => {
            let ledger = append_friday_release_checkpoint_signoff_to_ledger(
                resolve_repo_relative_path(&ledger_file),
                resolve_repo_relative_path(&review_file),
                FridayReleaseCheckpointSignoffRequest {
                    decision: FridayReleaseCheckpointSignoffDecision::parse(&decision)?,
                    operator,
                    reason,
                    acknowledgement_evidence_path: acknowledgement_evidence_file,
                    carryover_commitment,
                },
            )?;
            print_friday_release_checkpoint_signoff_ledger(&ledger);
        }

        Command::FridayReleaseCheckpointSignoffJson {
            ledger_file,
            review_file,
            decision,
            operator,
            reason,
            acknowledgement_evidence_file,
            carryover_commitment,
        } => {
            let ledger_path = resolve_repo_relative_path(&ledger_file);
            let review_path = resolve_repo_relative_path(&review_file);
            let mut records = read_friday_release_checkpoint_signoff_ledger(&ledger_path)
                .map(|ledger| ledger.records)
                .unwrap_or_default();
            records.push(friday_release_checkpoint_signoff_record_from_review(
                &review_path,
                FridayReleaseCheckpointSignoffRequest {
                    decision: FridayReleaseCheckpointSignoffDecision::parse(&decision)?,
                    operator,
                    reason,
                    acknowledgement_evidence_path: acknowledgement_evidence_file,
                    carryover_commitment,
                },
            )?);
            let ledger = friday_release_checkpoint_signoff_ledger_report(&ledger_path, records);
            println!("{}", ledger.to_pretty_json()?);
        }

        Command::FridayReleaseCheckpointSignoffList { ledger_file } => {
            let ledger = read_friday_release_checkpoint_signoff_ledger(
                resolve_repo_relative_path(&ledger_file),
            )?;
            print_friday_release_checkpoint_signoff_ledger(&ledger);
        }

        Command::FridayReleaseCheckpointSignoffExport {
            ledger_file,
            output_file,
        } => {
            let ledger = read_friday_release_checkpoint_signoff_ledger(
                resolve_repo_relative_path(&ledger_file),
            )?;
            write_friday_release_checkpoint_signoff_ledger(
                resolve_repo_relative_path(&output_file),
                &ledger,
            )?;
            print_friday_release_checkpoint_signoff_ledger(&ledger);
        }

        Command::FridayReleaseCheckpointEvidenceVault {
            vault_file,
            review_file,
            signoff_ledger_file,
        } => {
            let vault = friday_release_checkpoint_evidence_vault_report(
                resolve_repo_relative_path(&vault_file),
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&signoff_ledger_file),
            );
            write_friday_release_checkpoint_evidence_vault(
                resolve_repo_relative_path(&vault_file),
                &vault,
            )?;
            print_friday_release_checkpoint_evidence_vault(&vault);
        }

        Command::FridayReleaseCheckpointEvidenceVaultJson {
            vault_file,
            review_file,
            signoff_ledger_file,
        } => {
            let vault = friday_release_checkpoint_evidence_vault_report(
                resolve_repo_relative_path(&vault_file),
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&signoff_ledger_file),
            );
            println!("{}", vault.to_pretty_json()?);
        }

        Command::FridayReleaseEvidenceAttachmentReview {
            review_file,
            vault_file,
        } => {
            let review = friday_release_evidence_attachment_review_report(
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&vault_file),
            );
            write_friday_release_evidence_attachment_review(
                resolve_repo_relative_path(&review_file),
                &review,
            )?;
            print_friday_release_evidence_attachment_review(&review);
        }

        Command::FridayReleaseEvidenceAttachmentReviewJson {
            review_file,
            vault_file,
        } => {
            let review = friday_release_evidence_attachment_review_report(
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&vault_file),
            );
            println!("{}", review.to_pretty_json()?);
        }

        Command::FridayReleaseHandoffPacket {
            packet_file,
            attachment_review_file,
        } => {
            let packet = friday_release_handoff_packet_report(
                resolve_repo_relative_path(&packet_file),
                resolve_repo_relative_path(&attachment_review_file),
            );
            write_friday_release_handoff_packet(resolve_repo_relative_path(&packet_file), &packet)?;
            print_friday_release_handoff_packet(&packet);
        }

        Command::FridayReleaseHandoffPacketJson {
            packet_file,
            attachment_review_file,
        } => {
            let packet = friday_release_handoff_packet_report(
                resolve_repo_relative_path(&packet_file),
                resolve_repo_relative_path(&attachment_review_file),
            );
            println!("{}", packet.to_pretty_json()?);
        }

        Command::FridayReleaseHandoffAudit {
            trail_file,
            packet_file,
            state,
            operator,
            acknowledgement_note,
            supersedes_packet_id,
        } => {
            let trail = append_friday_release_handoff_audit_to_trail(
                resolve_repo_relative_path(&trail_file),
                resolve_repo_relative_path(&packet_file),
                FridayReleaseHandoffAuditRequest {
                    state: FridayReleaseHandoffAuditState::parse(&state)?,
                    operator,
                    acknowledgement_note,
                    supersedes_packet_id,
                },
            )?;
            print_friday_release_handoff_audit_trail(&trail);
        }

        Command::FridayReleaseHandoffAuditJson {
            trail_file,
            packet_file,
            state,
            operator,
            acknowledgement_note,
            supersedes_packet_id,
        } => {
            let trail_path = resolve_repo_relative_path(&trail_file);
            let packet_path = resolve_repo_relative_path(&packet_file);
            let mut records = read_friday_release_handoff_audit_trail(&trail_path)
                .map(|trail| trail.records)
                .unwrap_or_default();
            records.push(friday_release_handoff_audit_record_from_packet(
                &packet_path,
                FridayReleaseHandoffAuditRequest {
                    state: FridayReleaseHandoffAuditState::parse(&state)?,
                    operator,
                    acknowledgement_note,
                    supersedes_packet_id,
                },
            )?);
            let trail = friday_release_handoff_audit_trail_report(&trail_path, records);
            println!("{}", trail.to_pretty_json()?);
        }

        Command::FridayReleaseHandoffAuditList { trail_file } => {
            let trail =
                read_friday_release_handoff_audit_trail(resolve_repo_relative_path(&trail_file))?;
            print_friday_release_handoff_audit_trail(&trail);
        }

        Command::FridayReleaseHandoffAuditExport {
            trail_file,
            output_file,
        } => {
            let trail =
                read_friday_release_handoff_audit_trail(resolve_repo_relative_path(&trail_file))?;
            write_friday_release_handoff_audit_trail(
                resolve_repo_relative_path(&output_file),
                &trail,
            )?;
            print_friday_release_handoff_audit_trail(&trail);
        }

        Command::FridayReleaseHandoffGovernanceReview {
            review_file,
            trail_file,
        } => {
            let review = friday_release_handoff_governance_review_report(
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&trail_file),
            );
            write_friday_release_handoff_governance_review(
                resolve_repo_relative_path(&review_file),
                &review,
            )?;
            print_friday_release_handoff_governance_review(&review);
        }

        Command::FridayReleaseHandoffGovernanceReviewJson {
            review_file,
            trail_file,
        } => {
            let review = friday_release_handoff_governance_review_report(
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&trail_file),
            );
            println!("{}", review.to_pretty_json()?);
        }

        Command::FridayReleaseHandoffDispatchChecklist {
            checklist_file,
            governance_review_file,
            recipients,
            attachments,
            dispatch_note,
            privacy_note,
        } => {
            let checklist = friday_release_handoff_dispatch_checklist_report(
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&governance_review_file),
                FridayReleaseHandoffDispatchChecklistRequest {
                    recipients,
                    attachments,
                    dispatch_note,
                    privacy_note,
                },
            );
            write_friday_release_handoff_dispatch_checklist(
                resolve_repo_relative_path(&checklist_file),
                &checklist,
            )?;
            print_friday_release_handoff_dispatch_checklist(&checklist);
        }

        Command::FridayReleaseHandoffDispatchChecklistJson {
            checklist_file,
            governance_review_file,
            recipients,
            attachments,
            dispatch_note,
            privacy_note,
        } => {
            let checklist = friday_release_handoff_dispatch_checklist_report(
                resolve_repo_relative_path(&checklist_file),
                resolve_repo_relative_path(&governance_review_file),
                FridayReleaseHandoffDispatchChecklistRequest {
                    recipients,
                    attachments,
                    dispatch_note,
                    privacy_note,
                },
            );
            println!("{}", checklist.to_pretty_json()?);
        }

        Command::FridayReleaseHandoffDispatchAudit {
            trail_file,
            checklist_file,
            state,
            operator,
            final_decision_note,
            supersedes_checklist_id,
        } => {
            let trail = append_friday_release_handoff_dispatch_audit_to_trail(
                resolve_repo_relative_path(&trail_file),
                resolve_repo_relative_path(&checklist_file),
                FridayReleaseHandoffDispatchAuditRequest {
                    state: FridayReleaseHandoffDispatchAuditState::parse(&state)?,
                    operator,
                    final_decision_note,
                    supersedes_checklist_id,
                },
            )?;
            print_friday_release_handoff_dispatch_audit_trail(&trail);
        }

        Command::FridayReleaseHandoffDispatchAuditJson {
            trail_file,
            checklist_file,
            state,
            operator,
            final_decision_note,
            supersedes_checklist_id,
        } => {
            let trail_path = resolve_repo_relative_path(&trail_file);
            let checklist_path = resolve_repo_relative_path(&checklist_file);
            let mut records = read_friday_release_handoff_dispatch_audit_trail(&trail_path)
                .map(|trail| trail.records)
                .unwrap_or_default();
            records.push(friday_release_handoff_dispatch_audit_record_from_checklist(
                &checklist_path,
                FridayReleaseHandoffDispatchAuditRequest {
                    state: FridayReleaseHandoffDispatchAuditState::parse(&state)?,
                    operator,
                    final_decision_note,
                    supersedes_checklist_id,
                },
            )?);
            let trail = friday_release_handoff_dispatch_audit_trail_report(&trail_path, records);
            println!("{}", trail.to_pretty_json()?);
        }

        Command::FridayReleaseHandoffDispatchAuditList { trail_file } => {
            let trail = read_friday_release_handoff_dispatch_audit_trail(
                resolve_repo_relative_path(&trail_file),
            )?;
            print_friday_release_handoff_dispatch_audit_trail(&trail);
        }

        Command::FridayReleaseHandoffDispatchAuditExport {
            trail_file,
            output_file,
        } => {
            let trail = read_friday_release_handoff_dispatch_audit_trail(
                resolve_repo_relative_path(&trail_file),
            )?;
            write_friday_release_handoff_dispatch_audit_trail(
                resolve_repo_relative_path(&output_file),
                &trail,
            )?;
            print_friday_release_handoff_dispatch_audit_trail(&trail);
        }

        Command::FridayReleaseHandoffDispatchGovernance {
            review_file,
            trail_file,
        } => {
            let review = friday_release_handoff_dispatch_governance_review_report(
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&trail_file),
            );
            write_friday_release_handoff_dispatch_governance_review(
                resolve_repo_relative_path(&review_file),
                &review,
            )?;
            print_friday_release_handoff_dispatch_governance_review(&review);
        }

        Command::FridayReleaseHandoffDispatchGovernanceJson {
            review_file,
            trail_file,
        } => {
            let review = friday_release_handoff_dispatch_governance_review_report(
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&trail_file),
            );
            println!("{}", review.to_pretty_json()?);
        }

        Command::FridayReleaseHandoffCompletion {
            ledger_file,
            governance_review_file,
            state,
            operator,
            outcome_note,
            external_reference,
            supersedes_completion_id,
        } => {
            let ledger = append_friday_release_handoff_completion_to_ledger(
                resolve_repo_relative_path(&ledger_file),
                resolve_repo_relative_path(&governance_review_file),
                FridayReleaseHandoffCompletionRequest {
                    state: FridayReleaseHandoffCompletionState::parse(&state)?,
                    operator,
                    outcome_note,
                    external_reference,
                    supersedes_completion_id,
                },
            )?;
            print_friday_release_handoff_completion_ledger(&ledger);
        }

        Command::FridayReleaseHandoffCompletionJson {
            ledger_file,
            governance_review_file,
            state,
            operator,
            outcome_note,
            external_reference,
            supersedes_completion_id,
        } => {
            let ledger_path = resolve_repo_relative_path(&ledger_file);
            let governance_review_path = resolve_repo_relative_path(&governance_review_file);
            let mut records = read_friday_release_handoff_completion_ledger(&ledger_path)
                .map(|ledger| ledger.records)
                .unwrap_or_default();
            records.push(
                friday_release_handoff_completion_record_from_governance_review(
                    &governance_review_path,
                    FridayReleaseHandoffCompletionRequest {
                        state: FridayReleaseHandoffCompletionState::parse(&state)?,
                        operator,
                        outcome_note,
                        external_reference,
                        supersedes_completion_id,
                    },
                )?,
            );
            let ledger = friday_release_handoff_completion_ledger_report(&ledger_path, records);
            println!("{}", ledger.to_pretty_json()?);
        }

        Command::FridayReleaseHandoffCompletionList { ledger_file } => {
            let ledger = read_friday_release_handoff_completion_ledger(
                resolve_repo_relative_path(&ledger_file),
            )?;
            print_friday_release_handoff_completion_ledger(&ledger);
        }

        Command::FridayReleaseHandoffCompletionExport {
            ledger_file,
            output_file,
        } => {
            let ledger = read_friday_release_handoff_completion_ledger(
                resolve_repo_relative_path(&ledger_file),
            )?;
            write_friday_release_handoff_completion_ledger(
                resolve_repo_relative_path(&output_file),
                &ledger,
            )?;
            print_friday_release_handoff_completion_ledger(&ledger);
        }

        Command::FridayReleasePublicationControl {
            control_file,
            completion_ledger_file,
            state,
            operator,
            publication_note,
            manual_publication_reference,
        } => {
            let control = friday_release_publication_control_report(
                resolve_repo_relative_path(&control_file),
                resolve_repo_relative_path(&completion_ledger_file),
                FridayReleasePublicationRequest {
                    state: FridayReleasePublicationState::parse(&state)?,
                    operator,
                    publication_note,
                    manual_publication_reference,
                },
            );
            write_friday_release_publication_control(
                resolve_repo_relative_path(&control_file),
                &control,
            )?;
            print_friday_release_publication_control(&control);
        }

        Command::FridayReleasePublicationControlJson {
            control_file,
            completion_ledger_file,
            state,
            operator,
            publication_note,
            manual_publication_reference,
        } => {
            let control = friday_release_publication_control_report(
                resolve_repo_relative_path(&control_file),
                resolve_repo_relative_path(&completion_ledger_file),
                FridayReleasePublicationRequest {
                    state: FridayReleasePublicationState::parse(&state)?,
                    operator,
                    publication_note,
                    manual_publication_reference,
                },
            );
            println!("{}", control.to_pretty_json()?);
        }

        Command::FridayReleaseOutboundReview {
            ledger_file,
            publication_control_file,
            state,
            reviewer,
            review_note,
            manual_publication_reference,
            supersedes_review_id,
        } => {
            let ledger = append_friday_release_outbound_review_to_ledger(
                resolve_repo_relative_path(&ledger_file),
                resolve_repo_relative_path(&publication_control_file),
                FridayReleaseOutboundReviewRequest {
                    state: FridayReleaseOutboundReviewState::parse(&state)?,
                    reviewer,
                    review_note,
                    manual_publication_reference,
                    supersedes_review_id,
                },
            )?;
            print_friday_release_outbound_review_ledger(&ledger);
        }

        Command::FridayReleaseOutboundReviewJson {
            ledger_file,
            publication_control_file,
            state,
            reviewer,
            review_note,
            manual_publication_reference,
            supersedes_review_id,
        } => {
            let ledger_path = resolve_repo_relative_path(&ledger_file);
            let mut records = read_friday_release_outbound_review_ledger(&ledger_path)
                .map(|ledger| ledger.records)
                .unwrap_or_default();
            records.push(
                friday_release_outbound_review_record_from_publication_control(
                    resolve_repo_relative_path(&publication_control_file),
                    FridayReleaseOutboundReviewRequest {
                        state: FridayReleaseOutboundReviewState::parse(&state)?,
                        reviewer,
                        review_note,
                        manual_publication_reference,
                        supersedes_review_id,
                    },
                )?,
            );
            let ledger = friday_release_outbound_review_ledger_report(&ledger_path, records);
            println!("{}", ledger.to_pretty_json()?);
        }

        Command::FridayReleaseOutboundReviewList { ledger_file } => {
            let ledger = read_friday_release_outbound_review_ledger(resolve_repo_relative_path(
                &ledger_file,
            ))?;
            print_friday_release_outbound_review_ledger(&ledger);
        }

        Command::FridayReleaseOutboundReviewExport {
            ledger_file,
            output_file,
        } => {
            let ledger = read_friday_release_outbound_review_ledger(resolve_repo_relative_path(
                &ledger_file,
            ))?;
            write_friday_release_outbound_review_ledger(
                resolve_repo_relative_path(&output_file),
                &ledger,
            )?;
            print_friday_release_outbound_review_ledger(&ledger);
        }

        Command::FridayReleaseExternalReceipt {
            archive_file,
            outbound_review_ledger_file,
            state,
            receipt_kind,
            operator,
            receipt_note,
            evidence_path,
            external_reference,
            supersedes_receipt_id,
        } => {
            let archive = append_friday_release_external_receipt_to_archive(
                resolve_repo_relative_path(&archive_file),
                resolve_repo_relative_path(&outbound_review_ledger_file),
                FridayReleaseExternalReceiptRequest {
                    state: FridayReleaseExternalReceiptState::parse(&state)?,
                    receipt_kind: FridayReleaseExternalReceiptKind::parse(&receipt_kind),
                    operator,
                    receipt_note,
                    evidence_path,
                    external_reference,
                    supersedes_receipt_id,
                },
            )?;
            print_friday_release_external_receipt_archive(&archive);
        }

        Command::FridayReleaseExternalReceiptJson {
            archive_file,
            outbound_review_ledger_file,
            state,
            receipt_kind,
            operator,
            receipt_note,
            evidence_path,
            external_reference,
            supersedes_receipt_id,
        } => {
            let archive_path = resolve_repo_relative_path(&archive_file);
            let mut records = read_friday_release_external_receipt_archive(&archive_path)
                .map(|archive| archive.records)
                .unwrap_or_default();
            records.push(friday_release_external_receipt_record_from_outbound_review(
                resolve_repo_relative_path(&outbound_review_ledger_file),
                FridayReleaseExternalReceiptRequest {
                    state: FridayReleaseExternalReceiptState::parse(&state)?,
                    receipt_kind: FridayReleaseExternalReceiptKind::parse(&receipt_kind),
                    operator,
                    receipt_note,
                    evidence_path,
                    external_reference,
                    supersedes_receipt_id,
                },
            )?);
            let archive = friday_release_external_receipt_archive_report(&archive_path, records);
            println!("{}", archive.to_pretty_json()?);
        }

        Command::FridayReleaseExternalReceiptList { archive_file } => {
            let archive = read_friday_release_external_receipt_archive(
                resolve_repo_relative_path(&archive_file),
            )?;
            print_friday_release_external_receipt_archive(&archive);
        }

        Command::FridayReleaseExternalReceiptExport {
            archive_file,
            output_file,
        } => {
            let archive = read_friday_release_external_receipt_archive(
                resolve_repo_relative_path(&archive_file),
            )?;
            write_friday_release_external_receipt_archive(
                resolve_repo_relative_path(&output_file),
                &archive,
            )?;
            print_friday_release_external_receipt_archive(&archive);
        }

        Command::FridayReleaseReceiptReviewBoard {
            review_file,
            receipt_archive_file,
        } => {
            let report = friday_release_receipt_review_board_report(
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&receipt_archive_file),
            );
            write_friday_release_receipt_review_board_report(
                resolve_repo_relative_path(&review_file),
                &report,
            )?;
            print_friday_release_receipt_review_board(&report);
        }

        Command::FridayReleaseReceiptReviewBoardJson {
            review_file,
            receipt_archive_file,
        } => {
            let report = friday_release_receipt_review_board_report(
                resolve_repo_relative_path(&review_file),
                resolve_repo_relative_path(&receipt_archive_file),
            );
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayReleaseClosure {
            ledger_file,
            receipt_review_file,
            state,
            operator,
            closure_note,
            external_reference,
            carryover_commitment,
            supersedes_closure_id,
        } => {
            let ledger = append_friday_release_closure_to_ledger(
                resolve_repo_relative_path(&ledger_file),
                resolve_repo_relative_path(&receipt_review_file),
                FridayReleaseClosureRequest {
                    state: FridayReleaseClosureState::parse(&state)?,
                    operator,
                    closure_note,
                    external_reference,
                    carryover_commitment,
                    supersedes_closure_id,
                },
            )?;
            print_friday_release_closure_ledger(&ledger);
        }

        Command::FridayReleaseClosureJson {
            ledger_file,
            receipt_review_file,
            state,
            operator,
            closure_note,
            external_reference,
            carryover_commitment,
            supersedes_closure_id,
        } => {
            let ledger_path = resolve_repo_relative_path(&ledger_file);
            let mut records = read_friday_release_closure_ledger(&ledger_path)
                .map(|ledger| ledger.records)
                .unwrap_or_default();
            records.push(friday_release_closure_record_from_receipt_review(
                resolve_repo_relative_path(&receipt_review_file),
                FridayReleaseClosureRequest {
                    state: FridayReleaseClosureState::parse(&state)?,
                    operator,
                    closure_note,
                    external_reference,
                    carryover_commitment,
                    supersedes_closure_id,
                },
            )?);
            let ledger = friday_release_closure_ledger_report(&ledger_path, records);
            println!("{}", ledger.to_pretty_json()?);
        }

        Command::FridayReleaseClosureList { ledger_file } => {
            let ledger =
                read_friday_release_closure_ledger(resolve_repo_relative_path(&ledger_file))?;
            print_friday_release_closure_ledger(&ledger);
        }

        Command::FridayReleaseClosureExport {
            ledger_file,
            output_file,
        } => {
            let ledger =
                read_friday_release_closure_ledger(resolve_repo_relative_path(&ledger_file))?;
            write_friday_release_closure_ledger(resolve_repo_relative_path(&output_file), &ledger)?;
            print_friday_release_closure_ledger(&ledger);
        }

        Command::FridayReleaseContinuity {
            journal_file,
            closure_ledger_file,
            entry_kind,
            operator,
            note,
            owner,
            next_release_target,
            supersedes_entry_id,
        } => {
            let journal = append_friday_release_continuity_to_journal(
                resolve_repo_relative_path(&journal_file),
                resolve_repo_relative_path(&closure_ledger_file),
                FridayReleaseContinuityRequest {
                    entry_kind: FridayReleaseContinuityEntryKind::parse(&entry_kind)?,
                    operator,
                    note,
                    owner,
                    next_release_target,
                    supersedes_entry_id,
                },
            )?;
            print_friday_release_continuity_journal(&journal);
        }

        Command::FridayReleaseContinuityJson {
            journal_file,
            closure_ledger_file,
            entry_kind,
            operator,
            note,
            owner,
            next_release_target,
            supersedes_entry_id,
        } => {
            let journal_path = resolve_repo_relative_path(&journal_file);
            let mut records = read_friday_release_continuity_journal(&journal_path)
                .map(|journal| journal.records)
                .unwrap_or_default();
            records.push(friday_release_continuity_entry_from_closure_ledger(
                resolve_repo_relative_path(&closure_ledger_file),
                FridayReleaseContinuityRequest {
                    entry_kind: FridayReleaseContinuityEntryKind::parse(&entry_kind)?,
                    operator,
                    note,
                    owner,
                    next_release_target,
                    supersedes_entry_id,
                },
            )?);
            let journal = friday_release_continuity_journal_report(&journal_path, records);
            println!("{}", journal.to_pretty_json()?);
        }

        Command::FridayReleaseContinuityList { journal_file } => {
            let journal =
                read_friday_release_continuity_journal(resolve_repo_relative_path(&journal_file))?;
            print_friday_release_continuity_journal(&journal);
        }

        Command::FridayReleaseContinuityExport {
            journal_file,
            output_file,
        } => {
            let journal =
                read_friday_release_continuity_journal(resolve_repo_relative_path(&journal_file))?;
            write_friday_release_continuity_journal(
                resolve_repo_relative_path(&output_file),
                &journal,
            )?;
            print_friday_release_continuity_journal(&journal);
        }

        Command::FridayReleaseLearning {
            register_file,
            continuity_journal_file,
            category,
            operator,
            learning,
            owner,
            next_cycle_commitment,
            quality_gate,
            retires_learning_id,
        } => {
            let register = append_friday_release_learning_to_register(
                resolve_repo_relative_path(&register_file),
                resolve_repo_relative_path(&continuity_journal_file),
                FridayReleaseLearningRequest {
                    category: FridayReleaseLearningCategory::parse(&category)?,
                    operator,
                    learning,
                    owner,
                    next_cycle_commitment,
                    quality_gate,
                    retires_learning_id,
                },
            )?;
            print_friday_release_learning_register(&register);
        }

        Command::FridayReleaseLearningJson {
            register_file,
            continuity_journal_file,
            category,
            operator,
            learning,
            owner,
            next_cycle_commitment,
            quality_gate,
            retires_learning_id,
        } => {
            let register_path = resolve_repo_relative_path(&register_file);
            let mut records = read_friday_release_learning_register(&register_path)
                .map(|register| register.records)
                .unwrap_or_default();
            records.push(friday_release_learning_record_from_continuity_journal(
                resolve_repo_relative_path(&continuity_journal_file),
                FridayReleaseLearningRequest {
                    category: FridayReleaseLearningCategory::parse(&category)?,
                    operator,
                    learning,
                    owner,
                    next_cycle_commitment,
                    quality_gate,
                    retires_learning_id,
                },
            )?);
            let register = friday_release_learning_register_report(&register_path, records);
            println!("{}", register.to_pretty_json()?);
        }

        Command::FridayReleaseLearningList { register_file } => {
            let register =
                read_friday_release_learning_register(resolve_repo_relative_path(&register_file))?;
            print_friday_release_learning_register(&register);
        }

        Command::FridayReleaseLearningExport {
            register_file,
            output_file,
        } => {
            let register =
                read_friday_release_learning_register(resolve_repo_relative_path(&register_file))?;
            write_friday_release_learning_register(
                resolve_repo_relative_path(&output_file),
                &register,
            )?;
            print_friday_release_learning_register(&register);
        }

        Command::FridayTrustedHostLiveState {
            state_file,
            history_file,
        } => {
            let state = run_friday_trusted_host_live_state_command(&state_file, &history_file)?;
            print_friday_trusted_host_live_runner_state(&state);
        }

        Command::FridayTrustedHostLiveStateJson {
            state_file,
            history_file,
        } => {
            let state = run_friday_trusted_host_live_state_command(&state_file, &history_file)?;
            println!("{}", state.to_pretty_json()?);
        }

        Command::FridayTrustedHostBridgeRunner {
            input_dir,
            action_id,
            approve,
            execute,
            cancel,
            history_file,
            state_file,
            reason,
        } => {
            let report = run_friday_trusted_host_bridge_runner_command(
                &input_dir,
                action_id.as_deref(),
                approve,
                execute,
                cancel,
                &history_file,
                &state_file,
                reason.as_deref(),
            )?;
            print_friday_trusted_host_bridge_runner_report(&report);
        }

        Command::FridayTrustedHostBridgeRunnerJson {
            input_dir,
            action_id,
            approve,
            execute,
            cancel,
            history_file,
            state_file,
            reason,
        } => {
            let report = run_friday_trusted_host_bridge_runner_command(
                &input_dir,
                action_id.as_deref(),
                approve,
                execute,
                cancel,
                &history_file,
                &state_file,
                reason.as_deref(),
            )?;
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayLocalChecks => {
            print_friday_local_execution_checks();
        }

        Command::FridayLocalChecksJson => {
            println!(
                "{}",
                default_friday_local_execution_checks().to_pretty_json()?
            );
        }

        Command::FridayBrowserGate => {
            print_friday_browser_gate();
        }

        Command::FridayBrowserGateJson => {
            println!(
                "{}",
                default_friday_browser_verification_report().to_pretty_json()?
            );
        }

        Command::BrowserExtensionSmoke => {
            print_browser_extension_smoke();
        }

        Command::BrowserExtensionSmokeJson => {
            println!("{}", browser_extension_smoke_report().to_pretty_json()?);
        }

        Command::BrowserExtensionLaunchSmoke { execute } => {
            print_browser_extension_launch_smoke(execute);
        }

        Command::BrowserExtensionLaunchSmokeJson { execute } => {
            println!(
                "{}",
                browser_extension_launch_smoke_report(execute, 8_000).to_pretty_json()?
            );
        }

        Command::BrowserPackReuseSmoke => {
            print_browser_pack_reuse_smoke();
        }

        Command::BrowserPackReuseSmokeJson => {
            println!("{}", browser_pack_reuse_smoke_report().to_pretty_json()?);
        }

        Command::BrowserPackRecoverySmoke => {
            print_browser_pack_recovery_smoke();
        }

        Command::BrowserPackRecoverySmokeJson => {
            println!("{}", browser_pack_recovery_smoke_report().to_pretty_json()?);
        }

        Command::BrowserWebLlmAcceleration => {
            print_browser_webllm_acceleration();
        }

        Command::BrowserWebLlmAccelerationJson => {
            println!("{}", browser_webllm_acceleration_report().to_pretty_json()?);
        }

        Command::FridayOcrSmoke {
            output_dir,
            image,
            execute_model,
        } => {
            let report = run_friday_ocr_smoke(
                resolve_repo_relative_path(&output_dir),
                image.as_deref(),
                execute_model,
            )?;
            print_friday_ocr_smoke(&report);
        }

        Command::FridayOcrSmokeJson {
            output_dir,
            image,
            execute_model,
        } => {
            let report = run_friday_ocr_smoke(
                resolve_repo_relative_path(&output_dir),
                image.as_deref(),
                execute_model,
            )?;
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayVlmContract {
            output_dir,
            screenshot,
            prompt,
        } => {
            let report = run_friday_vlm_contract(
                resolve_repo_relative_path(&output_dir),
                screenshot.as_deref(),
                prompt.as_deref(),
            )?;
            print_friday_vlm_contract(&report);
        }

        Command::FridayVlmContractJson {
            output_dir,
            screenshot,
            prompt,
        } => {
            let report = run_friday_vlm_contract(
                resolve_repo_relative_path(&output_dir),
                screenshot.as_deref(),
                prompt.as_deref(),
            )?;
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayMultimodalRoute {
            request_kind,
            remote_allowed,
        } => {
            let request_kind = parse_friday_multimodal_request_kind(&request_kind)?;
            let route = friday_multimodal_route(request_kind, remote_allowed);
            print_friday_multimodal_route(&route);
        }

        Command::FridayMultimodalRouteJson {
            request_kind,
            remote_allowed,
        } => {
            let request_kind = parse_friday_multimodal_request_kind(&request_kind)?;
            let route = friday_multimodal_route(request_kind, remote_allowed);
            println!("{}", route.to_pretty_json()?);
        }

        Command::FridayMultimodalDiagnostics => {
            print_friday_multimodal_diagnostics(&friday_multimodal_ui_diagnostics());
        }

        Command::FridayMultimodalDiagnosticsJson => {
            println!("{}", friday_multimodal_ui_diagnostics().to_pretty_json()?);
        }

        Command::FridayMultimodalVisualCheck => {
            print_friday_multimodal_visual_check(&friday_multimodal_visual_check());
        }

        Command::FridayMultimodalVisualCheckJson => {
            println!("{}", friday_multimodal_visual_check().to_pretty_json()?);
        }

        Command::FridayScreenshotVlm {
            output_dir,
            screenshot,
            prompt,
        } => {
            let report = run_friday_screenshot_vlm_handoff(
                resolve_repo_relative_path(&output_dir),
                resolve_repo_relative_path(&screenshot),
                prompt.as_deref(),
            )?;
            print_friday_screenshot_vlm_handoff(&report);
        }

        Command::FridayScreenshotVlmJson {
            output_dir,
            screenshot,
            prompt,
        } => {
            let report = run_friday_screenshot_vlm_handoff(
                resolve_repo_relative_path(&output_dir),
                resolve_repo_relative_path(&screenshot),
                prompt.as_deref(),
            )?;
            println!("{}", report.to_pretty_json()?);
        }

        Command::FridayMediaAffordances => {
            print_friday_media_affordances(&friday_media_affordances());
        }

        Command::FridayMediaAffordancesJson => {
            println!(
                "{}",
                serde_json::to_string_pretty(&friday_media_affordances())?
            );
        }

        Command::AccessibilityDiagnostics { os, live } => {
            print_accessibility_diagnostics(os.as_deref(), live)?;
        }

        Command::AuditLog { state_file, limit } => {
            print_audit_log(&state_file, limit)?;
        }

        Command::Completion => {
            print_completion();
        }

        Command::CompletionJson => {
            print_completion_json()?;
        }

        Command::Models { modality } => {
            print_models(&RuntimeBroker::detect(), modality.as_deref())?;
        }

        Command::InstallModel { model } => {
            install_model_cli(&model)?;
        }

        Command::UiModelCandidates => {
            print_ui_model_candidates()?;
        }

        Command::ToolModelCandidates => {
            print_tool_model_candidates()?;
        }

        Command::ModelRoles => {
            print_model_roles();
        }

        Command::Uigen {
            model,
            output,
            prompt,
        } => {
            run_uigen(model.as_deref(), &output, &prompt).await?;
        }

        Command::UigenGoogle { model } => {
            run_uigen_google(model.as_deref()).await?;
        }

        Command::UigenVision {
            screenshot,
            output,
            prompt,
        } => {
            run_uigen_vision(&screenshot, &output, &prompt)?;
        }

        Command::UigenVisionGoogle => {
            run_uigen_vision_google()?;
        }

        Command::Plan { modality, model } => {
            let modality = parse_modality(&modality)?;
            let broker = RuntimeBroker::detect();
            let plan = broker.build_plan(BrokerRequest::new(modality).with_model(model));
            print_plan(&broker, &plan);
        }

        Command::Blueprint { host } => {
            let host = parse_host_surface(&host)?;
            print_blueprint(host);
        }

        Command::BrowserProfile { flavor } => {
            let flavor = parse_browser_flavor(&flavor)?;
            print_browser_profile(flavor);
        }

        Command::BrowserPlan {
            flavor,
            task,
            modality,
            model,
            remote_fallback,
        } => {
            let flavor = parse_browser_flavor(&flavor)?;
            let task = parse_browser_task(&task)?;
            let modality = parse_modality(&modality)?;
            print_browser_plan(flavor, task, modality, model, remote_fallback);
        }

        Command::BrowserPacks => {
            print_browser_packs();
        }

        Command::Grammar { text, fix } => {
            run_grammar(&text, fix)?;
        }

        Command::WakeWords => {
            print_wake_words(&RuntimeBroker::detect());
        }

        Command::ProductionConfig { target } => {
            let target = parse_integration_target(&target)?;
            print_production_config(target)?;
        }

        Command::ExportProductionBundle { output_dir } => {
            export_production_bundle_cli(&output_dir)?;
        }

        Command::ReleaseSummary => {
            print_release_summary()?;
        }

        Command::ExportReleaseSummary { output_dir } => {
            export_release_summary_cli(&output_dir)?;
        }
    }

    Ok(())
}

fn print_interactive_help() {
    println!("Flow");
    println!("====\n");
    println!("Commands:");
    println!("  --transcribe <file>      Transcribe an audio file");
    println!("  --wispr <file>           STT plus local cleanup");
    println!("  --speak <text>           Speak text with Kokoro");
    println!("  --live                   Live microphone mode");
    println!("  --dictate                Wake/hotkey dictation into focused input");
    println!("  --chat [model]           Interactive local chat");
    println!("  --tool-agent <prompt>    Run one bounded local tool-agent prompt");
    println!("  --tool-agent-tools <tools.json> <request>");
    println!("                           Run tool routing with a tools JSON file");
    println!("  --ocr <image> [prompt]   OCR with GLM-OCR");
    println!("  --profile                Show device profile and activation config");
    println!("  --projects               Show the DX project stack");
    println!("  --scorecard              Show the Flow competitive scorecard");
    println!("  --friday                 Show Friday's AI workspace capability plan");
    println!("  --friday-json            Print Friday's capability plan as JSON");
    println!("  --friday-search <query>  Plan a metasearch-first cited answer search");
    println!("  --friday-research <query>");
    println!("                           Plan a metasearch-first deep research run");
    println!("  --friday-research-workflow <query>");
    println!("                           Show the runnable Friday research workflow contract");
    println!("  --friday-metasearch <query>");
    println!("                           Execute answer search against local metasearch");
    println!("  --friday-research-report <query>");
    println!("                           Search locally and print a markdown research report");
    println!("  --friday-research-report-save <dir> <query>");
    println!("                           Persist report, citations, source groups, and events");
    println!("  --friday-research-synthesize <query>");
    println!("                           Search locally and synthesize a cited answer");
    println!("  --friday-workspace-init <dir>");
    println!("                           Seed durable Projects, Memory, and Connectors state");
    println!("  --friday-workspace-json [dir]");
    println!("                           Print seeded or persisted Friday workspace state");
    println!("  --friday-artifacts-init <dir>");
    println!("                           Seed Canvas, Artifacts, and Code checkpoint state");
    println!("  --friday-artifacts-json [dir]");
    println!("                           Print seeded or persisted artifact state");
    println!("  --friday-artifacts-index-multimodal <store-dir> <bundle-dir>");
    println!("                           Import OCR/VLM artifact metadata into the artifact store");
    println!("  --friday-artifacts-index-multimodal-json <store-dir> <bundle-dir>");
    println!("                           Print multimodal artifact import status as JSON");
    println!("  --friday-runtime-init <dir>");
    println!("                           Seed Voice, Multimodal, and Automation runtime state");
    println!("  --friday-runtime-json [dir]");
    println!("                           Print seeded or persisted runtime state");
    println!("  --friday-ui             Show Friday product UI integration contracts");
    println!("  --friday-ui-json        Print Friday UI integration contracts as JSON");
    println!("  --friday-live-ui-routes Show tracked Friday UI route file bindings");
    println!("  --friday-live-ui-routes-json");
    println!("                           Print tracked Friday UI route file bindings as JSON");
    println!("  --friday-readiness      Show Friday operator readiness summary");
    println!("  --friday-readiness-json Print Friday operator readiness as JSON");
    println!("  --friday-route-visuals  Show Friday route screenshot targets");
    println!("  --friday-route-visuals-json");
    println!("                           Print Friday route screenshot targets as JSON");
    println!("  --friday-execution-handoffs");
    println!("                           Show Friday desktop/web execution handoff contracts");
    println!("  --friday-execution-handoffs-json");
    println!("                           Print Friday execution handoff contracts as JSON");
    println!("  --friday-dashboard-export [dir]");
    println!("                           Export readiness bundle for Friday/DX dashboards");
    println!("  --friday-dashboard-export-json [dir]");
    println!("                           Export readiness bundle and print JSON");
    println!("  --friday-dashboard-panel [dir]");
    println!("                           Show dashboard UI panel data from an export bundle");
    println!("  --friday-dashboard-panel-json [dir]");
    println!("                           Print dashboard UI panel data as JSON");
    println!("  --friday-dashboard-product-ui [dir]");
    println!("                           Show product UI binding data for the dashboard");
    println!("  --friday-dashboard-product-ui-json [dir]");
    println!("                           Print product UI binding data as JSON");
    println!("  --friday-dashboard-product-ui-smoke [dir]");
    println!("                           Smoke-check the visible dashboard binding");
    println!("  --friday-dashboard-product-ui-smoke-json [dir]");
    println!("                           Print dashboard binding smoke check as JSON");
    println!("  --friday-dashboard-host-bridge [dir]");
    println!("                           Show trusted host command bridge handoffs");
    println!("  --friday-dashboard-host-bridge-json [dir]");
    println!("                           Print trusted host command bridge handoffs as JSON");
    println!("  --friday-trusted-host-runner [dir] [--action-id id] [--approve] [--execute]");
    println!("                           Run or dry-run one approved trusted host command");
    println!("  --friday-trusted-host-runner-json [dir] [--action-id id] [--approve] [--execute]");
    println!("                           Print trusted host runner result as JSON");
    println!("  --friday-trusted-host-runner-ux [history-file]");
    println!("                           Show grouped trusted runner history and retry UX");
    println!("  --friday-trusted-host-runner-ux-json [history-file]");
    println!("                           Print trusted runner history UX as JSON");
    println!("  --friday-trusted-host-runner-approval-ui [history-file]");
    println!("                           Show trusted runner approval modal contract");
    println!("  --friday-trusted-host-runner-approval-ui-json [history-file]");
    println!("                           Print trusted runner approval modal contract as JSON");
    println!("  --friday-trusted-host-runner-cancellation-ux [state-file]");
    println!("                           Show live runner cancellation and recovery controls");
    println!("  --friday-trusted-host-runner-cancellation-ux-json [state-file]");
    println!("                           Print live runner cancellation controls as JSON");
    println!("  --friday-trusted-host-runner-review [history-file] [--status status]");
    println!("                           Show trusted runner operator review and release gates");
    println!("  --friday-trusted-host-runner-review-json [history-file] [--status status]");
    println!("                           Print trusted runner operator review as JSON");
    println!("  --friday-trusted-host-runner-release-package [export-dir] [--output file]");
    println!("                           Write trusted runner release package JSON");
    println!("  --friday-trusted-host-runner-release-package-json [export-dir]");
    println!("                           Print trusted runner release package as JSON");
    println!("  --friday-trusted-runner-release-archive [timeline-file] [package-file]");
    println!("                           Append a package to the trusted runner release timeline");
    println!("  --friday-trusted-runner-release-timeline [timeline-file] [--package file]");
    println!("                           Show and write trusted runner release package timeline");
    println!("  --friday-trusted-runner-release-timeline-json [timeline-file] [--package file]");
    println!("                           Print trusted runner release package timeline as JSON");
    println!("  --friday-release-checklist [export-dir] [--output file]");
    println!("                           Write Friday release operator checklist JSON");
    println!("  --friday-release-checklist-json [export-dir]");
    println!("                           Print Friday release operator checklist as JSON");
    println!("  --friday-release-signoff [checklist-file] [--reason text]");
    println!("                           Append a local Friday release signoff record");
    println!("  --friday-release-signoff-json [checklist-file] [--reason text]");
    println!("                           Append a local Friday release signoff and print JSON");
    println!("  --friday-release-qa [export-dir] [--output file]");
    println!("                           Write Friday release QA command center JSON");
    println!("  --friday-release-qa-json [export-dir]");
    println!("                           Print Friday release QA command center as JSON");
    println!("  --friday-release-export-kit [export-dir]");
    println!("                           Write Friday release evidence export-kit JSON");
    println!("  --friday-release-export-kit-json [export-dir]");
    println!("                           Print Friday release evidence export kit as JSON");
    println!("  --friday-release-deployment-gate [export-dir]");
    println!("                           Write Friday release go/no-go deployment gate JSON");
    println!("  --friday-release-deployment-gate-json [export-dir]");
    println!("                           Print Friday release deployment gate as JSON");
    println!("  --friday-release-candidate-archive [archive-file] [--gate file]");
    println!("                           Append deployment gate(s) to candidate archive");
    println!("  --friday-release-candidate-archive-json [archive-file] [--gate file]");
    println!("                           Print release candidate archive as JSON");
    println!("  --friday-release-promotion-ledger [ledger-file] [--archive file]");
    println!(
        "                           Record candidate promotion, hold, rollback, or abandonment"
    );
    println!("  --friday-release-promotion-ledger-json [ledger-file] [--archive file]");
    println!("                           Record a promotion decision and print ledger JSON");
    println!("  --friday-release-post-promotion-monitor [export-dir] [--incident-note file]");
    println!("                           Write post-promotion readiness and incident monitor JSON");
    println!("  --friday-release-post-promotion-monitor-json [export-dir]");
    println!("                           Print post-promotion monitor as JSON");
    println!("  --friday-release-rollback-drill [export-dir] [--rollback-command cmd]");
    println!("                           Write local-only rollback drill readiness JSON");
    println!("  --friday-release-rollback-drill-json [export-dir]");
    println!("                           Print rollback drill readiness JSON");
    println!("  --friday-release-stability-board [export-dir]");
    println!("                           Write consolidated Friday release stability board JSON");
    println!("  --friday-release-stability-board-json [export-dir]");
    println!("                           Print release stability board as JSON");
    println!("  --friday-release-recovery-runbook [export-dir]");
    println!("                           Write local-only Friday release recovery runbook JSON");
    println!("  --friday-release-recovery-runbook-json [export-dir]");
    println!("                           Print release recovery runbook as JSON");
    println!("  --friday-release-incident-archive [export-dir] [--incident-note file]");
    println!("                           Append a local-only release incident archive entry");
    println!("  --friday-release-incident-archive-json [export-dir]");
    println!("                           Print release incident archive preview as JSON");
    println!("  --friday-release-incident-archive-list [export-dir]");
    println!("                           List an existing release incident archive");
    println!("  --friday-release-incident-archive-export [export-dir] [--output file]");
    println!("                           Export an existing release incident archive JSON");
    println!("  --friday-release-prevention-plan [export-dir]");
    println!("                           Write release prevention plan JSON without remediation");
    println!("  --friday-release-prevention-plan-json [export-dir]");
    println!("                           Print release prevention plan as JSON");
    println!("  --friday-release-owner-followup-board [export-dir]");
    println!("                           Write owner follow-up board JSON without remediation");
    println!("  --friday-release-owner-followup-board-json [export-dir]");
    println!("                           Print owner follow-up board as JSON");
    println!("  --friday-release-evidence-sla-monitor [export-dir]");
    println!("                           Write release evidence SLA monitor JSON");
    println!("  --friday-release-evidence-sla-monitor-json [export-dir]");
    println!("                           Print release evidence SLA monitor as JSON");
    println!("  --friday-release-escalation-ledger [--ledger file] [--monitor file]");
    println!("                           Append SLA escalations to a local release ledger");
    println!("  --friday-release-escalation-ledger-json [--ledger file] [--monitor file]");
    println!("                           Print escalation ledger preview as JSON");
    println!("  --friday-release-escalation-ledger-list [--ledger file]");
    println!("                           List an existing release escalation ledger");
    println!("  --friday-release-escalation-ledger-export [--ledger file] [--output file]");
    println!("                           Export an existing release escalation ledger JSON");
    println!("  --friday-release-checkpoint-review [export-dir]");
    println!(
        "                           Write release checkpoint review JSON without running commands"
    );
    println!("  --friday-release-checkpoint-review-json [export-dir]");
    println!("                           Print release checkpoint review as JSON");
    println!("  --friday-release-checkpoint-signoff [--ledger file] [--review file]");
    println!("                           Append checkpoint signoff to a local release ledger");
    println!("  --friday-release-checkpoint-signoff-json [--ledger file] [--review file]");
    println!("                           Print checkpoint signoff ledger preview as JSON");
    println!("  --friday-release-checkpoint-signoff-list [--ledger file]");
    println!("                           List an existing checkpoint signoff ledger");
    println!("  --friday-release-checkpoint-signoff-export [--ledger file] [--output file]");
    println!("                           Export an existing checkpoint signoff ledger JSON");
    println!("  --friday-release-checkpoint-evidence-vault [export-dir]");
    println!(
        "                           Write checkpoint evidence vault JSON without running commands"
    );
    println!("  --friday-release-checkpoint-evidence-vault-json [export-dir]");
    println!("                           Print checkpoint evidence vault as JSON");
    println!("  --friday-release-evidence-attachment-review [export-dir]");
    println!("                           Write evidence attachment review JSON without uploading");
    println!("  --friday-release-evidence-attachment-review-json [export-dir]");
    println!("                           Print evidence attachment review as JSON");
    println!("  --friday-release-handoff-packet [export-dir]");
    println!("                           Write release handoff packet JSON without uploading");
    println!("  --friday-release-handoff-packet-json [export-dir]");
    println!("                           Print release handoff packet as JSON");
    println!("  --friday-release-handoff-audit [--trail file] [--packet file]");
    println!("                           Append a local release handoff audit record");
    println!("  --friday-release-handoff-audit-json [--trail file] [--packet file]");
    println!("                           Print release handoff audit preview as JSON");
    println!("  --friday-release-handoff-audit-list [--trail file]");
    println!("                           List an existing release handoff audit trail");
    println!("  --friday-release-handoff-audit-export [--trail file] [--output file]");
    println!("                           Export an existing release handoff audit trail JSON");
    println!("  --friday-release-handoff-governance-review [export-dir]");
    println!("                           Write handoff governance review JSON without sending");
    println!("  --friday-release-handoff-governance-review-json [export-dir]");
    println!("                           Print handoff governance review as JSON");
    println!("  --friday-release-handoff-dispatch-checklist [export-dir]");
    println!("                           Write handoff dispatch checklist JSON without sending");
    println!("  --friday-release-handoff-dispatch-checklist-json [export-dir]");
    println!("                           Print handoff dispatch checklist as JSON");
    println!("  --friday-release-handoff-dispatch-audit [--trail file] [--checklist file]");
    println!("                           Append a local handoff dispatch audit record");
    println!("  --friday-release-handoff-dispatch-audit-json [--trail file] [--checklist file]");
    println!("                           Print handoff dispatch audit preview as JSON");
    println!("  --friday-release-handoff-dispatch-audit-list [--trail file]");
    println!("                           List an existing handoff dispatch audit trail");
    println!("  --friday-release-handoff-dispatch-audit-export [--trail file] [--output file]");
    println!("                           Export an existing handoff dispatch audit trail JSON");
    println!("  --friday-release-handoff-dispatch-governance [export-dir]");
    println!("                           Write handoff dispatch governance JSON without sending");
    println!("  --friday-release-handoff-dispatch-governance-json [export-dir]");
    println!("                           Print handoff dispatch governance as JSON");
    println!("  --friday-release-handoff-completion [--ledger file] [--governance-review file]");
    println!("                           Append a governed local handoff completion record");
    println!(
        "  --friday-release-handoff-completion-json [--ledger file] [--governance-review file]"
    );
    println!("                           Print handoff completion ledger preview as JSON");
    println!("  --friday-release-handoff-completion-list [--ledger file]");
    println!("                           List an existing handoff completion ledger");
    println!("  --friday-release-handoff-completion-export [--ledger file] [--output file]");
    println!("                           Export an existing handoff completion ledger JSON");
    println!("  --friday-release-publication-control [export-dir] [--completion-ledger file]");
    println!("                           Write local-only release publication control JSON");
    println!("  --friday-release-publication-control-json [export-dir] [--completion-ledger file]");
    println!("                           Print release publication control as JSON");
    println!("  --friday-release-outbound-review [--ledger file] [--publication-control file]");
    println!("                           Append a local outbound review record without publishing");
    println!(
        "  --friday-release-outbound-review-json [--ledger file] [--publication-control file]"
    );
    println!("                           Print outbound review ledger preview as JSON");
    println!("  --friday-release-outbound-review-list [--ledger file]");
    println!("                           List an existing outbound review ledger");
    println!("  --friday-release-outbound-review-export [--ledger file] [--output file]");
    println!("                           Export an existing outbound review ledger JSON");
    println!(
        "  --friday-release-external-receipt [--archive file] [--outbound-review-ledger file]"
    );
    println!("                           Append an operator-owned external receipt record");
    println!(
        "  --friday-release-external-receipt-json [--archive file] [--outbound-review-ledger file]"
    );
    println!("                           Print external receipt archive preview as JSON");
    println!("  --friday-release-external-receipt-list [--archive file]");
    println!("                           List an existing external receipt archive");
    println!("  --friday-release-external-receipt-export [--archive file] [--output file]");
    println!("                           Export an existing external receipt archive JSON");
    println!("  --friday-release-receipt-review-board [export-dir] [--receipt-archive file]");
    println!("                           Write a local release receipt review board JSON");
    println!("  --friday-release-receipt-review-board-json [export-dir] [--receipt-archive file]");
    println!("                           Print release receipt review board as JSON");
    println!("  --friday-release-closure [--ledger file] [--receipt-review file]");
    println!("                           Append a local release closure ledger record");
    println!("  --friday-release-closure-json [--ledger file] [--receipt-review file]");
    println!("                           Print release closure ledger preview as JSON");
    println!("  --friday-release-closure-list [--ledger file]");
    println!("                           List an existing release closure ledger");
    println!("  --friday-release-closure-export [--ledger file] [--output file]");
    println!("                           Export an existing release closure ledger JSON");
    println!("  --friday-release-continuity [--journal file] [--closure-ledger file]");
    println!("                           Append a local release continuity journal entry");
    println!("  --friday-release-continuity-json [--journal file] [--closure-ledger file]");
    println!("                           Print release continuity journal preview as JSON");
    println!("  --friday-release-continuity-list [--journal file]");
    println!("                           List an existing release continuity journal");
    println!("  --friday-release-continuity-export [--journal file] [--output file]");
    println!("                           Export an existing release continuity journal JSON");
    println!("  --friday-release-learning [--register file] [--continuity-journal file]");
    println!("                           Append a local release learning register record");
    println!("  --friday-release-learning-json [--register file] [--continuity-journal file]");
    println!("                           Print release learning register preview as JSON");
    println!("  --friday-release-learning-list [--register file]");
    println!("                           List an existing release learning register");
    println!("  --friday-release-learning-export [--register file] [--output file]");
    println!("                           Export an existing release learning register JSON");
    println!("  --friday-trusted-host-live-state [state-file] [--history file]");
    println!("                           Show trusted runner live state from local state/history");
    println!("  --friday-trusted-host-live-state-json [state-file] [--history file]");
    println!("                           Print trusted runner live state as JSON");
    println!(
        "  --friday-trusted-host-bridge-runner [dir] [--action-id id] [--approve] [--execute]"
    );
    println!("                           Run a trusted host command through the live-state bridge");
    println!(
        "  --friday-trusted-host-bridge-runner-json [dir] [--action-id id] [--approve] [--execute]"
    );
    println!("                           Print trusted host bridge runner events as JSON");
    println!("  --friday-local-checks   Run low-resource local execution checks");
    println!("  --friday-local-checks-json");
    println!("                           Print local execution checks as JSON");
    println!("  --friday-browser-gate   Show browser verification and deploy gate");
    println!("  --friday-browser-gate-json");
    println!("                           Print browser gate status as JSON");
    println!("  --browser-extension-smoke");
    println!(
        "                           Show packaged extension and installed-browser smoke readiness"
    );
    println!("  --browser-extension-smoke-json");
    println!("                           Print browser extension smoke readiness as JSON");
    println!("  --browser-extension-launch-smoke [--execute]");
    println!("                           Show bounded temporary-profile launch smoke readiness");
    println!("  --browser-extension-launch-smoke-json [--execute]");
    println!("                           Print browser extension launch smoke readiness as JSON");
    println!("  --browser-pack-reuse-smoke");
    println!("                           Show offline browser-pack reuse smoke readiness");
    println!("  --browser-pack-reuse-smoke-json");
    println!("                           Print offline browser-pack reuse smoke readiness as JSON");
    println!("  --browser-pack-recovery-smoke");
    println!("                           Show browser-pack recovery smoke readiness");
    println!("  --browser-pack-recovery-smoke-json");
    println!("                           Print browser-pack recovery smoke readiness as JSON");
    println!("  --browser-webllm-acceleration");
    println!("                           Show Chromium WebLLM acceleration readiness");
    println!("  --browser-webllm-acceleration-json");
    println!("                           Print Chromium WebLLM acceleration readiness as JSON");
    println!("  --friday-ocr-smoke <dir> [image] [--execute]");
    println!("                           Write a bounded OCR smoke artifact bundle");
    println!("  --friday-ocr-smoke-json <dir> [image] [--execute]");
    println!("                           Print OCR smoke artifact bundle status as JSON");
    println!("  --friday-vlm-contract <dir> [screenshot] [prompt]");
    println!("                           Write a VLM screenshot artifact contract");
    println!("  --friday-vlm-contract-json <dir> [screenshot] [prompt]");
    println!("                           Print the VLM screenshot contract as JSON");
    println!("  --friday-multimodal-route <ocr|vlm|audio|image|video> [--remote]");
    println!("                           Show local-first model routing policy");
    println!("  --friday-multimodal-route-json <kind> [--remote]");
    println!("                           Print multimodal routing policy as JSON");
    println!("  --friday-multimodal-diagnostics");
    println!("                           Show Multimodal UI and OCR diagnostics");
    println!("  --friday-multimodal-diagnostics-json");
    println!("                           Print Multimodal UI diagnostics as JSON");
    println!("  --friday-multimodal-visual-check");
    println!("                           Show the Multimodal route visual verification target");
    println!("  --friday-multimodal-visual-check-json");
    println!("                           Print the Multimodal visual check as JSON");
    println!("  --friday-screenshot-vlm <dir> <screenshot> [prompt]");
    println!("                           Validate a local screenshot and write VLM handoff files");
    println!("  --friday-screenshot-vlm-json <dir> <screenshot> [prompt]");
    println!("                           Print screenshot VLM handoff status as JSON");
    println!("  --friday-media-affordances");
    println!("                           Show image/video install and run affordances");
    println!("  --friday-media-affordances-json");
    println!("                           Print image/video affordances as JSON");
    println!("  --accessibility [os] [--dry-run]");
    println!("                           Diagnose host accessibility automation readiness");
    println!("  --audit-log <state-file> [limit]");
    println!("                           Review persisted host automation audit records");
    println!("  --completion             Show the active 100-point completion loop");
    println!("  --completion-json        Print the active completion loop as JSON");
    println!("  --models [modality]      Show broker model catalog");
    println!("  --install-model <key>    Download a known local model artifact");
    println!("  --ui-model-candidates    Show ranked local UI model options");
    println!("  --tool-model-candidates  Show ranked local tool-calling model options");
    println!("  --model-roles            Show Flow's local model routing policy");
    println!("  --verify-local-models    Run a bounded local model verification pass");
    println!("  --uigen <out.html> <prompt>");
    println!("                           Generate a single-file UI with the default UI model");
    println!("  --uigen-model <key> <out.html> <prompt>");
    println!("                           Generate with a selected UI model");
    println!("  --uigen-google [key]     Generate the Google homepage clone eval file");
    println!("  --uigen-vision <screenshot.png> <out.html> <prompt>");
    println!("                           Generate a UI from a screenshot with Gemma frontend");
    println!("  --uigen-vision-google    Capture Google and generate a vision UI clone");
    println!("  --plan <modality> [key]  Show runtime broker execution plan");
    println!("  --blueprint [host]       Show the embedding blueprint for a host");
    println!("  --browser-profile [flavor]");
    println!("                           Show default browser capability profile");
    println!("  --browser-plan <flavor> <task> <modality> [model] [--remote]");
    println!("                           Show browser execution plan");
    println!("  --browser-packs          Show registered browser-ready model packs");
    println!("  --grammar [--fix] <text> Analyze or correct text");
    println!("  --wakewords              Show local wake-word models");
    println!("  --production-config [target]");
    println!("                           Print the recommended production config JSON");
    println!("  --export-production-bundle [dir]");
    println!("                           Export all production configs and a manifest");
    println!("  --release-summary        Print the current release summary JSON");
    println!("  --export-release-summary [dir]");
    println!("                           Export release summary and handoff markdown");
    println!();
    println!("Examples:");
    println!("  cargo run --bin flow -- --profile");
    println!("  cargo run --bin flow -- --projects");
    println!("  cargo run --bin flow -- --scorecard");
    println!("  cargo run --bin flow -- --accessibility windows");
    println!("  cargo run --bin flow -- --audit-log tmp/flow-state.txt");
    println!("  cargo run --bin flow -- --completion");
    println!("  cargo run --bin flow -- --completion-json");
    println!("  cargo run --bin flow -- --friday-ocr-smoke tmp/friday-ocr-smoke");
    println!("  cargo run --bin flow -- --friday-vlm-contract tmp/friday-vlm-contract");
    println!("  cargo run --bin flow -- --friday-multimodal-route vlm");
    println!("  cargo run --bin flow -- --friday-multimodal-diagnostics");
    println!("  cargo run --bin flow -- --models chat");
    println!("  cargo run --release --bin flow -- --install-model qwen3-0.6b");
    println!("  cargo run --release --bin flow -- --install-model qwen35-4b-revised-q4km");
    println!("  cargo run --release --bin flow -- --install-model xlam2-3b-fc-r-q4km");
    println!("  cargo run --release --bin flow -- --install-model qwen35-9b-q4km");
    println!("  cargo run --release --bin flow -- --chat qwen35-4b-revised-q4km");
    println!("  cargo run --release --bin flow -- --tool-agent \"choose a tool for this request\"");
    println!(
        "  cargo run --release --bin flow -- --tool-agent-tools examples/tool-agent/weather-tools.json \"weather in Dhaka tomorrow\""
    );
    println!("  cargo run --release --bin flow -- --install-model webgen-4b-preview-i1-q4km");
    println!("  cargo run --release --bin flow -- --install-model qwendean-4b-q4km");
    println!("  cargo run --release --bin flow -- --install-model gemma4-e4b-frontend-q4km");
    println!("  cargo run --release --bin flow -- --ui-model-candidates");
    println!("  cargo run --release --bin flow -- --tool-model-candidates");
    println!("  cargo run --release --bin flow -- --model-roles");
    println!("  cargo run --release --bin flow -- --verify-local-models");
    println!("  cargo run --release --bin flow -- --models ui");
    println!("  cargo run --release --bin flow -- --models vlm");
    println!(
        "  cargo run --release --bin flow -- --uigen tmp/uigen-output/index.html \"make a shadcn-style Google homepage clone\""
    );
    println!("  cargo run --release --bin flow -- --uigen-google webgen-4b-preview-i1-q4km");
    println!("  cargo run --release --bin flow -- --uigen-google qwendean-4b-q4km");
    println!("  cargo run --release --bin flow -- --uigen-vision-google");
    println!("  cargo run --bin flow -- --plan chat qwen3-0.6b");
    println!("  cargo run --bin flow -- --blueprint dx");
    println!("  cargo run --bin flow -- --browser-profile chromium");
    println!("  cargo run --bin flow -- --browser-plan chromium rewrite-selection chat");
    println!("  cargo run --bin flow -- --browser-packs");
    println!("  cargo run --bin flow -- --grammar --fix \"This is an test.\"");
    println!("  cargo run --bin flow -- --production-config codex-fork");
    println!("  cargo run --bin flow -- --export-production-bundle configs/production");
    println!("  cargo run --bin flow -- --release-summary");
    println!("  cargo run --bin flow -- --export-release-summary release");
}

fn print_profile(broker: &RuntimeBroker) {
    let profile = broker.device_profile();
    println!("Flow Device Profile");
    println!("===================");
    println!("OS: {} / {}", profile.os, profile.arch);
    println!("CPU: {}", profile.cpu_model);
    println!(
        "Cores: {} physical / {} logical",
        profile.physical_cores, profile.logical_cores
    );
    println!(
        "Memory: {:.1} GB total / {:.1} GB available",
        profile.total_memory_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
        profile.available_memory_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    );
    println!("Tier: {:?}", profile.tier);
    println!();

    if profile.graphics.is_empty() {
        println!("Graphics: none detected");
    } else {
        println!("Graphics:");
        for gpu in &profile.graphics {
            let vram = gpu
                .vram_bytes
                .map(format_bytes)
                .unwrap_or_else(|| "unknown".to_string());
            println!(
                "  - {} ({}, {}, backends: {})",
                gpu.name,
                gpu.vendor.as_deref().unwrap_or("unknown"),
                vram,
                gpu.backends
                    .iter()
                    .map(|backend| format!("{backend:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    println!();
    println!(
        "Push-to-talk: {}",
        shortcut_label(
            &broker.activation().push_to_talk.modifiers,
            &broker.activation().push_to_talk.key
        )
    );
    println!(
        "Hands-free toggle: {}",
        shortcut_label(
            &broker.activation().hands_free_toggle.modifiers,
            &broker.activation().hands_free_toggle.key
        )
    );

    if broker.activation().wake_words.is_empty() {
        println!("Wake words: none detected in models/wake_words");
    } else {
        println!("Wake words:");
        for item in &broker.activation().wake_words {
            let aliases = if item.aliases.is_empty() {
                "-".to_string()
            } else {
                item.aliases.join(", ")
            };
            println!(
                "  - {} ({}) (aliases: {}, threshold: {}%, model: {})",
                item.command_key, item.phrase, aliases, item.threshold, item.model_path
            );
        }
    }
}

fn print_projects() {
    println!("DX Project Stack");
    println!("================");
    for project in dx_project_statuses() {
        println!(
            "- {}: {} ({}%)",
            project.key, project.role, project.completeness_score
        );
    }
}

fn print_scorecard() {
    let scorecard = default_competitive_scorecard();

    println!("Flow Competitive Scorecard");
    println!("==========================");
    println!("Measured on: {}", scorecard.measured_on);
    println!("Overall: {} / 100", scorecard.overall_score_out_of_100);
    println!(
        "Wispr replacement: {} / 100",
        scorecard.wispr_replacement_score_out_of_100
    );
    println!(
        "Grammarly replacement: {} / 100",
        scorecard.grammarly_replacement_score_out_of_100
    );
    println!(
        "Flow-native advantage: {} / 100",
        scorecard.flow_native_advantage_score_out_of_100
    );
    println!();
    println!("Top gaps:");
    for gap in scorecard.top_gaps {
        println!("  - {}", gap);
    }
}

fn print_friday_plan() {
    let plan = default_friday_product_plan();

    println!("Friday Competitive AI Workspace");
    println!("================================");
    println!("Measured on: {}", plan.measured_on);
    println!("Progress: {} / 100", plan.score_out_of_100);
    println!("Search engine: {}", plan.search_policy.engine);
    println!(
        "Perplexity Computer dependency: {}",
        if plan.search_policy.forbids_perplexity_computer {
            "forbidden"
        } else {
            "allowed"
        }
    );
    println!();

    println!("Workspace views:");
    for view in &plan.workspace_views {
        println!("  - {} ({})", view.title, view.route);
        println!("    {}", view.objective);
    }
    println!();

    println!("Capability map:");
    for capability in &plan.capabilities {
        println!(
            "  - [{}] {} / {} -> {}",
            capability.friday_status.label(),
            capability.competitor.label(),
            capability.area.label(),
            capability.feature
        );
        if capability.uses_metasearch {
            println!("    search: metasearch-first");
        }
        if capability.friday_status != FridayFeatureStatus::Shipped {
            println!("    next: {}", capability.implementation_note);
        }
    }
    println!();

    println!("Top priorities:");
    for priority in &plan.top_priorities {
        println!("  - {}", priority);
    }
}

fn print_friday_plan_json() -> Result<()> {
    println!("{}", default_friday_product_plan().to_pretty_json()?);
    Ok(())
}

fn print_friday_ui_plan() {
    let plan = default_friday_ui_integration_plan();

    println!("Friday Product UI Integration");
    println!("=============================");
    println!("Progress: {} / 100", plan.score_out_of_100);
    println!("Wired routes: {}", plan.ready_route_count());
    println!();

    for route in &plan.routes {
        println!(
            "- {} ({}) [{}]",
            route.title,
            route.route,
            route.status.label()
        );
        println!("  model: {}", route.model_role);
        println!("  command: {}", route.primary_command);
        println!(
            "  stream={}, citations={}, save_report={}",
            yes_no(route.stream_enabled),
            yes_no(route.citations_visible),
            yes_no(route.report_persistence)
        );
        println!(
            "  sources: {}",
            route
                .source_controls
                .iter()
                .filter(|source| source.default_enabled)
                .map(|source| source.key.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "  states: {}",
            route
                .states
                .iter()
                .map(|state| format!(
                    "{:?}/{}",
                    state.kind,
                    if state.blocks_interaction {
                        "blocking"
                    } else {
                        "nonblocking"
                    }
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );
        if route.status != FridayUiIntegrationStatus::Wired {
            println!("  next: connect route bindings");
        }
    }

    println!();
    println!("Next actions:");
    for action in &plan.next_actions {
        println!("  - {}", action);
    }
}

fn print_friday_live_ui_routes() {
    let report = friday_live_ui_route_binding_report();

    println!("Friday Live UI Route Bindings");
    println!("=============================");
    println!("{}", report.summary);
    println!("Score: {} / 100", report.score_out_of_100);
    println!(
        "Routes: {} passed, {} warning, {} blocking",
        report.passed_count, report.warning_count, report.blocking_count
    );
    println!();

    for route in &report.routes {
        println!(
            "- [{}] {} ({})",
            route.status.label(),
            route.title,
            route.route
        );
        println!("  command: {}", route.primary_command);
        for file in &route.source_files {
            println!(
                "  file: {} [{}] exists={}, bytes={}",
                file.path,
                file.role,
                yes_no(file.exists),
                file.bytes
            );
        }
        for item in &route.evidence {
            println!("  evidence: {}", item);
        }
        if route.status != crate::friday::FridayLiveUiBindingStatus::Passed {
            println!("  next: {}", route.next_action);
        }
    }
}

fn print_friday_readiness() {
    let report = friday_operator_readiness_report();

    println!("Friday Operator Readiness");
    println!("=========================");
    println!("{}", report.summary);
    println!("Score: {} / 100", report.score_out_of_100);
    println!(
        "Areas: {} passed, {} warning, {} blocking",
        report.passed_count, report.warning_count, report.blocking_count
    );
    println!();

    for item in &report.items {
        println!(
            "- [{}] {} ({} / 100)",
            item.status.label(),
            item.title,
            item.score_out_of_100
        );
        println!("  command: {}", item.command);
        println!("  local_only: {}", yes_no(item.local_only));
        for evidence in &item.evidence {
            println!("  evidence: {}", evidence);
        }
        if item.status != crate::friday::FridayOperatorReadinessStatus::Passed {
            println!("  next: {}", item.next_action);
        }
    }
}

fn print_friday_route_visuals() {
    let report = friday_route_visual_report();

    println!("Friday Route Screenshot Targets");
    println!("===============================");
    println!("{}", report.summary);
    println!("Score: {} / 100", report.score_out_of_100);
    println!("Artifact root: {}", report.artifact_root);
    println!(
        "Targets: {} passed, {} warning, {} blocking",
        report.passed_count, report.warning_count, report.blocking_count
    );
    println!();

    for target in &report.targets {
        println!(
            "- [{}] {} {} ({}x{})",
            target.status.label(),
            target.title,
            target.route,
            target.viewport.width,
            target.viewport.height
        );
        println!("  layout: {}", target.viewport.expected_layout);
        println!("  source: {}", target.source_file);
        println!("  screenshot: {}", target.screenshot_path);
        println!("  metadata: {}", target.metadata_path);
        println!("  capture: {}", target.capture_command);
        for evidence in &target.evidence {
            println!("  evidence: {}", evidence);
        }
        if target.status != crate::friday::FridayRouteVisualStatus::Passed {
            println!("  next: {}", target.next_action);
        }
    }
}

fn print_friday_execution_handoffs() {
    let report = friday_execution_handoff_report();

    println!("Friday Execution Handoffs");
    println!("=========================");
    println!("{}", report.summary);
    println!("Score: {} / 100", report.score_out_of_100);
    println!(
        "Handoffs: {} passed, {} warning, {} blocking",
        report.passed_count, report.warning_count, report.blocking_count
    );
    println!();

    for handoff in &report.handoffs {
        println!(
            "- [{}] {} / {} ({})",
            handoff.status.label(),
            handoff.title,
            handoff.id,
            handoff.surface.label()
        );
        println!("  route: {}", handoff.route);
        println!("  command: {}", handoff.command);
        println!("  source: {}", handoff.source_file);
        println!("  local_only: {}", yes_no(handoff.local_only));
        println!("  user_gesture: {}", yes_no(handoff.requires_user_gesture));
        println!("  permissions: {}", handoff.permission_scopes.join(", "));
        if let Some(path) = &handoff.artifact_path {
            println!("  artifact: {}", path);
        }
        println!("  recovery: {}", handoff.recovery_command);
        for evidence in &handoff.evidence {
            println!("  evidence: {}", evidence);
        }
        if handoff.status != crate::friday::FridayExecutionHandoffStatus::Passed {
            println!("  next: {}", handoff.next_action);
        }
    }
}

fn print_friday_dashboard_export(output_dir: &str) -> Result<()> {
    let bundle = export_friday_dashboard_bundle(resolve_repo_relative_path(output_dir))?;
    let manifest = &bundle.manifest;

    println!("Friday Dashboard Export");
    println!("=======================");
    println!("{}", manifest.summary);
    println!("Score: {} / 100", manifest.score_out_of_100);
    println!("Directory: {}", manifest.export_dir);
    println!();

    println!("Files:");
    for file in &manifest.files {
        println!("  - {} [{}] {} bytes", file.path, file.kind, file.bytes);
    }
    println!("  - {} [manifest]", manifest.manifest_json);
    println!();

    println!("Commands:");
    for command in &manifest.commands {
        println!("  - {}", command);
    }

    Ok(())
}

fn print_friday_dashboard_panel(input_dir: &str) -> Result<()> {
    let panel = friday_dashboard_panel_from_export(resolve_repo_relative_path(input_dir))?;

    println!("Friday Dashboard Panel");
    println!("======================");
    println!("{}", panel.summary);
    println!(
        "Status: {} ({} / 100)",
        panel.status.label(),
        panel.score_out_of_100
    );
    println!("Export: {}", panel.export_dir);
    println!();

    println!("Cards:");
    for card in &panel.cards {
        println!(
            "- [{}] {} ({} / 100)",
            card.status.label(),
            card.title,
            card.score_out_of_100
        );
        println!("  metric: {}", card.primary_metric);
        println!("  source: {}", card.source_json);
        for action in &card.actions {
            println!(
                "  action: [{}] {} -> {}",
                action.kind.label(),
                action.label,
                action.command
            );
        }
    }

    println!();
    println!(
        "Screenshots: {} captured, {} missing, {} missing metadata",
        panel.screenshot_history.captured_count,
        panel.screenshot_history.missing_count,
        panel.screenshot_history.metadata_missing_count
    );
    println!(
        "History: {} checkpoint(s), score delta {:+}, readiness delta {:+}",
        panel.export_history.record_count,
        panel.export_history.score_delta_from_previous,
        panel.export_history.readiness_delta_from_previous
    );
    println!(
        "Release review: {} ({}/{} checks ready)",
        panel.release_review.status.label(),
        panel.release_review.ready_count,
        panel.release_review.total_count
    );
    for record in panel
        .screenshot_history
        .records
        .iter()
        .filter(|record| record.status != crate::friday::FridayDashboardScreenshotStatus::Captured)
        .take(5)
    {
        println!(
            "  - [{}] {} {}: {}",
            record.status.label(),
            record.title,
            record.viewport_id,
            record.prompt
        );
    }

    if !panel.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &panel.warnings {
            println!("  - {}", warning);
        }
    }

    Ok(())
}

fn print_friday_dashboard_product_ui(input_dir: &str) -> Result<()> {
    let binding =
        friday_dashboard_product_ui_binding_from_export(resolve_repo_relative_path(input_dir))?;

    println!("Friday Dashboard Product UI Binding");
    println!("===================================");
    println!("{}", binding.summary);
    println!(
        "Route: {} -> {} ({})",
        binding.route,
        binding.source_file,
        binding.status.label()
    );
    println!("Score: {} / 100", binding.score_out_of_100);
    println!("Panel JSON: {}", binding.panel_json_command);
    println!("Export refresh: {}", binding.export_command);
    println!(
        "Cards: {}/{} bound, actions: {}, warnings: {}, blocking: {}",
        binding.bound_card_count,
        binding.card_count,
        binding.action_count,
        binding.warning_count,
        binding.blocking_count
    );
    println!(
        "Release links: {}, screenshot prompts: {}",
        binding.release_links.len(),
        binding.screenshot_prompts.len()
    );
    println!();

    println!("Data bindings:");
    for data_binding in &binding.data_bindings {
        println!(
            "  - {} -> {} ({})",
            data_binding.source, data_binding.writes_to, data_binding.command
        );
    }

    println!();
    println!("Cards:");
    for card in &binding.cards {
        println!(
            "  - [{}] {}: {}",
            card.status.label(),
            card.title,
            card.primary_metric
        );
    }

    println!();
    println!("Release links:");
    for link in &binding.release_links {
        println!("  - [{}] {} -> {}", link.section, link.label, link.path);
    }

    Ok(())
}

fn print_friday_dashboard_product_ui_smoke(input_dir: &str) -> Result<()> {
    let report =
        friday_dashboard_product_ui_smoke_from_export(resolve_repo_relative_path(input_dir))?;

    println!("Friday Dashboard Product UI Smoke");
    println!("=================================");
    println!("{}", report.summary);
    println!(
        "Route: {} -> {} ({})",
        report.route,
        report.source_file,
        report.status.label()
    );
    println!("Score: {} / 100", report.score_out_of_100);
    println!(
        "Checks: {} passed, {} warning, {} blocking / {} total",
        report.passed_count, report.warning_count, report.blocking_count, report.check_count
    );
    println!();

    for check in &report.checks {
        println!(
            "  - [{}] {}: {}",
            check.status.label(),
            check.title,
            check.evidence.join(", ")
        );
        if check.status != crate::friday::FridayDashboardProductUiSmokeStatus::Passed {
            println!("    next: {}", check.next_action);
        }
    }

    Ok(())
}

fn print_friday_dashboard_host_command_bridge(input_dir: &str) -> Result<()> {
    let report =
        friday_dashboard_host_command_bridge_from_export(resolve_repo_relative_path(input_dir))?;

    println!("Friday Dashboard Host Command Bridge");
    println!("====================================");
    println!("{}", report.summary);
    println!("Route: {}", report.route);
    println!("Source: {}", report.source_command);
    println!(
        "Commands: {}, awaiting approval: {}, blocked: {}, audit records: {}",
        report.command_count,
        report.awaiting_approval_count,
        report.blocked_count,
        report.audit_count
    );
    println!();

    for record in &report.records {
        println!(
            "  - [{}] {} ({})",
            record.status.label(),
            record.label,
            record.approval_state.label()
        );
        println!("    command: {}", record.command);
        println!(
            "    silent_execution_allowed={}, can_execute_after_approval={}",
            yes_no(record.silent_execution_allowed),
            yes_no(record.can_execute_after_approval)
        );
        if let Some(reason) = &record.blocked_reason {
            println!("    blocked: {}", reason);
        }
        println!(
            "    audit: {} | stdout={} | stderr={}",
            record.audit.event, record.audit.stdout_summary, record.audit.stderr_summary
        );
    }

    Ok(())
}

fn run_friday_trusted_host_runner_command(
    input_dir: &str,
    action_id: Option<&str>,
    approve: bool,
    execute: bool,
    cancel: bool,
    history_file: &str,
    reason: Option<&str>,
) -> Result<FridayTrustedHostRunnerResult> {
    let bridge =
        friday_dashboard_host_command_bridge_from_export(resolve_repo_relative_path(input_dir))?;
    let record = if let Some(action_id) = action_id {
        bridge
            .records
            .iter()
            .find(|record| record.action_id == action_id)
            .with_context(|| format!("No trusted host command record found for `{action_id}`"))?
    } else {
        bridge
            .records
            .first()
            .context("No trusted host command records are available")?
    };
    let request = FridayTrustedHostRunnerRequest {
        approved: approve && execute,
        cancel_requested: cancel,
        operator_reason: reason.map(str::to_string),
        ..Default::default()
    };
    let result = run_friday_trusted_host_command(record, &request);
    append_friday_trusted_host_runner_history(
        resolve_repo_relative_path(history_file),
        result.clone(),
    )?;
    Ok(result)
}

fn print_friday_trusted_host_runner_result(result: &FridayTrustedHostRunnerResult) {
    println!("Friday Trusted Host Runner");
    println!("==========================");
    println!("Action: {} ({})", result.label, result.action_id);
    println!("Status: {}", result.status.label());
    println!("Command: {}", result.command);
    println!("Approved: {}", yes_no(result.approved));
    println!("Cancelled: {}", yes_no(result.cancelled));
    println!(
        "Reason: {}",
        result.operator_reason.as_deref().unwrap_or("not recorded")
    );
    println!(
        "Exit code: {}",
        result
            .exit_code
            .map_or("n/a".to_string(), |code| code.to_string())
    );
    println!(
        "Duration: {}ms / timeout {}ms",
        result.duration_ms, result.timeout_ms
    );
    println!(
        "Stdout: {}{}",
        result.stdout_summary,
        if result.stdout_truncated {
            " [truncated]"
        } else {
            ""
        }
    );
    println!(
        "Stderr: {}{}",
        result.stderr_summary,
        if result.stderr_truncated {
            " [truncated]"
        } else {
            ""
        }
    );
}

fn print_friday_trusted_host_runner_ux_report(report: &FridayTrustedHostRunnerUxReport) {
    println!("Friday Dashboard Runner UX");
    println!("==========================");
    println!(
        "History: {} ({} result(s))",
        report.history_json, report.result_count
    );
    println!(
        "Latest status: {}",
        report
            .latest_status
            .map(|status| status.label().to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!();
    println!("Status groups:");
    for summary in &report.status_summaries {
        println!(
            "  - {}: {} [{}]",
            summary.title, summary.count, summary.tone
        );
        println!("    {}", summary.description);
    }
    println!();
    println!("Affordances:");
    for affordance in &report.affordances {
        println!(
            "  - {} ({}, approval: {}, disabled: {})",
            affordance.label,
            affordance.kind,
            yes_no(affordance.requires_approval),
            yes_no(affordance.disabled)
        );
        println!("    command: {}", affordance.command);
        println!("    {}", affordance.detail);
        if let Some(reason) = &affordance.disabled_reason {
            println!("    disabled: {}", reason);
        }
    }
    println!();
    println!("Operator notes:");
    for note in &report.operator_notes {
        println!("  - {}: {}", note.label, note.detail);
        println!("    release review: {}", note.release_review_path);
    }
}

fn print_friday_trusted_host_runner_approval_ui_report(
    report: &FridayTrustedHostRunnerApprovalUiReport,
) {
    println!("Friday Runner Approval UI");
    println!("=========================");
    println!("{}", report.title);
    println!("{}", report.body);
    println!("History: {}", report.history_json);
    println!("Command: {}", report.command_preview);
    println!(
        "Reason required: {} ({})",
        yes_no(report.audit_reason_required),
        report.reason_label
    );
    println!();
    println!("Controls:");
    for control in &report.controls {
        println!(
            "  - {} ({}, approval: {}, reason: {}, disabled: {})",
            control.label,
            control.kind,
            yes_no(control.requires_approval),
            yes_no(control.requires_reason),
            yes_no(control.disabled)
        );
        if !control.command.is_empty() {
            println!("    command: {}", control.command);
        }
        if let Some(shortcut) = &control.keyboard_shortcut {
            println!("    shortcut: {} - {}", shortcut.key, shortcut.detail);
        }
        if let Some(reason) = &control.disabled_reason {
            println!("    disabled: {}", reason);
        }
    }
    println!();
    println!("Snooze:");
    for option in &report.snooze_options {
        println!("  - {} ({}s)", option.label, option.duration_seconds);
    }
    println!("Undo: {}", report.undo_note);
    println!("Release review: {}", report.release_review_path);
}

fn print_friday_trusted_host_runner_cancellation_ux_report(
    report: &FridayTrustedHostRunnerCancellationUxReport,
) {
    println!("Friday Runner Cancellation UX");
    println!("=============================");
    println!("State: {}", report.state_json);
    println!(
        "Records: {} | active: {} | stale: {} | denied: {}",
        report.record_count, report.active_count, report.stale_count, report.denial_count
    );
    println!("Drafts: {}", report.draft.autosave_hint);
    println!();
    println!("Guidance:");
    for item in &report.guidance {
        println!("  - {}", item);
    }
    println!();
    println!("Controls:");
    for control in &report.controls {
        println!(
            "  - {} ({}, reason: {}, disabled: {})",
            control.label,
            control.kind,
            yes_no(control.requires_reason),
            yes_no(control.disabled)
        );
        println!("    command: {}", control.command);
        println!("    {}", control.detail);
        if let Some(reason) = &control.disabled_reason {
            println!("    disabled: {}", reason);
        }
    }
}

fn trusted_host_runner_review_filter(
    status: &Option<String>,
    action_id: &Option<String>,
    since_ms: &Option<String>,
    until_ms: &Option<String>,
    limit: usize,
) -> Result<FridayTrustedHostRunnerOperatorReviewFilter> {
    Ok(FridayTrustedHostRunnerOperatorReviewFilter {
        status: match status
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some("all") | None => None,
            Some(value) => Some(parse_trusted_host_runner_status(value)?),
        },
        action_id: action_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        since_unix_ms: parse_optional_u128_flag(since_ms, "--since-ms")?,
        until_unix_ms: parse_optional_u128_flag(until_ms, "--until-ms")?,
        limit,
    })
}

fn parse_optional_u128_flag(value: &Option<String>, name: &str) -> Result<Option<u128>> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .parse::<u128>()
                .with_context(|| format!("Invalid {name} value `{value}`"))
        })
        .transpose()
}

fn parse_trusted_host_runner_status(value: &str) -> Result<FridayTrustedHostRunnerStatus> {
    match value {
        "succeeded" | "success" => Ok(FridayTrustedHostRunnerStatus::Succeeded),
        "failed" | "failure" => Ok(FridayTrustedHostRunnerStatus::Failed),
        "timed-out" | "timeout" => Ok(FridayTrustedHostRunnerStatus::TimedOut),
        "cancelled" | "canceled" => Ok(FridayTrustedHostRunnerStatus::Cancelled),
        "denied" | "blocked" => Ok(FridayTrustedHostRunnerStatus::Denied),
        _ => anyhow::bail!(
            "Invalid trusted runner status `{value}`. Use succeeded, failed, timed-out, cancelled, denied, or all."
        ),
    }
}

fn print_friday_trusted_host_runner_operator_review_report(
    report: &FridayTrustedHostRunnerOperatorReviewReport,
) {
    println!("Friday Runner Operator Review");
    println!("=============================");
    println!("History: {}", report.history_json);
    println!(
        "Gate: {} | matched: {} / {} | blocked: {} | ready: {}",
        report.release_gate_status,
        report.matched_count,
        report.record_count,
        report.blocked_count,
        report.ready_count
    );
    println!();
    println!("Filters:");
    println!(
        "  status: {}",
        report
            .filters
            .status
            .map(|status| status.label().to_string())
            .unwrap_or_else(|| "all".to_string())
    );
    println!(
        "  action: {}",
        report.filters.action_id.as_deref().unwrap_or("all")
    );
    println!("  limit: {}", report.filters.limit);
    println!();
    println!("Release gate:");
    for summary in &report.release_gate_summaries {
        println!(
            "  - {}: {} [{}]",
            summary.title, summary.count, summary.severity
        );
        println!("    {}", summary.detail);
        println!("    next: {}", summary.next_action);
    }
    println!();
    println!("Incident notes:");
    for note in &report.incident_notes {
        println!("  - {} [{}]", note.title, note.severity);
        println!("    {}", note.body);
    }
}

fn print_friday_trusted_runner_release_package_report(
    report: &FridayTrustedRunnerReleasePackageReport,
) {
    println!("Friday Trusted Runner Release Package");
    println!("=====================================");
    println!("{}", report.summary);
    println!("Ready to ship: {}", yes_no(report.ready_to_ship));
    println!("Package: {}", report.manifest.package_json);
    println!("Signature: {}", report.manifest.package_signature);
    println!(
        "Evidence: {} file(s), missing: {}, warnings: {}",
        report.manifest.evidence_count,
        report.manifest.missing_count,
        report.manifest.warning_count
    );
    println!();
    println!("Warnings:");
    if report.warnings.is_empty() {
        println!("  - none");
    } else {
        for warning in &report.warnings {
            println!("  - {}", warning);
        }
    }
    println!();
    println!("Evidence files:");
    for file in &report.manifest.files {
        println!(
            "  - {} [{}] present={} bytes={} sha256={}",
            file.label,
            file.kind,
            yes_no(file.present),
            file.bytes,
            file.sha256.as_deref().unwrap_or("missing")
        );
        println!("    {}", file.path);
    }
}

fn print_friday_trusted_runner_release_timeline(timeline: &FridayTrustedRunnerReleaseTimeline) {
    println!("Friday Trusted Runner Release Timeline");
    println!("======================================");
    println!("Timeline: {}", timeline.timeline_json);
    println!(
        "Packages: {} | ready: {} | blocked: {} | regressions: {} missing, {} warning | signature changes: {}",
        timeline.package_count,
        timeline.ready_count,
        timeline.blocked_count,
        timeline.missing_evidence_regressions,
        timeline.warning_regressions,
        timeline.signature_changes
    );
    if let Some(latest) = &timeline.latest_package_id {
        println!("Latest: {}", latest);
    }
    println!();
    println!("Warnings:");
    if timeline.warnings.is_empty() {
        println!("  - none");
    } else {
        for warning in &timeline.warnings {
            println!("  - {}", warning);
        }
    }
    println!();
    println!("Packages:");
    for entry in &timeline.entries {
        println!(
            "  - {} ready={} missing={} warnings={} stale={} signature={}",
            entry.package_id,
            yes_no(entry.ready_to_ship),
            entry.missing_count,
            entry.warning_count,
            entry.stale_warning_count,
            entry.package_signature
        );
        println!("    {}", entry.package_json);
    }
    println!();
    println!("Diffs:");
    if timeline.diffs.is_empty() {
        println!("  - none");
    } else {
        for diff in &timeline.diffs {
            println!(
                "  - {} -> {} regression={} missing_delta={} warning_delta={}",
                diff.from_package_id,
                diff.to_package_id,
                yes_no(diff.regression),
                diff.missing_delta,
                diff.warning_delta
            );
            println!("    {}", diff.summary);
        }
    }
}

fn print_friday_release_operator_checklist(report: &FridayReleaseOperatorChecklistReport) {
    println!("Friday Release Operator Checklist");
    println!("=================================");
    println!("{}", report.summary);
    println!("Ready to ship: {}", yes_no(report.ready_to_ship));
    println!("Checklist: {}", report.checklist_json);
    println!(
        "Items: {}/{} ready | warnings: {} | blocking: {} | signoffs: {}",
        report.ready_count,
        report.total_count,
        report.warning_count,
        report.blocking_count,
        report.signoff_count
    );
    println!();
    println!("Blockers:");
    if report.blockers.is_empty() {
        println!("  - none");
    } else {
        for blocker in &report.blockers {
            println!(
                "  - {} [{}:{}]",
                blocker.title,
                blocker.category,
                blocker.severity.label()
            );
            println!("    {}", blocker.detail);
            println!("    next: {}", blocker.next_action);
        }
    }
    println!();
    println!("Checklist:");
    for item in &report.checklist {
        println!(
            "  - {} ready={} ({})",
            item.title,
            yes_no(item.ready),
            item.source_path
        );
        println!("    {}", item.detail);
    }
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {}", command);
    }
}

fn print_friday_release_signoffs(signoffs: &[FridayReleaseChecklistSignoff]) {
    println!("Friday Release Signoffs");
    println!("=======================");
    println!("Records: {}", signoffs.len());
    for signoff in signoffs.iter().rev().take(5) {
        println!(
            "  - {} by {} for {}",
            signoff.decision.label(),
            signoff.operator,
            signoff.checklist_id
        );
        println!("    {}", signoff.reason);
    }
}

fn print_friday_release_qa_command_center(report: &FridayReleaseQaCommandCenterReport) {
    println!("Friday Release QA Command Center");
    println!("================================");
    println!("{}", report.summary);
    println!("Ready to ship: {}", yes_no(report.ready_to_ship));
    println!("Score: {} / 100", report.score_out_of_100);
    println!(
        "Warnings: {} | blocking: {} | stale: {} | missing: {}",
        report.warning_count, report.blocking_count, report.stale_count, report.missing_count
    );
    println!();
    println!("Checks:");
    for check in &report.checks {
        println!(
            "  - {} [{}] present={} stale={}",
            check.label,
            check.status.label(),
            yes_no(check.present),
            yes_no(check.stale)
        );
        println!("    command: {}", check.command);
        println!("    result: {}", check.result_path);
        println!("    next: {}", check.next_action);
    }
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {}", command);
    }
}

fn print_friday_release_evidence_export_kit(report: &FridayReleaseEvidenceExportKitReport) {
    println!("Friday Release Evidence Export Kit");
    println!("==================================");
    println!("{}", report.summary);
    println!("Ready to attach: {}", yes_no(report.ready_to_attach));
    println!(
        "Files: {} | required: {} | missing: {} | stale: {} | warnings: {}",
        report.manifest.file_count,
        report.manifest.required_count,
        report.manifest.missing_count,
        report.manifest.stale_count,
        report.manifest.warning_count
    );
    println!("Kit: {}", report.manifest.kit_json);
    println!("Manifest checksum: {}", report.manifest.manifest_sha256);
    println!();
    println!("Evidence:");
    for file in &report.manifest.files {
        println!(
            "  - {} [{}] present={} stale={} bytes={} sha256={}",
            file.label,
            file.kind,
            yes_no(file.present),
            yes_no(file.stale),
            file.bytes,
            file.sha256.as_deref().unwrap_or("missing")
        );
        println!("    path: {}", file.path);
        if let Some(warning) = &file.warning {
            println!("    warning: {warning}");
        }
    }
    if !report.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }
    println!();
    println!("Operator copy:");
    println!("{}", report.operator_copy);
    println!();
    println!("Commands:");
    for command in &report.manifest.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_deployment_gate(report: &FridayReleaseDeploymentGateReport) {
    println!("Friday Release Deployment Gate");
    println!("==============================");
    println!("{}", report.summary);
    println!("Decision: {}", report.decision.label());
    println!("Ready to deploy: {}", yes_no(report.ready_to_deploy));
    println!("Score: {} / 100", report.score_out_of_100);
    println!(
        "Target: {} ({}, {})",
        report.target.label, report.target.environment, report.target.provider
    );
    println!(
        "Reasons: {} blocking, {} warning(s) | Checklist: {}/{} ready",
        report.no_deploy_reason_count, report.warning_count, report.ready_count, report.total_count
    );
    println!("Gate: {}", report.gate_json);
    println!();
    println!("No-deploy reasons:");
    if report.reasons.is_empty() {
        println!("  - none");
    } else {
        for reason in &report.reasons {
            println!(
                "  - {} [{}:{}]",
                reason.title,
                reason.category.label(),
                reason.severity.label()
            );
            println!("    {}", reason.detail);
            println!("    source: {}", reason.source_path);
            println!("    next: {}", reason.next_action);
        }
    }
    println!();
    println!("Deployment checklist:");
    for item in &report.checklist {
        println!(
            "  - {} ready={} ({})",
            item.title,
            yes_no(item.ready),
            item.source_path
        );
        println!("    {}", item.detail);
    }
    println!();
    println!("Operator copy:");
    println!("{}", report.operator_copy);
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {command}");
    }
}

fn run_friday_release_candidate_archive(
    archive_file: &str,
    gate_files: &[String],
    write_archive: bool,
) -> Result<FridayReleaseCandidateArchive> {
    let archive_path = resolve_repo_relative_path(archive_file);
    if gate_files.is_empty() {
        return Ok(
            read_friday_release_candidate_archive(&archive_path).unwrap_or_else(|_| {
                friday_release_candidate_archive_report(&archive_path, Vec::new())
            }),
        );
    }

    let mut archive = read_friday_release_candidate_archive(&archive_path)
        .unwrap_or_else(|_| friday_release_candidate_archive_report(&archive_path, Vec::new()));
    for gate_file in gate_files {
        archive = if write_archive {
            append_friday_release_candidate_to_archive(
                &archive_path,
                resolve_repo_relative_path(gate_file),
            )?
        } else {
            let mut entries = archive.entries;
            entries.push(friday_release_candidate_entry_from_gate(
                resolve_repo_relative_path(gate_file),
            )?);
            friday_release_candidate_archive_report(&archive_path, entries)
        };
    }
    Ok(archive)
}

fn print_friday_release_candidate_archive(archive: &FridayReleaseCandidateArchive) {
    println!("Friday Release Candidate Archive");
    println!("================================");
    println!(
        "Candidates: {} | go: {} | no-go: {} | draft: {} | regressions: {}",
        archive.candidate_count,
        archive.go_count,
        archive.no_go_count,
        archive.draft_count,
        archive.regression_count
    );
    println!("Archive: {}", archive.archive_json);
    if let Some(latest) = &archive.latest_candidate_id {
        println!("Latest: {}", latest);
    }
    if let Some(decision) = archive.latest_decision {
        println!("Latest decision: {}", decision.label());
    }
    if let Some(score) = archive.latest_score_out_of_100 {
        println!("Latest score: {} / 100", score);
    }
    println!();
    println!("Candidates:");
    if archive.entries.is_empty() {
        println!("  - none");
    } else {
        for entry in archive.entries.iter().rev().take(5) {
            println!(
                "  - {} [{}] score={} blockers={} target={}",
                entry.candidate_id,
                entry.decision.label(),
                entry.score_out_of_100,
                entry.no_deploy_reason_count,
                entry.target.label
            );
            println!("    gate: {}", entry.gate_json);
            println!("    export kit: {}", entry.export_kit_json);
            println!("    rollback: {}", entry.rollback_note);
        }
    }
    println!();
    println!("Diffs:");
    if archive.diffs.is_empty() {
        println!("  - none");
    } else {
        for diff in archive.diffs.iter().rev().take(5) {
            println!(
                "  - {} -> {} delta={} regression={}",
                diff.from_candidate_id,
                diff.to_candidate_id,
                diff.score_delta,
                yes_no(diff.regression)
            );
            println!("    {}", diff.summary);
            if !diff.new_blocker_ids.is_empty() {
                println!("    new blockers: {}", diff.new_blocker_ids.join(", "));
            }
            if !diff.resolved_blocker_ids.is_empty() {
                println!(
                    "    resolved blockers: {}",
                    diff.resolved_blocker_ids.join(", ")
                );
            }
        }
    }
    println!();
    println!("Commands:");
    for command in &archive.commands {
        println!("  - {command}");
    }
}

#[allow(clippy::too_many_arguments)]
fn run_friday_release_promotion_ledger(
    ledger_file: &str,
    archive_file: &str,
    candidate_id: Option<String>,
    decision: &str,
    operator: &str,
    reason: &str,
    deployment_note: &str,
    rollback_reference: &str,
    post_check_files: Vec<String>,
) -> Result<FridayReleasePromotionLedger> {
    let ledger_path = resolve_repo_relative_path(ledger_file);
    let archive_path = resolve_repo_relative_path(archive_file);

    if !archive_path.exists() {
        return Ok(read_friday_release_promotion_ledger(&ledger_path)
            .unwrap_or_else(|_| friday_release_promotion_ledger_report(&ledger_path, Vec::new())));
    }

    append_friday_release_promotion_to_ledger(
        &ledger_path,
        &archive_path,
        FridayReleasePromotionRecordRequest {
            candidate_id,
            decision: FridayReleasePromotionDecision::parse(decision)?,
            operator: operator.to_string(),
            reason: reason.to_string(),
            deployment_note: deployment_note.to_string(),
            rollback_reference: rollback_reference.to_string(),
            post_check_files,
        },
    )
}

fn print_friday_release_promotion_ledger(ledger: &FridayReleasePromotionLedger) {
    println!("Friday Release Promotion Ledger");
    println!("===============================");
    println!(
        "Records: {} | promoted: {} | held: {} | rolled-back: {} | superseded: {} | abandoned: {}",
        ledger.record_count,
        ledger.promoted_count,
        ledger.held_count,
        ledger.rolled_back_count,
        ledger.superseded_count,
        ledger.abandoned_count
    );
    println!("Ledger: {}", ledger.ledger_json);
    if let Some(decision) = ledger.latest_decision {
        println!("Latest decision: {}", decision.label());
    }
    if let Some(candidate_id) = &ledger.active_candidate_id {
        println!("Active candidate: {}", candidate_id);
    }
    if let Some(rollback) = &ledger.active_rollback_reference {
        println!("Active rollback: {}", rollback);
    }
    if ledger.post_promotion_missing_count > 0 {
        println!(
            "Missing post-promotion checks: {}",
            ledger.post_promotion_missing_count
        );
    }
    if !ledger.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &ledger.warnings {
            println!("  - {warning}");
        }
    }
    println!();
    println!("Promotion records:");
    if ledger.records.is_empty() {
        println!("  - none");
    } else {
        for record in ledger.records.iter().rev().take(5) {
            println!(
                "  - {} [{}] candidate={} score={} missing-checks={}",
                record.promotion_id,
                record.decision.label(),
                record.candidate_id,
                record.candidate_score_out_of_100,
                record.post_promotion_missing_count
            );
            println!("    operator: {}", record.operator);
            println!("    reason: {}", record.reason);
            println!("    deployment note: {}", record.deployment_note);
            println!("    rollback: {}", record.rollback_reference);
            for check in record.post_promotion_checks.iter().take(4) {
                println!(
                    "    check {}: {} ({})",
                    check.id,
                    if check.present { "present" } else { "missing" },
                    check.result_path
                );
            }
        }
    }
    println!();
    println!("Commands:");
    for command in &ledger.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_post_promotion_monitor(report: &FridayReleasePostPromotionMonitorReport) {
    println!("Friday Release Post-Promotion Monitor");
    println!("=====================================");
    println!(
        "Score: {} / 100 | status: {} | stable: {}",
        report.score_out_of_100,
        report.status.label(),
        yes_no(report.ready_for_stable)
    );
    println!(
        "Blocking: {} | warnings: {} | stale: {} | missing evidence: {}",
        report.blocking_count,
        report.warning_count,
        report.stale_count,
        report.missing_evidence_count
    );
    println!("Monitor: {}", report.monitor_json);
    if let Some(candidate_id) = &report.active_candidate_id {
        println!("Active candidate: {}", candidate_id);
    }
    if let Some(rollback) = &report.active_rollback_reference {
        println!("Rollback: {}", rollback);
    }
    if !report.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }
    println!();
    println!("Checks:");
    for check in &report.checks {
        println!(
            "  - {} [{}] {}",
            check.label,
            check.status.label(),
            check.summary
        );
        println!("    source: {}", check.source_path);
        println!("    next: {}", check.next_action);
    }
    if !report.incident_notes.is_empty() {
        println!();
        println!("Incident notes:");
        for note in &report.incident_notes {
            println!(
                "  - {} ({}) {}",
                note.id,
                if note.present { "present" } else { "missing" },
                note.path
            );
        }
    }
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_rollback_drill(report: &FridayReleaseRollbackDrillReport) {
    println!("Friday Release Rollback Drill");
    println!("=============================");
    println!(
        "Score: {} / 100 | status: {} | rollback ready: {} | stable: {}",
        report.score_out_of_100,
        report.status.label(),
        yes_no(report.ready_to_rollback),
        yes_no(report.ready_for_stable)
    );
    println!(
        "Blocking: {} | warnings: {} | stale: {} | missing evidence: {}",
        report.blocking_count,
        report.warning_count,
        report.stale_count,
        report.missing_evidence_count
    );
    println!("Drill: {}", report.drill_json);
    println!("Operator: {}", report.operator);
    println!("Reason: {}", report.reason);
    if let Some(candidate_id) = &report.active_candidate_id {
        println!("Active candidate: {}", candidate_id);
    }
    if let Some(rollback) = &report.active_rollback_reference {
        println!("Rollback reference: {}", rollback);
    }
    println!("Dry run: {}", report.dry_run_command);
    if !report.blocked_reasons.is_empty() {
        println!();
        println!("Blocked reasons:");
        for reason in &report.blocked_reasons {
            println!("  - {reason}");
        }
    }
    println!();
    println!("Checks:");
    for check in &report.checks {
        println!(
            "  - {} [{}] {}",
            check.label,
            check.status.label(),
            check.summary
        );
        println!("    source: {}", check.source_path);
        println!("    next: {}", check.next_action);
    }
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_stability_board(report: &FridayReleaseStabilityBoardReport) {
    println!("Friday Release Stability Evidence Board");
    println!("=======================================");
    println!(
        "Score: {} / 100 | status: {} | checkpoint ready: {}",
        report.score_out_of_100,
        report.status.label(),
        yes_no(report.ready_for_checkpoint)
    );
    println!(
        "Deploy: {} | stable: {} | recoverable: {}",
        yes_no(report.ready_to_deploy),
        yes_no(report.stable_after_promotion),
        yes_no(report.recoverable)
    );
    println!(
        "Blocking: {} | warnings: {} | stale: {} | missing evidence: {}",
        report.blocking_count,
        report.warning_count,
        report.stale_count,
        report.missing_evidence_count
    );
    println!("Board: {}", report.board_json);
    if let Some(candidate_id) = &report.active_candidate_id {
        println!("Active candidate: {}", candidate_id);
    }
    if let Some(rollback) = &report.active_rollback_reference {
        println!("Rollback reference: {}", rollback);
    }
    if !report.active_risks.is_empty() {
        println!();
        println!("Active risks:");
        for risk in &report.active_risks {
            println!("  - {risk}");
        }
    }
    println!();
    println!("Checks:");
    for check in &report.checks {
        println!(
            "  - {} [{}] {}",
            check.label,
            check.status.label(),
            check.summary
        );
        println!("    category: {}", check.category.label());
        println!("    source: {}", check.source_path);
        println!("    next: {}", check.next_action);
    }
    println!();
    println!("Evidence:");
    for link in &report.evidence_links {
        println!(
            "  - {} ({}) {}",
            link.label,
            if link.present { "present" } else { "missing" },
            link.path
        );
    }
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_recovery_runbook(report: &FridayReleaseRecoveryRunbookReport) {
    println!("Friday Release Recovery Runbook");
    println!("===============================");
    println!(
        "Score: {} / 100 | status: {} | review ready: {} | executable: {}",
        report.score_out_of_100,
        report.status.label(),
        yes_no(report.ready_for_operator_review),
        yes_no(report.ready_to_execute_recovery)
    );
    println!(
        "Phases: {} | blocked: {} | approvals: {} unsatisfied / {} total",
        report.phase_count,
        report.blocked_phase_count,
        report.unsatisfied_approval_gate_count,
        report.approval_gate_count
    );
    println!("Runbook: {}", report.runbook_json);
    if let Some(candidate_id) = &report.active_candidate_id {
        println!("Active candidate: {}", candidate_id);
    }
    if let Some(rollback) = &report.active_rollback_reference {
        println!("Rollback reference: {}", rollback);
    }
    if !report.active_risks.is_empty() {
        println!();
        println!("Active risks:");
        for risk in &report.active_risks {
            println!("  - {risk}");
        }
    }
    println!();
    println!("Phases:");
    for phase in &report.phases {
        println!(
            "  {}. {} [{}]",
            phase.order,
            phase.label,
            phase.status.label()
        );
        println!("    objective: {}", phase.objective);
        println!("    command: {}", phase.command);
        println!("    verify: {}", phase.verification);
        println!("    next: {}", phase.next_action);
    }
    println!();
    println!("Approval gates:");
    for gate in &report.approval_gates {
        println!(
            "  - {} [{}] {}",
            gate.label,
            if gate.satisfied {
                "satisfied"
            } else {
                "pending"
            },
            gate.summary
        );
    }
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_incident_archive(archive: &FridayReleaseIncidentArchive) {
    println!("Friday Release Incident Archive");
    println!("===============================");
    println!(
        "Incidents: {} | open: {} | monitoring: {} | resolved: {} | rolled back: {} | prevented: {}",
        archive.incident_count,
        archive.open_count,
        archive.monitoring_count,
        archive.resolved_count,
        archive.rolled_back_count,
        archive.prevented_count
    );
    println!(
        "Severity: {} critical | {} blocking total | follow-ups: {}",
        archive.critical_count, archive.blocking_count, archive.follow_up_count
    );
    println!("Archive: {}", archive.archive_json);
    if let Some(latest) = &archive.latest_incident_id {
        println!("Latest incident: {latest}");
    }
    if let Some(reference) = &archive.latest_rollback_reference {
        println!("Latest rollback reference: {reference}");
    }
    println!();
    println!("Entries:");
    for entry in &archive.entries {
        println!(
            "  - {} [{} / {}]",
            entry.title,
            entry.severity.label(),
            entry.outcome.label()
        );
        println!("    {}", entry.summary);
        println!("    runbook: {}", entry.recovery_runbook_json);
        if !entry.follow_up_actions.is_empty() {
            println!("    follow-up: {}", entry.follow_up_actions[0]);
        }
    }
    println!();
    println!("Commands:");
    for command in &archive.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_prevention_plan(report: &FridayReleasePreventionPlanReport) {
    println!("Friday Release Prevention Plan");
    println!("==============================");
    println!(
        "Score: {} / 100 | status: {} | ready for next checkpoint: {}",
        report.score_out_of_100,
        report.status.label(),
        yes_no(report.ready_for_next_checkpoint)
    );
    println!(
        "Findings: {} | recurring: {} | blockers: {} | actions: {} | owner-ready: {}",
        report.finding_count,
        report.recurring_issue_count,
        report.blocker_count,
        report.action_count,
        report.owner_ready_count
    );
    println!("Plan: {}", report.plan_json);
    if let Some(reference) = &report.active_rollback_reference {
        println!("Rollback reference: {reference}");
    }
    println!();
    println!("Findings:");
    for finding in &report.findings {
        println!(
            "  - {} [{} / {}]",
            finding.title,
            finding.kind.label(),
            finding.severity.label()
        );
        println!("    {}", finding.summary);
        println!("    next: {}", finding.next_action);
    }
    println!();
    println!("Actions:");
    for action in &report.actions {
        println!("  - {} [{}]", action.title, action.status.label());
        println!("    owner: {}", action.owner);
        println!("    command: {}", action.command);
        println!("    next: {}", action.next_action);
    }
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_owner_followup_board(report: &FridayReleaseOwnerFollowUpBoardReport) {
    println!("Friday Release Owner Follow-up Board");
    println!("====================================");
    println!(
        "Score: {} / 100 | status: {} | ready for next checkpoint: {}",
        report.score_out_of_100,
        report.status.label(),
        yes_no(report.ready_for_next_checkpoint)
    );
    println!(
        "Owners: {} | records: {} | ready: {} | waiting: {} | overdue: {} | gate blocks: {}",
        report.owner_count,
        report.record_count,
        report.ready_count,
        report.waiting_count,
        report.overdue_count,
        report.gate_blocking_count
    );
    println!("Board: {}", report.board_json);
    println!("Prevention plan: {}", report.prevention_plan_json);
    println!();
    println!("Owner groups:");
    for group in &report.owner_groups {
        println!(
            "  - @{}: {} record(s), {} missing evidence, {} overdue",
            group.owner, group.record_count, group.evidence_missing_count, group.overdue_count
        );
    }
    println!();
    println!("Assignments:");
    for record in &report.records {
        println!(
            "  - @{} {} [{} / {}]",
            record.owner,
            record.title,
            record.completion_state.label(),
            record.evidence_state.label()
        );
        println!("    due before: {}", record.due_before_unix_ms);
        println!("    evidence: {}", record.evidence_request);
        println!("    command: {}", record.command);
    }
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_evidence_sla_monitor(report: &FridayReleaseEvidenceSlaMonitorReport) {
    println!("Friday Release Evidence SLA Monitor");
    println!("===================================");
    println!(
        "Score: {} / 100 | status: {} | ready for next checkpoint: {}",
        report.score_out_of_100,
        report.status.label(),
        yes_no(report.ready_for_next_checkpoint)
    );
    println!(
        "Requirements: {} | owners: {} | overdue: {} | missing: {} | escalations: {} | gate blocks: {}",
        report.requirement_count,
        report.owner_count,
        report.overdue_count,
        report.missing_count,
        report.escalation_count,
        report.gate_blocking_count
    );
    println!("Monitor: {}", report.monitor_json);
    println!("Owner board: {}", report.owner_followup_board_json);
    println!();
    println!("Owner groups:");
    for group in &report.owner_groups {
        println!(
            "  - @{}: {} requirement(s), {} overdue, {} missing, {} escalation(s)",
            group.owner,
            group.requirement_count,
            group.overdue_count,
            group.missing_count,
            group.escalation_count
        );
    }
    println!();
    println!("SLA requirements:");
    for requirement in &report.requirements {
        println!(
            "  - @{} {} [{} / {}]",
            requirement.owner,
            requirement.title,
            requirement.state.label(),
            requirement.escalation_level.label()
        );
        println!("    evidence: {}", requirement.evidence_path);
        println!("    due before: {}", requirement.due_before_unix_ms);
        println!("    next: {}", requirement.next_action);
    }
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_escalation_ledger(ledger: &FridayReleaseEscalationLedger) {
    println!("Friday Release Escalation Ledger");
    println!("================================");
    println!(
        "Entries: {} | active: {} | carryover: {} | acknowledgement blockers: {} | gate blocks: {}",
        ledger.entry_count,
        ledger.active_count,
        ledger.carryover_count,
        ledger.acknowledgement_blocker_count,
        ledger.release_gate_blocking_count
    );
    println!("Ledger: {}", ledger.ledger_json);
    if let Some(outcome) = ledger.latest_gate_outcome {
        println!("Latest gate outcome: {}", outcome.label());
    }
    println!();
    println!("Owner groups:");
    for group in &ledger.owner_groups {
        println!(
            "  - @{}: {} entry(s), {} active, {} acknowledgement blocker(s)",
            group.owner, group.entry_count, group.active_count, group.acknowledgement_blocker_count
        );
    }
    println!();
    println!("Escalations:");
    for entry in &ledger.entries {
        println!(
            "  - @{} {} [{} / {}]",
            entry.owner,
            entry.title,
            entry.owner_response.label(),
            entry.gate_outcome.label()
        );
        println!("    SLA: {}", entry.sla_state.label());
        println!("    evidence: {}", entry.evidence_path);
        println!("    next: {}", entry.next_action);
    }
    println!();
    println!("Commands:");
    for command in &ledger.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_checkpoint_review_board(report: &FridayReleaseCheckpointReviewBoardReport) {
    println!("Friday Release Checkpoint Review");
    println!("================================");
    println!(
        "Decision: {} | score: {} / 100 | ready for checkpoint: {}",
        report.decision.label(),
        report.score_out_of_100,
        yes_no(report.ready_for_checkpoint)
    );
    println!(
        "Items: {} | hold: {} | carryover: {} | acknowledgement blockers: {} | gate blocks: {}",
        report.item_count,
        report.hold_count,
        report.carryover_count,
        report.acknowledgement_blocker_count,
        report.release_gate_blocking_count
    );
    println!("Review: {}", report.review_json);
    println!();
    println!("Owner groups:");
    for group in &report.owner_groups {
        println!(
            "  - @{}: {} item(s), {} hold, {} carryover, {} acknowledgement blocker(s)",
            group.owner,
            group.item_count,
            group.hold_count,
            group.carryover_count,
            group.acknowledgement_blocker_count
        );
    }
    println!();
    println!("Review items:");
    for item in &report.items {
        println!(
            "  - @{} {} [{} / {}]",
            item.owner,
            item.title,
            item.source.label(),
            item.state.label()
        );
        println!("    evidence: {}", item.evidence_path);
        println!("    next: {}", item.next_action);
    }
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_checkpoint_signoff_ledger(ledger: &FridayReleaseCheckpointSignoffLedger) {
    println!("Friday Release Checkpoint Signoff Ledger");
    println!("========================================");
    println!(
        "Records: {} | signed off: {} | held: {} | carried over: {} | missing acknowledgement evidence: {}",
        ledger.record_count,
        ledger.signed_off_count,
        ledger.held_count,
        ledger.carried_over_count,
        ledger.acknowledgement_evidence_missing_count
    );
    if let Some(decision) = ledger.active_decision {
        println!("Active decision: {}", decision.label());
    }
    println!("Ledger: {}", ledger.ledger_json);
    println!();
    println!("Signoffs:");
    for record in &ledger.records {
        println!(
            "  - {} {} [{}]",
            record.operator,
            record.review_id,
            record.decision.label()
        );
        println!("    reason: {}", record.reason);
        println!("    carryover: {}", record.carryover_commitment);
        println!(
            "    acknowledgement evidence: {}",
            record.acknowledgement_evidence_path
        );
    }
    println!();
    println!("Commands:");
    for command in &ledger.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_checkpoint_evidence_vault(vault: &FridayReleaseCheckpointEvidenceVault) {
    println!("Friday Release Checkpoint Evidence Vault");
    println!("========================================");
    println!(
        "Ready to archive: {} | entries: {} | missing: {} | checksums: {}",
        yes_no(vault.ready_to_archive),
        vault.entry_count,
        vault.missing_count,
        vault.checksum_count
    );
    println!(
        "Active holds: {} | carryovers: {} | gate blocks: {} | missing ack evidence: {}",
        vault.active_hold_count,
        vault.active_carryover_count,
        vault.release_gate_blocking_count,
        vault.acknowledgement_evidence_missing_count
    );
    println!("Vault: {}", vault.vault_json);
    println!("Manifest checksum: {}", vault.manifest_sha256);
    println!();
    println!("Evidence entries:");
    for entry in &vault.entries {
        println!(
            "  - {} [{}] {}",
            entry.label,
            entry.kind.label(),
            if entry.present { "present" } else { "missing" }
        );
        println!("    path: {}", entry.path);
        if let Some(sha256) = &entry.sha256 {
            println!("    sha256: {sha256}");
        }
        if let Some(warning) = &entry.warning {
            println!("    warning: {warning}");
        }
    }
    println!();
    println!("Commands:");
    for command in &vault.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_evidence_attachment_review(review: &FridayReleaseEvidenceAttachmentReview) {
    println!("Friday Release Evidence Attachment Review");
    println!("=========================================");
    println!(
        "Ready for handoff: {} | items: {} | attachable: {} | missing: {} | blocked: {}",
        yes_no(review.ready_for_handoff),
        review.item_count,
        review.attachable_count,
        review.missing_count,
        review.blocked_count
    );
    println!(
        "Inline only: {} | checksum missing: {} | gate blocks: {}",
        review.inline_only_count, review.checksum_missing_count, review.release_gate_blocking_count
    );
    println!("Review: {}", review.review_json);
    println!("Vault: {}", review.vault_json);
    println!("Manifest checksum: {}", review.manifest_sha256);
    if let Some(blocker) = &review.first_blocker {
        println!("First blocker: {blocker}");
    }
    println!();
    println!("Attachment items:");
    for item in &review.items {
        println!(
            "  - {} [{}] {}",
            item.label,
            item.kind.label(),
            item.state.label()
        );
        println!("    path: {}", item.path);
        println!("    next: {}", item.next_action);
    }
    println!();
    println!("Commands:");
    for command in &review.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_handoff_packet(packet: &FridayReleaseHandoffPacket) {
    println!("Friday Release Handoff Packet");
    println!("=============================");
    println!(
        "Ready to send: {} | sections: {} | files: {} | inline notes: {} | blockers: {}",
        yes_no(packet.ready_to_send),
        packet.section_count,
        packet.attachable_file_count,
        packet.inline_note_count,
        packet.unresolved_blocker_count
    );
    println!(
        "Included: {} | missing: {} | checksums: {}",
        packet.included_count, packet.missing_count, packet.checksum_count
    );
    println!("Packet: {}", packet.packet_json);
    println!("Attachment review: {}", packet.attachment_review_json);
    println!("Manifest checksum: {}", packet.manifest_sha256);
    if let Some(blocker) = &packet.first_blocker {
        println!("First blocker: {blocker}");
    }
    println!();
    println!("Packet sections:");
    for section in &packet.sections {
        println!(
            "  - {} [{}] {}",
            section.title,
            section.kind.label(),
            if section.included {
                "included"
            } else {
                "pending"
            }
        );
        println!("    path: {}", section.path);
        println!("    next: {}", section.next_action);
    }
    println!();
    println!("Commands:");
    for command in &packet.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_handoff_audit_trail(trail: &FridayReleaseHandoffAuditTrail) {
    println!("Friday Release Handoff Audit Trail");
    println!("===================================");
    println!(
        "Records: {} | ready: {} | sent: {} | blocked: {} | carryover records: {}",
        trail.record_count,
        trail.ready_count,
        trail.sent_count,
        trail.blocked_count,
        trail.blocker_carryover_count
    );
    println!(
        "Draft: {} | superseded: {} | revoked: {} | acknowledgements: {}",
        trail.draft_count, trail.superseded_count, trail.revoked_count, trail.acknowledgement_count
    );
    if let Some(state) = trail.latest_state {
        println!("Latest state: {}", state.label());
    }
    if let Some(packet_id) = &trail.active_packet_id {
        println!("Active packet: {packet_id}");
    }
    println!(
        "Unresolved carryover blockers: {}",
        trail.unresolved_blocker_count
    );
    println!("Trail: {}", trail.trail_json);
    println!();
    println!("Audit records:");
    for record in &trail.records {
        println!(
            "  - {} [{}] {}",
            record.operator,
            record.state.label(),
            record.packet_id
        );
        println!("    packet: {}", record.packet_json);
        println!("    acknowledgement: {}", record.acknowledgement_note);
        println!("    carryover blockers: {}", record.blocker_carryover);
    }
    println!();
    println!("Commands:");
    for command in &trail.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_handoff_governance_review(review: &FridayReleaseHandoffGovernanceReview) {
    println!("Friday Release Handoff Governance Review");
    println!("========================================");
    println!(
        "Approved: {} | state: {} | score: {}/100 | findings: {}",
        yes_no(review.approved_for_external_handoff),
        review.state.label(),
        review.score_out_of_100,
        review.finding_count
    );
    println!(
        "Acknowledgement gaps: {} | stale packet warnings: {} | blocker carryover: {} | release gate blocks: {}",
        review.acknowledgement_gap_count,
        review.stale_active_packet_count,
        review.blocked_carryover_count,
        review.release_gate_blocking_count
    );
    if let Some(packet_id) = &review.latest_packet_id {
        println!("Latest packet: {packet_id}");
    }
    if let Some(packet_id) = &review.active_packet_id {
        println!("Active packet: {packet_id}");
    }
    println!("Review: {}", review.review_json);
    println!("Trail: {}", review.trail_json);
    println!();
    println!("Governance findings:");
    for finding in &review.findings {
        println!(
            "  - {} [{}] {}",
            finding.title,
            finding.source.label(),
            finding.state.label()
        );
        println!("    packet: {}", finding.packet_id);
        println!("    evidence: {}", finding.evidence_path);
        println!("    next: {}", finding.next_action);
    }
    println!();
    println!("Commands:");
    for command in &review.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_handoff_dispatch_checklist(
    checklist: &FridayReleaseHandoffDispatchChecklist,
) {
    println!("Friday Release Handoff Dispatch Checklist");
    println!("=========================================");
    println!(
        "Ready: {} | state: {} | items: {}/{} | recipients: {} | attachments: {}",
        yes_no(checklist.ready_to_dispatch),
        checklist.state.label(),
        checklist.ready_count,
        checklist.item_count,
        checklist.recipient_count,
        checklist.attachment_count
    );
    println!(
        "Privacy notes: {} | no-send safeguards: {} | blockers: {}",
        checklist.privacy_boundary_count,
        checklist.no_send_safeguard_count,
        checklist.release_gate_blocking_count
    );
    println!(
        "Governance: {} [{}] approved: {}",
        checklist.governance_review_id,
        checklist.governance_state.label(),
        yes_no(checklist.approved_for_external_handoff)
    );
    if let Some(packet_id) = &checklist.latest_packet_id {
        println!("Latest packet: {packet_id}");
    }
    if let Some(packet_id) = &checklist.active_packet_id {
        println!("Active packet: {packet_id}");
    }
    println!("Checklist: {}", checklist.checklist_json);
    println!("Governance review: {}", checklist.governance_review_json);
    println!();
    println!("Dispatch items:");
    for item in &checklist.items {
        println!(
            "  - {} [{}] {} ready: {}",
            item.title,
            item.source.label(),
            item.state.label(),
            yes_no(item.ready)
        );
        if !item.evidence_path.is_empty() {
            println!("    evidence: {}", item.evidence_path);
        }
        println!("    next: {}", item.next_action);
    }
    println!();
    println!("Commands:");
    for command in &checklist.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_handoff_dispatch_audit_trail(
    trail: &FridayReleaseHandoffDispatchAuditTrail,
) {
    println!("Friday Release Handoff Dispatch Audit");
    println!("=====================================");
    println!(
        "Records: {} | approved: {} | sent manually: {} | held: {} | blocked: {}",
        trail.record_count,
        trail.approved_count,
        trail.sent_manually_count,
        trail.held_count,
        trail.blocked_count
    );
    println!(
        "Latest ready: {} | active blockers: {} | final decisions: {}",
        yes_no(trail.latest_ready_to_dispatch),
        trail.unresolved_blocker_count,
        trail.final_decision_count
    );
    if let Some(checklist_id) = &trail.latest_checklist_id {
        println!("Latest checklist: {checklist_id}");
    }
    if let Some(checklist_id) = &trail.active_checklist_id {
        println!("Active checklist: {checklist_id}");
    }
    println!("Trail: {}", trail.trail_json);
    println!();
    println!("Dispatch audit records:");
    for record in &trail.records {
        println!(
            "  - {} [{}] {}",
            record.operator,
            record.state.label(),
            record.checklist_id
        );
        println!("    checklist: {}", record.checklist_json);
        println!("    final decision: {}", record.final_decision_note);
        println!("    blocker carryover: {}", record.blocker_carryover);
    }
    println!();
    println!("Commands:");
    for command in &trail.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_handoff_dispatch_governance_review(
    review: &FridayReleaseHandoffDispatchGovernanceReview,
) {
    println!("Friday Release Handoff Dispatch Governance");
    println!("===========================================");
    println!(
        "Approved: {} | state: {} | score: {}/100 | findings: {}",
        yes_no(review.approved_for_external_handoff),
        review.state.label(),
        review.score_out_of_100,
        review.finding_count
    );
    println!(
        "Final decision gaps: {} | stale checklists: {} | revoked decisions: {} | blocker carryover: {} | release gate blocks: {}",
        review.final_decision_gap_count,
        review.stale_checklist_count,
        review.revoked_active_decision_count,
        review.blocked_carryover_count,
        review.release_gate_blocking_count
    );
    if let Some(checklist_id) = &review.latest_checklist_id {
        println!("Latest checklist: {checklist_id}");
    }
    if let Some(checklist_id) = &review.active_checklist_id {
        println!("Active checklist: {checklist_id}");
    }
    println!("Review: {}", review.review_json);
    println!("Trail: {}", review.trail_json);
    println!();
    println!("Dispatch governance findings:");
    for finding in &review.findings {
        println!(
            "  - {} [{}] {}",
            finding.title,
            finding.source.label(),
            finding.state.label()
        );
        println!("    checklist: {}", finding.checklist_id);
        println!("    evidence: {}", finding.evidence_path);
        println!("    next: {}", finding.next_action);
    }
    println!();
    println!("Commands:");
    for command in &review.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_handoff_completion_ledger(ledger: &FridayReleaseHandoffCompletionLedger) {
    println!("Friday Release Handoff Completion Ledger");
    println!("=========================================");
    println!(
        "Records: {} | completed: {} | manually sent: {} | held: {} | blocked: {}",
        ledger.record_count,
        ledger.completed_count,
        ledger.manually_sent_count,
        ledger.held_count,
        ledger.blocked_count
    );
    println!(
        "Approved outcomes: {} | blocked outcomes: {} | active gate blocks: {} | active unresolved blockers: {}",
        ledger.approved_outcome_count,
        ledger.blocked_outcome_count,
        ledger.release_gate_blocking_count,
        ledger.unresolved_blocker_count
    );
    if let Some(completion_id) = &ledger.active_completion_id {
        println!("Active completion: {completion_id}");
    }
    if let Some(review_id) = &ledger.latest_governance_review_id {
        println!("Latest governance review: {review_id}");
    }
    if let Some(state) = ledger.latest_state {
        println!("Latest state: {}", state.label());
    }
    println!("Ledger: {}", ledger.ledger_json);
    println!();
    println!("Completion records:");
    for record in ledger.records.iter().rev().take(8) {
        println!(
            "  - {} [{}] {}",
            record.operator,
            record.state.label(),
            record.governance_review_id
        );
        println!("    outcome: {}", record.outcome_note);
        println!(
            "    governance: {} | approved: {} | blockers: {}",
            record.governance_state.label(),
            yes_no(record.approved_for_external_handoff),
            record.release_gate_blocking_count
        );
        if let Some(reference) = &record.external_reference {
            println!("    reference: {reference}");
        }
    }
    println!();
    println!("Commands:");
    for command in &ledger.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_publication_control(control: &FridayReleasePublicationControl) {
    println!("Friday Release Publication Control");
    println!("==================================");
    println!(
        "Ready: {} | state: {} | score: {}/100 | blockers: {}",
        yes_no(control.ready_to_publish),
        control.state.label(),
        control.score_out_of_100,
        control.publication_blocker_count
    );
    println!(
        "Completion records: {} | approved outcomes: {} | blocked outcomes: {} | gate blocks: {}",
        control.completion_record_count,
        control.approved_outcome_count,
        control.blocked_outcome_count,
        control.release_gate_blocking_count
    );
    if let Some(completion_id) = &control.active_completion_id {
        println!("Active completion: {completion_id}");
    }
    if let Some(review_id) = &control.latest_governance_review_id {
        println!("Latest governance review: {review_id}");
    }
    println!("Control: {}", control.control_json);
    println!("Ledger: {}", control.ledger_json);
    println!();
    println!("Publication blockers:");
    for blocker in &control.blockers {
        println!("  - {} [{}]", blocker.summary, blocker.kind.label());
        println!("    evidence: {}", blocker.evidence_path);
        println!("    next: {}", blocker.next_action);
    }
    if control.blockers.is_empty() {
        println!("  - none");
    }
    println!();
    println!("Commands:");
    for command in &control.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_outbound_review_ledger(ledger: &FridayReleaseOutboundReviewLedger) {
    println!("Friday Release Outbound Review");
    println!("==============================");
    println!(
        "Records: {} | reviewed: {} | manual: {} | blocked: {} | copy-safe: {}",
        ledger.record_count,
        ledger.reviewed_count,
        ledger.manual_publication_count,
        ledger.blocked_count,
        ledger.copy_safe_count
    );
    println!(
        "Active review: {} | latest state: {} | latest publication: {}",
        ledger.active_review_id.as_deref().unwrap_or("none"),
        ledger
            .latest_state
            .map(|state| state.label())
            .unwrap_or("none"),
        ledger
            .latest_publication_state
            .map(|state| state.label())
            .unwrap_or("none")
    );
    println!(
        "Gate blockers: {} | unresolved blockers: {} | blocked reviews: {}",
        ledger.release_gate_blocking_count,
        ledger.unresolved_blocker_count,
        ledger.blocked_review_count
    );
    println!("Ledger: {}", ledger.ledger_json);
    println!();
    println!("Recent reviews:");
    for record in ledger.records.iter().rev().take(8) {
        println!(
            "  - {} [{}] {}",
            record.reviewer,
            record.state.label(),
            record.publication_control_id
        );
        println!(
            "    publication: {} | ready: {} | blockers: {} | copy-safe: {}",
            record.publication_state.label(),
            yes_no(record.ready_to_publish),
            record.release_gate_blocking_count,
            yes_no(record.copy_safe)
        );
        println!("    note: {}", record.review_note);
        if let Some(reference) = &record.manual_publication_reference {
            println!("    manual reference: {reference}");
        }
    }
    println!();
    println!("Commands:");
    for command in &ledger.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_external_receipt_archive(archive: &FridayReleaseExternalReceiptArchive) {
    println!("Friday Release External Receipt Archive");
    println!("=======================================");
    println!(
        "Records: {} | attached: {} | verified: {} | stale/missing: {} | blocked: {}",
        archive.record_count,
        archive.attached_receipt_count,
        archive.verified_receipt_count,
        archive.stale_or_missing_count,
        archive.blocked_receipt_count
    );
    println!(
        "Active receipt: {} | latest state: {} | latest outbound review: {}",
        archive.active_receipt_id.as_deref().unwrap_or("none"),
        archive
            .latest_state
            .map(|state| state.label())
            .unwrap_or("none"),
        archive
            .latest_outbound_review_id
            .as_deref()
            .unwrap_or("none")
    );
    println!(
        "Gate blockers: {} | unresolved blockers: {}",
        archive.release_gate_blocking_count, archive.unresolved_blocker_count
    );
    println!("Archive: {}", archive.archive_json);
    println!();
    println!("Recent receipts:");
    for record in archive.records.iter().rev().take(8) {
        println!(
            "  - {} [{}:{}] {}",
            record.operator,
            record.receipt_kind.label(),
            record.state.label(),
            record.outbound_review_id
        );
        println!(
            "    attached: {} | verified: {} | Friday mutated external systems: {}",
            yes_no(record.receipt_attached),
            yes_no(record.receipt_verified),
            yes_no(record.externally_mutated_by_friday)
        );
        if let Some(path) = &record.evidence_path {
            println!("    evidence: {path}");
        }
        if let Some(reference) = &record.external_reference {
            println!("    reference: {reference}");
        }
        println!("    note: {}", record.receipt_note);
    }
    println!();
    println!("Commands:");
    for command in &archive.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_receipt_review_board(report: &FridayReleaseReceiptReviewBoardReport) {
    println!("Friday Release Receipt Review Board");
    println!("===================================");
    println!(
        "Decision: {} | ready: {} | score: {}/100 | findings: {}",
        report.decision.label(),
        yes_no(report.ready_for_external_completion),
        report.score_out_of_100,
        report.finding_count
    );
    println!(
        "Receipts: {} | attached: {} | verified: {} | stale/missing: {} | gate blockers: {}",
        report.record_count,
        report.attached_receipt_count,
        report.verified_receipt_count,
        report.stale_or_missing_count,
        report.release_gate_blocking_count
    );
    println!(
        "Active receipt: {} | latest outbound review: {}",
        report.active_receipt_id.as_deref().unwrap_or("none"),
        report
            .latest_outbound_review_id
            .as_deref()
            .unwrap_or("none")
    );
    println!("Review: {}", report.review_json);
    println!("Archive: {}", report.archive_json);
    println!();
    println!("Findings:");
    for finding in &report.findings {
        println!(
            "  - {} [{}] {}",
            finding.summary,
            finding.decision.label(),
            finding.source.label()
        );
        println!("    evidence: {}", finding.evidence_path);
        println!("    next: {}", finding.next_action);
    }
    if report.findings.is_empty() {
        println!("  - none");
    }
    println!();
    println!("Commands:");
    for command in &report.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_closure_ledger(ledger: &FridayReleaseClosureLedger) {
    println!("Friday Release Closure Ledger");
    println!("=============================");
    println!(
        "Records: {} | closed: {} | held: {} | carryover: {} | blocked: {}",
        ledger.record_count,
        ledger.closed_count,
        ledger.held_count,
        ledger.carryover_count,
        ledger.blocked_count
    );
    println!(
        "Active closure: {} | latest review: {} | latest decision: {}",
        ledger.active_closure_id.as_deref().unwrap_or("none"),
        ledger.latest_receipt_review_id.as_deref().unwrap_or("none"),
        ledger
            .latest_review_decision
            .map(|decision| decision.label())
            .unwrap_or("none")
    );
    println!(
        "Closed outcomes: {} | carryover outcomes: {} | blocked outcomes: {}",
        ledger.closed_outcome_count, ledger.carryover_outcome_count, ledger.blocked_outcome_count
    );
    println!(
        "Gate blockers: {} | unresolved blockers: {}",
        ledger.release_gate_blocking_count, ledger.unresolved_blocker_count
    );
    println!("Ledger: {}", ledger.ledger_json);
    println!();
    println!("Records:");
    for record in &ledger.records {
        println!(
            "  - {} [{}] {}",
            record.closure_id,
            record.state.label(),
            record.summary
        );
        println!("    review: {}", record.receipt_review_json);
        println!("    note: {}", record.closure_note);
        if let Some(carryover) = &record.carryover_commitment {
            println!("    carryover: {carryover}");
        }
    }
    if ledger.records.is_empty() {
        println!("  - none");
    }
    println!();
    println!("Commands:");
    for command in &ledger.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_continuity_journal(journal: &FridayReleaseContinuityJournal) {
    println!("Friday Release Continuity Journal");
    println!("=================================");
    println!(
        "Entries: {} | outcomes: {} | carryover: {} | blockers: {} | next notes: {}",
        journal.entry_count,
        journal.outcome_entry_count,
        journal.carryover_entry_count,
        journal.blocker_pattern_count,
        journal.next_release_note_count
    );
    println!(
        "Active entry: {} | latest closure ledger: {} | latest closure state: {}",
        journal.active_entry_id.as_deref().unwrap_or("none"),
        journal
            .latest_closure_ledger_id
            .as_deref()
            .unwrap_or("none"),
        journal
            .latest_closure_state
            .map(|state| state.label())
            .unwrap_or("none")
    );
    println!(
        "Closed outcomes: {} | carryover commitments: {} | recurring blockers: {}",
        journal.closed_outcome_count,
        journal.carryover_commitment_count,
        journal.recurring_blocker_count
    );
    println!(
        "Gate blockers: {} | unresolved blockers: {}",
        journal.release_gate_blocking_count, journal.unresolved_blocker_count
    );
    println!("Journal: {}", journal.journal_json);
    println!();
    println!("Entries:");
    for record in &journal.records {
        println!(
            "  - {} [{}] {}",
            record.entry_id,
            record.entry_kind.label(),
            record.summary
        );
        println!("    closure ledger: {}", record.closure_ledger_json);
        println!("    note: {}", record.note);
        if let Some(owner) = &record.owner {
            println!("    owner: {owner}");
        }
        if let Some(target) = &record.next_release_target {
            println!("    next release: {target}");
        }
    }
    if journal.records.is_empty() {
        println!("  - none");
    }
    println!();
    println!("Commands:");
    for command in &journal.commands {
        println!("  - {command}");
    }
}

fn print_friday_release_learning_register(register: &FridayReleaseLearningRegister) {
    println!("Friday Release Learning Register");
    println!("================================");
    println!(
        "Records: {} | lessons: {} | experiments: {} | decisions: {} | gates: {} | commitments: {}",
        register.record_count,
        register.lesson_count,
        register.prevention_experiment_count,
        register.decision_pattern_count,
        register.quality_gate_count,
        register.owner_commitment_count
    );
    println!(
        "Active learning: {} | latest continuity journal: {} | latest category: {}",
        register.active_learning_id.as_deref().unwrap_or("none"),
        register
            .latest_continuity_journal_id
            .as_deref()
            .unwrap_or("none"),
        register
            .latest_category
            .map(|category| category.label())
            .unwrap_or("none")
    );
    println!(
        "Repeated lessons: {} | next-cycle commitments: {}",
        register.repeated_lesson_count, register.next_cycle_commitment_count
    );
    println!(
        "Gate blockers: {} | unresolved blockers: {}",
        register.release_gate_blocking_count, register.unresolved_blocker_count
    );
    println!("Register: {}", register.register_json);
    println!();
    println!("Records:");
    for record in &register.records {
        println!(
            "  - {} [{}] {}",
            record.learning_id,
            record.category.label(),
            record.summary
        );
        println!("    continuity journal: {}", record.continuity_journal_json);
        println!("    learning: {}", record.learning);
        if let Some(owner) = &record.owner {
            println!("    owner: {owner}");
        }
        if let Some(gate) = &record.quality_gate {
            println!("    quality gate: {gate}");
        }
        if let Some(commitment) = &record.next_cycle_commitment {
            println!("    next cycle: {commitment}");
        }
    }
    if register.records.is_empty() {
        println!("  - none");
    }
    println!();
    println!("Commands:");
    for command in &register.commands {
        println!("  - {command}");
    }
}

fn friday_release_signoff_decision(value: &str) -> Result<FridayReleaseChecklistSignoffDecision> {
    match value.trim().to_ascii_lowercase().as_str() {
        "approved" | "approve" | "ready" => Ok(FridayReleaseChecklistSignoffDecision::Approved),
        "needs-changes" | "changes" | "needs_changes" => {
            Ok(FridayReleaseChecklistSignoffDecision::NeedsChanges)
        }
        "blocked" | "block" => Ok(FridayReleaseChecklistSignoffDecision::Blocked),
        other => anyhow::bail!(
            "Unknown Friday release signoff decision `{}`. Use approved, needs-changes, or blocked.",
            other
        ),
    }
}

fn run_friday_trusted_host_live_state_command(
    state_file: &str,
    history_file: &str,
) -> Result<FridayTrustedHostLiveRunnerState> {
    let state_path = resolve_repo_relative_path(state_file);
    let history_path = resolve_repo_relative_path(history_file);
    let state =
        friday_trusted_host_live_runner_state_from_history_file(&history_path, &state_path)?;
    write_friday_trusted_host_live_runner_state(&state_path, state.records.clone())
}

fn print_friday_trusted_host_live_runner_state(state: &FridayTrustedHostLiveRunnerState) {
    println!("Friday Trusted Host Live State");
    println!("==============================");
    println!("State: {}", state.state_json);
    println!(
        "Records: {} | pending: {} | running: {} | finished: {} | stale: {}",
        state.record_count,
        state.pending_count,
        state.running_count,
        state.finished_count,
        state.stale_count
    );
    if state.stale_count > 0 {
        println!("Stale recovery: {}", state.stale_recovery_copy);
    }
    println!();
    for record in &state.records {
        println!(
            "  - [{}] {} ({})",
            record.status.label(),
            record.label,
            record.action_id
        );
        println!("    {}", record.message);
        println!("    command: {}", record.command);
        println!("    recover: {}", record.recovery_command);
        println!("    cleanup: {}", record.cleanup_command);
    }
}

fn run_friday_trusted_host_bridge_runner_command(
    input_dir: &str,
    action_id: Option<&str>,
    approve: bool,
    execute: bool,
    cancel: bool,
    history_file: &str,
    state_file: &str,
    reason: Option<&str>,
) -> Result<FridayTrustedHostRunnerBridgeReport> {
    let bridge =
        friday_dashboard_host_command_bridge_from_export(resolve_repo_relative_path(input_dir))?;
    let record = if let Some(action_id) = action_id {
        bridge
            .records
            .iter()
            .find(|record| record.action_id == action_id)
            .with_context(|| format!("No trusted host command record found for `{action_id}`"))?
    } else {
        bridge
            .records
            .first()
            .context("No trusted host command records are available")?
    };
    let request = FridayTrustedHostRunnerRequest {
        approved: approve && execute,
        cancel_requested: cancel,
        operator_reason: reason.map(str::to_string),
        ..Default::default()
    };
    let cancellation = if cancel {
        FridayTrustedHostRunnerCancellationToken::requested(
            reason.unwrap_or("operator requested cancellation"),
        )
    } else {
        FridayTrustedHostRunnerCancellationToken::none()
    };
    run_friday_trusted_host_command_bridge(
        record,
        &request,
        resolve_repo_relative_path(state_file),
        resolve_repo_relative_path(history_file),
        &cancellation,
    )
}

fn print_friday_trusted_host_bridge_runner_report(report: &FridayTrustedHostRunnerBridgeReport) {
    println!("Friday Trusted Host Bridge Runner");
    println!("=================================");
    println!("State: {}", report.state_json);
    println!("History: {}", report.history_json);
    println!("Result: {}", report.result.status.label());
    println!("Events: {}", report.event_count);
    println!("Guidance: {}", report.dashboard_import_guidance);
    println!();
    for event in &report.events {
        println!("  - [{}] {}", event.status.label(), event.message);
        println!("    state: {}", event.state_json);
        println!("    command: {}", event.record.command);
    }
}

fn print_friday_local_execution_checks() {
    let report = default_friday_local_execution_checks();

    println!("Friday Local Execution Checks");
    println!("=============================");
    println!("{}", report.summary);
    println!(
        "Passed: {}, warnings: {}, blocking: {}",
        report.passed_count(),
        report.warning_count(),
        report.blocking_count()
    );
    println!();

    for check in &report.checks {
        println!(
            "- [{}] {} / {}",
            check.status.label(),
            check.area.label(),
            check.title
        );
        println!("  command: {}", check.command);
        println!(
            "  local_only={}, loads_model={}, network={}",
            yes_no(check.local_only),
            yes_no(check.loads_model),
            yes_no(check.touches_network)
        );
        for item in &check.evidence {
            println!("  evidence: {}", item);
        }
        if check.status != crate::friday::FridayLocalCheckStatus::Passed {
            println!("  next: {}", check.next_action);
        }
    }
}

fn print_friday_browser_gate() {
    let report = default_friday_browser_verification_report();

    println!("Friday Browser Verification Gate");
    println!("================================");
    println!("{}", report.summary);
    println!(
        "Targets: {}/{} passed, blocking: {}",
        report.passed_target_count(),
        report.targets.len(),
        report.blocking_count()
    );
    println!(
        "Deploy allowed: {}",
        yes_no(report.deploy_gate.deployment_allowed)
    );
    println!(
        "Major feature: {}",
        report.deploy_gate.major_user_visible_feature
    );
    println!(
        "Required check: {}",
        report.deploy_gate.required_verification_command
    );
    println!("Rule: {}", report.deploy_gate.deploy_rule);
    println!();

    for target in &report.targets {
        println!("- [{}] {}", target.status.label(), target.surface);
        println!("  command: {}", target.command);
        for item in &target.evidence {
            println!("  evidence: {}", item);
        }
        if target.status != crate::friday::FridayVerificationStatus::Passed {
            println!("  next: {}", target.next_action);
        }
    }

    if !report.deploy_gate.notes.is_empty() {
        println!();
        println!("Notes:");
        for note in &report.deploy_gate.notes {
            println!("  - {}", note);
        }
    }
}

fn print_browser_extension_smoke() {
    let report = browser_extension_smoke_report();

    println!("Browser Extension Smoke Readiness");
    println!("=================================");
    println!("{}", report.summary);
    println!("Score: {} / 100", report.score_out_of_100);
    println!("Local only: {}", yes_no(report.local_only));
    println!("Touches network: {}", yes_no(report.touches_network));
    println!(
        "Targets: {} passed, {} warning, {} blocking",
        report.passed_count(),
        report.warning_count(),
        report.blocking_count()
    );
    println!();

    for target in &report.targets {
        println!(
            "- [{}] {} ({})",
            target.status.label(),
            target.browser_name,
            target.extension_target
        );
        println!("  dist: {}", target.dist_dir);
        println!("  package: {}", target.package_zip);
        println!(
            "  browser: {}",
            target
                .detected_executable
                .as_deref()
                .unwrap_or("<not detected>")
        );
        println!("  launch: {}", target.launch_command_hint);
        for item in &target.evidence {
            println!("  evidence: {}", item);
        }
        if target.status != BrowserExtensionSmokeStatus::Passed {
            println!("  next: {}", target.next_action);
        }
    }
}

fn print_browser_extension_launch_smoke(execute: bool) {
    let report = browser_extension_launch_smoke_report(execute, 8_000);

    println!("Browser Extension Launch Smoke");
    println!("==============================");
    println!("{}", report.summary);
    println!("Score: {} / 100", report.score_out_of_100);
    println!("Execute: {}", yes_no(report.execute));
    println!("Timeout: {}ms", report.timeout_ms);
    println!("Local only: {}", yes_no(report.local_only));
    println!("Touches network: {}", yes_no(report.touches_network));
    println!(
        "Targets: {} passed, {} warning, {} blocking",
        report.passed_count(),
        report.warning_count(),
        report.blocking_count()
    );
    println!();

    for target in &report.targets {
        println!(
            "- [{}] {} ({})",
            target.status.label(),
            target.browser_name,
            target.extension_target
        );
        println!(
            "  executable: {}",
            target.executable.as_deref().unwrap_or("<not detected>")
        );
        println!(
            "  profile: {}",
            target.profile_dir.as_deref().unwrap_or("<none>")
        );
        println!("  command: {}", target.command_preview);
        println!(
            "  executed={}, timed_out={}, exit_code={:?}, duration={}ms",
            yes_no(target.executed),
            yes_no(target.timed_out),
            target.exit_code,
            target.duration_ms
        );
        if !target.stdout_preview.trim().is_empty() {
            println!("  stdout: {}", target.stdout_preview.trim());
        }
        if !target.stderr_preview.trim().is_empty() {
            println!("  stderr: {}", target.stderr_preview.trim());
        }
        for item in &target.evidence {
            println!("  evidence: {}", item);
        }
        if target.status != BrowserExtensionSmokeStatus::Passed {
            println!("  next: {}", target.next_action);
        }
    }
}

fn print_browser_pack_reuse_smoke() {
    let report = browser_pack_reuse_smoke_report();

    println!("Browser Pack Offline Reuse Smoke");
    println!("================================");
    println!("{}", report.summary);
    println!("Score: {} / 100", report.score_out_of_100);
    println!("Local only: {}", yes_no(report.local_only));
    println!("Touches network: {}", yes_no(report.touches_network));
    println!(
        "Targets: {} passed, {} warning, {} blocking",
        report.passed_count(),
        report.warning_count(),
        report.blocking_count()
    );
    println!();

    for target in &report.targets {
        println!(
            "- [{}] {} ({})",
            target.status.label(),
            target.display_name,
            target.pack_key
        );
        println!("  model: {}", target.model_key);
        println!("  task: {}", target.task.label());
        println!("  cache: {}", target.cache_namespace);
        println!(
            "  selected_pack={:?}, local_only={}, remote_allowed={}",
            target.selected_pack_key,
            yes_no(target.local_only),
            yes_no(target.remote_allowed)
        );
        println!(
            "  cached files: {}/{} via {:?}",
            target.files_cached, target.files_total, target.storage_backend
        );
        for item in &target.evidence {
            println!("  evidence: {}", item);
        }
        if target.status != crate::browser::BrowserPackReuseStatus::Passed {
            println!("  next: {}", target.next_action);
        }
    }
}

fn print_browser_pack_recovery_smoke() {
    let report = browser_pack_recovery_smoke_report();

    println!("Browser Pack Recovery Smoke");
    println!("===========================");
    println!("{}", report.summary);
    println!("Score: {} / 100", report.score_out_of_100);
    println!("Local only: {}", yes_no(report.local_only));
    println!("Touches network: {}", yes_no(report.touches_network));
    println!(
        "Targets: {} passed, {} warning, {} blocking",
        report.passed_count(),
        report.warning_count(),
        report.blocking_count()
    );
    println!();

    for target in &report.targets {
        println!(
            "- [{}] {} ({})",
            target.status().label(),
            target.display_name,
            target.pack_key
        );
        println!("  model: {}", target.model_key);
        println!(
            "  cache files: {}, required_bytes={}, available_before={}, evicted_stale={}, available_after={}",
            target.files_total,
            target.required_bytes,
            target.available_bytes_before,
            target.stale_bytes_evicted,
            target.available_bytes_after
        );
        for scenario in &target.scenarios {
            println!(
                "  - [{}] {}",
                scenario.status.label(),
                scenario.kind.label()
            );
            println!("    action: {}", scenario.action);
            for item in &scenario.evidence {
                println!("    evidence: {}", item);
            }
        }
    }
}

fn print_browser_webllm_acceleration() {
    let report = browser_webllm_acceleration_report();

    println!("Chromium WebLLM Acceleration");
    println!("============================");
    println!("{}", report.summary);
    println!("Score: {} / 100", report.score_out_of_100);
    println!("Local only: {}", yes_no(report.local_only));
    println!("Touches network: {}", yes_no(report.touches_network));
    println!(
        "Targets: {} passed, {} warning, {} blocking",
        report.passed_count(),
        report.warning_count(),
        report.blocking_count()
    );
    println!();

    for target in &report.targets {
        println!(
            "- [{}] {} ({})",
            target.status.label(),
            target.display_name,
            target.pack_key
        );
        println!("  model: {}", target.model_key);
        println!("  host: {:?}", target.host_flavor);
        println!("  task: {}", target.task.label());
        println!(
            "  backend: {:?} with {:?} fallback",
            target.acceleration_backend, target.fallback_backend
        );
        println!(
            "  worker={:?}, device={:?}, local_only={}, remote_allowed={}",
            target.worker_kind,
            target.device_target,
            yes_no(target.local_only),
            yes_no(target.remote_allowed)
        );
        println!("  requirements: {}", target.requirements.join(", "));
        for item in &target.evidence {
            println!("  evidence: {}", item);
        }
        if target.status != crate::browser::BrowserWebLlmAccelerationStatus::Passed {
            println!("  next: {}", target.next_action);
        }
    }

    println!();
    println!("Guardrails:");
    for guardrail in &report.guardrails {
        println!(
            "  - {:?}: acceleration_allowed={}, fallback={:?}",
            guardrail.host_flavor,
            yes_no(guardrail.acceleration_allowed),
            guardrail.fallback_backend
        );
        for item in &guardrail.evidence {
            println!("    evidence: {}", item);
        }
    }
}

fn print_friday_ocr_smoke(report: &crate::friday::FridayOcrSmokeReport) {
    println!("Friday OCR Smoke");
    println!("================");
    println!("Status: {}", report.status.label());
    println!(
        "Mode: {}",
        if report.model_execution {
            "model"
        } else {
            "fixture"
        }
    );
    println!("Source: {}", report.image_source);
    println!("Output dir: {}", report.output_dir);
    println!("Markdown: {}", report.output_markdown);
    println!("Artifact: {}", report.artifact_json);
    println!("Checkpoint: {}", report.checkpoint_json);
    println!("Metadata: {}", report.metadata_json);
    println!("Report: {}", report.report_json);
    println!(
        "Artifact record: {} -> {}",
        report.artifact.id, report.artifact.current_checkpoint_id
    );
    if !report.extracted_text_preview.is_empty() {
        println!("Preview: {}", report.extracted_text_preview);
    }
    if report.findings.is_empty() {
        println!("Findings: none");
    } else {
        println!("Findings:");
        for finding in &report.findings {
            println!("  - {}", finding);
        }
    }
}

fn print_friday_vlm_contract(report: &crate::friday::FridayVlmContractReport) {
    println!("Friday VLM Screenshot Contract");
    println!("==============================");
    println!("Status: {}", report.status.label());
    println!("Model: {}", report.model_key);
    println!("Source: {}", report.screenshot_source);
    println!("Prompt: {}", report.prompt);
    println!("Output dir: {}", report.output_dir);
    println!("Markdown: {}", report.output_markdown);
    println!("Artifact: {}", report.artifact_json);
    println!("Checkpoint: {}", report.checkpoint_json);
    println!("Metadata: {}", report.metadata_json);
    println!("Report: {}", report.report_json);
    println!("Model execution: {}", yes_no(report.model_execution));
    println!("Model files:");
    for file in &report.model_files {
        println!(
            "  - [{}] {} ({})",
            if file.present { "present" } else { "missing" },
            file.path,
            file.purpose
        );
    }
    if report.findings.is_empty() {
        println!("Findings: none");
    } else {
        println!("Findings:");
        for finding in &report.findings {
            println!("  - {}", finding);
        }
    }
}

fn print_friday_multimodal_route(route: &crate::friday::FridayMultimodalRouteDecision) {
    println!("Friday Multimodal Route");
    println!("=======================");
    println!("Request: {}", route.request_kind.label());
    println!("Status: {}", route.status.label());
    println!("Local first: {}", yes_no(route.local_first));
    println!("Remote allowed: {}", yes_no(route.remote_allowed));
    if let Some(selected) = &route.selected {
        println!("Selected: {} ({})", selected.model_key, selected.purpose);
        println!("Command: {}", selected.command);
        println!("Resident: {}", yes_no(selected.resident));
        for file in &selected.files {
            println!(
                "  - [{}] {} ({})",
                if file.present { "present" } else { "missing" },
                file.path,
                file.purpose
            );
        }
    } else {
        println!("Selected: none");
    }
    if !route.fallbacks.is_empty() {
        println!("Fallbacks:");
        for fallback in &route.fallbacks {
            println!("  - {} ({})", fallback.model_key, fallback.purpose);
        }
    }
    println!("Rationale:");
    for item in &route.rationale {
        println!("  - {}", item);
    }
    println!("Next: {}", route.next_action);
}

fn print_friday_multimodal_diagnostics(diagnostics: &crate::friday::FridayMultimodalUiDiagnostics) {
    println!("Friday Multimodal Diagnostics");
    println!("=============================");
    println!("Route: {}", diagnostics.route);
    println!("Score: {} / 100", diagnostics.score_out_of_100);
    println!("Ready: {}", diagnostics.ready_count());
    println!("Warnings: {}", diagnostics.warning_count());
    println!("Primary command: {}", diagnostics.primary_command);
    println!();
    for item in &diagnostics.items {
        println!("- [{}] {}", item.status.label(), item.title);
        println!("  command: {}", item.command);
        println!("  output: {}", item.artifact_output);
        println!(
            "  local_only={}, loads_model={}",
            yes_no(item.local_only),
            yes_no(item.loads_model)
        );
        for evidence in &item.evidence {
            println!("  evidence: {}", evidence);
        }
        if item.status != crate::friday::FridayMultimodalDiagnosticStatus::Ready {
            println!("  next: {}", item.next_action);
        }
    }
    if !diagnostics.findings.is_empty() {
        println!();
        println!("Findings:");
        for finding in &diagnostics.findings {
            println!("  - {}", finding);
        }
    }
}

fn print_friday_multimodal_visual_check(report: &crate::friday::FridayMultimodalVisualCheckReport) {
    println!("Friday Multimodal Visual Check");
    println!("==============================");
    println!("Route: {}", report.route);
    println!("Surface: {}", report.target_surface);
    println!("Status: {}", report.status.label());
    println!("Score: {} / 100", report.score_out_of_100);
    println!("Command: {}", report.verification_command);
    println!();
    println!("Viewports:");
    for viewport in &report.viewports {
        println!(
            "  - {}: {}x{} ({})",
            viewport.id, viewport.width, viewport.height, viewport.expected_layout
        );
    }
    println!();
    println!("Requirements:");
    for requirement in &report.requirements {
        println!("  - [{}] {}", requirement.status.label(), requirement.label);
        for evidence in &requirement.evidence {
            println!("    evidence: {}", evidence);
        }
        if requirement.status != crate::friday::FridayUiVisualCheckStatus::Passed {
            println!("    next: {}", requirement.next_action);
        }
    }
    if !report.notes.is_empty() {
        println!();
        println!("Notes:");
        for note in &report.notes {
            println!("  - {}", note);
        }
    }
}

fn print_friday_screenshot_vlm_handoff(report: &crate::friday::FridayScreenshotVlmHandoffReport) {
    println!("Friday Screenshot VLM Handoff");
    println!("=============================");
    println!("Source: {}", report.source.path);
    println!("Mime: {}", report.source.mime);
    println!("Bytes: {}", report.source.bytes);
    println!("Accepted: {}", yes_no(report.source.accepted));
    println!("Source JSON: {}", report.source_json);
    println!("VLM report: {}", report.vlm_report.report_json);
    println!("VLM metadata: {}", report.vlm_report.metadata_json);
    println!("Artifact: {}", report.vlm_report.artifact_json);
    if report.findings.is_empty() {
        println!("Findings: none");
    } else {
        println!("Findings:");
        for finding in &report.findings {
            println!("  - {}", finding);
        }
    }
}

fn print_friday_media_affordances(affordances: &[crate::friday::FridayMediaAffordance]) {
    println!("Friday Media Affordances");
    println!("========================");
    for item in affordances {
        println!("- [{}] {}", item.status.label(), item.label);
        println!("  kind: {}", item.request_kind.label());
        println!("  model: {} ({})", item.model_key, item.repo_id);
        println!("  install: {}", item.install_command);
        println!("  run: {}", item.run_command);
        println!(
            "  local_only={}, resident={}",
            yes_no(item.local_only),
            yes_no(item.resident)
        );
        for note in &item.notes {
            println!("  note: {}", note);
        }
    }
}

fn print_friday_multimodal_artifact_import(
    report: &crate::friday::FridayMultimodalArtifactImportReport,
) {
    println!("Friday Multimodal Artifact Import");
    println!("=================================");
    println!("Store: {}", report.store_dir);
    println!("Bundle: {}", report.import.bundle_dir);
    println!(
        "Artifacts: {}",
        report.import.imported_artifact_ids.join(", ")
    );
    println!(
        "Checkpoints: {}",
        report.import.imported_checkpoint_ids.join(", ")
    );
    println!(
        "Metadata records: {}",
        report.import.imported_metadata_count
    );
    println!("Metadata JSON: {}", report.multimodal_metadata_json);
    println!("Manifest: {}", report.manifest_json);
    if report.findings.is_empty() {
        println!("Findings: none");
    } else {
        println!("Findings:");
        for finding in &report.findings {
            println!("  - {}", finding.message);
        }
    }
}

fn index_friday_multimodal_artifact_bundle(
    store_dir: &str,
    bundle_dir: &str,
) -> Result<crate::friday::FridayMultimodalArtifactImportReport> {
    let store_dir = resolve_repo_relative_path(store_dir);
    let bundle_dir = resolve_repo_relative_path(bundle_dir);
    let mut store = FridayArtifactStore::read_or_seed_from_dir(&store_dir)?;
    let import = store.import_multimodal_bundle(&bundle_dir)?;
    let snapshot = store.write_to_dir(&store_dir)?;

    Ok(crate::friday::FridayMultimodalArtifactImportReport {
        store_dir: snapshot.root_dir.to_string_lossy().into_owned(),
        multimodal_metadata_json: snapshot
            .multimodal_metadata_json
            .to_string_lossy()
            .into_owned(),
        manifest_json: snapshot.manifest_json.to_string_lossy().into_owned(),
        import,
        findings: snapshot.manifest.findings,
    })
}

fn print_search_request_plan(title: &str, plan: &crate::search::SearchRequestPlan) -> Result<()> {
    println!("{}", title);
    println!("{}", "=".repeat(title.len()));
    println!("Query: {}", plan.query);
    println!("Intent: {:?}", plan.intent);
    println!(
        "Adjacent metasearch: {}",
        yes_no(plan.use_adjacent_metasearch)
    );
    println!(
        "Verticals: {}",
        plan.verticals
            .iter()
            .map(|vertical| format!("{:?}", vertical))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
    println!("Notes:");
    for note in &plan.notes {
        println!("  - {}", note);
    }
    Ok(())
}

fn parse_friday_multimodal_request_kind(
    value: &str,
) -> Result<crate::friday::FridayMultimodalRequestKind> {
    crate::friday::FridayMultimodalRequestKind::parse(value)
        .with_context(|| format!("unknown multimodal request kind `{value}`"))
}

fn print_friday_research_workflow(workflow: &FridayResearchWorkflow) {
    println!("Friday Research Workflow");
    println!("========================");
    println!("Query: {}", workflow.query);
    println!("Local-first: {}", yes_no(workflow.local_first));
    println!("API path: {}", workflow.local_metasearch_api_path);
    println!(
        "Perplexity Computer dependency: {}",
        if workflow.forbids_perplexity_computer {
            "forbidden"
        } else {
            "allowed"
        }
    );
    println!("Ready stages: {}", workflow.ready_stage_count());
    println!();

    println!("Metasearch targets:");
    for target in &workflow.metasearch_targets {
        println!("  - {:?}: {}", target.mode, target.endpoint_or_command);
        println!("    default: {}", yes_no(target.available_by_default));
        for note in &target.notes {
            println!("    note: {}", note);
        }
    }
    println!();

    println!("Stages:");
    for stage in &workflow.stages {
        println!("  - {:?} [{}]", stage.kind, stage.status.label());
        println!("    {}", stage.output_contract);
    }
    println!();

    println!("Export formats: {}", workflow.export_formats.join(", "));
}

fn print_friday_metasearch_response(response: &MetasearchApiResponse) {
    println!("Friday Metasearch Results");
    println!("=========================");
    println!("Query: {}", response.query);
    println!("Results: {}", response.number_of_results);
    println!("Engines: {}", response.engines_used.join(", "));
    if !response.engines_failed.is_empty() {
        println!("Failed engines: {}", response.engines_failed.join(", "));
    }
    println!("Time: {} ms", response.search_time_ms);
    println!("Cached: {}", yes_no(response.cached));
    println!();

    for (index, result) in response.results.iter().take(10).enumerate() {
        println!("{}. {}", index + 1, result.title);
        println!("   {}", result.url);
        if !result.content.trim().is_empty() {
            println!("   {}", result.content.trim());
        }
        if !result.engine.trim().is_empty() {
            println!("   source: {}", result.engine);
        }
    }
}

fn print_accessibility_diagnostics(os: Option<&str>, live: bool) -> Result<()> {
    let os = match os {
        Some(value) => parse_operating_system(value)?,
        None => {
            let broker = RuntimeBroker::detect();
            OperatingSystemFamily::from_host_label(&broker.device_profile().os)
        }
    };
    let runtime = if live {
        FlowAccessibilityRuntime::live(os)
    } else {
        FlowAccessibilityRuntime::dry_run(os)
    };
    let diagnostic = runtime.diagnostic();

    println!("Flow Accessibility Diagnostics");
    println!("==============================");
    println!("OS: {:?}", diagnostic.os);
    println!("Probe mode: {}", if live { "live" } else { "dry-run" });
    println!("Backend: {:?}", diagnostic.backend);
    println!("Mode: {:?}", diagnostic.mode);
    println!("Severity: {}", diagnostic.severity.label());
    println!("Summary: {}", diagnostic.summary);
    println!(
        "Ready: full={}, selection_rewrite={}, shortcuts={}",
        yes_no(diagnostic.ready_for_full_automation),
        yes_no(diagnostic.ready_for_selection_rewrite),
        yes_no(diagnostic.ready_for_shortcuts)
    );
    print_accessibility_notes(&diagnostic);

    Ok(())
}

fn print_accessibility_notes(diagnostic: &FlowAccessibilityDiagnostic) {
    if !diagnostic.notes.is_empty() {
        println!();
        println!("Notes:");
        for note in &diagnostic.notes {
            println!("  - {}", note);
        }
    }

    println!();
    println!("Actions:");
    for action in &diagnostic.actions {
        println!("  - {}", action);
    }
}

fn print_audit_log(state_file: &str, limit: usize) -> Result<()> {
    let state_path = resolve_repo_relative_path(state_file);
    let store = FlowFileStateStore::new(state_path.clone());
    let state = store
        .load_state()
        .with_context(|| format!("Could not load Flow state from {}", state_path.display()))?;
    let summary = state.audit_summary(limit);

    println!("Flow Host Audit Log");
    println!("===================");
    println!("State: {}", state_path.display());
    println!("Entries: {}", summary.total_entries);
    println!("Approved: {}", summary.approved_entries);
    println!("Denied: {}", summary.denied_entries);
    println!("Showing: {}", summary.recent_entries.len());

    if summary.recent_entries.is_empty() {
        println!();
        println!("No persisted audit records yet.");
        return Ok(());
    }

    println!();
    println!("Recent records:");
    for entry in &summary.recent_entries {
        println!(
            "  - [{}] {} on {}",
            if entry.approved { "approved" } else { "denied" },
            entry.capability,
            entry.surface
        );
        println!("    {}", entry.description);
    }

    Ok(())
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn print_completion() {
    let set = active_completion_set();

    println!("Flow Completion Loop");
    println!("====================");
    println!("Active set: {}", set.name);
    println!(
        "Current: {} / {}",
        set.current_score_out_of_100, set.target_score_out_of_100
    );
    println!("Rule: {}", set.loop_rule);
    println!();

    println!("Items:");
    for item in &set.items {
        println!(
            "  - [{}] {} ({} pts)",
            item.status.label(),
            item.title,
            item.weight
        );
        println!("    proof: {}", item.proof);
        if item.status != crate::competitive::CompletionItemStatus::Done {
            println!("    next: {}", item.next_action);
        }
    }

    println!();
    println!("Next actions:");
    let remaining = set
        .items
        .iter()
        .filter(|item| item.status != crate::competitive::CompletionItemStatus::Done)
        .collect::<Vec<_>>();
    if remaining.is_empty() {
        println!("  - Open the next 100-point feature set in TODO.md and keep the loop moving.");
    } else {
        for item in remaining {
            println!("  - {}", item.next_action);
        }
    }
}

fn print_completion_json() -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&active_completion_set())?
    );
    Ok(())
}

fn print_models(broker: &RuntimeBroker, modality_filter: Option<&str>) -> Result<()> {
    let filter = modality_filter.map(parse_modality).transpose()?;
    println!("Flow Model Catalog");
    println!("==================");

    for manifest in broker.catalog() {
        if filter.is_some() && filter != Some(manifest.modality) {
            continue;
        }

        let local = if matches!(manifest.modality, Modality::SpeechToText) {
            LocalSttEngine::model_files_ready(&manifest.key, manifest.local_path.as_deref())
        } else {
            model_artifact_ready(manifest)
        };
        println!("{} - {}", manifest.key, manifest.display_name);
        println!(
            "  modality={:?}, runtime={:?}, format={:?}, min_mem={}",
            manifest.modality,
            manifest.preferred_runtime,
            manifest.artifact_format,
            format_bytes(manifest.minimum_memory_bytes)
        );
        println!(
            "  repo={}, local={}, conversion={}",
            manifest.repo_id,
            if local { "present" } else { "missing" },
            manifest
                .conversion_lanes
                .iter()
                .map(|lane| format!("{lane:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        if let Some(path) = &manifest.local_path {
            println!("  path={}", path);
        }
        if !manifest.tags.is_empty() {
            println!("  tags={}", manifest.tags.join(", "));
        }
        println!();
    }

    Ok(())
}

fn print_ui_model_candidates() -> Result<()> {
    println!("Flow UI Model Candidates");
    println!("========================");
    println!(
        "Device note: CPU-first is recommended on this machine class; tiny iGPU VRAM is not enough for meaningful offload."
    );
    println!();

    let candidates = [
        (
            "1",
            GEMMA4_FRONTEND_MODEL_KEY,
            "DuoNeural/Gemma-4-E4B-Frontend-GGUF",
            "image-text-to-text GGUF + mmproj",
            "apache-2.0",
            "Yes",
            "Best local visual candidate, but initial Google clone eval failed complete HTML; not proven.",
        ),
        (
            "2",
            "zai-org-ui2code-n",
            "zai-org/UI2Code_N",
            "image-text-to-text safetensors",
            "mit",
            "Yes",
            "Best quality candidate, but not GGUF-simple and likely too heavy locally.",
        ),
        (
            "3",
            "allenai-molmoweb-4b",
            "allenai/MolmoWeb-4B",
            "image-text-to-text safetensors/custom code",
            "apache-2.0",
            "Yes",
            "Useful as a visual web evaluator/agent, not the direct HTML generator.",
        ),
        (
            "4",
            "uigen-t3-8b-preview-q4km",
            "QuantFactory/UIGEN-T3-8B-Preview-GGUF",
            "text-only GGUF",
            "check-before-production",
            "No",
            "Text-only fallback only; do not use first for screenshot cloning.",
        ),
        (
            "5",
            QWENDEAN_MODEL_KEY,
            QWENDEAN_MODEL_REPO,
            "text-only GGUF",
            "apache-2.0",
            "No",
            "Already tested: complete HTML but poor visual clone.",
        ),
        (
            "6",
            WEBGEN_MODEL_KEY,
            WEBGEN_MODEL_REPO,
            "text-only GGUF",
            "apache-2.0",
            "No",
            "Already tested: incomplete standalone HTML in this runtime.",
        ),
    ];

    for (rank, key, repo, runtime, license, screenshot, recommendation) in candidates {
        let local = download_spec_for_model(key)
            .map(|spec| model_download_spec_ready(&spec))
            .unwrap_or(false);
        println!("{rank}. {key}");
        println!("   repo={repo}");
        println!("   runtime={runtime}");
        println!("   license={license}");
        println!("   screenshot_support={screenshot}");
        println!("   local={}", if local { "present" } else { "missing" });
        println!("   recommendation={recommendation}");
        println!();
    }

    Ok(())
}

fn print_tool_model_candidates() -> Result<()> {
    println!("Flow Tool-Calling Model Candidates");
    println!("===================================");
    println!("Ranking is for local CPU-first agent routing, JSON/tool calls, and reasoning.");
    println!("Commercial-safe means Apache/MIT-style licensing for future product use.");
    println!();

    let candidates = [
        (
            "1",
            XLAM2_3B_TOOL_MODEL_KEY,
            XLAM2_3B_TOOL_MODEL_REPO,
            "specialist function-calling GGUF",
            "cc-by-nc-4.0",
            "No",
            "Best small pure tool-calling candidate; install for local research/eval, not commercial default.",
        ),
        (
            "2",
            MINISTRAL3_3B_MODEL_KEY,
            MINISTRAL3_3B_MODEL_REPO,
            "general instruct + native tool/JSON GGUF",
            "apache-2.0",
            "Yes",
            "Best commercial-safe small general agent/chat replacement candidate.",
        ),
        (
            "3",
            GRANITE4_H_MICRO_MODEL_KEY,
            GRANITE4_H_MICRO_MODEL_REPO,
            "low-latency structured-output GGUF",
            "apache-2.0",
            "Yes",
            "Best tiny commercial-safe router candidate for strict JSON/function-call workflows.",
        ),
        (
            "4",
            PHI4_MINI_MODEL_KEY,
            PHI4_MINI_MODEL_REPO,
            "reasoning-focused instruct GGUF",
            "mit",
            "Yes",
            "Strong small reasoning backup with documented function-calling format.",
        ),
        (
            "5",
            "qwen35-4b-revised-q4km",
            QWEN35_4B_REVISED_MODEL_REPO,
            "general smart/coding GGUF",
            "apache-2.0",
            "Yes",
            "Already installed and smart, but not the cleanest dedicated tool-call model.",
        ),
        (
            "6",
            SMOLLM3_3B_MODEL_KEY,
            SMOLLM3_3B_MODEL_REPO,
            "fast general small GGUF",
            "apache-2.0",
            "Yes",
            "Good small fallback; weaker tool specialization than xLAM, Ministral, or Granite.",
        ),
    ];

    for (rank, key, repo, runtime, license, commercial_safe, recommendation) in candidates {
        let local = download_spec_for_model(key)
            .map(|spec| model_download_spec_ready(&spec))
            .unwrap_or_else(|| {
                LocalLlm::model_path_for_key(key)
                    .map(|path| Path::new(&path).exists())
                    .unwrap_or(false)
            });
        println!("{rank}. {key}");
        println!("   repo={repo}");
        println!("   runtime={runtime}");
        println!("   license={license}");
        println!("   commercial_safe={commercial_safe}");
        println!("   local={}", if local { "present" } else { "missing" });
        println!("   recommendation={recommendation}");
        println!();
    }

    println!("Install the top research model:");
    println!("  cargo run --release --bin flow -- --install-model {XLAM2_3B_TOOL_MODEL_KEY}");
    println!("Commercial-safe runner-up:");
    println!("  cargo run --release --bin flow -- --install-model {MINISTRAL3_3B_MODEL_KEY}");

    Ok(())
}

async fn run_tool_agent(tools_path: Option<&str>, request: &str) -> Result<()> {
    let spec = download_spec_for_model(XLAM2_3B_TOOL_MODEL_KEY)
        .context("No built-in tool-agent model is registered")?;
    if !model_download_spec_ready(&spec) {
        return Err(anyhow::anyhow!(
            "Tool-agent model '{}' is missing. Run: cargo run --release --bin flow -- --install-model {}",
            XLAM2_3B_TOOL_MODEL_KEY,
            XLAM2_3B_TOOL_MODEL_KEY
        ));
    }

    let tools_json = match tools_path {
        Some(path) => read_tool_agent_tools(path)?,
        None => "[]".to_string(),
    };

    let llm = LocalLlm::for_tool_agent();
    let started = Instant::now();
    llm.initialize().await?;
    let load_time_ms = started.elapsed().as_millis();
    let (response, metrics) = llm
        .generate_tool_call_with_metrics(&tools_json, request)
        .await?;

    let json_state = if response.trim_start().starts_with('[') {
        match serde_json::from_str::<Value>(&response) {
            Ok(_) => "valid",
            Err(_) => "invalid",
        }
    } else {
        "not-json"
    };

    println!("{}", response);
    println!();
    println!(
        "[tool-agent] json={} load={:.2}s prompt_tokens={} generated_tokens={} total={:.2}s gen={:.2}s speed={:.2} tok/s",
        json_state,
        load_time_ms as f64 / 1000.0,
        metrics.prompt_tokens,
        metrics.generated_tokens,
        metrics.total_time_ms as f64 / 1000.0,
        metrics.generation_time_ms as f64 / 1000.0,
        metrics.tokens_per_second
    );

    Ok(())
}

fn read_tool_agent_tools(path: &str) -> Result<String> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("Failed to read tool schema file: {path}"))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("Invalid JSON in tool schema: {path}"))?;
    if !value.is_array() {
        return Err(anyhow::anyhow!(
            "Tool schema must be a JSON array of tool definitions: {path}"
        ));
    }
    serde_json::to_string(&value).context("Failed to compact tool schema JSON")
}

#[derive(Debug)]
struct LocalModelProbeResult {
    role: &'static str,
    model_key: &'static str,
    best_for: &'static str,
    path: String,
    local_state: &'static str,
    status: String,
    load_time_ms: Option<u128>,
    metrics: Option<GenerationMetrics>,
    signal: String,
    output: String,
}

async fn verify_local_models() -> Result<()> {
    println!("Flow Local Model Verification");
    println!("=============================");
    println!("CPU-first role check for this Windows machine. Keep prompts short and bounded.");
    println!();

    let mut results = Vec::new();

    let result = verify_helper_model().await;
    print_local_model_probe(&result);
    flush_stdout();
    results.push(result);

    let result = verify_tool_agent_model().await;
    print_local_model_probe(&result);
    flush_stdout();
    results.push(result);

    let result = verify_coding_model().await;
    print_local_model_probe(&result);
    flush_stdout();
    results.push(result);

    let result = verify_quality_chat_model().await;
    print_local_model_probe(&result);
    flush_stdout();
    results.push(result);

    print_slow_backup_state();
    print_local_model_ranking(&results);

    Ok(())
}

async fn verify_helper_model() -> LocalModelProbeResult {
    let llm = LocalLlm::for_helper();
    let prompt = "Rewrite this as one clean professional sentence: um i need like the final file now and make it professional";
    let best_for = "fast prompt enhancer, text cleanup, tiny rewrites, labels, conversion";

    verify_model_probe(
        "helper",
        FLOW_HELPER_MODEL_KEY,
        best_for,
        llm,
        "sentence cleanup",
        |output| {
            if output.len() <= 160 && !output.to_ascii_lowercase().contains("um") {
                "concise-cleanup=pass".to_string()
            } else {
                "concise-cleanup=weak".to_string()
            }
        },
        |llm| Box::pin(async move { llm.generate_helper_with_metrics(prompt).await }),
    )
    .await
}

async fn verify_tool_agent_model() -> LocalModelProbeResult {
    let llm = LocalLlm::for_tool_agent();
    let tools_json = r#"[{"name":"get_weather","description":"Get weather for a location and date.","parameters":{"type":"object","properties":{"location":{"type":"string"},"date":{"type":"string"}},"required":["location","date"]}}]"#;
    let request = "weather in Dhaka tomorrow";
    let best_for = "strict JSON tool routing and function-call decisions";

    verify_model_probe(
        "tool-agent",
        FLOW_TOOL_MODEL_KEY,
        best_for,
        llm,
        "tool JSON",
        |output| {
            let json_state = if output.trim_start().starts_with('[') {
                match serde_json::from_str::<Value>(output) {
                    Ok(_) => "valid",
                    Err(_) => "invalid",
                }
            } else {
                "not-json"
            };
            format!("json={json_state}")
        },
        |llm| {
            Box::pin(async move {
                llm.generate_tool_call_with_metrics(tools_json, request)
                    .await
            })
        },
    )
    .await
}

async fn verify_coding_model() -> LocalModelProbeResult {
    let llm = LocalLlm::for_coding();
    let prompt = "Return only TypeScript code for a small cx(...classes) helper that joins truthy class names.";
    let best_for = "coding, shadcn/ui edits, React, TypeScript, Tailwind, Rust";

    verify_model_probe(
        "coding",
        FLOW_CODING_MODEL_KEY,
        best_for,
        llm,
        "TypeScript utility",
        |output| {
            let has_signature = output.contains("cx") && output.contains("classes");
            let has_truthy_filter = output.contains("filter(Boolean)") || output.contains("filter");
            format!(
                "ts-helper={}",
                if has_signature && has_truthy_filter {
                    "pass"
                } else {
                    "check"
                }
            )
        },
        |llm| Box::pin(async move { llm.generate_coding_with_metrics(prompt).await }),
    )
    .await
}

async fn verify_quality_chat_model() -> LocalModelProbeResult {
    let llm = LocalLlm::for_quality_chat();
    let prompt =
        "In two concise sentences, explain when Flow should use local AI versus remote AI.";
    let best_for = "commercial-safe daily chat, synthesis, short reasoning, normal useful answers";

    verify_model_probe(
        "quality-chat",
        FLOW_QUALITY_CHAT_MODEL_KEY,
        best_for,
        llm,
        "daily answer",
        |output| {
            let clean = !output.to_ascii_lowercase().contains("<think>");
            let concise = output.split_whitespace().count() <= 90;
            format!(
                "clean-concise={}",
                if clean && concise { "pass" } else { "check" }
            )
        },
        |llm| Box::pin(async move { llm.generate_quality_chat_with_metrics(prompt).await }),
    )
    .await
}

async fn verify_model_probe<F, G>(
    role: &'static str,
    model_key: &'static str,
    best_for: &'static str,
    llm: LocalLlm,
    task_label: &'static str,
    signal_fn: F,
    generate_fn: G,
) -> LocalModelProbeResult
where
    F: Fn(&str) -> String,
    G: for<'a> FnOnce(
        &'a LocalLlm,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(String, GenerationMetrics)>> + 'a>,
    >,
{
    let path = llm.model_path().to_string();
    if !Path::new(&path).exists() {
        return LocalModelProbeResult {
            role,
            model_key,
            best_for,
            path,
            local_state: "missing",
            status: "skipped".to_string(),
            load_time_ms: None,
            metrics: None,
            signal: "missing-local-file".to_string(),
            output: format!(
                "Install with: cargo run --release --bin flow -- --install-model {model_key}"
            ),
        };
    }

    let load_start = Instant::now();
    if let Err(error) = llm.initialize().await {
        return LocalModelProbeResult {
            role,
            model_key,
            best_for,
            path,
            local_state: "present",
            status: "load-failed".to_string(),
            load_time_ms: Some(load_start.elapsed().as_millis()),
            metrics: None,
            signal: task_label.to_string(),
            output: error.to_string(),
        };
    }
    let load_time_ms = load_start.elapsed().as_millis();

    match generate_fn(&llm).await {
        Ok((output, metrics)) => {
            let trimmed_output = output.trim();
            let empty_output = trimmed_output.is_empty();
            let signal = if empty_output {
                "empty-output=fail".to_string()
            } else {
                signal_fn(trimmed_output)
            };
            LocalModelProbeResult {
                role,
                model_key,
                best_for,
                path,
                local_state: "present",
                status: if empty_output { "empty-output" } else { "ok" }.to_string(),
                load_time_ms: Some(load_time_ms),
                metrics: Some(metrics),
                signal,
                output,
            }
        }
        Err(error) => LocalModelProbeResult {
            role,
            model_key,
            best_for,
            path,
            local_state: "present",
            status: "generation-failed".to_string(),
            load_time_ms: Some(load_time_ms),
            metrics: None,
            signal: task_label.to_string(),
            output: error.to_string(),
        },
    }
}

fn print_local_model_probe(result: &LocalModelProbeResult) {
    println!("{} - {}", result.role, result.model_key);
    println!("  best_for={}", result.best_for);
    println!("  path={}", result.path);
    println!("  local={}", result.local_state);
    println!("  status={}", result.status);

    if let Some(load_time_ms) = result.load_time_ms {
        println!("  load={:.2}s", load_time_ms as f64 / 1000.0);
    }

    if let Some(metrics) = &result.metrics {
        println!(
            "  tokens: prompt={} generated={} total={:.2}s gen={:.2}s speed={:.2} tok/s",
            metrics.prompt_tokens,
            metrics.generated_tokens,
            metrics.total_time_ms as f64 / 1000.0,
            metrics.generation_time_ms as f64 / 1000.0,
            metrics.tokens_per_second
        );
    }

    println!("  signal={}", result.signal);
    println!("  sample={}", one_line_sample(&result.output, 260));
    println!();
}

fn print_slow_backup_state() {
    let state = if Path::new(QWEN35_9B_MODEL_PATH).exists() {
        "present"
    } else {
        "missing"
    };

    println!("slow-backup - {QWEN35_9B_MODEL_KEY}");
    println!("  best_for=last-resort coding quality when latency is acceptable");
    println!("  path={QWEN35_9B_MODEL_PATH}");
    println!("  local={state}");
    println!("  status=not-run-by-default");
    println!(
        "  reason=9B Q4_K_M is useful as backup, but too slow for every local verification pass on this OS."
    );
    println!();
}

fn print_local_model_ranking(results: &[LocalModelProbeResult]) {
    println!("Recommended Local Routing");
    println!("-------------------------");
    let helper_ok = results
        .iter()
        .any(|result| result.role == "helper" && result.status == "ok");
    if helper_ok {
        println!("1. fastest helper: {FLOW_HELPER_MODEL_KEY}");
    } else {
        println!(
            "1. helper: {FLOW_QUALITY_CHAT_MODEL_KEY} until {FLOW_HELPER_MODEL_KEY} passes visible-output verification"
        );
    }
    println!("2. tool calling: {FLOW_TOOL_MODEL_KEY}");
    println!("3. coding/shadcn/Tailwind edits: {FLOW_CODING_MODEL_KEY}");
    println!("4. daily smart chat: {FLOW_QUALITY_CHAT_MODEL_KEY}");
    println!("5. slow backup: {QWEN35_9B_MODEL_KEY}");
    println!();

    let fastest = results
        .iter()
        .filter_map(|result| {
            if result.status != "ok" {
                return None;
            }
            result
                .metrics
                .as_ref()
                .map(|metrics| (result.model_key, metrics.tokens_per_second, result.role))
        })
        .max_by(|left, right| left.1.total_cmp(&right.1));

    if let Some((model_key, speed, role)) = fastest {
        println!("Fastest successful measured role: {role} / {model_key} ({speed:.2} tok/s)");
    }
}

fn one_line_sample(text: &str, max_chars: usize) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= max_chars {
        return compact;
    }

    let mut sample = compact.chars().take(max_chars).collect::<String>();
    sample.push_str("...");
    sample
}

fn flush_stdout() {
    let _ = std::io::stdout().flush();
}

fn print_model_roles() {
    println!("Flow Local Model Roles");
    println!("======================");
    println!("This machine policy keeps local GGUF models active by purpose:");
    println!();

    for role in LocalLlm::model_roles() {
        let path = Path::new(role.model_path);
        let state = if path.exists() { "present" } else { "missing" };
        println!("{} - {}", role.role, role.model_key);
        println!("  path={}", role.model_path);
        println!("  local={state}");
        println!("  purpose={}", role.purpose);
        println!();
    }

    println!("Commands:");
    println!("  cargo run --release --bin flow -- --verify-local-models");
    println!("  cargo run --release --bin flow -- --chat qwen3-0.6b");
    println!("  cargo run --release --bin flow -- --tool-agent \"choose a tool for this request\"");
    println!("  cargo run --release --bin flow -- --chat qwen35-4b-revised-q4km");
    println!("  cargo run --release --bin flow -- --chat ministral3-3b-instruct-q4km");
    println!("  cargo run --release --bin flow -- --chat qwen35-9b-q4km");
}

fn install_model_cli(model_key: &str) -> Result<()> {
    let spec = download_spec_for_model(model_key)
        .with_context(|| format!("No built-in installer is registered for model '{model_key}'"))?;

    if model_download_spec_ready(&spec) {
        println!(
            "Model '{}' is already installed ({} file(s)).",
            spec.model_key,
            spec.files.len()
        );
        return Ok(());
    }

    for file in spec.files {
        install_model_file(&spec, file)?;
    }

    println!(
        "Installed '{}' ({} file(s)).",
        spec.model_key,
        spec.files.len()
    );
    Ok(())
}

fn install_model_file(spec: &ModelDownloadSpec, file: &ModelDownloadFile) -> Result<()> {
    let local_path = PathBuf::from(file.local_path);

    if local_path.exists() && fs::metadata(&local_path)?.len() >= file.expected_bytes {
        println!(
            "{} is already installed at {}",
            file.filename,
            local_path.display()
        );
        return Ok(());
    }

    if let Some(parent) = local_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create model directory {}", parent.display()))?;
    }

    if local_path.exists() {
        println!(
            "{} has a partial local file at {} ({} / {} bytes); resuming.",
            file.filename,
            local_path.display(),
            fs::metadata(&local_path)?.len(),
            file.expected_bytes
        );
    }

    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        spec.repo_id, file.filename
    );
    println!("Downloading {} from {}", file.filename, spec.repo_id);
    println!("Expected size: {}", format_bytes(file.expected_bytes));

    let script_path = Path::new("scripts/download_hf_file_resume.ps1");
    if cfg!(windows) && script_path.exists() {
        let log_path = format!(
            "tmp/downloads/{}-{}.log",
            sanitize_model_key_for_path(spec.model_key),
            sanitize_model_key_for_path(file.filename)
        );
        let status = std::process::Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(script_path)
            .arg("-Url")
            .arg(&url)
            .arg("-Output")
            .arg(file.local_path)
            .arg("-ExpectedBytes")
            .arg(file.expected_bytes.to_string())
            .arg("-LogPath")
            .arg(&log_path)
            .status()
            .with_context(|| {
                format!("Failed to start resumable downloader for {}", file.filename)
            })?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Resumable download failed for {}. See {}",
                file.filename,
                log_path
            ));
        }
    } else {
        let api = Api::new().context("Failed to initialize Hugging Face Hub API")?;
        let repo = api.model(spec.repo_id.to_string());
        let cached_path = repo
            .get(file.filename)
            .with_context(|| format!("Failed to download {}", file.filename))?;

        fs::copy(&cached_path, &local_path).with_context(|| {
            format!(
                "Failed to copy {} to {}",
                cached_path.display(),
                local_path.display()
            )
        })?;
    }

    let bytes = fs::metadata(&local_path)
        .with_context(|| format!("Downloaded file not found: {}", local_path.display()))?
        .len();
    if bytes < file.expected_bytes {
        return Err(anyhow::anyhow!(
            "{} is incomplete: {} / {} bytes",
            file.filename,
            bytes,
            file.expected_bytes
        ));
    }

    println!("Installed {} at {}", file.filename, local_path.display());
    Ok(())
}

fn run_uigen_output_path(output: &str) -> Result<PathBuf> {
    let output_path = PathBuf::from(output);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory {}", parent.display()))?;
    }
    Ok(output_path)
}

fn default_ui_model_key() -> String {
    std::env::var("FLOW_UIGEN_MODEL").unwrap_or_else(|_| DEFAULT_UI_MODEL_KEY.to_string())
}

fn sanitize_model_key_for_path(model_key: &str) -> String {
    model_key
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn resolve_ui_model_spec(model_key: Option<&str>) -> Result<ModelDownloadSpec> {
    let owned_default;
    let key = match model_key {
        Some(key) if !key.trim().is_empty() => key.trim(),
        _ => {
            owned_default = default_ui_model_key();
            owned_default.as_str()
        }
    };

    download_spec_for_model(key).with_context(|| {
        format!(
            "No built-in UI model is registered for '{key}'. Known UI models: {}, {}",
            WEBGEN_MODEL_KEY, QWENDEAN_MODEL_KEY
        )
    })
}

async fn run_uigen_google(model_key: Option<&str>) -> Result<()> {
    let spec = resolve_ui_model_spec(model_key)?;
    let output = format!(
        "{}/{}/index.html",
        UIMODEL_GOOGLE_OUTPUT_ROOT,
        sanitize_model_key_for_path(spec.model_key)
    );
    run_uigen(Some(spec.model_key), &output, GOOGLE_HOMEPAGE_PROMPT).await
}

async fn run_uigen(model_key: Option<&str>, output: &str, user_prompt: &str) -> Result<()> {
    let spec = resolve_ui_model_spec(model_key)?;
    let model_file = spec.primary_file();
    let model_path = PathBuf::from(model_file.local_path);
    if !model_path.exists() {
        return Err(anyhow::anyhow!(
            "UI model '{}' is missing at {}. Run: cargo run --release --bin flow -- --install-model {}",
            spec.model_key,
            model_path.display(),
            spec.model_key
        ));
    }
    if !model_download_spec_ready(&spec) {
        return Err(anyhow::anyhow!(
            "UI model '{}' is incomplete. Run: cargo run --release --bin flow -- --install-model {}",
            spec.model_key,
            spec.model_key
        ));
    }

    let output_path = run_uigen_output_path(output)?;
    let prompt = build_uigen_prompt(spec.model_key, user_prompt);
    let llm = LocalLlm::with_config(
        model_path.to_string_lossy().into_owned(),
        crate::models::LocalLlmConfig::uigen(),
    );

    println!(
        "Loading {} ({}) from {}",
        spec.display_name,
        spec.model_key,
        model_path.display()
    );
    llm.initialize().await?;
    println!("Generating UI into {}", output_path.display());

    let (raw, metrics) = llm.generate_ui_with_metrics(&prompt).await?;
    let html = strip_script_blocks(&clean_generated_code(&raw));
    validate_generated_html(&html).with_context(|| {
        let partial_path = output_path.with_extension("partial.html");
        let _ = fs::write(&partial_path, &html);
        format!(
            "{} returned incomplete HTML. Partial output saved to {}",
            spec.display_name,
            partial_path.display()
        )
    })?;
    fs::write(&output_path, html)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    println!(
        "Wrote {} ({} tokens in {:.2}s @ {:.1} tok/s)",
        output_path.display(),
        metrics.generated_tokens,
        metrics.total_time_ms as f64 / 1000.0,
        metrics.tokens_per_second
    );
    println!(
        "For screenshots, run: powershell -ExecutionPolicy Bypass -File scripts\\uigen_google_eval.ps1 -ModelKey {}",
        spec.model_key
    );
    Ok(())
}

fn run_uigen_vision_google() -> Result<()> {
    let output_dir = format!(
        "{}/{}",
        UIVISION_GOOGLE_OUTPUT_ROOT,
        sanitize_model_key_for_path(GEMMA4_FRONTEND_MODEL_KEY)
    );
    fs::create_dir_all(&output_dir).with_context(|| format!("Failed to create {}", output_dir))?;
    let screenshot = format!("{}/google-desktop.png", output_dir);
    let output = format!("{}/index.html", output_dir);
    capture_browser_screenshot(GOOGLE_URL, &screenshot, "1365,768")?;
    run_uigen_vision(&screenshot, &output, GOOGLE_VISION_PROMPT)
}

fn run_uigen_vision(screenshot: &str, output: &str, user_prompt: &str) -> Result<()> {
    let spec = download_spec_for_model(GEMMA4_FRONTEND_MODEL_KEY)
        .context("Gemma frontend vision model spec is missing")?;
    if !model_download_spec_ready(&spec) {
        return Err(anyhow::anyhow!(
            "Vision UI model '{}' is missing or incomplete. Run: cargo run --release --bin flow -- --install-model {}",
            spec.model_key,
            spec.model_key
        ));
    }

    let screenshot_path = PathBuf::from(screenshot);
    if !screenshot_path.exists() {
        return Err(anyhow::anyhow!(
            "Screenshot not found: {}",
            screenshot_path.display()
        ));
    }

    let output_path = run_uigen_output_path(output)?;
    let raw_path = output_path.with_extension("raw.txt");
    let model_file = spec
        .files
        .iter()
        .find(|file| file.role == ModelFileRole::Model)
        .context("Gemma frontend model file is missing from spec")?;
    let mmproj_file = spec
        .files
        .iter()
        .find(|file| file.role == ModelFileRole::Mmproj)
        .context("Gemma frontend mmproj file is missing from spec")?;

    println!(
        "Loading {} through llama-cpp-python vision bridge",
        spec.display_name
    );
    println!("Screenshot: {}", screenshot_path.display());
    println!("Output: {}", output_path.display());

    let output = std::process::Command::new("python")
        .arg("scripts/uigen_vision_llama_cpp.py")
        .arg("--model")
        .arg(model_file.local_path)
        .arg("--mmproj")
        .arg(mmproj_file.local_path)
        .arg("--image")
        .arg(&screenshot_path)
        .arg("--prompt")
        .arg(build_uigen_vision_prompt(user_prompt))
        .arg("--max-tokens")
        .arg(std::env::var("FLOW_UIGEN_VISION_MAX_TOKENS").unwrap_or_else(|_| "1800".to_string()))
        .arg("--ctx")
        .arg(std::env::var("FLOW_UIGEN_VISION_CTX").unwrap_or_else(|_| "8192".to_string()))
        .arg("--threads")
        .arg(uigen_thread_count().to_string())
        .output()
        .context("Failed to run scripts/uigen_vision_llama_cpp.py")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Vision UI generation failed. {}",
            stderr.trim()
        ));
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    fs::write(&raw_path, &raw)
        .with_context(|| format!("Failed to write {}", raw_path.display()))?;
    let html = strip_script_blocks(&clean_generated_code(&raw));
    validate_generated_html(&html).with_context(|| {
        let partial_path = output_path.with_extension("partial.html");
        let _ = fs::write(&partial_path, &html);
        format!(
            "{} returned incomplete HTML. Partial output saved to {}",
            spec.display_name,
            partial_path.display()
        )
    })?;
    fs::write(&output_path, html)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    println!("Wrote {}", output_path.display());
    Ok(())
}

fn build_uigen_vision_prompt(user_prompt: &str) -> String {
    format!(
        "{}\n\nOutput contract:\n- Return only a complete HTML document.\n- One <style> tag, no Markdown fences.\n- No external URLs, fonts, scripts, CDNs, or image assets.\n- Preserve the screenshot layout, spacing, and visual hierarchy.\n",
        user_prompt
    )
}

fn capture_browser_screenshot(url: &str, screenshot_path: &str, window_size: &str) -> Result<()> {
    let browser = resolve_headless_browser()
        .context("No headless Edge or Chrome executable was found for screenshot capture")?;
    let full_path = PathBuf::from(screenshot_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    let status = std::process::Command::new(&browser)
        .arg("--headless=new")
        .arg("--disable-gpu")
        .arg("--hide-scrollbars")
        .arg(format!("--window-size={window_size}"))
        .arg(format!("--screenshot={}", full_path.display()))
        .arg(url)
        .status()
        .with_context(|| format!("Failed to launch {}", browser.display()))?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "Browser screenshot failed for {url} with status {status}"
        ));
    }
    Ok(())
}

fn resolve_headless_browser() -> Option<PathBuf> {
    let candidates = [
        std::env::var("ProgramFiles")
            .ok()
            .map(|base| PathBuf::from(base).join("Microsoft/Edge/Application/msedge.exe")),
        std::env::var("ProgramFiles(x86)")
            .ok()
            .map(|base| PathBuf::from(base).join("Microsoft/Edge/Application/msedge.exe")),
        std::env::var("ProgramFiles")
            .ok()
            .map(|base| PathBuf::from(base).join("Google/Chrome/Application/chrome.exe")),
        std::env::var("ProgramFiles(x86)")
            .ok()
            .map(|base| PathBuf::from(base).join("Google/Chrome/Application/chrome.exe")),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|candidate| candidate.exists())
}

fn uigen_thread_count() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get().saturating_sub(1).max(1))
        .unwrap_or(4)
}

fn build_uigen_prompt(_model_key: &str, user_prompt: &str) -> String {
    format!(
        "Create a complete single-file HTML document for this UI request:\n\n{}\n\nRequirements:\n- Return only HTML code.\n- Include all CSS in one <style> tag.\n- Do not use external images, scripts, CDNs, fonts, Tailwind, React imports, or Google assets.\n- Do not use @import or external URLs.\n- Keep CSS under 120 lines and the whole file under 220 lines.\n- Emit <body> content immediately after </style>.\n- Include </body> and </html>.\n- If you prefer React or shadcn/ui, translate that design sense into plain standalone HTML/CSS for this local screenshot eval.\n- Use a shadcn/ui-like product UI sense: calm spacing, clean borders, accessible controls, responsive layout.\n- Preserve the requested visual structure closely.\n",
        user_prompt
    )
}

fn clean_generated_code(raw: &str) -> String {
    let trimmed = LocalLlm::strip_thinking_tags(raw).trim().to_string();
    if let Some(start) = trimmed.find("```") {
        let after_start = &trimmed[start + 3..];
        let after_lang = after_start
            .strip_prefix("html")
            .or_else(|| after_start.strip_prefix("HTML"))
            .unwrap_or(after_start)
            .trim_start_matches(['\r', '\n']);
        if let Some(end) = after_lang.find("```") {
            return after_lang[..end].trim().to_string();
        }
    }
    if let Some(end) = trimmed.to_ascii_lowercase().find("</html>") {
        let closing_end = end + "</html>".len();
        return trimmed[..closing_end].trim().to_string();
    }
    trimmed
}

fn validate_generated_html(html: &str) -> Result<()> {
    let lower = html.to_ascii_lowercase();
    let has_document = lower.contains("<!doctype html") || lower.contains("<html");
    if !has_document
        || !lower.contains("<body")
        || !lower.contains("</body>")
        || !lower.contains("</html>")
    {
        return Err(anyhow::anyhow!(
            "generated output does not contain a complete HTML document"
        ));
    }

    if lower.contains("<script src")
        || lower.contains("cdn.tailwindcss.com")
        || lower.contains("@import")
        || lower.contains("fonts.googleapis.com")
    {
        return Err(anyhow::anyhow!(
            "generated output contains external scripts, fonts, imports, or CDN dependencies"
        ));
    }

    Ok(())
}

fn strip_script_blocks(html: &str) -> String {
    let mut output = html.to_string();
    loop {
        let lower = output.to_ascii_lowercase();
        let Some(start) = lower.find("<script") else {
            break;
        };
        let Some(relative_end) = lower[start..].find("</script>") else {
            output.truncate(start);
            break;
        };
        let end = start + relative_end + "</script>".len();
        output.replace_range(start..end, "");
    }
    output.trim().to_string()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModelFileRole {
    Model,
    Mmproj,
}

#[derive(Clone, Copy)]
struct ModelDownloadFile {
    filename: &'static str,
    local_path: &'static str,
    expected_bytes: u64,
    role: ModelFileRole,
}

#[derive(Clone, Copy)]
struct ModelDownloadSpec {
    model_key: &'static str,
    display_name: &'static str,
    repo_id: &'static str,
    files: &'static [ModelDownloadFile],
}

impl ModelDownloadSpec {
    fn primary_file(&self) -> &'static ModelDownloadFile {
        self.files
            .iter()
            .find(|file| file.role == ModelFileRole::Model)
            .unwrap_or(&self.files[0])
    }
}

fn download_spec_for_model(model_key: &str) -> Option<ModelDownloadSpec> {
    const QWEN3_06B_FILES: &[ModelDownloadFile] = &[ModelDownloadFile {
        filename: QWEN3_06B_MODEL_FILE,
        local_path: QWEN3_06B_MODEL_PATH,
        expected_bytes: 396_705_472,
        role: ModelFileRole::Model,
    }];
    const WEBGEN_FILES: &[ModelDownloadFile] = &[ModelDownloadFile {
        filename: WEBGEN_MODEL_FILE,
        local_path: WEBGEN_MODEL_PATH,
        expected_bytes: 2_497_286_912,
        role: ModelFileRole::Model,
    }];
    const QWENDEAN_FILES: &[ModelDownloadFile] = &[ModelDownloadFile {
        filename: QWENDEAN_MODEL_FILE,
        local_path: QWENDEAN_MODEL_PATH,
        expected_bytes: 2_497_280_928,
        role: ModelFileRole::Model,
    }];
    const QWEN35_9B_FILES: &[ModelDownloadFile] = &[ModelDownloadFile {
        filename: QWEN35_9B_MODEL_FILE,
        local_path: QWEN35_9B_MODEL_PATH,
        expected_bytes: 5_680_522_464,
        role: ModelFileRole::Model,
    }];
    const QWEN35_4B_REVISED_FILES: &[ModelDownloadFile] = &[ModelDownloadFile {
        filename: QWEN35_4B_REVISED_MODEL_FILE,
        local_path: QWEN35_4B_REVISED_MODEL_PATH,
        expected_bytes: 2_708_808_096,
        role: ModelFileRole::Model,
    }];
    const XLAM2_3B_TOOL_FILES: &[ModelDownloadFile] = &[ModelDownloadFile {
        filename: XLAM2_3B_TOOL_MODEL_FILE,
        local_path: XLAM2_3B_TOOL_MODEL_PATH,
        expected_bytes: 1_929_902_656,
        role: ModelFileRole::Model,
    }];
    const MINISTRAL3_3B_FILES: &[ModelDownloadFile] = &[ModelDownloadFile {
        filename: MINISTRAL3_3B_MODEL_FILE,
        local_path: MINISTRAL3_3B_MODEL_PATH,
        expected_bytes: 2_146_497_824,
        role: ModelFileRole::Model,
    }];
    const GRANITE4_H_MICRO_FILES: &[ModelDownloadFile] = &[ModelDownloadFile {
        filename: GRANITE4_H_MICRO_MODEL_FILE,
        local_path: GRANITE4_H_MICRO_MODEL_PATH,
        expected_bytes: 1_942_564_512,
        role: ModelFileRole::Model,
    }];
    const PHI4_MINI_FILES: &[ModelDownloadFile] = &[ModelDownloadFile {
        filename: PHI4_MINI_MODEL_FILE,
        local_path: PHI4_MINI_MODEL_PATH,
        expected_bytes: 2_493_840_192,
        role: ModelFileRole::Model,
    }];
    const SMOLLM3_3B_FILES: &[ModelDownloadFile] = &[ModelDownloadFile {
        filename: SMOLLM3_3B_MODEL_FILE,
        local_path: SMOLLM3_3B_MODEL_PATH,
        expected_bytes: 1_915_305_312,
        role: ModelFileRole::Model,
    }];
    const GEMMA4_FRONTEND_FILES: &[ModelDownloadFile] = &[
        ModelDownloadFile {
            filename: GEMMA4_FRONTEND_MODEL_FILE,
            local_path: GEMMA4_FRONTEND_MODEL_PATH,
            expected_bytes: 5_335_285_376,
            role: ModelFileRole::Model,
        },
        ModelDownloadFile {
            filename: GEMMA4_FRONTEND_MMPROJ_FILE,
            local_path: GEMMA4_FRONTEND_MMPROJ_PATH,
            expected_bytes: 991_551_904,
            role: ModelFileRole::Mmproj,
        },
    ];
    const UIGEN_FX_FILES: &[ModelDownloadFile] = &[ModelDownloadFile {
        filename: UIGEN_FX_MODEL_FILE,
        local_path: UIGEN_FX_MODEL_PATH,
        expected_bytes: 2_716_064_480,
        role: ModelFileRole::Model,
    }];

    match model_key {
        QWEN3_06B_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: QWEN3_06B_MODEL_KEY,
            display_name: "Qwen3 0.6B Q4_K_M",
            repo_id: QWEN3_06B_MODEL_REPO,
            files: QWEN3_06B_FILES,
        }),
        WEBGEN_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: WEBGEN_MODEL_KEY,
            display_name: "WEBGEN 4B Preview i1 Q4_K_M",
            repo_id: WEBGEN_MODEL_REPO,
            files: WEBGEN_FILES,
        }),
        QWENDEAN_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: QWENDEAN_MODEL_KEY,
            display_name: "Qwendean 4B Q4_K_M",
            repo_id: QWENDEAN_MODEL_REPO,
            files: QWENDEAN_FILES,
        }),
        QWEN35_9B_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: QWEN35_9B_MODEL_KEY,
            display_name: "Qwen3.5 9B Q4_K_M",
            repo_id: QWEN35_9B_MODEL_REPO,
            files: QWEN35_9B_FILES,
        }),
        QWEN35_4B_REVISED_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: QWEN35_4B_REVISED_MODEL_KEY,
            display_name: "Qwen3.5 4B Revised Q4_K_M",
            repo_id: QWEN35_4B_REVISED_MODEL_REPO,
            files: QWEN35_4B_REVISED_FILES,
        }),
        XLAM2_3B_TOOL_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: XLAM2_3B_TOOL_MODEL_KEY,
            display_name: "xLAM-2 3B Function Calling Q4_K_M",
            repo_id: XLAM2_3B_TOOL_MODEL_REPO,
            files: XLAM2_3B_TOOL_FILES,
        }),
        MINISTRAL3_3B_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: MINISTRAL3_3B_MODEL_KEY,
            display_name: "Ministral 3 3B Instruct Q4_K_M",
            repo_id: MINISTRAL3_3B_MODEL_REPO,
            files: MINISTRAL3_3B_FILES,
        }),
        GRANITE4_H_MICRO_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: GRANITE4_H_MICRO_MODEL_KEY,
            display_name: "Granite 4.0 H Micro Q4_K_M",
            repo_id: GRANITE4_H_MICRO_MODEL_REPO,
            files: GRANITE4_H_MICRO_FILES,
        }),
        PHI4_MINI_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: PHI4_MINI_MODEL_KEY,
            display_name: "Phi-4 Mini Instruct Q4_K_M",
            repo_id: PHI4_MINI_MODEL_REPO,
            files: PHI4_MINI_FILES,
        }),
        SMOLLM3_3B_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: SMOLLM3_3B_MODEL_KEY,
            display_name: "SmolLM3 3B Q4_K_M",
            repo_id: SMOLLM3_3B_MODEL_REPO,
            files: SMOLLM3_3B_FILES,
        }),
        GEMMA4_FRONTEND_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: GEMMA4_FRONTEND_MODEL_KEY,
            display_name: "Gemma 4 E4B Frontend Q4_K_M + BF16 mmproj",
            repo_id: GEMMA4_FRONTEND_MODEL_REPO,
            files: GEMMA4_FRONTEND_FILES,
        }),
        UIGEN_FX_MODEL_KEY => Some(ModelDownloadSpec {
            model_key: UIGEN_FX_MODEL_KEY,
            display_name: "UIGEN-FX 4B Preview Q4_K_M",
            repo_id: UIGEN_FX_MODEL_REPO,
            files: UIGEN_FX_FILES,
        }),
        _ => None,
    }
}

fn model_download_spec_ready(spec: &ModelDownloadSpec) -> bool {
    spec.files.iter().all(|file| {
        let path = Path::new(file.local_path);
        path.exists()
            && fs::metadata(path)
                .map(|metadata| metadata.len() >= file.expected_bytes)
                .unwrap_or(false)
    })
}

fn model_artifact_ready(manifest: &crate::runtime::ModelManifest) -> bool {
    let Some(local_path) = manifest.local_path.as_deref() else {
        return false;
    };
    let path = Path::new(local_path);
    if !path.exists() {
        return false;
    }
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if let Some(spec) = download_spec_for_model(&manifest.key) {
        return model_download_spec_ready(&spec);
    }
    metadata.len() > 0
}

fn print_plan(broker: &RuntimeBroker, plan: &ExecutionPlan) {
    println!("Flow Runtime Plan");
    println!("=================");
    println!("Modality: {:?}", plan.modality);
    println!("Device tier: {:?}", plan.device_tier);
    println!(
        "Selected model: {}",
        plan.selected_model.as_deref().unwrap_or("none")
    );
    println!(
        "Runtime: {}",
        plan.selected_runtime
            .map(|runtime| format!("{runtime:?}"))
            .unwrap_or_else(|| "none".to_string())
    );
    println!("Launch: {:?}", plan.launch);

    if let Some(bytes) = plan.estimated_memory_bytes {
        println!("Estimated memory: {}", format_bytes(bytes));
    }

    println!();
    println!("Reasons:");
    for reason in &plan.reasons {
        println!("  - {}", reason);
    }

    if let Some(artifact) = &plan.artifact {
        println!();
        println!("Artifact:");
        println!(
            "  {} files in {}, runtime {:?}, redistributable={}",
            artifact.files.len(),
            artifact.root_dir,
            artifact.runtime,
            artifact.redistributable
        );
    }

    if let Some(job) = &plan.conversion_job {
        println!();
        println!("Conversion:");
        println!(
            "  lane={:?}, target={:?}, command={}",
            job.lane,
            job.target_format,
            job.command_preview.join(" ")
        );
    }

    if let Some(publish) = &plan.publish_record {
        println!();
        println!(
            "Publish: {:?} -> {}",
            publish.status, publish.destination_repo
        );
        if let Some(reason) = &publish.reason {
            println!("  reason={}", reason);
        }
    }

    if let Some(reason) = &plan.unsupported_reason {
        println!();
        println!("Unsupported: {}", reason);
    }

    if matches!(plan.modality, Modality::Chat) {
        let recommended = broker.models_for(Modality::Chat);
        if !recommended.is_empty() {
            println!();
            println!("Local chat candidates:");
            for candidate in recommended {
                let local = candidate
                    .local_path
                    .as_deref()
                    .map(Path::new)
                    .map(Path::exists)
                    .unwrap_or(false);
                println!(
                    "  - {} [{}]",
                    candidate.key,
                    if local { "local" } else { "missing" }
                );
            }
        }
    }
}

fn run_grammar(text: &str, fix: bool) -> Result<()> {
    let checker = HarperGrammarChecker::new();
    let issues = checker.analyze(text)?;

    println!("Grammar");
    println!("=======\n");
    println!("Input: {}", text);

    if issues.is_empty() {
        println!("Issues: none");
    } else {
        println!("Issues:");
        for issue in &issues {
            println!(
                "  - {}..{}: {}{}",
                issue.start,
                issue.end,
                issue.message,
                issue
                    .replacement
                    .as_ref()
                    .map(|replacement| format!(" -> {}", replacement))
                    .unwrap_or_default()
            );
        }
    }

    if fix {
        let corrected = checker.correct(text)?;
        println!();
        println!("Corrected:");
        println!("{}", corrected);
    }

    Ok(())
}

fn print_wake_words(broker: &RuntimeBroker) {
    println!("Wake Words");
    println!("==========");
    println!(
        "Frontend resources: {}",
        if WakeWordDetector::is_available() {
            "present"
        } else {
            "missing"
        }
    );

    let installed = broker.activation().wake_words.iter().collect::<Vec<_>>();
    for definition in wake_command_definitions() {
        let item = installed
            .iter()
            .copied()
            .find(|item| item.command_key == definition.command_key);
        let model_path = Path::new("models/wake_words").join(definition.model_filename);
        println!("{} ({})", definition.command_key, definition.phrase);
        println!(
            "  status={}",
            if item.is_some() {
                "installed"
            } else {
                "missing"
            }
        );
        println!("  model={}", model_path.to_string_lossy());
        println!("  threshold={}%", definition.threshold);
        println!(
            "  aliases={}",
            if definition.aliases.is_empty() {
                "-".to_string()
            } else {
                definition.aliases.join(", ")
            }
        );
    }
}

fn print_blueprint(host: HostSurface) {
    let registry = FlowEmbeddingRegistry::detect();
    let blueprint = registry.blueprint(host);

    println!("Flow Embedding Blueprint");
    println!("========================");
    println!("Host: {:?}", blueprint.host);
    println!("Mode: {:?}", blueprint.integration_mode);
    println!(
        "Device: {} / {:?}",
        blueprint.device_profile.os, blueprint.device_profile.tier
    );
    println!();

    println!("Core subsystems:");
    for subsystem in &blueprint.core_subsystems {
        println!("  - {:?}", subsystem);
    }

    if !blueprint.optional_subsystems.is_empty() {
        println!();
        println!("Optional subsystems:");
        for subsystem in &blueprint.optional_subsystems {
            println!("  - {:?}", subsystem);
        }
    }

    println!();
    println!("Adjacent projects:");
    for project in &blueprint.adjacent_projects {
        println!(
            "  - {} [{}] {}",
            project.key,
            if project.detected {
                "detected"
            } else {
                "missing"
            },
            project.purpose
        );
    }

    println!();
    println!("Workspace projects:");
    for project in &blueprint.workspace_projects {
        println!(
            "  - {}: {} ({}%)",
            project.key, project.role, project.completeness_score
        );
    }

    println!();
    println!(
        "Providers: folder_present={}, auto_switch_local_and_remote={}",
        blueprint.provider_strategy.folder_present,
        blueprint.provider_strategy.auto_switch_local_and_remote
    );
    println!(
        "Provider catalog sources: {}",
        blueprint
            .provider_catalog_plan
            .sources
            .iter()
            .map(|source| format!("{source:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "Serializer: folder_present={}, format={}, rkyv={}, memmap={}",
        blueprint.serializer_strategy.folder_present,
        blueprint.serializer_strategy.format_name,
        blueprint.serializer_strategy.uses_rkyv,
        blueprint.serializer_strategy.uses_memmap
    );
    println!(
        "Metasearch: folder_present={}",
        blueprint.search_strategy.folder_present
    );
    println!(
        "RLM: folder_present={}",
        blueprint.long_context_strategy.folder_present
    );
    println!(
        "Forge: folder_present={}, multi_remote={}",
        blueprint.forge_strategy.folder_present, blueprint.forge_strategy.supports_multi_remote
    );

    println!();
    println!("Notes:");
    for note in &blueprint.notes {
        println!("  - {}", note);
    }
}

fn print_browser_profile(flavor: BrowserHostFlavor) {
    let engine = FlowBrowserEngine::detect();
    let profile = engine.detect_browser_capabilities(flavor, None, None, None, None, None);

    println!("Flow Browser Capability Profile");
    println!("===============================");
    println!("Flavor: {:?}", profile.flavor);
    println!("webgpu={}", profile.webgpu);
    println!("wasm_threads={}", profile.wasm_threads);
    println!("cross_origin_isolated={}", profile.cross_origin_isolated);
    println!("opfs={}", profile.opfs);
    println!("indexeddb={}", profile.indexeddb);
    println!("side_panel={}", profile.side_panel);
    println!("sidebar_action={}", profile.sidebar_action);
    println!("offscreen_document={}", profile.offscreen_document);
    println!(
        "background_service_worker={}",
        profile.background_service_worker
    );
    println!();
    println!("Notes:");
    for note in profile.notes {
        println!("  - {}", note);
    }
}

fn print_browser_plan(
    flavor: BrowserHostFlavor,
    task: BrowserTask,
    modality: Modality,
    model: Option<String>,
    remote_fallback: bool,
) {
    let engine = FlowBrowserEngine::detect();
    let capabilities = engine.detect_browser_capabilities(flavor, None, None, None, None, None);
    let plan = engine.plan_browser_execution(crate::browser::BrowserExecutionRequest {
        task,
        modality,
        local_only: !remote_fallback,
        preferred_model: model,
        allow_remote_fallback: remote_fallback,
        capabilities,
    });

    println!("Flow Browser Execution Plan");
    println!("===========================");
    println!("Task: {:?}", plan.task);
    println!("Modality: {:?}", plan.modality);
    println!(
        "Selected model: {}",
        plan.selected_model.unwrap_or_else(|| "-".to_string())
    );
    println!(
        "Pack key: {}",
        plan.pack_key.unwrap_or_else(|| "-".to_string())
    );
    println!("Backend: {:?}", plan.backend);
    println!("Storage: {:?}", plan.storage_backend);
    println!(
        "Worker: {}",
        plan.worker_kind
            .map(|kind| format!("{kind:?}"))
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "Device target: {}",
        plan.device_target
            .map(|target| format!("{target:?}"))
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "UI surfaces: {}",
        plan.ui_surfaces
            .iter()
            .map(|surface| format!("{surface:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("Local only: {}", plan.local_only);
    println!("Remote allowed: {}", plan.remote_allowed);
    println!();
    println!("Enabled features:");
    for feature in &plan.enabled_features {
        println!("  - {}", feature);
    }
    if !plan.disabled_features.is_empty() {
        println!();
        println!("Disabled features:");
        for feature in &plan.disabled_features {
            println!("  - {}", feature);
        }
    }
    println!();
    println!("Reasons:");
    for reason in &plan.reasons {
        println!("  - {}", reason);
    }
    if let Some(reason) = &plan.unsupported_reason {
        println!();
        println!("Unsupported: {}", reason);
    }
}

fn print_browser_packs() {
    let catalog = default_browser_pack_catalog();

    println!("Flow Browser Pack Catalog");
    println!("=========================");
    for pack in catalog {
        println!("{}", pack.display_name);
        println!("  model_key={}", pack.model_key);
        println!("  pack_key={}", pack.pack_key);
        println!("  modality={:?}", pack.modality);
        println!("  backend={:?}", pack.backend);
        println!(
            "  support=chromium:{} firefox:{} safari:{} web:{} webgpu:{}",
            pack.browser_support.chromium,
            pack.browser_support.firefox,
            pack.browser_support.safari,
            pack.browser_support.standalone_web,
            pack.browser_support.requires_webgpu
        );
        println!(
            "  files={}",
            pack.files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

fn print_production_config(target: FlowIntegrationTarget) -> Result<()> {
    let runtime = crate::dx::DxFlowRuntime::detect();
    println!("{}", runtime.production_config_json(target)?);
    Ok(())
}

fn export_production_bundle_cli(output_dir: &str) -> Result<()> {
    let runtime = crate::dx::DxFlowRuntime::detect();
    let manifest = runtime.export_production_bundle(resolve_repo_relative_path(output_dir))?;
    println!("Exported Flow production bundle");
    println!("===============================");
    println!("Directory: {}", output_dir);
    println!("Device tier: {}", manifest.device_tier);
    println!(
        "Models: text={} stt={} tts={}",
        manifest.selected_text_model.as_deref().unwrap_or("none"),
        manifest.selected_stt_model.as_deref().unwrap_or("none"),
        manifest.selected_tts_model.as_deref().unwrap_or("none"),
    );
    println!("All models ready: {}", manifest.all_models_ready);
    println!("Files:");
    for entry in &manifest.entries {
        println!("  - {}", entry.filename);
    }
    println!("  - manifest.json");
    println!("  - README.txt");
    Ok(())
}

fn print_release_summary() -> Result<()> {
    let runtime = crate::dx::DxFlowRuntime::detect();
    println!("{}", runtime.release_summary()?.to_pretty_json()?);
    Ok(())
}

fn export_release_summary_cli(output_dir: &str) -> Result<()> {
    let runtime = crate::dx::DxFlowRuntime::detect();
    let summary = runtime.export_release_summary(resolve_repo_relative_path(output_dir))?;
    println!("Exported Flow release summary");
    println!("=============================");
    println!("Directory: {}", output_dir);
    println!(
        "Production bundle ready: {}",
        summary.production_bundle_ready
    );
    println!(
        "Artifacts ready: {} / {}",
        summary
            .browser_release_artifacts
            .iter()
            .filter(|artifact| artifact.exists)
            .count(),
        summary.browser_release_artifacts.len()
    );
    println!("Files:");
    println!("  - flow-release-summary.json");
    println!("  - FLOW_RELEASE_HANDOFF.md");
    Ok(())
}

fn resolve_repo_relative_path(path: &str) -> std::path::PathBuf {
    let path = std::path::PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
    }
}

async fn load_chat_llm(broker: &RuntimeBroker) -> Result<LocalLlm> {
    let mut request = BrokerRequest::new(Modality::Chat);
    request.allow_conversion = false;
    request.allow_publish = false;
    let plan = broker.build_plan(request);

    let selected = plan
        .selected_model
        .clone()
        .context("No chat model selected by the runtime broker")?;
    let manifest = broker
        .catalog()
        .iter()
        .find(|candidate| candidate.key == selected)
        .context("Selected chat model is missing from the broker catalog")?;
    let model_path = manifest
        .local_path
        .clone()
        .context("Selected chat model has no local path")?;

    if !Path::new(&model_path).exists() {
        return Err(anyhow::anyhow!(
            "Selected chat model '{}' is not present at {}",
            manifest.key,
            model_path
        ));
    }

    let llm = LocalLlm::with_model_path(model_path);
    llm.initialize().await?;
    Ok(llm)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiveSessionMode {
    VoiceRoundTrip,
    DictateToFocusedInput,
}

impl LiveSessionMode {
    fn title(self) -> &'static str {
        match self {
            Self::VoiceRoundTrip => "Flow Live",
            Self::DictateToFocusedInput => "Flow Dictation",
        }
    }

    fn terminal_title(self) -> &'static str {
        match self {
            Self::VoiceRoundTrip => "Flow Live - listening",
            Self::DictateToFocusedInput => "Flow Dictation - listening",
        }
    }

    fn uses_focused_input(self) -> bool {
        matches!(self, Self::DictateToFocusedInput)
    }

    fn allows_bare_hotkeys(self) -> bool {
        matches!(self, Self::VoiceRoundTrip)
    }
}

async fn run_live_mode() -> Result<()> {
    run_live_session(LiveSessionMode::VoiceRoundTrip).await
}

async fn run_dictation_mode() -> Result<()> {
    run_live_session(LiveSessionMode::DictateToFocusedInput).await
}

async fn run_live_session(mode: LiveSessionMode) -> Result<()> {
    let broker = RuntimeBroker::detect();

    set_terminal_title(mode.terminal_title());
    println!("{}", mode.title());
    println!("{}", "=".repeat(mode.title().len()));
    println!("Device tier: {:?}", broker.device_profile().tier);
    println!(
        "Push-to-talk: {}",
        shortcut_label(
            &broker.activation().push_to_talk.modifiers,
            &broker.activation().push_to_talk.key
        )
    );
    println!(
        "Hands-free toggle: {}",
        shortcut_label(
            &broker.activation().hands_free_toggle.modifiers,
            &broker.activation().hands_free_toggle.key
        )
    );
    if !broker.activation().wake_words.is_empty() {
        println!(
            "Wake words: {}",
            broker
                .activation()
                .wake_words
                .iter()
                .map(|item| item.command_key.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!();

    if mode.uses_focused_input() {
        println!(
            "[indicator] compact status lives in this terminal title; keep your target input focused"
        );
        println!("[init] arming microphone and wake detector; STT loads only after speech");
    } else {
        println!("[init] arming microphone and wake detector; STT/TTS/LLM load on demand");
    }
    let mut stt: Option<LocalSttEngine> = if mode.uses_focused_input() {
        Some(load_live_stt_engine(&broker, mode)?)
    } else {
        None
    };
    let mut tts: Option<KokoroTTS> = None;
    let mut llm: Option<LocalLlm> = None;
    let mut input_bridge = mode.uses_focused_input().then(|| {
        NativeSelectionBridge::live(OperatingSystemFamily::from_host_label(std::env::consts::OS))
    });

    let wakeword_detector = WakeWordDetector::from_config(&broker.activation().wake_words)?
        .map(|detector| Arc::new(Mutex::new(detector)));
    let wakeword_available = wakeword_detector.is_some();

    let sample_rate = 16_000_u32;
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow::anyhow!("No input device found"))?;
    let config = device.default_input_config()?;
    let channels = config.channels() as usize;
    let input_sample_rate = config.sample_rate();

    println!(
        "[audio] input={} Hz, channels={}, processing={} Hz",
        input_sample_rate, channels, sample_rate
    );
    if mode.allows_bare_hotkeys() {
        println!("[ready] press Enter to record, Space to stop and process");
    }
    println!("[ready] press Ctrl+Shift+Space to toggle recording");
    if wakeword_available {
        println!("[ready] local wake words are armed");
    } else if mode.uses_focused_input() {
        println!(
            "[ready] wake models are missing; speech activity will start recording automatically"
        );
        println!("[ready] Ctrl+Shift+Space still works as a manual override");
    } else {
        println!(
            "[ready] wake models are missing; Ctrl+Shift+Space is the local fallback until ONNX wake files exist"
        );
    }
    println!("[ready] press Ctrl+C to exit\n");

    let is_recording = Arc::new(AtomicBool::new(false));
    let should_process = Arc::new(AtomicBool::new(false));
    let speech_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let last_voice_at = Arc::new(Mutex::new(Instant::now()));

    let is_recording_kb = Arc::clone(&is_recording);
    let should_process_kb = Arc::clone(&should_process);
    let speech_buffer_kb = Arc::clone(&speech_buffer);
    let last_voice_at_kb = Arc::clone(&last_voice_at);
    let allow_bare_hotkeys = mode.allows_bare_hotkeys();

    std::thread::spawn(move || {
        use rdev::{Event, EventType, Key, listen};

        let ctrl_down = Arc::new(AtomicBool::new(false));
        let shift_down = Arc::new(AtomicBool::new(false));
        let ctrl_down_cb = Arc::clone(&ctrl_down);
        let shift_down_cb = Arc::clone(&shift_down);

        let callback = move |event: Event| match event.event_type {
            EventType::KeyPress(Key::ControlLeft | Key::ControlRight) => {
                ctrl_down_cb.store(true, Ordering::Relaxed);
            }
            EventType::KeyRelease(Key::ControlLeft | Key::ControlRight) => {
                ctrl_down_cb.store(false, Ordering::Relaxed);
            }
            EventType::KeyPress(Key::ShiftLeft | Key::ShiftRight) => {
                shift_down_cb.store(true, Ordering::Relaxed);
            }
            EventType::KeyRelease(Key::ShiftLeft | Key::ShiftRight) => {
                shift_down_cb.store(false, Ordering::Relaxed);
            }
            EventType::KeyPress(Key::Return) if allow_bare_hotkeys => {
                if !is_recording_kb.load(Ordering::Relaxed) {
                    if let Ok(mut buffer) = speech_buffer_kb.lock() {
                        buffer.clear();
                    }
                    if let Ok(mut last_voice) = last_voice_at_kb.lock() {
                        *last_voice = Instant::now();
                    }
                    is_recording_kb.store(true, Ordering::Relaxed);
                    println!("[record] started");
                    set_terminal_title("Flow - recording");
                }
            }
            EventType::KeyPress(Key::Space)
                if ctrl_down_cb.load(Ordering::Relaxed)
                    && shift_down_cb.load(Ordering::Relaxed) =>
            {
                let new_state = !is_recording_kb.load(Ordering::Relaxed);
                if new_state {
                    if let Ok(mut buffer) = speech_buffer_kb.lock() {
                        buffer.clear();
                    }
                    if let Ok(mut last_voice) = last_voice_at_kb.lock() {
                        *last_voice = Instant::now();
                    }
                    is_recording_kb.store(true, Ordering::Relaxed);
                    println!("[record] toggled on");
                    set_terminal_title("Flow - recording");
                } else {
                    is_recording_kb.store(false, Ordering::Relaxed);
                    should_process_kb.store(true, Ordering::Relaxed);
                    println!("[record] toggled off, processing");
                    set_terminal_title("Flow - processing");
                }
            }
            EventType::KeyPress(Key::Space) if allow_bare_hotkeys => {
                if is_recording_kb.load(Ordering::Relaxed) {
                    is_recording_kb.store(false, Ordering::Relaxed);
                    should_process_kb.store(true, Ordering::Relaxed);
                    println!("[record] stopped, processing");
                    set_terminal_title("Flow - processing");
                }
            }
            _ => {}
        };

        if let Err(error) = listen(callback) {
            eprintln!("[error] keyboard listener: {:?}", error);
        }
    });

    let is_recording_audio = Arc::clone(&is_recording);
    let should_process_audio = Arc::clone(&should_process);
    let speech_buffer_audio = Arc::clone(&speech_buffer);
    let pre_roll_buffer_audio = Arc::new(Mutex::new(Vec::<f32>::new()));
    let wakeword_detector_audio = wakeword_detector.clone();
    let last_voice_at_audio = Arc::clone(&last_voice_at);
    let mut vad_gate = NoiseGateVAD::new(sample_rate).ok();
    let silence_timeout = Duration::from_millis(1500);
    let min_auto_stop_samples = sample_rate as usize / 2;
    let auto_record_on_voice = mode.uses_focused_input() && !wakeword_available;
    let auto_start_min_rms = 0.00002_f32;
    let pre_roll_limit_samples = (sample_rate as usize * 3) / 4;
    let mut last_idle_meter_at = Instant::now();

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &_| {
            let mono: Vec<f32> = if channels == 2 {
                data.chunks(2)
                    .map(|chunk| (chunk[0] + chunk.get(1).copied().unwrap_or(0.0)) / 2.0)
                    .collect()
            } else {
                data.to_vec()
            };

            let processed = if input_sample_rate != sample_rate && input_sample_rate > sample_rate {
                let ratio = (input_sample_rate / sample_rate).max(1) as usize;
                mono.iter().step_by(ratio).copied().collect::<Vec<_>>()
            } else {
                mono
            };

            let input_rms = rms_energy(&processed);
            let (speech_samples, is_speech) = if let Some(vad) = vad_gate.as_mut() {
                let (gated, is_speech, _) = vad.process(&processed);
                (gated, is_speech)
            } else {
                (processed.clone(), input_rms > auto_start_min_rms)
            };
            let voice_active = if auto_record_on_voice {
                input_rms >= auto_start_min_rms || is_speech
            } else {
                is_speech
            };
            let record_samples = if auto_record_on_voice {
                &processed
            } else {
                &speech_samples
            };

            if !is_recording_audio.load(Ordering::Relaxed) {
                if auto_record_on_voice {
                    if let Ok(mut pre_roll) = pre_roll_buffer_audio.try_lock() {
                        pre_roll.extend_from_slice(record_samples);
                        if pre_roll.len() > pre_roll_limit_samples {
                            let excess = pre_roll.len() - pre_roll_limit_samples;
                            pre_roll.drain(..excess);
                        }
                    }
                }

                if auto_record_on_voice && last_idle_meter_at.elapsed() >= Duration::from_secs(2) {
                    println!("[meter] idle rms={:.7}", input_rms);
                    last_idle_meter_at = Instant::now();
                }

                if let Some(detector) = &wakeword_detector_audio {
                    if let Ok(mut detector) = detector.try_lock() {
                        match detector.feed_f32(&processed) {
                            Ok(Some(detection)) => {
                                if let Ok(mut buffer) = speech_buffer_audio.try_lock() {
                                    buffer.clear();
                                }
                                if let Ok(mut last_voice) = last_voice_at_audio.try_lock() {
                                    *last_voice = Instant::now();
                                }
                                is_recording_audio.store(true, Ordering::Relaxed);
                                println!(
                                    "[wake] '{}' ({}) detected at {:.0}% confidence",
                                    detection.command_key,
                                    detection.phrase,
                                    detection.confidence * 100.0
                                );
                            }
                            Ok(None) => {}
                            Err(error) => {
                                eprintln!("[warn] wake-word detection error: {}", error);
                            }
                        }
                    }
                }

                if auto_record_on_voice && voice_active {
                    if let Ok(mut buffer) = speech_buffer_audio.try_lock() {
                        buffer.clear();
                        if let Ok(pre_roll) = pre_roll_buffer_audio.try_lock() {
                            buffer.extend_from_slice(&pre_roll);
                        }
                        buffer.extend_from_slice(record_samples);
                    }
                    if let Ok(mut pre_roll) = pre_roll_buffer_audio.try_lock() {
                        pre_roll.clear();
                    }
                    if let Ok(mut last_voice) = last_voice_at_audio.try_lock() {
                        *last_voice = Instant::now();
                    }
                    is_recording_audio.store(true, Ordering::Relaxed);
                    println!("[voice] speech detected, recording (rms={:.5})", input_rms);
                    set_terminal_title("Flow - recording");
                }
                return;
            }

            if let Ok(mut buffer) = speech_buffer_audio.try_lock() {
                buffer.extend_from_slice(record_samples);
                if voice_active {
                    if let Ok(mut last_voice) = last_voice_at_audio.try_lock() {
                        *last_voice = Instant::now();
                    }
                } else if buffer.len() >= min_auto_stop_samples {
                    if let Ok(last_voice) = last_voice_at_audio.try_lock() {
                        if last_voice.elapsed() >= silence_timeout {
                            is_recording_audio.store(false, Ordering::Relaxed);
                            should_process_audio.store(true, Ordering::Relaxed);
                            println!("[record] silence detected, processing");
                            set_terminal_title("Flow - processing");
                        }
                    }
                }
            }
        },
        |error| eprintln!("[error] audio stream: {}", error),
        None,
    )?;

    stream.play()?;

    let mut recording_counter = 1_u32;

    loop {
        if should_process.load(Ordering::Relaxed) {
            should_process.store(false, Ordering::Relaxed);

            let recorded_samples = {
                let mut buffer = speech_buffer.lock().unwrap();
                let samples = buffer.clone();
                buffer.clear();
                samples
            };

            if recorded_samples.is_empty() {
                println!("[warn] no audio recorded\n");
                continue;
            }

            let energy = recorded_samples
                .iter()
                .map(|sample| sample * sample)
                .sum::<f32>()
                / recorded_samples.len() as f32;
            let rms = energy.sqrt();

            println!(
                "[process] {} samples ({:.2}s), rms={:.6}",
                recorded_samples.len(),
                recorded_samples.len() as f32 / sample_rate as f32,
                rms
            );

            let prepared = prepare_recording_for_stt(&recorded_samples, sample_rate, mode);
            if prepared.samples.len() < minimum_stt_samples(sample_rate, mode) {
                println!(
                    "[warn] captured clip is too short after cleanup ({:.2}s); keep speaking a little longer\n",
                    prepared.samples.len() as f32 / sample_rate as f32
                );
                set_terminal_title(mode.terminal_title());
                continue;
            }

            println!(
                "[process] prepared {:.2}s, noise_floor={:.7}, gain={:.1}x, final_rms={:.6}",
                prepared.samples.len() as f32 / sample_rate as f32,
                prepared.noise_floor,
                prepared.gain,
                prepared.final_rms
            );

            let numbered_file = format!("recording_{recording_counter:04}.wav");
            recording_counter += 1;
            write_wav(&numbered_file, sample_rate, &prepared.samples)?;
            write_wav("temp_live_recording.wav", sample_rate, &prepared.samples)?;
            println!("[file] saved {}", numbered_file);

            set_terminal_title("Flow - transcribing");
            print!("[stt] transcribing... ");
            std::io::stdout().flush()?;
            if stt.is_none() {
                stt = Some(load_live_stt_engine(&broker, mode)?);
            }
            let raw_text = stt
                .as_mut()
                .context("Local STT engine was not initialized")?
                .transcribe("temp_live_recording.wav")?;
            println!("\"{}\"", raw_text);

            if raw_text.trim().len() < 3 {
                println!("[warn] transcription too short\n");
                set_terminal_title(mode.terminal_title());
                continue;
            }

            match mode {
                LiveSessionMode::DictateToFocusedInput => {
                    let dictated_text = prepare_dictation_text(&raw_text);
                    print!("[input] inserting into focused input... ");
                    std::io::stdout().flush()?;
                    let inserted = input_bridge
                        .as_mut()
                        .map(|bridge| bridge.replace_selection(&dictated_text))
                        .unwrap_or(false);
                    if inserted {
                        println!("done");
                    } else {
                        println!("failed");
                        println!(
                            "[warn] desktop paste bridge is unavailable; transcript: {}",
                            dictated_text
                        );
                    }
                    set_terminal_title(mode.terminal_title());
                    println!();
                }
                LiveSessionMode::VoiceRoundTrip => {
                    print!("[ai] cleaning... ");
                    std::io::stdout().flush()?;
                    if llm.is_none() {
                        llm = Some(load_chat_llm(&broker).await?);
                    }
                    let cleaned_text = llm
                        .as_ref()
                        .context("Local chat model was not initialized")?
                        .clean_speech(&raw_text)
                        .await?;
                    println!("\"{}\"", cleaned_text);

                    print!("[tts] speaking... ");
                    std::io::stdout().flush()?;
                    if tts.is_none() {
                        tts = Some(KokoroTTS::new_async().await?);
                    }
                    tts.as_mut()
                        .context("Kokoro TTS was not initialized")?
                        .speak(&cleaned_text)?;
                    println!("done\n");
                    set_terminal_title(mode.terminal_title());
                }
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

fn load_live_stt_engine(broker: &RuntimeBroker, mode: LiveSessionMode) -> Result<LocalSttEngine> {
    if mode.uses_focused_input()
        && LocalSttEngine::model_files_ready(PARAKEET_STT_KEY, Some(PARAKEET_STT_PATH))
    {
        #[cfg(feature = "sherpa-stt")]
        {
            println!("[stt] preloading Parakeet TDT 0.6B v3 INT8...");
            let started = Instant::now();
            let engine = LocalSttEngine::from_selection(PARAKEET_STT_KEY, Some(PARAKEET_STT_PATH))?;
            println!(
                "[stt] Parakeet ready in {:.1}s",
                started.elapsed().as_secs_f32()
            );
            return Ok(engine);
        }

        #[cfg(not(feature = "sherpa-stt"))]
        {
            println!(
                "[stt] Parakeet files are present, but this binary was built without --features sherpa-stt"
            );
        }
    }

    println!("[stt] preloading local broker-selected STT...");
    let started = Instant::now();
    let engine = LocalSttEngine::from_broker(broker)?;
    println!(
        "[stt] broker-selected STT ready in {:.1}s",
        started.elapsed().as_secs_f32()
    );
    Ok(engine)
}

struct PreparedRecording {
    samples: Vec<f32>,
    noise_floor: f32,
    gain: f32,
    final_rms: f32,
}

fn prepare_recording_for_stt(
    samples: &[f32],
    sample_rate: u32,
    mode: LiveSessionMode,
) -> PreparedRecording {
    let mut cleaned = remove_dc_offset(samples);
    let noise_floor = estimate_noise_floor(&cleaned, sample_rate);

    if mode.uses_focused_input() {
        let gate_threshold = (noise_floor * 2.0).clamp(0.000005, 0.003);
        for sample in &mut cleaned {
            if sample.abs() < gate_threshold {
                *sample *= 0.35;
            }
        }
        cleaned = trim_low_energy_edges(&cleaned, sample_rate, gate_threshold * 1.25);
    }

    let target_rms = if mode.uses_focused_input() {
        0.08_f32
    } else {
        0.1_f32
    };
    let max_gain = if mode.uses_focused_input() {
        50.0_f32
    } else {
        10.0_f32
    };
    let current_rms = rms_energy(&cleaned);
    let gain = if current_rms > 0.000001 {
        (target_rms / current_rms).clamp(1.0, max_gain)
    } else {
        1.0
    };
    if gain > 1.5 {
        println!("[process] boosting input by {:.1}x", gain);
    }

    let samples = cleaned
        .iter()
        .map(|sample| (*sample * gain).clamp(-1.0, 1.0))
        .collect::<Vec<_>>();
    let final_rms = rms_energy(&samples);

    PreparedRecording {
        samples,
        noise_floor,
        gain,
        final_rms,
    }
}

fn remove_dc_offset(samples: &[f32]) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }

    let mean = samples.iter().sum::<f32>() / samples.len() as f32;
    samples
        .iter()
        .map(|sample| (*sample - mean).clamp(-1.0, 1.0))
        .collect()
}

fn estimate_noise_floor(samples: &[f32], sample_rate: u32) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let window = ((sample_rate as usize) / 5).clamp(1, samples.len());
    let head = rms_energy(&samples[..window]);
    let tail = rms_energy(&samples[samples.len().saturating_sub(window)..]);
    head.min(tail)
}

fn trim_low_energy_edges(samples: &[f32], sample_rate: u32, threshold: f32) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }

    let first = samples.iter().position(|sample| sample.abs() >= threshold);
    let last = samples.iter().rposition(|sample| sample.abs() >= threshold);
    let (Some(first), Some(last)) = (first, last) else {
        return samples.to_vec();
    };

    let keep = (sample_rate as usize / 4).max(1);
    let start = first.saturating_sub(keep);
    let end = (last + keep).min(samples.len().saturating_sub(1));
    if end <= start || end - start < sample_rate as usize / 4 {
        return samples.to_vec();
    }

    samples[start..=end].to_vec()
}

fn minimum_stt_samples(sample_rate: u32, mode: LiveSessionMode) -> usize {
    if mode.uses_focused_input() {
        (sample_rate as f32 * 0.75) as usize
    } else {
        (sample_rate as f32 * 0.35) as usize
    }
}

fn prepare_dictation_text(raw_text: &str) -> String {
    raw_text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn rms_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let energy = samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32;
    energy.sqrt()
}

fn set_terminal_title(title: &str) {
    let sanitized = title.replace(['\x07', '\x1b'], "");
    print!("\x1b]0;{}\x07", sanitized);
    let _ = std::io::stdout().flush();
}

fn write_wav(path: &str, sample_rate: u32, samples: &[f32]) -> Result<()> {
    use hound::{SampleFormat, WavSpec, WavWriter};

    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec)?;
    for sample in samples {
        let scaled = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(scaled)?;
    }
    writer.finalize()?;
    Ok(())
}

fn parse_modality(value: &str) -> Result<Modality> {
    match value.to_ascii_lowercase().as_str() {
        "chat" | "llm" => Ok(Modality::Chat),
        "text" => Ok(Modality::Text),
        "ui" | "uigen" | "ui-generation" | "ui_generation" | "frontend" => {
            Ok(Modality::UiGeneration)
        }
        "vlm" | "vision-language" | "vision_language" => Ok(Modality::VisionLanguage),
        "stt" | "speech" | "speech-to-text" | "speech_to_text" => Ok(Modality::SpeechToText),
        "tts" | "text-to-speech" | "text_to_speech" => Ok(Modality::TextToSpeech),
        "ocr" => Ok(Modality::Ocr),
        "image" | "image-generation" | "image_generation" => Ok(Modality::ImageGeneration),
        "video" | "video-generation" | "video_generation" => Ok(Modality::VideoGeneration),
        "wake" | "wakeword" | "wake-word" | "wake_words" => Ok(Modality::WakeWord),
        "grammar" => Ok(Modality::Grammar),
        other => Err(anyhow::anyhow!("Unsupported modality '{}'", other)),
    }
}

fn parse_host_surface(value: &str) -> Result<HostSurface> {
    match value.to_ascii_lowercase().as_str() {
        "dx" => Ok(HostSurface::Dx),
        "flow" | "flow-app" => Ok(HostSurface::FlowApp),
        "zeroclaw" => Ok(HostSurface::ZeroclawFork),
        "codex" => Ok(HostSurface::CodexFork),
        "zed" => Ok(HostSurface::ZedFork),
        "desktop" => Ok(HostSurface::Desktop),
        "android" => Ok(HostSurface::AndroidNative),
        "ios" => Ok(HostSurface::IosNative),
        "tauri" => Ok(HostSurface::Tauri),
        "flutter" => Ok(HostSurface::Flutter),
        "browser" | "wasm" | "browser-wasm" => Ok(HostSurface::BrowserWasm),
        "vps" => Ok(HostSurface::Vps),
        "raspberry-pi" | "raspberrypi" | "pi" => Ok(HostSurface::RaspberryPi),
        "watch" => Ok(HostSurface::Watch),
        "tv" => Ok(HostSurface::Tv),
        "tablet" => Ok(HostSurface::Tablet),
        "custom" => Ok(HostSurface::CustomRustHost),
        other => Err(anyhow::anyhow!("Unsupported host surface '{}'", other)),
    }
}

fn parse_operating_system(value: &str) -> Result<OperatingSystemFamily> {
    match value.to_ascii_lowercase().as_str() {
        "windows" | "win" | "win32" => Ok(OperatingSystemFamily::Windows),
        "macos" | "mac" | "darwin" | "osx" => Ok(OperatingSystemFamily::Macos),
        "linux" => Ok(OperatingSystemFamily::Linux),
        "android" => Ok(OperatingSystemFamily::Android),
        "ios" => Ok(OperatingSystemFamily::Ios),
        "browser" | "wasm" | "web" | "browser-wasm" => Ok(OperatingSystemFamily::BrowserWasm),
        "server" | "daemon" => Ok(OperatingSystemFamily::Server),
        other => Err(anyhow::anyhow!("Unsupported operating system '{}'", other)),
    }
}

fn parse_browser_flavor(value: &str) -> Result<BrowserHostFlavor> {
    match value.to_ascii_lowercase().as_str() {
        "chromium" | "chrome" | "edge" => Ok(BrowserHostFlavor::ChromiumExtension),
        "firefox" | "gecko" => Ok(BrowserHostFlavor::FirefoxExtension),
        "safari" => Ok(BrowserHostFlavor::SafariWebExtension),
        "web" | "standalone" | "webapp" => Ok(BrowserHostFlavor::StandaloneWebApp),
        other => Err(anyhow::anyhow!("Unsupported browser flavor '{}'", other)),
    }
}

fn parse_browser_task(value: &str) -> Result<BrowserTask> {
    match value.to_ascii_lowercase().as_str() {
        "rewrite" | "rewrite-selection" | "rewrite_selection" => Ok(BrowserTask::RewriteSelection),
        "summarize-selection" | "summarize_selection" => Ok(BrowserTask::SummarizeSelection),
        "summarize-page" | "summarize_page" | "summarize" => Ok(BrowserTask::SummarizePage),
        "compose" | "compose-draft" | "compose_draft" => Ok(BrowserTask::ComposeDraft),
        "explain" | "explain-page" | "explain_page" => Ok(BrowserTask::ExplainPage),
        "ocr" | "ocr-image" | "ocr_image" => Ok(BrowserTask::OcrImage),
        "vlm" | "multimodal" | "multimodal-ask" | "multimodal_ask" => {
            Ok(BrowserTask::MultimodalAsk)
        }
        other => Err(anyhow::anyhow!("Unsupported browser task '{}'", other)),
    }
}

fn parse_integration_target(value: &str) -> Result<FlowIntegrationTarget> {
    match value.to_ascii_lowercase().as_str() {
        "dx" | "dx-desktop" | "desktop" => Ok(FlowIntegrationTarget::DxDesktop),
        "browser" | "browser-extension" | "webext" => Ok(FlowIntegrationTarget::BrowserExtension),
        "zed" | "zed-fork" => Ok(FlowIntegrationTarget::ZedFork),
        "codex" | "codex-fork" => Ok(FlowIntegrationTarget::CodexFork),
        "zeroclaw" | "zeroclaw-fork" | "openclaw" => Ok(FlowIntegrationTarget::ZeroClawFork),
        other => Err(anyhow::anyhow!(
            "Unsupported integration target '{}'",
            other
        )),
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{:.0} B", bytes)
    }
}

fn shortcut_label(modifiers: &[String], key: &str) -> String {
    if modifiers.is_empty() {
        key.to_string()
    } else {
        format!("{}+{}", modifiers.join("+"), key)
    }
}
