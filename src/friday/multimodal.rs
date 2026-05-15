use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayArtifactCheckpoint, FridayArtifactKind, FridayArtifactRecord, FridayCheckpointReason,
    FridayPreviewRunner, FridayWorkspaceArea,
};
use crate::models::GlmOcr;

const OCR_MODEL_KEY: &str = "glm-ocr-q4km";
const VLM_MODEL_KEY: &str = "gemma4-e4b-frontend-q4km";
const VLM_MODEL_PATH: &str = "models/llm/gemma-4-E4B-it.Q4_K_M.gguf";
const VLM_MMPROJ_PATH: &str = "models/llm/gemma-4-E4B-it.BF16-mmproj.gguf";
const DEFAULT_VLM_PROMPT: &str =
    "Describe the screenshot, visible text, primary UI regions, and likely user intent.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayMultimodalRequestKind {
    Ocr,
    Vlm,
    Audio,
    Image,
    Video,
}

impl FridayMultimodalRequestKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "ocr" | "image-ocr" => Some(Self::Ocr),
            "vlm" | "screenshot" | "vision" => Some(Self::Vlm),
            "audio" | "speech" | "stt" | "tts" => Some(Self::Audio),
            "image" | "image-generation" => Some(Self::Image),
            "video" | "video-understanding" => Some(Self::Video),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Ocr => "ocr",
            Self::Vlm => "vlm",
            Self::Audio => "audio",
            Self::Image => "image",
            Self::Video => "video",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayMultimodalRouteStatus {
    Ready,
    NeedsModel,
    Planned,
}

