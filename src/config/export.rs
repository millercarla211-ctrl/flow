use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::competitive::{CompletionItem, active_completion_set};
use crate::runtime::FlowLocalRuntimeSummary;

use super::{FlowIntegrationTarget, FlowProductionConfig};

pub const VALIDATED_RELEASE_COMMANDS: [&str; 8] = [
    "cargo check",
    "cargo test",
    "cargo build",
    "cargo check -p flow-browser-core",
    "cargo check --features example-binaries --examples",
    "npm run typecheck (extensions/flow-webext)",
    "npm run build:all (extensions/flow-webext)",
    "npm run package:all (extensions/flow-webext)",
];

pub const BROWSER_RELEASE_ARTIFACTS: [&str; 3] = [
    "extensions/flow-webext/artifacts/flow-webext-chromium-v0.1.0.zip",
    "extensions/flow-webext/artifacts/flow-webext-firefox-v0.1.0.zip",
    "extensions/flow-webext/artifacts/flow-webext-safari-v0.1.0.zip",
];

pub const HANDOFF_DOCUMENT_SNAPSHOTS: [&str; 2] = ["TODO.md", "CHANGELOG.md"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowProductionBundleEntry {
    pub target: FlowIntegrationTarget,
    pub filename: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowProductionBundleDocument {
    pub source: String,
    pub filename: String,
    pub copied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowProductionBundleManifest {
    pub project: String,
    pub crate_version: String,
    pub generated_at_unix_ms: u128,
    pub device_tier: String,
    pub selected_text_model: Option<String>,
    pub selected_stt_model: Option<String>,
    pub selected_tts_model: Option<String>,
    pub all_models_ready: bool,
    pub missing_model_paths: Vec<String>,
    pub entries: Vec<FlowProductionBundleEntry>,
    pub handoff_documents: Vec<FlowProductionBundleDocument>,
    pub active_completion_set: String,
    pub completion_score_out_of_100: u8,
    pub completion_target_score_out_of_100: u8,
    pub completion_items: Vec<CompletionItem>,
    pub browser_release_artifacts: Vec<String>,
    pub validated_commands: Vec<String>,
    pub notes: Vec<String>,
}

impl FlowProductionBundleManifest {
    pub fn for_summary(
        summary: &FlowLocalRuntimeSummary,
        entries: Vec<FlowProductionBundleEntry>,
        handoff_documents: Vec<FlowProductionBundleDocument>,
    ) -> Self {
        let completion_set = active_completion_set();

        Self {
            project: "flow".to_string(),
            crate_version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            device_tier: format!("{:?}", summary.device_profile.tier),
            selected_text_model: summary.chat.model_key.clone(),
            selected_stt_model: summary.speech_to_text.model_key.clone(),
            selected_tts_model: summary.text_to_speech.model_key.clone(),
            all_models_ready: summary.all_ready(),
            missing_model_paths: summary.missing_model_paths(),
            entries,
            handoff_documents,
            active_completion_set: completion_set.name,
            completion_score_out_of_100: completion_set.current_score_out_of_100,
            completion_target_score_out_of_100: completion_set.target_score_out_of_100,
            completion_items: completion_set.items,
            browser_release_artifacts: BROWSER_RELEASE_ARTIFACTS
                .into_iter()
                .map(str::to_string)
                .collect(),
            validated_commands: VALIDATED_RELEASE_COMMANDS
                .into_iter()
                .map(str::to_string)
                .collect(),
            notes: vec![
                "This bundle contains low-end-safe production defaults for every supported Flow host target.".to_string(),
                "TODO.md and CHANGELOG.md snapshots are included when the command is run from the repository root.".to_string(),
                "Firebase wiring, browser-store publishing, and vendor-side signing stay external to this repository.".to_string(),
            ],
        }
    }

    pub fn to_pretty_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

pub fn recommended_production_configs(
    summary: &FlowLocalRuntimeSummary,
) -> Vec<FlowProductionConfig> {
    FlowIntegrationTarget::all()
        .iter()
        .copied()
        .map(|target| FlowProductionConfig::recommended_for_target(target, summary))
        .collect()
}

pub fn export_production_bundle(
    summary: &FlowLocalRuntimeSummary,
    output_dir: impl AsRef<Path>,
) -> Result<FlowProductionBundleManifest> {
    let repo_root = std::env::current_dir()?;
    export_production_bundle_from_repo(summary, repo_root, output_dir)
}

pub fn export_production_bundle_from_repo(
    summary: &FlowLocalRuntimeSummary,
    repo_root: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<FlowProductionBundleManifest> {
    let repo_root = repo_root.as_ref();
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    let mut entries = Vec::new();
    for target in FlowIntegrationTarget::all().iter().copied() {
        let config = FlowProductionConfig::recommended_for_target(target, summary);
        let filename = format!("{}.json", target.slug());
        let path = output_dir.join(&filename);
        fs::write(&path, config.to_pretty_json()?)?;
        entries.push(FlowProductionBundleEntry { target, filename });
    }

    let handoff_documents = copy_handoff_documents(repo_root, output_dir)?;
    let manifest = FlowProductionBundleManifest::for_summary(summary, entries, handoff_documents);
    fs::write(output_dir.join("manifest.json"), manifest.to_pretty_json()?)?;
    fs::write(
        output_dir.join("README.txt"),
        build_bundle_readme(summary, &manifest),
    )?;

    Ok(manifest)
}

fn copy_handoff_documents(
    repo_root: &Path,
    output_dir: &Path,
) -> Result<Vec<FlowProductionBundleDocument>> {
    let handoff_dir = output_dir.join("handoff");
    fs::create_dir_all(&handoff_dir)?;

    HANDOFF_DOCUMENT_SNAPSHOTS
        .iter()
        .map(|source| {
            let filename = source.to_ascii_lowercase().replace(".md", "-snapshot.md");
            let copied = match fs::copy(repo_root.join(source), handoff_dir.join(&filename)) {
                Ok(_) => true,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => return Err(error.into()),
            };

            Ok(FlowProductionBundleDocument {
                source: (*source).to_string(),
                filename: format!("handoff/{filename}"),
                copied,
            })
        })
        .collect()
}

fn build_bundle_readme(
    summary: &FlowLocalRuntimeSummary,
    manifest: &FlowProductionBundleManifest,
) -> String {
    let mut lines = vec![
        "Flow Production Bundle".to_string(),
        "======================".to_string(),
        String::new(),
        format!("crate_version={}", manifest.crate_version),
        format!("device_tier={}", manifest.device_tier),
        format!(
            "text_model={}",
            summary.chat.model_key.as_deref().unwrap_or("none")
        ),
        format!(
            "stt_model={}",
            summary
                .speech_to_text
                .model_key
                .as_deref()
                .unwrap_or("none")
        ),
        format!(
            "tts_model={}",
            summary
                .text_to_speech
                .model_key
                .as_deref()
                .unwrap_or("none")
        ),
        format!("all_models_ready={}", manifest.all_models_ready),
        String::new(),
        "Included configs:".to_string(),
    ];

    for entry in &manifest.entries {
        lines.push(format!("  - {} -> {}", entry.target.slug(), entry.filename));
    }

    lines.push(String::new());
    lines.push("Handoff snapshots:".to_string());
    for document in &manifest.handoff_documents {
        lines.push(format!(
            "  - {} -> {} ({})",
            document.source,
            document.filename,
            if document.copied { "copied" } else { "missing" }
        ));
    }

    lines.push(String::new());
    lines.push(format!(
        "Completion loop: {} ({}/{})",
        manifest.active_completion_set,
        manifest.completion_score_out_of_100,
        manifest.completion_target_score_out_of_100
    ));

    if !manifest.missing_model_paths.is_empty() {
        lines.push(String::new());
        lines.push("Missing local model paths:".to_string());
        for path in &manifest.missing_model_paths {
            lines.push(format!("  - {}", path));
        }
    }

    lines.push(String::new());
    lines.push("Validated commands:".to_string());
    for command in &manifest.validated_commands {
        lines.push(format!("  - {}", command));
    }

    lines.push(String::new());
    lines.push("Browser release artifacts:".to_string());
    for artifact in &manifest.browser_release_artifacts {
        lines.push(format!("  - {}", artifact));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{
        ComputeBackend, DeviceProfile, DeviceTier, FlowLocalRuntime, GraphicsDevice,
    };

    fn low_end_runtime_summary() -> FlowLocalRuntimeSummary {
        FlowLocalRuntime::for_device_profile(DeviceProfile {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            cpu_model: "Test CPU".to_string(),
            physical_cores: 4,
            logical_cores: 8,
            total_memory_bytes: 6 * 1024 * 1024 * 1024,
            available_memory_bytes: 4 * 1024 * 1024 * 1024,
            battery_powered: None,
            thermal_class: None,
            graphics: vec![GraphicsDevice {
                name: "Integrated GPU".to_string(),
                vendor: Some("intel".to_string()),
                vram_bytes: None,
                integrated: true,
                backends: vec![ComputeBackend::Cpu],
            }],
            tier: DeviceTier::Low,
        })
        .unwrap()
        .summary()
        .clone()
    }

    #[test]
    fn production_bundle_covers_all_targets() {
        let configs = recommended_production_configs(&low_end_runtime_summary());
        assert_eq!(configs.len(), FlowIntegrationTarget::all().len());
        assert!(
            configs
                .iter()
                .any(|config| config.target == FlowIntegrationTarget::BrowserExtension)
        );
    }

    #[test]
    fn export_production_bundle_writes_manifest_and_configs() {
        let temp_dir = std::env::temp_dir().join(format!(
            "flow_bundle_test_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let repo_root = temp_dir.join("repo");
        let output = temp_dir.join("out");
        fs::create_dir_all(&repo_root).unwrap();
        fs::write(repo_root.join("TODO.md"), "# TODO").unwrap();
        fs::write(repo_root.join("CHANGELOG.md"), "# Changelog").unwrap();

        let manifest =
            export_production_bundle_from_repo(&low_end_runtime_summary(), &repo_root, &output)
                .unwrap();

        assert!(output.join("manifest.json").exists());
        assert!(output.join("README.txt").exists());
        for entry in &manifest.entries {
            assert!(output.join(&entry.filename).exists());
        }
        assert!(
            manifest
                .handoff_documents
                .iter()
                .all(|document| document.copied)
        );
        assert!(output.join("handoff/todo-snapshot.md").exists());
        assert!(output.join("handoff/changelog-snapshot.md").exists());
        assert!(manifest.completion_score_out_of_100 <= 100);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
