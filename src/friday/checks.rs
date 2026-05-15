use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::{
    FridayArtifactStore, FridayPreviewRunner, FridayRuntimeSurfaceStore, FridayWorkspaceArea,
    friday_answer_search_plan,
};
use crate::models::{GlmOcr, KokoroTTS, LocalSttEngine};
use crate::runtime::{Modality, default_model_catalog};
use crate::search::{MetasearchServerConfig, metasearch_categories};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayLocalCheckKind {
    ModelArtifacts,
    RuntimeStore,
    MetasearchRequest,
    ArtifactPreview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayLocalCheckStatus {
    Passed,
    Warning,
    Failed,
    Skipped,
}

impl FridayLocalCheckStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayLocalExecutionCheck {
    pub id: String,
    pub area: FridayWorkspaceArea,
    pub kind: FridayLocalCheckKind,
    pub title: String,
    pub command: String,
    pub status: FridayLocalCheckStatus,
    pub local_only: bool,
    pub loads_model: bool,
    pub touches_network: bool,
    pub evidence: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayLocalExecutionReport {
    pub generated_at_unix_ms: u128,
    pub summary: String,
    pub checks: Vec<FridayLocalExecutionCheck>,
}

impl FridayLocalExecutionReport {
    pub fn passed_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|check| check.status == FridayLocalCheckStatus::Passed)
            .count()
    }

    pub fn blocking_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|check| check.status == FridayLocalCheckStatus::Failed)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|check| check.status == FridayLocalCheckStatus::Warning)
            .count()
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn default_friday_local_execution_checks() -> FridayLocalExecutionReport {
    let checks = vec![
        stt_check(),
        tts_check(),
        ocr_check(),
        metasearch_request_check(),
        artifact_preview_check(),
        runtime_store_check(),
    ];

    let passed = checks
        .iter()
        .filter(|check| check.status == FridayLocalCheckStatus::Passed)
        .count();
    let failed = checks
        .iter()
        .filter(|check| check.status == FridayLocalCheckStatus::Failed)
        .count();

    FridayLocalExecutionReport {
        generated_at_unix_ms: unix_ms(),
        summary: format!(
            "{passed}/{} local checks are ready; {failed} blocking issue(s) need attention before this is production-live.",
            checks.len()
        ),
        checks,
    }
}

fn stt_check() -> FridayLocalExecutionCheck {
    let runtime = FridayRuntimeSurfaceStore::seed_local_first();
    let model_key = runtime.voice.stt_model_key;
    let manifest = default_model_catalog().into_iter().find(|candidate| {
        candidate.key == model_key && candidate.modality == Modality::SpeechToText
    });

    let (status, mut evidence, next_action) = match manifest {
        Some(manifest) => {
            let ready =
                LocalSttEngine::model_files_ready(&model_key, manifest.local_path.as_deref());
            let mut evidence = vec![
                format!("model_key={model_key}"),
                format!("runtime={:?}", manifest.preferred_runtime),
                format!(
                    "path={}",
                    manifest.local_path.as_deref().unwrap_or("<missing>")
                ),
                format!("files={}", present_label(ready)),
            ];
            evidence.extend(path_evidence(manifest.local_path.as_deref().into_iter()));

            if ready {
                (
                    FridayLocalCheckStatus::Passed,
                    evidence,
                    "Run a short fixture transcription when changing the STT engine.".to_string(),
                )
            } else {
                (
                    FridayLocalCheckStatus::Failed,
                    evidence,
                    format!(
                        "Install the default STT model with `flow --install-model {model_key}`."
                    ),
                )
            }
        }
        None => (
            FridayLocalCheckStatus::Failed,
            vec![
                format!("model_key={model_key}"),
                "catalog=missing".to_string(),
            ],
            "Add the selected STT model to the runtime catalog.".to_string(),
        ),
    };

    evidence.push("resource_policy=no model load, no microphone capture".to_string());
    check(
        "stt-parakeet-artifacts",
        FridayWorkspaceArea::Voice,
        FridayLocalCheckKind::ModelArtifacts,
        "Default STT artifact readiness",
        "flow --transcribe <short-wav>",
        status,
        evidence,
        next_action,
    )
}