impl FridayMultimodalRouteStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NeedsModel => "needs-model",
            Self::Planned => "planned",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMultimodalModelRoute {
    pub model_key: String,
    pub purpose: String,
    pub local_only: bool,
    pub resident: bool,
    pub command: String,
    pub files: Vec<FridayMultimodalModelFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMultimodalRouteDecision {
    pub request_kind: FridayMultimodalRequestKind,
    pub status: FridayMultimodalRouteStatus,
    pub local_first: bool,
    pub remote_allowed: bool,
    pub selected: Option<FridayMultimodalModelRoute>,
    pub fallbacks: Vec<FridayMultimodalModelRoute>,
    pub rationale: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMultimodalArtifactMetadata {
    pub artifact_id: String,
    pub request_kind: FridayMultimodalRequestKind,
    pub source_uri: String,
    pub source_mime: Option<String>,
    pub model_key: String,
    pub prompt: Option<String>,
    pub output_format: String,
    pub local_only: bool,
    pub model_execution: bool,
    pub duration_ms: u128,
    pub confidence_percent: Option<u8>,
    pub created_at_unix_ms: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayMultimodalDiagnosticStatus {
    Ready,
    Warning,
    Planned,
}

impl FridayMultimodalDiagnosticStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Warning => "warning",
            Self::Planned => "planned",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMultimodalDiagnosticItem {
    pub id: String,
    pub title: String,
    pub status: FridayMultimodalDiagnosticStatus,
    pub command: String,
    pub artifact_output: String,
    pub local_only: bool,
    pub loads_model: bool,
    pub evidence: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMultimodalUiDiagnostics {
    pub generated_at_unix_ms: u128,
    pub area: FridayWorkspaceArea,
    pub route: String,
    pub score_out_of_100: u8,
    pub primary_command: String,
    pub items: Vec<FridayMultimodalDiagnosticItem>,
    pub findings: Vec<String>,
}

impl FridayMultimodalUiDiagnostics {
    pub fn ready_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| item.status == FridayMultimodalDiagnosticStatus::Ready)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| item.status == FridayMultimodalDiagnosticStatus::Warning)
            .count()
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

impl FridayMultimodalArtifactMetadata {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

impl FridayMultimodalRouteDecision {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayOcrSmokeStatus {
    Passed,
    Failed,
}

impl FridayOcrSmokeStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayOcrSmokeReport {
    pub generated_at_unix_ms: u128,
    pub status: FridayOcrSmokeStatus,
    pub output_dir: String,
    pub image_source: String,
    pub model_execution: bool,
    pub model_paths: Vec<String>,
    pub artifact_json: String,
    pub checkpoint_json: String,
    pub metadata_json: String,
    pub output_markdown: String,
    pub report_json: String,
    pub artifact: FridayArtifactRecord,
    pub checkpoint: FridayArtifactCheckpoint,
    pub metadata: FridayMultimodalArtifactMetadata,
    pub extracted_text_preview: String,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayVlmContractStatus {
    Ready,
    Warning,
}

impl FridayVlmContractStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMultimodalModelFile {
    pub path: String,
    pub purpose: String,
    pub present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayVlmContractReport {
    pub generated_at_unix_ms: u128,
    pub status: FridayVlmContractStatus,
    pub output_dir: String,
    pub screenshot_source: String,
    pub prompt: String,
    pub model_key: String,
    pub model_execution: bool,
    pub model_files: Vec<FridayMultimodalModelFile>,
    pub artifact_json: String,
    pub checkpoint_json: String,
    pub metadata_json: String,
    pub output_markdown: String,
    pub report_json: String,
    pub artifact: FridayArtifactRecord,
    pub checkpoint: FridayArtifactCheckpoint,
    pub metadata: FridayMultimodalArtifactMetadata,
    pub summary_preview: String,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayScreenshotSourceRecord {
    pub path: String,
    pub bytes: u64,
    pub mime: String,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayScreenshotVlmHandoffReport {
    pub generated_at_unix_ms: u128,
    pub output_dir: String,
    pub source_json: String,
    pub source: FridayScreenshotSourceRecord,
    pub vlm_report: FridayVlmContractReport,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayMediaAffordanceStatus {
    Ready,
    NeedsInstaller,
    Planned,
}

impl FridayMediaAffordanceStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NeedsInstaller => "needs-installer",
            Self::Planned => "planned",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMediaAffordance {
    pub request_kind: FridayMultimodalRequestKind,
    pub model_key: String,
    pub label: String,
    pub repo_id: String,
    pub status: FridayMediaAffordanceStatus,
    pub install_command: String,
    pub run_command: String,
    pub local_only: bool,
    pub resident: bool,
    pub notes: Vec<String>,
}

impl FridayOcrSmokeReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

impl FridayVlmContractReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

impl FridayScreenshotVlmHandoffReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_media_affordances() -> Vec<FridayMediaAffordance> {
    vec![
        FridayMediaAffordance {
            request_kind: FridayMultimodalRequestKind::Image,
            model_key: "sd-turbo".to_string(),
            label: "SD-Turbo local image generation".to_string(),
            repo_id: "stabilityai/sd-turbo".to_string(),
            status: FridayMediaAffordanceStatus::NeedsInstaller,
            install_command: "flow --models image".to_string(),
            run_command: "flow --plan image sd-turbo".to_string(),
            local_only: true,
            resident: false,
            notes: vec![
                "Catalog entry exists, but no built-in downloader is registered yet.".to_string(),
                "Keep this as an explicit install/run affordance before exposing image generation in the UI.".to_string(),
            ],
        },
        FridayMediaAffordance {
            request_kind: FridayMultimodalRequestKind::Video,
            model_key: "wan2.1-14b".to_string(),
            label: "Wan2.1 high-end local video generation".to_string(),
            repo_id: "Wan-AI/Wan2.1-FLF2V-14B-720P".to_string(),
            status: FridayMediaAffordanceStatus::Planned,
            install_command: "flow --models video".to_string(),
            run_command: "flow --plan video wan2.1-14b".to_string(),
            local_only: true,
            resident: false,
            notes: vec![
                "Marked high-end because this model is not appropriate for the default low-resource idle path.".to_string(),
                "UI should present this as an explicit advanced workflow, not a default feature.".to_string(),
            ],
        },
    ]
}

pub fn friday_multimodal_route(
    request_kind: FridayMultimodalRequestKind,
    remote_allowed: bool,
) -> FridayMultimodalRouteDecision {
    match request_kind {
        FridayMultimodalRequestKind::Ocr => route_decision(
            request_kind,
            remote_allowed,
            ocr_route(),
            Vec::new(),
            vec![
                "OCR uses GLM-OCR only when explicitly invoked.".to_string(),
                "Readiness checks can verify files without loading the model.".to_string(),
            ],
            "Use `flow --friday-ocr-smoke <dir> <image> --execute` for a bounded local OCR run.",
        ),
        FridayMultimodalRequestKind::Vlm => route_decision(
            request_kind,
            remote_allowed,
            vlm_route(),
            Vec::new(),
            vec![
                "Screenshot understanding is separated from OCR so prompts, artifacts, and model files stay explicit.".to_string(),
                "The VLM contract writes metadata first and avoids resident model loading.".to_string(),
            ],
            "Use `flow --friday-vlm-contract <dir> <screenshot> <prompt>` before enabling real VLM execution.",
        ),
        FridayMultimodalRequestKind::Audio => route_decision(
            request_kind,
            remote_allowed,
            audio_route(),
            vec![tts_route()],
            vec![
                "Audio stays local-first through Parakeet STT and Kokoro TTS.".to_string(),
                "Models should warm on demand, not remain resident in the idle overlay.".to_string(),
            ],
            "Keep STT/TTS local by default and expose remote audio providers only behind explicit settings.",
        ),
        FridayMultimodalRequestKind::Image => planned_decision(
            request_kind,
            remote_allowed,
            "Local image generation is not resident yet; route to artifact planning and require explicit model installation before execution.",
        ),
        FridayMultimodalRequestKind::Video => planned_decision(
            request_kind,
            remote_allowed,
            "Video understanding is planned as frame sampling plus VLM/OCR artifacts, not a resident video model.",
        ),
    }
}

pub fn friday_multimodal_ui_diagnostics() -> FridayMultimodalUiDiagnostics {
    let ocr_route = friday_multimodal_route(FridayMultimodalRequestKind::Ocr, false);
    let vlm_route = friday_multimodal_route(FridayMultimodalRequestKind::Vlm, false);
    let audio_route = friday_multimodal_route(FridayMultimodalRequestKind::Audio, false);
    let image_route = friday_multimodal_route(FridayMultimodalRequestKind::Image, false);
    let video_route = friday_multimodal_route(FridayMultimodalRequestKind::Video, false);

    let items = vec![
        diagnostic_item(
            "ocr-fixture-smoke",
            "OCR fixture artifact path",
            FridayMultimodalDiagnosticStatus::Ready,
            "flow --friday-ocr-smoke tmp/friday-ocr-smoke",
            "ocr-smoke-output.md + artifact/checkpoint/metadata/report JSON",
            false,
            vec![
                "Fixture mode proves UI diagnostics can write artifacts without loading GLM-OCR."
                    .to_string(),
                "The metadata sidecar links source, request kind, model key, timing, and local-only policy."
                    .to_string(),
            ],
            "Surface the latest OCR smoke report in the Multimodal and OCR pages.",
        ),
        diagnostic_item(
            "ocr-model-run",
            "OCR local model execution",
            route_status_to_diagnostic(ocr_route.status),
            "flow --friday-ocr-smoke <dir> <image> --execute",
            "OCR markdown, artifact, checkpoint, metadata, and report bundle",
            true,
            route_evidence(&ocr_route),
            "Run explicit OCR only after the user supplies an image path.",
        ),
        diagnostic_item(
            "vlm-contract",
            "VLM screenshot contract",
            route_status_to_diagnostic(vlm_route.status),
            "flow --friday-vlm-contract <dir> <screenshot> <prompt>",
            "VLM contract markdown, artifact, checkpoint, metadata, and report bundle",
            false,
            route_evidence(&vlm_route),
            "Connect the next slice to a real screenshot capture path.",
        ),
        diagnostic_item(
            "media-routing",
            "Image, audio, and video route policy",
            FridayMultimodalDiagnosticStatus::Ready,
            "flow --friday-multimodal-route <ocr|vlm|audio|image|video>",
            "local-first route decision JSON",
            false,
            vec![
                format!("audio={}", audio_route.status.label()),
                format!("image={}", image_route.status.label()),
                format!("video={}", video_route.status.label()),
            ],
            "Add install/run affordances for planned image and video paths.",
        ),
        diagnostic_item(
            "artifact-metadata",
            "Multimodal artifact metadata",
            FridayMultimodalDiagnosticStatus::Ready,
            "flow --friday-ocr-smoke <dir> or flow --friday-vlm-contract <dir>",
            "FridayMultimodalArtifactMetadata JSON sidecar",
            false,
            vec![
                "Metadata includes source URI, request kind, model key, prompt, output format, local-only flag, execution flag, timing, and confidence."
                    .to_string(),
            ],
            "Persist metadata inside the durable Friday artifact store.",
        ),
    ];
    let score_out_of_100 = diagnostic_score(&items);
    let findings = items
        .iter()
        .filter(|item| item.status != FridayMultimodalDiagnosticStatus::Ready)
        .map(|item| format!("{}: {}", item.id, item.next_action))
        .collect();

    FridayMultimodalUiDiagnostics {
        generated_at_unix_ms: unix_ms(),
        area: FridayWorkspaceArea::Multimodal,
        route: FridayWorkspaceArea::Multimodal.route().to_string(),
        score_out_of_100,
        primary_command: "flow --friday-multimodal-diagnostics".to_string(),
        items,
        findings,
    }
}

pub fn run_friday_ocr_smoke(
    output_dir: impl AsRef<Path>,
    image_path: Option<&str>,
    execute_model: bool,
) -> Result<FridayOcrSmokeReport> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create OCR smoke dir {}", output_dir.display()))?;

    let generated_at_unix_ms = unix_ms();
    let image_source = image_path
        .map(str::to_string)
        .unwrap_or_else(|| "fixture://friday-ocr-smoke".to_string());
    let model_paths = GlmOcr::resolved_model_paths()
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    let extraction_started = Instant::now();
    let extracted_text = if execute_model {
        let image_path = image_path.context("OCR smoke execution requires an image path")?;
        GlmOcr::new()?.ocr_image(image_path)?
    } else {
        "Friday OCR smoke fixture\nLocal multimodal artifacts stay on this machine.\nGenerated by the bounded smoke path."
            .to_string()
    };
    let duration_ms = extraction_started.elapsed().as_millis();

    let status = if extracted_text.trim().is_empty() {
        FridayOcrSmokeStatus::Failed
    } else {
        FridayOcrSmokeStatus::Passed
    };
    let output_markdown = output_dir.join("ocr-smoke-output.md");
    let artifact_json = output_dir.join("ocr-smoke-artifact.json");
    let checkpoint_json = output_dir.join("ocr-smoke-checkpoint.json");
    let metadata_json = output_dir.join("ocr-smoke-metadata.json");
    let report_json = output_dir.join("ocr-smoke-report.json");
    let artifact_id = "ocr-smoke-output".to_string();
    let checkpoint_id = "ocr-smoke-output-cp1".to_string();

    let markdown = format!(
        "# Friday OCR Smoke Output\n\n- Source: `{}`\n- Model execution: `{}`\n- Status: `{}`\n\n## Extracted Text\n\n{}\n",
        image_source,
        execute_model,
        status.label(),
        extracted_text.trim()
    );
    fs::write(&output_markdown, markdown)
        .with_context(|| format!("failed to write {}", output_markdown.display()))?;

    let artifact = FridayArtifactRecord {
        id: artifact_id.clone(),
        project_id: "friday-local".to_string(),
        kind: FridayArtifactKind::Markdown,
        title: "OCR smoke output".to_string(),
        path: output_markdown.to_string_lossy().into_owned(),
        language: Some("markdown".to_string()),
        preview_runner: FridayPreviewRunner::Markdown,
        current_checkpoint_id: checkpoint_id.clone(),
        created_at_unix_ms: generated_at_unix_ms,
        updated_at_unix_ms: generated_at_unix_ms,
    };
    let checkpoint = FridayArtifactCheckpoint {
        id: checkpoint_id,
        artifact_id,
        reason: if execute_model {
            FridayCheckpointReason::Generated
        } else {
            FridayCheckpointReason::Created
        },
        summary: if execute_model {
            "OCR smoke artifact generated from a local image.".to_string()
        } else {
            "OCR smoke artifact generated from the built-in fixture.".to_string()
        },
        content_hash: stable_hash(&extracted_text),
        created_at_unix_ms: generated_at_unix_ms,
    };
    let metadata = FridayMultimodalArtifactMetadata {
        artifact_id: artifact.id.clone(),
        request_kind: FridayMultimodalRequestKind::Ocr,
        source_uri: image_source.clone(),
        source_mime: None,
        model_key: OCR_MODEL_KEY.to_string(),
        prompt: None,
        output_format: "markdown".to_string(),
        local_only: true,
        model_execution: execute_model,
        duration_ms,
        confidence_percent: None,
        created_at_unix_ms: generated_at_unix_ms,
    };

    write_json(&artifact_json, &artifact)?;
    write_json(&checkpoint_json, &checkpoint)?;
    write_json(&metadata_json, &metadata)?;

    let findings = if status == FridayOcrSmokeStatus::Passed {
        Vec::new()
    } else {
        vec!["OCR smoke produced no extracted text.".to_string()]
    };
    let report = FridayOcrSmokeReport {
        generated_at_unix_ms,
        status,
        output_dir: output_dir.to_string_lossy().into_owned(),
        image_source,
        model_execution: execute_model,
        model_paths,
        artifact_json: artifact_json.to_string_lossy().into_owned(),
        checkpoint_json: checkpoint_json.to_string_lossy().into_owned(),
        metadata_json: metadata_json.to_string_lossy().into_owned(),
        output_markdown: output_markdown.to_string_lossy().into_owned(),
        report_json: report_json.to_string_lossy().into_owned(),
        artifact,
        checkpoint,
        metadata,
        extracted_text_preview: extracted_text
            .split_whitespace()
            .take(32)
            .collect::<Vec<_>>()
            .join(" "),
        findings,
    };
    write_json(&report_json, &report)?;

    Ok(report)
}

fn diagnostic_item(
    id: &str,
    title: &str,
    status: FridayMultimodalDiagnosticStatus,
    command: &str,
    artifact_output: &str,
    loads_model: bool,
    evidence: Vec<String>,
    next_action: &str,
) -> FridayMultimodalDiagnosticItem {
    FridayMultimodalDiagnosticItem {
        id: id.to_string(),
        title: title.to_string(),
        status,
        command: command.to_string(),
        artifact_output: artifact_output.to_string(),
        local_only: true,
        loads_model,
        evidence,
        next_action: next_action.to_string(),
    }
}

fn route_status_to_diagnostic(
    status: FridayMultimodalRouteStatus,
) -> FridayMultimodalDiagnosticStatus {
    match status {
        FridayMultimodalRouteStatus::Ready => FridayMultimodalDiagnosticStatus::Ready,
        FridayMultimodalRouteStatus::NeedsModel => FridayMultimodalDiagnosticStatus::Warning,
        FridayMultimodalRouteStatus::Planned => FridayMultimodalDiagnosticStatus::Planned,
    }
}

fn route_evidence(route: &FridayMultimodalRouteDecision) -> Vec<String> {
    let selected = route.selected.as_ref();
    let mut evidence = vec![
        format!("status={}", route.status.label()),
        format!("remote_allowed={}", route.remote_allowed),
    ];
    if let Some(selected) = selected {
        evidence.push(format!("model={}", selected.model_key));
        evidence.extend(selected.files.iter().map(|file| {
            format!(
                "{}={} ({})",
                file.purpose,
                if file.present { "present" } else { "missing" },
                file.path
            )
        }));
    }
    evidence
}

fn diagnostic_score(items: &[FridayMultimodalDiagnosticItem]) -> u8 {
    if items.is_empty() {
        return 0;
    }

    let earned = items
        .iter()
        .map(|item| match item.status {
            FridayMultimodalDiagnosticStatus::Ready => 1.0,
            FridayMultimodalDiagnosticStatus::Warning => 0.5,
            FridayMultimodalDiagnosticStatus::Planned => 0.0,
        })
        .sum::<f32>();

    ((earned / items.len() as f32) * 100.0).round() as u8
}

fn route_decision(
    request_kind: FridayMultimodalRequestKind,
    remote_allowed: bool,
    selected: FridayMultimodalModelRoute,
    fallbacks: Vec<FridayMultimodalModelRoute>,
    rationale: Vec<String>,
    next_action: &str,
) -> FridayMultimodalRouteDecision {
    let all_files_ready = selected.files.iter().all(|file| file.present)
        && fallbacks
            .iter()
            .flat_map(|route| &route.files)
            .all(|file| file.present);
    FridayMultimodalRouteDecision {
        request_kind,
        status: if all_files_ready {
            FridayMultimodalRouteStatus::Ready
        } else {
            FridayMultimodalRouteStatus::NeedsModel
        },
        local_first: true,
        remote_allowed,
        selected: Some(selected),
        fallbacks,
        rationale,
        next_action: next_action.to_string(),
    }
}

fn planned_decision(
    request_kind: FridayMultimodalRequestKind,
    remote_allowed: bool,
    next_action: &str,
) -> FridayMultimodalRouteDecision {
    FridayMultimodalRouteDecision {
        request_kind,
        status: FridayMultimodalRouteStatus::Planned,
        local_first: true,
        remote_allowed,
        selected: None,
        fallbacks: Vec::new(),
        rationale: vec![
            "Keep local-only mode as the default.".to_string(),
            "Do not silently call cloud providers for multimodal work.".to_string(),
        ],
        next_action: next_action.to_string(),
    }
}

fn ocr_route() -> FridayMultimodalModelRoute {
    FridayMultimodalModelRoute {
        model_key: OCR_MODEL_KEY.to_string(),
        purpose: "image OCR".to_string(),
        local_only: true,
        resident: false,
        command: "flow --friday-ocr-smoke <dir> <image> --execute".to_string(),
        files: GlmOcr::resolved_model_paths()
            .iter()
            .enumerate()
            .map(|(index, path)| FridayMultimodalModelFile {
                path: path.to_string_lossy().into_owned(),
                purpose: if index == 0 {
                    "ocr model"
                } else {
                    "ocr projector"
                }
                .to_string(),
                present: path.exists(),
            })
            .collect(),
    }
}

fn vlm_route() -> FridayMultimodalModelRoute {
    FridayMultimodalModelRoute {
        model_key: VLM_MODEL_KEY.to_string(),
        purpose: "screenshot understanding".to_string(),
        local_only: true,
        resident: false,
        command: "flow --friday-vlm-contract <dir> <screenshot> <prompt>".to_string(),
        files: vlm_model_files(),
    }
}

fn audio_route() -> FridayMultimodalModelRoute {
    FridayMultimodalModelRoute {
        model_key: "parakeet-unified-en-0.6b-int8".to_string(),
        purpose: "speech-to-text".to_string(),
        local_only: true,
        resident: false,
        command: "flow --dictate".to_string(),
        files: vec![FridayMultimodalModelFile {
            path: "models/stt/parakeet-unified-en-0.6b-int8/encoder.int8.onnx".to_string(),
            purpose: "stt encoder".to_string(),
            present: Path::new("models/stt/parakeet-unified-en-0.6b-int8/encoder.int8.onnx")
                .exists(),
        }],
    }
}

fn tts_route() -> FridayMultimodalModelRoute {
    FridayMultimodalModelRoute {
        model_key: "kokoro-int8".to_string(),
        purpose: "text-to-speech".to_string(),
        local_only: true,
        resident: false,
        command: "flow --speak <text>".to_string(),
        files: vec![FridayMultimodalModelFile {
            path: "models/tts/kokoro.onnx".to_string(),
            purpose: "tts model".to_string(),
            present: Path::new("models/tts/kokoro.onnx").exists(),
        }],
    }
}

pub fn run_friday_vlm_contract(
    output_dir: impl AsRef<Path>,
    screenshot_path: Option<&str>,
    prompt: Option<&str>,
) -> Result<FridayVlmContractReport> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create VLM contract dir {}", output_dir.display()))?;

    let generated_at_unix_ms = unix_ms();
    let screenshot_source = screenshot_path
        .map(str::to_string)
        .unwrap_or_else(|| "fixture://friday-vlm-screenshot".to_string());
    let prompt = prompt
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_VLM_PROMPT)
        .to_string();
    let model_files = vlm_model_files();
    let missing_model_files = model_files
        .iter()
        .filter(|file| !file.present)
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let status = if missing_model_files.is_empty() {
        FridayVlmContractStatus::Ready
    } else {
        FridayVlmContractStatus::Warning
    };

    let output_markdown = output_dir.join("vlm-screenshot-contract.md");
    let artifact_json = output_dir.join("vlm-screenshot-artifact.json");
    let checkpoint_json = output_dir.join("vlm-screenshot-checkpoint.json");
    let metadata_json = output_dir.join("vlm-screenshot-metadata.json");
    let report_json = output_dir.join("vlm-screenshot-report.json");
    let artifact_id = "vlm-screenshot-contract".to_string();
    let checkpoint_id = "vlm-screenshot-contract-cp1".to_string();
    let model_file_lines = model_files
        .iter()
        .map(|file| {
            format!(
                "- `{}` ({}) present={}",
                file.path, file.purpose, file.present
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let summary = format!(
        "Friday VLM contract for `{}` using `{}`. The runtime must keep execution local, read one screenshot, emit one artifact, and avoid loading the VLM until the user explicitly runs screenshot understanding.",
        screenshot_source, VLM_MODEL_KEY
    );

    let markdown = format!(
        "# Friday VLM Screenshot Contract\n\n- Source: `{}`\n- Prompt: `{}`\n- Model: `{}`\n- Status: `{}`\n- Model execution: `false`\n\n## Model Files\n\n{}\n\n## Boundary\n\n{}\n",
        screenshot_source,
        prompt,
        VLM_MODEL_KEY,
        status.label(),
        model_file_lines,
        summary
    );
    fs::write(&output_markdown, markdown)
        .with_context(|| format!("failed to write {}", output_markdown.display()))?;

    let artifact = FridayArtifactRecord {
        id: artifact_id.clone(),
        project_id: "friday-local".to_string(),
        kind: FridayArtifactKind::Markdown,
        title: "VLM screenshot contract".to_string(),
        path: output_markdown.to_string_lossy().into_owned(),
        language: Some("markdown".to_string()),
        preview_runner: FridayPreviewRunner::Markdown,
        current_checkpoint_id: checkpoint_id.clone(),
        created_at_unix_ms: generated_at_unix_ms,
        updated_at_unix_ms: generated_at_unix_ms,
    };
    let checkpoint = FridayArtifactCheckpoint {
        id: checkpoint_id,
        artifact_id,
        reason: FridayCheckpointReason::Created,
        summary:
            "VLM screenshot contract created with explicit local model and artifact boundaries."
                .to_string(),
        content_hash: stable_hash(&summary),
        created_at_unix_ms: generated_at_unix_ms,
    };
    let metadata = FridayMultimodalArtifactMetadata {
        artifact_id: artifact.id.clone(),
        request_kind: FridayMultimodalRequestKind::Vlm,
        source_uri: screenshot_source.clone(),
        source_mime: None,
        model_key: VLM_MODEL_KEY.to_string(),
        prompt: Some(prompt.clone()),
        output_format: "markdown".to_string(),
        local_only: true,
        model_execution: false,
        duration_ms: 0,
        confidence_percent: None,
        created_at_unix_ms: generated_at_unix_ms,
    };

    write_json(&artifact_json, &artifact)?;
    write_json(&checkpoint_json, &checkpoint)?;
    write_json(&metadata_json, &metadata)?;

    let findings = missing_model_files
        .iter()
        .map(|path| format!("VLM model file is not present yet: {path}"))
        .collect::<Vec<_>>();
    let report = FridayVlmContractReport {
        generated_at_unix_ms,
        status,
        output_dir: output_dir.to_string_lossy().into_owned(),
        screenshot_source,
        prompt,
        model_key: VLM_MODEL_KEY.to_string(),
        model_execution: false,
        model_files,
        artifact_json: artifact_json.to_string_lossy().into_owned(),
        checkpoint_json: checkpoint_json.to_string_lossy().into_owned(),
        metadata_json: metadata_json.to_string_lossy().into_owned(),
        output_markdown: output_markdown.to_string_lossy().into_owned(),
        report_json: report_json.to_string_lossy().into_owned(),
        artifact,
        checkpoint,
        metadata,
        summary_preview: summary,
        findings,
    };
    write_json(&report_json, &report)?;

    Ok(report)
}

pub fn run_friday_screenshot_vlm_handoff(
    output_dir: impl AsRef<Path>,
    screenshot_path: impl AsRef<Path>,
    prompt: Option<&str>,
) -> Result<FridayScreenshotVlmHandoffReport> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "failed to create screenshot VLM handoff dir {}",
            output_dir.display()
        )
    })?;

    let screenshot_path = screenshot_path.as_ref();
    let metadata = fs::metadata(screenshot_path)
        .with_context(|| format!("failed to read screenshot {}", screenshot_path.display()))?;
    if !metadata.is_file() {
        anyhow::bail!(
            "screenshot path is not a file: {}",
            screenshot_path.display()
        );
    }

    let mime = screenshot_mime(screenshot_path)
        .with_context(|| format!("unsupported screenshot type {}", screenshot_path.display()))?;
    let screenshot = screenshot_path.to_string_lossy().into_owned();
    let vlm_report = run_friday_vlm_contract(output_dir, Some(&screenshot), prompt)?;
    let source_json = output_dir.join("vlm-screenshot-source.json");
    let source = FridayScreenshotSourceRecord {
        path: screenshot,
        bytes: metadata.len(),
        mime,
        accepted: true,
    };
    write_json(&source_json, &source)?;

    Ok(FridayScreenshotVlmHandoffReport {
        generated_at_unix_ms: unix_ms(),
        output_dir: output_dir.to_string_lossy().into_owned(),
        source_json: source_json.to_string_lossy().into_owned(),
        source,
        findings: vlm_report.findings.clone(),
        vlm_report,
    })
}

fn screenshot_mime(path: &Path) -> Result<String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .context("screenshot file needs an extension")?;
    let mime = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => anyhow::bail!("unsupported screenshot extension: {extension}"),
    };
    Ok(mime.to_string())
}

fn vlm_model_files() -> Vec<FridayMultimodalModelFile> {
    [
        (VLM_MODEL_PATH, "vision-language model"),
        (VLM_MMPROJ_PATH, "vision projector"),
    ]
    .into_iter()
    .map(|(path, purpose)| FridayMultimodalModelFile {
        path: path.to_string(),
        purpose: purpose.to_string(),
        present: Path::new(path).exists(),
    })
    .collect()
}

fn stable_hash(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn ocr_smoke_writes_artifact_checkpoint_and_report() {
        let root = std::env::temp_dir().join(format!(
            "friday-ocr-smoke-{}-{}",
            std::process::id(),
            unix_ms()
        ));
        let _ = fs::remove_dir_all(&root);

        let report = run_friday_ocr_smoke(&root, None, false).unwrap();
        assert_eq!(report.status, FridayOcrSmokeStatus::Passed);
        assert!(!report.model_execution);
        assert!(PathBuf::from(&report.artifact_json).exists());
        assert!(PathBuf::from(&report.checkpoint_json).exists());
        assert!(PathBuf::from(&report.metadata_json).exists());
        assert!(PathBuf::from(&report.output_markdown).exists());
        assert!(PathBuf::from(&report.report_json).exists());
        assert_eq!(report.artifact.id, "ocr-smoke-output");
        assert_eq!(report.checkpoint.artifact_id, report.artifact.id);
        assert_eq!(report.metadata.artifact_id, report.artifact.id);
        assert_eq!(
            report.metadata.request_kind,
            FridayMultimodalRequestKind::Ocr
        );
        assert_eq!(report.artifact.current_checkpoint_id, report.checkpoint.id);
        assert_eq!(
            report.artifact.preview_runner,
            FridayPreviewRunner::Markdown
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn vlm_contract_writes_local_artifact_boundary() {
        let root = std::env::temp_dir().join(format!(
            "friday-vlm-contract-{}-{}",
            std::process::id(),
            unix_ms()
        ));
        let _ = fs::remove_dir_all(&root);

        let report = run_friday_vlm_contract(&root, None, Some("read visible UI text")).unwrap();
        assert!(!report.model_execution);
        assert_eq!(report.model_key, VLM_MODEL_KEY);
        assert_eq!(report.model_files.len(), 2);
        assert!(PathBuf::from(&report.artifact_json).exists());
        assert!(PathBuf::from(&report.checkpoint_json).exists());
        assert!(PathBuf::from(&report.metadata_json).exists());
        assert!(PathBuf::from(&report.output_markdown).exists());
        assert!(PathBuf::from(&report.report_json).exists());
        assert_eq!(report.artifact.current_checkpoint_id, report.checkpoint.id);
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
    fn multimodal_metadata_round_trips() {
        let metadata = FridayMultimodalArtifactMetadata {
            artifact_id: "artifact-1".to_string(),
            request_kind: FridayMultimodalRequestKind::Video,
            source_uri: "file:///tmp/frame.png".to_string(),
            source_mime: Some("image/png".to_string()),
            model_key: VLM_MODEL_KEY.to_string(),
            prompt: Some("summarize frame".to_string()),
            output_format: "markdown".to_string(),
            local_only: true,
            model_execution: false,
            duration_ms: 12,
            confidence_percent: Some(80),
            created_at_unix_ms: 42,
        };

        let restored: FridayMultimodalArtifactMetadata =
            serde_json::from_str(&metadata.to_pretty_json().unwrap()).unwrap();
        assert_eq!(restored, metadata);
    }

    #[test]
    fn multimodal_route_keeps_cloud_explicit() {
        let route = friday_multimodal_route(FridayMultimodalRequestKind::Vlm, false);
        assert!(route.local_first);
        assert!(!route.remote_allowed);
        assert_eq!(route.selected.unwrap().model_key, VLM_MODEL_KEY);

        let image = friday_multimodal_route(FridayMultimodalRequestKind::Image, false);
        assert_eq!(image.status, FridayMultimodalRouteStatus::Planned);
        assert!(image.selected.is_none());
    }

    #[test]
    fn multimodal_ui_diagnostics_connect_report_commands() {
        let diagnostics = friday_multimodal_ui_diagnostics();
        assert_eq!(diagnostics.area, FridayWorkspaceArea::Multimodal);
        assert!(diagnostics.score_out_of_100 >= 60);
        assert!(
            diagnostics
                .items
                .iter()
                .any(|item| item.command.contains("--friday-ocr-smoke"))
        );
        assert!(
            diagnostics
                .items
                .iter()
                .any(|item| item.artifact_output.contains("metadata"))
        );
    }

    #[test]
    fn screenshot_vlm_handoff_validates_local_image_path() {
        let root = std::env::temp_dir().join(format!(
            "friday-screenshot-vlm-{}-{}",
            std::process::id(),
            unix_ms()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let screenshot = root.join("screen.png");
        fs::write(&screenshot, b"not-a-real-png-but-valid-handoff-fixture").unwrap();
        let out = root.join("out");

        let report =
            run_friday_screenshot_vlm_handoff(&out, &screenshot, Some("describe visible text"))
                .unwrap();
        assert_eq!(report.source.mime, "image/png");
        assert!(report.source.accepted);
        assert!(PathBuf::from(&report.source_json).exists());
        assert_eq!(
            report.vlm_report.screenshot_source,
            screenshot.to_string_lossy()
        );
        assert!(PathBuf::from(&report.vlm_report.report_json).exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn media_affordances_expose_explicit_install_and_run_commands() {
        let affordances = friday_media_affordances();
        assert!(affordances.iter().any(|item| item.request_kind
            == FridayMultimodalRequestKind::Image
            && item.install_command.contains("--models image")
            && item.run_command.contains("--plan image")));
        assert!(affordances.iter().any(|item| item.request_kind
            == FridayMultimodalRequestKind::Video
            && item.local_only
            && !item.resident));
    }
}
