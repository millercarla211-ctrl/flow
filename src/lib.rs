// Flow - Open-source voice assistant library
//
// This library provides STT, LLM, and TTS capabilities for building
// voice-enabled applications.

pub mod audio;
pub mod browser;
pub mod cli;
pub mod codex;
pub mod competitive;
pub mod config;
pub mod dx;
pub mod embed;
pub mod experience;
pub mod forge_bridge;
pub mod friday;
pub mod long_context;
pub mod models;
pub mod pipeline;
pub mod prompt;
pub mod provider_catalog;
pub mod remote;
pub mod runtime;
pub mod search;
pub mod storage;
pub mod utils;
pub mod workspace;
pub mod writing;
pub mod zed;
pub mod zeroclaw;

// Re-export commonly used types
pub use audio::{AudioLoader, MelSpectrogramConfig, compute_mel_spectrogram};
pub use browser::{
    BrowserCapabilityProfile, BrowserChatMessage, BrowserDeviceTarget, BrowserExecutionBackend,
    BrowserExecutionPlan, BrowserExecutionRequest, BrowserExtensionInstallProbe,
    BrowserExtensionLaunchReport, BrowserExtensionLaunchTarget, BrowserExtensionMessage,
    BrowserExtensionSmokeReport, BrowserExtensionSmokeStatus, BrowserExtensionSmokeTarget,
    BrowserHostFlavor, BrowserInferenceInvocation, BrowserInferenceRequest, BrowserPackFile,
    BrowserPackManifest, BrowserPackRecoveryFile, BrowserPackRecoveryReport,
    BrowserPackRecoveryScenario, BrowserPackRecoveryScenarioKind, BrowserPackRecoveryStatus,
    BrowserPackRecoveryTarget, BrowserPackResolution, BrowserPackReuseFile,
    BrowserPackReuseReport, BrowserPackReuseStatus, BrowserPackReuseTarget, BrowserPackSupport,
    BrowserStorageBackend, BrowserTask, BrowserTokenStreamPlan, BrowserUiSurface,
    BrowserWebLlmAccelerationReport, BrowserWebLlmAccelerationStatus,
    BrowserWebLlmAccelerationTarget, BrowserWebLlmGuardrail, BrowserWorkerKind, FlowBrowserEngine,
    browser_extension_launch_smoke_report, browser_extension_launch_smoke_report_for_root,
    browser_extension_smoke_report, browser_extension_smoke_report_for_root,
    browser_pack_recovery_smoke_report, browser_pack_recovery_smoke_report_for_catalog,
    browser_pack_reuse_smoke_report, browser_pack_reuse_smoke_report_for_catalog,
    browser_webllm_acceleration_report, browser_webllm_acceleration_report_for_catalog,
    default_browser_pack_catalog,
};
pub use cli::{Args, Command, execute};
pub use codex::{
    CodexApprovalMode, CodexAttachment, CodexAttachmentKind, CodexContextItem,
    CodexExecutionTarget, CodexFlowAdapter, CodexFollowUpRequest, CodexLocalModelStatus,
    CodexReasoningEffort, CodexReviewFinding, CodexReviewRequest, CodexReviewResponse,
    CodexReviewSeverity, CodexSurface, CodexTaskCandidate, CodexTaskKind, CodexTaskRequest,
    CodexTaskResponse,
};
pub use competitive::{
    CompetitiveFeature, CompetitiveScorecard, CompetitiveSegment, CompletionItem,
    CompletionItemStatus, CompletionSet, FeatureStatus, active_completion_set,
    default_competitive_scorecard,
};
pub use config::{
    FlowBrowserProductionConfig, FlowCodexProductionConfig, FlowDeploymentEnvironment,
    FlowIntegrationTarget, FlowProductionBundleDocument, FlowProductionBundleEntry,
    FlowProductionBundleManifest, FlowProductionConfig, FlowReleaseFileRecord, FlowReleaseSummary,
    FlowReleaseTask, FlowReleaseTaskStatus, FlowRuntimeProductionConfig, FlowZedProductionConfig,
    FlowZeroClawProductionConfig,
};
pub use dx::DxFlowRuntime;
pub use embed::{
    AdjacentProject, FlowEmbeddingRegistry, FlowLibraryBlueprint, FlowSubsystem, ForgeStrategy,
    HostSurface, IntegrationMode, LongContextStrategy, ProviderAuthKind, ProviderStrategy,
    SearchStrategy, SerializerStrategy,
};
pub use experience::{
    AcademicCitationNeed, AcademicClaimReview, AcademicClaimStatus, AcademicReviewReport,
    AcademicReviewRequest, AcademicSource, AccessibilityBackend, AccessibilityDiagnosticSeverity,
    AccessibilityMode, AppContext, AppUsageStat, DictationAssistRequest, DictationAssistResult,
    DictionaryEntry, ExpandedSnippet, FlowAccessibilityDiagnostic, FlowAccessibilityRuntime,
    FlowDictationEngine, FlowExperienceHub, FlowHostDictationBlocker, FlowHostDictationExecution,
    FlowHostDictationReadiness, FlowHostDictationRequest, FlowHostPauseController,
    FlowHostPauseSnapshot, FlowTypingAssistant, FlowWorkspaceProfile, SnippetEntry, StylePreset,
    StyleRule, TextCommandRequest, TextCommandResult, ToneStyle, TypingAssistRequest,
    TypingAssistResult, UsageDashboardSnapshot, WritingChangeExplanation, WritingChangeKind,
    WritingDomain,
};
pub use forge_bridge::{ForgeAssetKind, ForgeBridge, ForgeRemoteKind, ForgeSyncPlan};
pub use friday::{
    FridayAnswerDelta, FridayAnswerDeltaKind, FridayArtifactCheckpoint, FridayArtifactDiff,
    FridayArtifactFinding, FridayArtifactKind, FridayArtifactManifest, FridayArtifactRecord,
    FridayArtifactSnapshot, FridayArtifactStore, FridayAutomationRuntimeRecord,
    FridayAutomationTrigger, FridayBoundarySeverity, FridayCapability, FridayCheckpointReason,
    FridayCitationRecord, FridayCodeTaskRecord, FridayCodeTaskStatus, FridayCompetitor,
    FridayConnectorAuthState, FridayConnectorKind, FridayConnectorRecord,
    FridayDashboardAction, FridayDashboardActionKind, FridayDashboardCard,
    FridayDashboardExportBundle, FridayDashboardExportFile, FridayDashboardExportHistory,
    FridayDashboardExportManifest, FridayDashboardHistoryRecord, FridayDashboardPanel,
    FridayDashboardPanelStatus, FridayDashboardReleaseReviewHandoff,
    FridayDashboardReleaseReviewItem, FridayDashboardReleaseReviewLink,
    FridayDashboardProductUiActionBinding, FridayDashboardProductUiBinding,
    FridayDashboardProductUiCardBinding,
    FridayDashboardScreenshotHistory, FridayDashboardScreenshotRecord, FridayDashboardScreenshotStatus,
    FridayExecutionHandoff, FridayExecutionHandoffReport, FridayExecutionHandoffStatus,
    FridayExecutionSurface, FridayFeatureStatus, FridayGenerationSummary, FridayLocalCheckKind,
    FridayLocalCheckStatus,
    FridayBrowserVerificationReport, FridayBrowserVerificationTarget, FridayDeployGate,
    FridayLiveUiBindingStatus, FridayLiveUiFileBinding, FridayLiveUiRouteBinding,
    FridayLiveUiRouteBindingReport, FridayLocalExecutionCheck, FridayLocalExecutionReport,
    FridayMemoryRecord, FridayMemoryState, FridayMediaAffordance, FridayMediaAffordanceStatus,
    FridayMultimodalArtifactImport,
    FridayMultimodalArtifactImportReport, FridayMultimodalArtifactMetadata,
    FridayMultimodalDiagnosticItem,
    FridayMultimodalDiagnosticStatus, FridayMultimodalModelFile, FridayMultimodalModelRoute,
    FridayMultimodalRequestKind, FridayMultimodalRouteDecision, FridayMultimodalRouteStatus,
    FridayMultimodalRuntimeRecord, FridayMultimodalSurface, FridayMultimodalUiDiagnostics,
    FridayOcrSmokeReport, FridayOcrSmokeStatus, FridayOperatorReadinessItem,
    FridayOperatorReadinessReport, FridayOperatorReadinessStatus, FridayPermissionFinding,
    FridayPermissionScope, FridayPreviewRunner, FridayRouteVisualReport, FridayRouteVisualStatus,
    FridayRouteVisualTarget, FridayScreenshotSourceRecord, FridayScreenshotVlmHandoffReport,
    FridayProductPlan, FridayProjectFile, FridayProjectRecord, FridayResearchEventKind,
    FridayResearchExportManifest,
    FridayResearchReport, FridayResearchRunEvent, FridayResearchStage, FridayResearchStageKind,
    FridayResearchStageStatus, FridayResearchWorkflow, FridayRuntimeFinding,
    FridayRuntimeRecordStatus, FridayRuntimeSurfaceManifest, FridayRuntimeSurfaceSnapshot,
    FridayRuntimeSurfaceStore, FridaySearchPolicy, FridaySourceGroup, FridaySynthesizedAnswer,
    FridayMultimodalVisualCheckReport, FridayUiDataBinding, FridayUiIntegrationPlan,
    FridayUiIntegrationStatus,
    FridayUiRouteContract, FridayUiSourceControl, FridayUiState, FridayUiStateKind,
    FridayUiStateTone, FridayUiVisualCheckStatus, FridayUiVisualRequirement,
    FridayUiVisualViewport,
    FridayVoiceRuntimeRecord, FridayWorkspaceArea, FridayWorkspaceManifest,
    FridayWorkspaceSnapshot, FridayWorkspaceStore, FridayWorkspaceView, MetasearchExecutionMode,
    FridayVerificationStatus, FridayVlmContractReport, FridayVlmContractStatus,
    MetasearchExecutionTarget,
    default_friday_browser_verification_report, default_friday_local_execution_checks,
    default_friday_product_plan, default_friday_ui_integration_plan, export_friday_dashboard_bundle,
    friday_answer_search_plan, friday_dashboard_export_history_from_export,
    friday_dashboard_panel_from_export, friday_dashboard_product_ui_binding_from_export,
    friday_dashboard_release_review_from_export, friday_dashboard_screenshot_history,
    friday_execution_handoff_report,
    friday_live_ui_route_binding_report,
    friday_live_ui_route_binding_report_for_root, friday_media_affordances,
    friday_multimodal_route, friday_multimodal_ui_diagnostics, friday_multimodal_visual_check,
    friday_operator_readiness_report,
    friday_research_search_plan, friday_route_visual_report, friday_route_visual_report_for_root,
    run_friday_ocr_smoke, run_friday_screenshot_vlm_handoff, run_friday_vlm_contract,
};
pub use long_context::{LongContextExecutionPlan, LongContextTask, RlmBridge};
pub use models::{KokoroTTS, LocalLlm, LocalSttEngine, MoonshineSTT};
pub use pipeline::VoicePipeline;
pub use prompt::{DxSerializer, SerializedPromptEnvelope};
pub use provider_catalog::{CatalogSource, ProviderCatalogBridge, ProviderCatalogPlan};
pub use remote::{
    AccessTier, RemoteCapability, RemoteModelEndpoint, RemoteProviderRouter, SeamlessRoutePlan,
    UnifiedUsagePolicy,
};
pub use runtime::{
    ActivationConfig, BrokerRequest, DeviceProfile, ExecutionPlan, FlowLocalRuntime,
    FlowLocalRuntimeSummary, LocalModelSelection, LocalSpeechAudio, LocalSpeechCleanup,
    LocalSpeechRoundtrip, Modality, RuntimeBroker,
};
pub use search::{
    MetasearchApiResponse, MetasearchApiResult, MetasearchBridge, MetasearchServerConfig,
    SearchIntent, SearchRequestPlan, SearchVertical, metasearch_categories,
};
pub use storage::{FlowPackManifest, FlowPackStore, PromptCacheIndex};
pub use utils::{check_memory_requirements, detect_device_profile, get_memory_info};
pub use workspace::{DxProjectStatus, dx_project_statuses};
pub use writing::HarperGrammarChecker;
pub use zed::{
    ZedAgentPanelRequest, ZedAgentPanelResponse, ZedAgentProfile, ZedAiSurface, ZedContextItem,
    ZedEditPredictionRequest, ZedEditPredictionResponse, ZedFlowAdapter, ZedInlineAssistRequest,
    ZedInlineAssistResponse, ZedLocalModelStatus, ZedToolPermissionMode,
};
pub use zeroclaw::{
    ZeroClawAutonomyLevel, ZeroClawChannel, ZeroClawContextItem, ZeroClawExecutionTarget,
    ZeroClawFlowAdapter, ZeroClawFollowUpRequest, ZeroClawLocalModelStatus, ZeroClawSurface,
    ZeroClawTaskCandidate, ZeroClawTaskRequest, ZeroClawTaskResponse, ZeroClawToolClass,
    ZeroClawToolPolicy,
};