fn tts_check() -> FridayLocalExecutionCheck {
    let ready = KokoroTTS::is_available();
    let mut evidence = vec![
        "model_key=kokoro-int8".to_string(),
        format!("files={}", present_label(ready)),
    ];
    evidence.extend(path_evidence([
        "models/tts/kokoro-v1.0.int8.onnx",
        "models/tts/voices-v1.0.bin",
        "models/tts/config.json",
    ]));
    evidence.push("resource_policy=no model load, no audio playback".to_string());

    check(
        "tts-kokoro-artifacts",
        FridayWorkspaceArea::Voice,
        FridayLocalCheckKind::ModelArtifacts,
        "Default TTS artifact readiness",
        "flow --speak <text>",
        if ready {
            FridayLocalCheckStatus::Passed
        } else {
            FridayLocalCheckStatus::Failed
        },
        evidence,
        if ready {
            "Keep Kokoro warmed by the desktop host only after user opt-in.".to_string()
        } else {
            "Run `scripts/download_ultralight_models.ps1` or install the Kokoro ONNX and voices assets under `models/tts`.".to_string()
        },
    )
}

fn ocr_check() -> FridayLocalExecutionCheck {
    let ready = GlmOcr::is_available();
    let paths = GlmOcr::resolved_model_paths();
    let mut evidence = vec![
        "model_key=glm-ocr".to_string(),
        format!("files={}", present_label(ready)),
    ];
    evidence.extend(pathbuf_evidence(paths.iter()));
    evidence.push("resource_policy=no Python worker launch, no image read".to_string());

    check(
        "ocr-glm-artifacts",
        FridayWorkspaceArea::Multimodal,
        FridayLocalCheckKind::ModelArtifacts,
        "Default OCR artifact readiness",
        "flow --ocr <image>",
        if ready {
            FridayLocalCheckStatus::Passed
        } else {
            FridayLocalCheckStatus::Failed
        },
        evidence,
        if ready {
            "Run a small screenshot OCR smoke test after frontend wiring changes.".to_string()
        } else {
            "Place GLM-OCR GGUF and mmproj files under `models/ocr` on this G: workspace."
                .to_string()
        },
    )
}

fn metasearch_request_check() -> FridayLocalExecutionCheck {
    let plan = friday_answer_search_plan("friday local check");
    let server = MetasearchServerConfig::default();
    let api_path = server.api_path_for_plan(&plan);
    let categories = metasearch_categories(&plan.verticals).join(",");
    let ready = plan.use_adjacent_metasearch
        && api_path.contains("/api/v1/search")
        && !categories.trim().is_empty();

    check(
        "metasearch-request-path",
        FridayWorkspaceArea::Search,
        FridayLocalCheckKind::MetasearchRequest,
        "Metasearch request contract",
        "flow --friday-metasearch-json <query>",
        if ready {
            FridayLocalCheckStatus::Passed
        } else {
            FridayLocalCheckStatus::Failed
        },
        vec![
            format!(
                "endpoint=http://{}:{}{}",
                server.host, server.port, api_path
            ),
            format!("categories={categories}"),
            format!(
                "adjacent_metasearch={}",
                present_label(plan.use_adjacent_metasearch)
            ),
            "resource_policy=no network call in default check".to_string(),
        ],
        "Start the adjacent metasearch server only for a live search smoke test.".to_string(),
    )
}

