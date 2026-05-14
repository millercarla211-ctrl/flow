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
    BrowserExecutionPlan, BrowserExecutionRequest, BrowserExtensionMessage, BrowserHostFlavor,
    BrowserInferenceInvocation, BrowserInferenceRequest, BrowserPackFile, BrowserPackManifest,
    BrowserPackResolution, BrowserPackSupport, BrowserStorageBackend, BrowserTask,
    BrowserTokenStreamPlan, BrowserUiSurface, BrowserWorkerKind, FlowBrowserEngine,
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
    FridayConnectorAuthState, FridayConnectorKind, FridayConnectorRecord, FridayFeatureStatus,
    FridayGenerationSummary, FridayLocalCheckKind, FridayLocalCheckStatus,
    FridayBrowserVerificationReport, FridayBrowserVerificationTarget, FridayDeployGate,
    FridayLocalExecutionCheck, FridayLocalExecutionReport, FridayMemoryRecord, FridayMemoryState,
    FridayMultimodalModelFile, FridayMultimodalRuntimeRecord, FridayMultimodalSurface,
    FridayOcrSmokeReport, FridayOcrSmokeStatus, FridayPermissionFinding, FridayPermissionScope,
    FridayPreviewRunner,
    FridayProductPlan, FridayProjectFile, FridayProjectRecord, FridayResearchEventKind,
    FridayResearchExportManifest,
    FridayResearchReport, FridayResearchRunEvent, FridayResearchStage, FridayResearchStageKind,
    FridayResearchStageStatus, FridayResearchWorkflow, FridayRuntimeFinding,
    FridayRuntimeRecordStatus, FridayRuntimeSurfaceManifest, FridayRuntimeSurfaceSnapshot,
    FridayRuntimeSurfaceStore, FridaySearchPolicy, FridaySourceGroup, FridaySynthesizedAnswer,
    FridayUiDataBinding, FridayUiIntegrationPlan, FridayUiIntegrationStatus,
    FridayUiRouteContract, FridayUiSourceControl, FridayUiState, FridayUiStateKind,
    FridayUiStateTone,
    FridayVoiceRuntimeRecord, FridayWorkspaceArea, FridayWorkspaceManifest,
    FridayWorkspaceSnapshot, FridayWorkspaceStore, FridayWorkspaceView, MetasearchExecutionMode,
    FridayVerificationStatus, FridayVlmContractReport, FridayVlmContractStatus,
    MetasearchExecutionTarget,
    default_friday_browser_verification_report, default_friday_local_execution_checks,
    default_friday_product_plan, default_friday_ui_integration_plan, friday_answer_search_plan,
    friday_research_search_plan, run_friday_ocr_smoke, run_friday_vlm_contract,
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
