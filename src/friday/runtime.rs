use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::runtime::wake_command_definitions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayMultimodalSurface {
    Ocr,
    Vision,
    Image,
    Audio,
    Video,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayAutomationTrigger {
    Manual,
    WakeFollowup,
    Scheduled,
    BackgroundResearch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayRuntimeRecordStatus {
    Ready,
    NeedsModel,
    NeedsPermission,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayVoiceRuntimeRecord {
    pub id: String,
    pub stt_model_key: String,
    pub tts_model_key: String,
    pub wake_commands: Vec<String>,
    pub overlay_timeout_ms: u64,
    pub default_volume_percent: u8,
    pub duplex_voice_enabled: bool,
    pub audit_stream: String,
    pub status: FridayRuntimeRecordStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMultimodalRuntimeRecord {
    pub id: String,
    pub surface: FridayMultimodalSurface,
    pub default_model_key: String,
    pub input_boundary: String,
    pub output_artifact_kind: String,
    pub status: FridayRuntimeRecordStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayAutomationRuntimeRecord {
    pub id: String,
    pub title: String,
    pub trigger: FridayAutomationTrigger,
    pub route: String,
    pub approval_required: bool,
    pub audit_state_file: String,
    pub status: FridayRuntimeRecordStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayRuntimeFinding {
    pub subject_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayRuntimeSurfaceStore {
    pub version: u16,
    pub voice: FridayVoiceRuntimeRecord,
    pub multimodal: Vec<FridayMultimodalRuntimeRecord>,
    pub automations: Vec<FridayAutomationRuntimeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayRuntimeSurfaceManifest {
    pub version: u16,
    pub generated_at_unix_ms: u128,
    pub voice_json: String,
    pub multimodal_json: String,
    pub automations_json: String,
    pub multimodal_count: usize,
    pub automation_count: usize,
    pub findings: Vec<FridayRuntimeFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayRuntimeSurfaceSnapshot {
    pub root_dir: PathBuf,
    pub voice_json: PathBuf,
    pub multimodal_json: PathBuf,
    pub automations_json: PathBuf,
    pub manifest_json: PathBuf,
    pub manifest: FridayRuntimeSurfaceManifest,
}

impl FridayRuntimeSurfaceStore {
    pub fn seed_local_first() -> Self {
        Self {
            version: 1,
            voice: FridayVoiceRuntimeRecord {
                id: "voice-local-runtime".to_string(),
                stt_model_key: "parakeet-tdt-0.6b-v3-int8".to_string(),
                tts_model_key: "kokoro-int8".to_string(),
                wake_commands: wake_command_definitions()
                    .iter()
                    .map(|definition| definition.command_key.to_string())
                    .collect(),
                overlay_timeout_ms: 30_000,
                default_volume_percent: 10,
                duplex_voice_enabled: true,
                audit_stream: "tmp/friday-voice-audit.txt".to_string(),
                status: FridayRuntimeRecordStatus::Ready,
            },
            multimodal: vec![
                FridayMultimodalRuntimeRecord {
                    id: "ocr-local".to_string(),
                    surface: FridayMultimodalSurface::Ocr,
                    default_model_key: "glm-ocr".to_string(),
                    input_boundary: "Images stay local and produce reviewable text artifacts."
                        .to_string(),
                    output_artifact_kind: "research-report".to_string(),
                    status: FridayRuntimeRecordStatus::Ready,
                },
                FridayMultimodalRuntimeRecord {
                    id: "vision-local".to_string(),
                    surface: FridayMultimodalSurface::Vision,
                    default_model_key: "gemma4-e4b-frontend-q4km".to_string(),
                    input_boundary: "Screenshots stay local and can feed UI-generation artifacts."
                        .to_string(),
                    output_artifact_kind: "ui-snippet".to_string(),
                    status: FridayRuntimeRecordStatus::NeedsModel,
                },
            ],
            automations: vec![
                FridayAutomationRuntimeRecord {
                    id: "wake-followup-hide-overlay".to_string(),
                    title: "Hide inactive wake overlay after follow-up window".to_string(),
                    trigger: FridayAutomationTrigger::WakeFollowup,
                    route: "/voice".to_string(),
                    approval_required: false,
                    audit_state_file: "tmp/friday-voice-audit.txt".to_string(),
                    status: FridayRuntimeRecordStatus::Ready,
                },
                FridayAutomationRuntimeRecord {
                    id: "background-research-run".to_string(),
                    title: "Approved background research run".to_string(),
                    trigger: FridayAutomationTrigger::BackgroundResearch,
                    route: "/automations".to_string(),
                    approval_required: true,
                    audit_state_file: "tmp/friday-automation-audit.txt".to_string(),
                    status: FridayRuntimeRecordStatus::Ready,
                },
            ],
        }
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn findings(&self) -> Vec<FridayRuntimeFinding> {
        let mut findings = Vec::new();

        if self.voice.wake_commands.is_empty() {
            findings.push(finding(
                &self.voice.id,
                "Voice runtime has no wake commands configured.",
            ));
        }
        if self.voice.default_volume_percent > 100 {
            findings.push(finding(
                &self.voice.id,
                "Voice runtime volume must stay within 0-100 percent.",
            ));
        }
        if self.voice.status == FridayRuntimeRecordStatus::Ready
            && (self.voice.stt_model_key.trim().is_empty()
                || self.voice.tts_model_key.trim().is_empty())
        {
            findings.push(finding(
                &self.voice.id,
                "Ready voice runtime requires both STT and TTS model keys.",
            ));
        }

        for item in &self.multimodal {
            if item.status == FridayRuntimeRecordStatus::Ready
                && item.default_model_key.trim().is_empty()
            {
                findings.push(finding(
                    &item.id,
                    "Ready multimodal surface requires a default model key.",
                ));
            }
        }

        for automation in &self.automations {
            if automation.route.trim().is_empty() {
                findings.push(finding(
                    &automation.id,
                    "Automation record requires a product route.",
                ));
            }
            if automation.trigger == FridayAutomationTrigger::BackgroundResearch
                && !automation.approval_required
            {
                findings.push(finding(
                    &automation.id,
                    "Background research automations must require approval by default.",
                ));
            }
        }

        findings
    }

    pub fn write_to_dir(&self, root: impl AsRef<Path>) -> Result<FridayRuntimeSurfaceSnapshot> {
        let root = root.as_ref();
        fs::create_dir_all(root)
            .with_context(|| format!("failed to create Friday runtime dir {}", root.display()))?;

        let voice_json = root.join("voice.json");
        let multimodal_json = root.join("multimodal.json");
        let automations_json = root.join("automations.json");
        let manifest_json = root.join("manifest.json");

        write_json(&voice_json, &self.voice)?;
        write_json(&multimodal_json, &self.multimodal)?;
        write_json(&automations_json, &self.automations)?;

        let manifest = FridayRuntimeSurfaceManifest {
            version: self.version,
            generated_at_unix_ms: unix_ms(),
            voice_json: voice_json.to_string_lossy().into_owned(),
            multimodal_json: multimodal_json.to_string_lossy().into_owned(),
            automations_json: automations_json.to_string_lossy().into_owned(),
            multimodal_count: self.multimodal.len(),
            automation_count: self.automations.len(),
            findings: self.findings(),
        };
        write_json(&manifest_json, &manifest)?;

        Ok(FridayRuntimeSurfaceSnapshot {
            root_dir: root.to_path_buf(),
            voice_json,
            multimodal_json,
            automations_json,
            manifest_json,
            manifest,
        })
    }

    pub fn read_from_dir(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        Ok(Self {
            version: 1,
            voice: read_json(&root.join("voice.json"))?,
            multimodal: read_json(&root.join("multimodal.json"))?,
            automations: read_json(&root.join("automations.json"))?,
        })
    }
}

fn finding(subject_id: &str, message: impl Into<String>) -> FridayRuntimeFinding {
    FridayRuntimeFinding {
        subject_id: subject_id.to_string(),
        message: message.into(),
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn read_json<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let json = fs::read_to_string(path)
        .with_context(|| format!("failed to read Friday runtime file {}", path.display()))?;
    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
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
    fn seed_runtime_links_voice_multimodal_and_automations() {
        let store = FridayRuntimeSurfaceStore::seed_local_first();
        assert_eq!(store.voice.stt_model_key, "parakeet-tdt-0.6b-v3-int8");
        assert_eq!(store.voice.tts_model_key, "kokoro-int8");
        assert!(store.voice.wake_commands.contains(&"hello".to_string()));
        assert_eq!(store.voice.default_volume_percent, 10);
        assert!(store.findings().is_empty());
        assert!(
            store
                .multimodal
                .iter()
                .any(|item| item.surface == FridayMultimodalSurface::Ocr)
        );
        assert!(store.automations.iter().any(|item| item.trigger
            == FridayAutomationTrigger::BackgroundResearch
            && item.approval_required));
    }

    #[test]
    fn runtime_surface_store_round_trips_as_separate_json_files() {
        let root = std::env::temp_dir().join(format!(
            "friday-runtime-{}-{}",
            std::process::id(),
            unix_ms()
        ));
        let _ = fs::remove_dir_all(&root);

        let store = FridayRuntimeSurfaceStore::seed_local_first();
        let snapshot = store.write_to_dir(&root).unwrap();
        assert!(snapshot.voice_json.exists());
        assert!(snapshot.multimodal_json.exists());
        assert!(snapshot.automations_json.exists());
        assert!(snapshot.manifest_json.exists());

        let restored = FridayRuntimeSurfaceStore::read_from_dir(&root).unwrap();
        assert_eq!(restored, store);

        let _ = fs::remove_dir_all(&root);
    }
}