fn artifact_preview_check() -> FridayLocalExecutionCheck {
    let store = FridayArtifactStore::seed_for_local_workspace();
    let findings = store.findings();
    let has_markdown = store
        .artifacts
        .iter()
        .any(|artifact| artifact.preview_runner == FridayPreviewRunner::Markdown);
    let has_html = store
        .artifacts
        .iter()
        .any(|artifact| artifact.preview_runner == FridayPreviewRunner::Html);
    let ready = findings.is_empty() && has_markdown && has_html;

    check(
        "artifact-preview-records",
        FridayWorkspaceArea::Artifacts,
        FridayLocalCheckKind::ArtifactPreview,
        "Artifact preview contract",
        "flow --friday-artifacts-json [dir]",
        if ready {
            FridayLocalCheckStatus::Passed
        } else {
            FridayLocalCheckStatus::Failed
        },
        vec![
            format!("artifacts={}", store.artifacts.len()),
            format!("checkpoints={}", store.checkpoints.len()),
            format!("markdown_preview={}", present_label(has_markdown)),
            format!("html_preview={}", present_label(has_html)),
            format!("findings={}", findings.len()),
        ],
        "Keep every previewable artifact tied to a checkpoint and explicit preview runner."
            .to_string(),
    )
}

fn runtime_store_check() -> FridayLocalExecutionCheck {
    let store = FridayRuntimeSurfaceStore::seed_local_first();
    let findings = store.findings();
    let ready = findings.is_empty()
        && !store.voice.stt_model_key.trim().is_empty()
        && !store.voice.tts_model_key.trim().is_empty()
        && !store.automations.is_empty();

    check(
        "runtime-surface-records",
        FridayWorkspaceArea::Voice,
        FridayLocalCheckKind::RuntimeStore,
        "Voice and automation runtime contract",
        "flow --friday-runtime-json [dir]",
        if ready {
            FridayLocalCheckStatus::Passed
        } else {
            FridayLocalCheckStatus::Failed
        },
        vec![
            format!("wake_commands={}", store.voice.wake_commands.join(",")),
            format!("volume_percent={}", store.voice.default_volume_percent),
            format!("multimodal_surfaces={}", store.multimodal.len()),
            format!("automations={}", store.automations.len()),
            format!("findings={}", findings.len()),
        ],
        "Wire the desktop route to these records before enabling background execution.".to_string(),
    )
}

fn check(
    id: &str,
    area: FridayWorkspaceArea,
    kind: FridayLocalCheckKind,
    title: &str,
    command: &str,
    status: FridayLocalCheckStatus,
    evidence: Vec<String>,
    next_action: String,
) -> FridayLocalExecutionCheck {
    FridayLocalExecutionCheck {
        id: id.to_string(),
        area,
        kind,
        title: title.to_string(),
        command: command.to_string(),
        status,
        local_only: true,
        loads_model: false,
        touches_network: false,
        evidence,
        next_action,
    }
}

fn path_evidence<'a>(paths: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    paths
        .into_iter()
        .map(|path| {
            let present = Path::new(path).exists()
                && fs::metadata(path)
                    .map(|metadata| metadata.len() > 0)
                    .unwrap_or(false);
            format!("{path}={}", present_label(present))
        })
        .collect()
}

fn pathbuf_evidence<'a>(paths: impl IntoIterator<Item = &'a std::path::PathBuf>) -> Vec<String> {
    paths
        .into_iter()
        .map(|path| {
            let present = path.exists()
                && fs::metadata(path)
                    .map(|metadata| metadata.len() > 0)
                    .unwrap_or(false);
            format!("{}={}", path.display(), present_label(present))
        })
        .collect()
}

fn present_label(value: bool) -> &'static str {
    if value { "present" } else { "missing" }
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

    #[test]
    fn local_execution_report_covers_core_friday_flows() {
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
    }

    #[test]
    fn local_execution_checks_stay_low_resource_by_default() {
        let report = default_friday_local_execution_checks();

        assert!(report.checks.iter().all(|check| check.local_only));
        assert!(report.checks.iter().all(|check| !check.loads_model));
        assert!(report.checks.iter().all(|check| !check.touches_network));
    }

    #[test]
    fn artifact_preview_check_passes_seed_contracts() {
        let report = default_friday_local_execution_checks();
        let check = report
            .checks
            .iter()
            .find(|check| check.id == "artifact-preview-records")
            .unwrap();

        assert_eq!(check.status, FridayLocalCheckStatus::Passed);
        assert!(
            check
                .evidence
                .iter()
                .any(|item| item == "markdown_preview=present")
        );
        assert!(
            check
                .evidence
                .iter()
                .any(|item| item == "html_preview=present")
        );
    }
}
