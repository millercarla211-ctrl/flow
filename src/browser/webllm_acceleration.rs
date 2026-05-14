use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::runtime::Modality;

use super::{
    BrowserDeviceTarget, BrowserExecutionBackend, BrowserHostFlavor, BrowserPackManifest,
    BrowserTask, BrowserWorkerKind, default_browser_pack_catalog,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserWebLlmAccelerationStatus {
    Passed,
    Warning,
    Failed,
}

impl BrowserWebLlmAccelerationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }

    fn score(self) -> f32 {
        match self {
            Self::Passed => 1.0,
            Self::Warning => 0.5,
            Self::Failed => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserWebLlmAccelerationTarget {
    pub pack_key: String,
    pub model_key: String,
    pub display_name: String,
    pub status: BrowserWebLlmAccelerationStatus,
    pub host_flavor: BrowserHostFlavor,
    pub task: BrowserTask,
    pub modality: Modality,
    pub acceleration_backend: BrowserExecutionBackend,
    pub fallback_backend: BrowserExecutionBackend,
    pub device_target: BrowserDeviceTarget,
    pub worker_kind: BrowserWorkerKind,
    pub local_only: bool,
    pub remote_allowed: bool,
    pub requirements: Vec<String>,
    pub evidence: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserWebLlmGuardrail {
    pub host_flavor: BrowserHostFlavor,
    pub acceleration_allowed: bool,
    pub fallback_backend: BrowserExecutionBackend,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserWebLlmAccelerationReport {
    pub generated_at_unix_ms: u128,
    pub summary: String,
    pub score_out_of_100: u8,
    pub local_only: bool,
    pub touches_network: bool,
    pub targets: Vec<BrowserWebLlmAccelerationTarget>,
    pub guardrails: Vec<BrowserWebLlmGuardrail>,
}

impl BrowserWebLlmAccelerationReport {
    pub fn passed_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserWebLlmAccelerationStatus::Passed)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserWebLlmAccelerationStatus::Warning)
            .count()
    }

    pub fn blocking_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserWebLlmAccelerationStatus::Failed)
            .count()
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn browser_webllm_acceleration_report() -> BrowserWebLlmAccelerationReport {
    browser_webllm_acceleration_report_for_catalog(&default_browser_pack_catalog())
}

pub fn browser_webllm_acceleration_report_for_catalog(
    catalog: &[BrowserPackManifest],
) -> BrowserWebLlmAccelerationReport {
    let targets = catalog
        .iter()
        .filter(|pack| is_webllm_candidate(pack))
        .map(webllm_target)
        .collect::<Vec<_>>();
    let guardrails = webllm_guardrails();
    let score_out_of_100 = score_targets(&targets, &guardrails);
    let passed = targets
        .iter()
        .filter(|target| target.status == BrowserWebLlmAccelerationStatus::Passed)
        .count();
    let warnings = targets
        .iter()
        .filter(|target| target.status == BrowserWebLlmAccelerationStatus::Warning)
        .count();
    let blocking = targets
        .iter()
        .filter(|target| target.status == BrowserWebLlmAccelerationStatus::Failed)
        .count();

    BrowserWebLlmAccelerationReport {
        generated_at_unix_ms: unix_ms(),
        summary: format!(
            "{passed}/{} Chromium WebLLM acceleration target(s) are gated and ready; {warnings} warning(s), {blocking} blocking issue(s).",
            targets.len()
        ),
        score_out_of_100,
        local_only: true,
        touches_network: false,
        targets,
        guardrails,
    }
}

fn webllm_target(pack: &BrowserPackManifest) -> BrowserWebLlmAccelerationTarget {
    let has_required_files = pack.files.iter().any(|file| file.path == "config.json")
        && pack
            .files
            .iter()
            .any(|file| file.path == pack.tokenizer.as_deref().unwrap_or("tokenizer.json"))
        && pack
            .files
            .iter()
            .any(|file| file.purpose.contains("model") || file.purpose.contains("weights"));
    let chromium_ready = pack.browser_support.chromium;
    let local_only = true;
    let remote_allowed = false;
    let requirements = vec![
        "chromium-extension".to_string(),
        "webgpu".to_string(),
        "cross-origin-isolated".to_string(),
        "opfs-cache".to_string(),
        "dedicated-worker".to_string(),
        "explicit-user-opt-in".to_string(),
    ];
    let status = if chromium_ready && has_required_files && local_only && !remote_allowed {
        BrowserWebLlmAccelerationStatus::Passed
    } else if chromium_ready && has_required_files {
        BrowserWebLlmAccelerationStatus::Warning
    } else {
        BrowserWebLlmAccelerationStatus::Failed
    };

    BrowserWebLlmAccelerationTarget {
        pack_key: pack.pack_key.clone(),
        model_key: pack.model_key.clone(),
        display_name: pack.display_name.clone(),
        status,
        host_flavor: BrowserHostFlavor::ChromiumExtension,
        task: BrowserTask::RewriteSelection,
        modality: Modality::Chat,
        acceleration_backend: BrowserExecutionBackend::WebLlmWorker,
        fallback_backend: pack.backend,
        device_target: BrowserDeviceTarget::WebGpu,
        worker_kind: BrowserWorkerKind::DedicatedWorker,
        local_only,
        remote_allowed,
        requirements,
        evidence: vec![
            format!("chromium_ready={}", yes_no(chromium_ready)),
            format!("has_required_files={}", yes_no(has_required_files)),
            format!("fallback_backend={:?}", pack.backend),
            format!("local_only={}", yes_no(local_only)),
            format!("remote_allowed={}", yes_no(remote_allowed)),
            "default_backend_unchanged=yes".to_string(),
        ],
        next_action: if status == BrowserWebLlmAccelerationStatus::Passed {
            "Expose this as an opt-in Chromium acceleration toggle after extension worker wiring lands."
                .to_string()
        } else {
            "Keep the Transformers.js ONNX backend active until the Qwen browser pack can satisfy WebLLM requirements."
                .to_string()
        },
    }
}

fn webllm_guardrails() -> Vec<BrowserWebLlmGuardrail> {
    [
        BrowserHostFlavor::FirefoxExtension,
        BrowserHostFlavor::SafariWebExtension,
        BrowserHostFlavor::StandaloneWebApp,
    ]
    .into_iter()
    .map(|host_flavor| BrowserWebLlmGuardrail {
        host_flavor,
        acceleration_allowed: false,
        fallback_backend: BrowserExecutionBackend::TransformersJsOnnx,
        evidence: vec![
            "webllm_requires_chromium_extension_gate=yes".to_string(),
            "fallback_backend=TransformersJsOnnx".to_string(),
            "remote_allowed=no".to_string(),
        ],
    })
    .collect()
}

fn is_webllm_candidate(pack: &BrowserPackManifest) -> bool {
    pack.model_key == "qwen3-0.6b"
        && pack.modality == Modality::Chat
        && pack.browser_support.chromium
        && pack.tags.iter().any(|tag| tag == "local-first")
}

fn score_targets(
    targets: &[BrowserWebLlmAccelerationTarget],
    guardrails: &[BrowserWebLlmGuardrail],
) -> u8 {
    if targets.is_empty()
        || guardrails
            .iter()
            .any(|guardrail| guardrail.acceleration_allowed)
    {
        return 0;
    }

    let earned = targets
        .iter()
        .map(|target| target.status.score())
        .sum::<f32>();
    ((earned / targets.len() as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
