use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::FridayMultimodalArtifactMetadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayArtifactKind {
    Document,
    Markdown,
    Code,
    UiSnippet,
    Chart,
    Media,
    ResearchReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayPreviewRunner {
    None,
    Markdown,
    Html,
    CodeDiff,
    StaticFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayCheckpointReason {
    Created,
    Generated,
    Edited,
    Reviewed,
    Reverted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayCodeTaskStatus {
    Draft,
    Ready,
    Running,
    Done,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayArtifactRecord {
    pub id: String,
    pub project_id: String,
    pub kind: FridayArtifactKind,
    pub title: String,
    pub path: String,
    pub language: Option<String>,
    pub preview_runner: FridayPreviewRunner,
    pub current_checkpoint_id: String,
    pub created_at_unix_ms: u128,
    pub updated_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayArtifactCheckpoint {
    pub id: String,
    pub artifact_id: String,
    pub reason: FridayCheckpointReason,
    pub summary: String,
    pub content_hash: String,
    pub created_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayArtifactDiff {
    pub id: String,
    pub artifact_id: String,
    pub from_checkpoint_id: String,
    pub to_checkpoint_id: String,
    pub summary: String,
    pub changed_files: Vec<String>,
    pub created_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayCodeTaskRecord {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: FridayCodeTaskStatus,
    pub artifact_ids: Vec<String>,
    pub checkpoint_ids: Vec<String>,
    pub tool_boundary: String,
    pub created_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayArtifactFinding {
    pub subject_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayArtifactStore {
    pub version: u16,
    pub artifacts: Vec<FridayArtifactRecord>,
    pub checkpoints: Vec<FridayArtifactCheckpoint>,
    pub diffs: Vec<FridayArtifactDiff>,
    pub code_tasks: Vec<FridayCodeTaskRecord>,
    pub multimodal_metadata: Vec<FridayMultimodalArtifactMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMultimodalArtifactImport {
    pub bundle_dir: String,
    pub imported_artifact_ids: Vec<String>,
    pub imported_checkpoint_ids: Vec<String>,
    pub imported_metadata_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMultimodalArtifactImportReport {
    pub store_dir: String,
    pub multimodal_metadata_json: String,
    pub manifest_json: String,
    pub import: FridayMultimodalArtifactImport,
    pub findings: Vec<FridayArtifactFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayArtifactManifest {
    pub version: u16,
    pub generated_at_unix_ms: u128,
    pub artifacts_json: String,
    pub checkpoints_json: String,
    pub diffs_json: String,
    pub code_tasks_json: String,
    pub multimodal_metadata_json: String,
    pub artifact_count: usize,
    pub checkpoint_count: usize,
    pub diff_count: usize,
    pub code_task_count: usize,
    pub multimodal_metadata_count: usize,
    pub findings: Vec<FridayArtifactFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayArtifactSnapshot {
    pub root_dir: PathBuf,
    pub artifacts_json: PathBuf,
    pub checkpoints_json: PathBuf,
    pub diffs_json: PathBuf,
    pub code_tasks_json: PathBuf,
    pub multimodal_metadata_json: PathBuf,
    pub manifest_json: PathBuf,
    pub manifest: FridayArtifactManifest,
}

impl FridayArtifactStore {
    pub fn seed_for_local_workspace() -> Self {
        let now = unix_ms();

        Self {
            version: 1,
            artifacts: vec![
                FridayArtifactRecord {
                    id: "research-answer-draft".to_string(),
                    project_id: "friday-local".to_string(),
                    kind: FridayArtifactKind::Markdown,
                    title: "Research answer draft".to_string(),
                    path: "artifacts/research-answer.md".to_string(),
                    language: Some("markdown".to_string()),
                    preview_runner: FridayPreviewRunner::Markdown,
                    current_checkpoint_id: "research-answer-draft-cp1".to_string(),
                    created_at_unix_ms: now,
                    updated_at_unix_ms: now,
                },
                FridayArtifactRecord {
                    id: "ui-prototype".to_string(),
                    project_id: "friday-local".to_string(),
                    kind: FridayArtifactKind::UiSnippet,
                    title: "Generated UI prototype".to_string(),
                    path: "artifacts/ui-prototype.html".to_string(),
                    language: Some("html".to_string()),
                    preview_runner: FridayPreviewRunner::Html,
                    current_checkpoint_id: "ui-prototype-cp1".to_string(),
                    created_at_unix_ms: now,
                    updated_at_unix_ms: now,
                },
            ],
            checkpoints: vec![
                FridayArtifactCheckpoint {
                    id: "research-answer-draft-cp1".to_string(),
                    artifact_id: "research-answer-draft".to_string(),
                    reason: FridayCheckpointReason::Created,
                    summary: "Initial cited research answer canvas.".to_string(),
                    content_hash: stable_hash("research-answer-draft-v1"),
                    created_at_unix_ms: now,
                },
                FridayArtifactCheckpoint {
                    id: "ui-prototype-cp1".to_string(),
                    artifact_id: "ui-prototype".to_string(),
                    reason: FridayCheckpointReason::Created,
                    summary: "Initial generated UI artifact preview.".to_string(),
                    content_hash: stable_hash("ui-prototype-v1"),
                    created_at_unix_ms: now,
                },
            ],
            diffs: vec![FridayArtifactDiff {
                id: "ui-prototype-diff1".to_string(),
                artifact_id: "ui-prototype".to_string(),
                from_checkpoint_id: "ui-prototype-cp1".to_string(),
                to_checkpoint_id: "ui-prototype-cp1".to_string(),
                summary: "No changes yet; checkpoint is ready for first edited diff.".to_string(),
                changed_files: vec!["artifacts/ui-prototype.html".to_string()],
                created_at_unix_ms: now,
            }],
            code_tasks: vec![FridayCodeTaskRecord {
                id: "artifact-canvas-checkpoint".to_string(),
                project_id: "friday-local".to_string(),
                title: "Prepare editable artifact canvas and code checkpoint state".to_string(),
                status: FridayCodeTaskStatus::Ready,
                artifact_ids: vec![
                    "research-answer-draft".to_string(),
                    "ui-prototype".to_string(),
                ],
                checkpoint_ids: vec![
                    "research-answer-draft-cp1".to_string(),
                    "ui-prototype-cp1".to_string(),
                ],
                tool_boundary:
                    "Code tools must write through approved artifact/checkpoint records before host execution."
                        .to_string(),
                created_at_unix_ms: now,
            }],
            multimodal_metadata: Vec::new(),
        }
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn artifact(&self, id: &str) -> Option<&FridayArtifactRecord> {
        self.artifacts.iter().find(|artifact| artifact.id == id)
    }

    pub fn checkpoints_for_artifact(&self, artifact_id: &str) -> Vec<&FridayArtifactCheckpoint> {
        self.checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.artifact_id == artifact_id)
            .collect()
    }

    pub fn diffs_for_artifact(&self, artifact_id: &str) -> Vec<&FridayArtifactDiff> {
        self.diffs
            .iter()
            .filter(|diff| diff.artifact_id == artifact_id)
            .collect()
    }

    pub fn task_artifacts(&self, task_id: &str) -> Vec<&FridayArtifactRecord> {
        let Some(task) = self.code_tasks.iter().find(|task| task.id == task_id) else {
            return Vec::new();
        };

        task.artifact_ids
            .iter()
            .filter_map(|artifact_id| self.artifact(artifact_id))
            .collect()
    }

    pub fn read_or_seed_from_dir(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        if root.join("artifacts.json").exists() {
            Self::read_from_dir(root)
        } else {
            Ok(Self::seed_for_local_workspace())
        }
    }

    pub fn multimodal_metadata_for_artifact(
        &self,
        artifact_id: &str,
    ) -> Vec<&FridayMultimodalArtifactMetadata> {
        self.multimodal_metadata
            .iter()
            .filter(|metadata| metadata.artifact_id == artifact_id)
            .collect()
    }

    pub fn index_multimodal_metadata(
        &mut self,
        metadata: FridayMultimodalArtifactMetadata,
    ) -> bool {
        if let Some(existing) = self.multimodal_metadata.iter_mut().find(|existing| {
            existing.artifact_id == metadata.artifact_id
                && existing.request_kind == metadata.request_kind
                && existing.source_uri == metadata.source_uri
        }) {
            *existing = metadata;
            false
        } else {
            self.multimodal_metadata.push(metadata);
            true
        }
    }

    pub fn import_multimodal_bundle(
        &mut self,
        bundle_dir: impl AsRef<Path>,
    ) -> Result<FridayMultimodalArtifactImport> {
        let bundle_dir = bundle_dir.as_ref();
        let candidates = [
            (
                "ocr-smoke-artifact.json",
                "ocr-smoke-checkpoint.json",
                "ocr-smoke-metadata.json",
            ),
            (
                "vlm-screenshot-artifact.json",
                "vlm-screenshot-checkpoint.json",
                "vlm-screenshot-metadata.json",
            ),
        ];

        let mut imported_artifact_ids = Vec::new();
        let mut imported_checkpoint_ids = Vec::new();
        let mut imported_metadata_count = 0;

        for (artifact_file, checkpoint_file, metadata_file) in candidates {
            let artifact_path = bundle_dir.join(artifact_file);
            let checkpoint_path = bundle_dir.join(checkpoint_file);
            let metadata_path = bundle_dir.join(metadata_file);
            if !artifact_path.exists() && !checkpoint_path.exists() && !metadata_path.exists() {
                continue;
            }

            let artifact: FridayArtifactRecord = read_json(&artifact_path)?;
            let checkpoint: FridayArtifactCheckpoint = read_json(&checkpoint_path)?;
            let metadata: FridayMultimodalArtifactMetadata = read_json(&metadata_path)?;
            let artifact_id = artifact.id.clone();
            let checkpoint_id = checkpoint.id.clone();

            upsert_by_id(&mut self.artifacts, artifact_id.clone(), artifact, |item| {
                &item.id
            });
            upsert_by_id(
                &mut self.checkpoints,
                checkpoint_id.clone(),
                checkpoint,
                |item| &item.id,
            );
            self.index_multimodal_metadata(metadata);

            imported_artifact_ids.push(artifact_id);
            imported_checkpoint_ids.push(checkpoint_id);
            imported_metadata_count += 1;
        }

        if imported_artifact_ids.is_empty() && imported_checkpoint_ids.is_empty() {
            anyhow::bail!(
                "No supported multimodal artifact bundle files found in {}",
                bundle_dir.display()
            );
        }

        Ok(FridayMultimodalArtifactImport {
            bundle_dir: bundle_dir.to_string_lossy().into_owned(),
            imported_artifact_ids,
            imported_checkpoint_ids,
            imported_metadata_count,
        })
    }

    pub fn findings(&self) -> Vec<FridayArtifactFinding> {
        let artifact_ids = self
            .artifacts
            .iter()
            .map(|artifact| artifact.id.as_str())
            .collect::<BTreeSet<_>>();
        let checkpoint_ids = self
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.id.as_str())
            .collect::<BTreeSet<_>>();

        let mut findings = Vec::new();

        for artifact in &self.artifacts {
            if !checkpoint_ids.contains(artifact.current_checkpoint_id.as_str()) {
                findings.push(finding(
                    &artifact.id,
                    format!(
                        "Artifact current checkpoint `{}` is missing.",
                        artifact.current_checkpoint_id
                    ),
                ));
            }
        }

        for checkpoint in &self.checkpoints {
            if !artifact_ids.contains(checkpoint.artifact_id.as_str()) {
                findings.push(finding(
                    &checkpoint.id,
                    format!(
                        "Checkpoint references missing artifact `{}`.",
                        checkpoint.artifact_id
                    ),
                ));
            }
        }

        for diff in &self.diffs {
            if !artifact_ids.contains(diff.artifact_id.as_str()) {
                findings.push(finding(
                    &diff.id,
                    format!("Diff references missing artifact `{}`.", diff.artifact_id),
                ));
            }
            if !checkpoint_ids.contains(diff.from_checkpoint_id.as_str())
                || !checkpoint_ids.contains(diff.to_checkpoint_id.as_str())
            {
                findings.push(finding(
                    &diff.id,
                    "Diff references a missing checkpoint.".to_string(),
                ));
            }
        }

        for task in &self.code_tasks {
            for artifact_id in &task.artifact_ids {
                if !artifact_ids.contains(artifact_id.as_str()) {
                    findings.push(finding(
                        &task.id,
                        format!("Code task references missing artifact `{artifact_id}`."),
                    ));
                }
            }
            for checkpoint_id in &task.checkpoint_ids {
                if !checkpoint_ids.contains(checkpoint_id.as_str()) {
                    findings.push(finding(
                        &task.id,
                        format!("Code task references missing checkpoint `{checkpoint_id}`."),
                    ));
                }
            }
        }

        for metadata in &self.multimodal_metadata {
            if !artifact_ids.contains(metadata.artifact_id.as_str()) {
                findings.push(finding(
                    &metadata.artifact_id,
                    format!(
                        "Multimodal metadata references missing artifact `{}`.",
                        metadata.artifact_id
                    ),
                ));
            }
        }

        findings
    }

    pub fn write_to_dir(&self, root: impl AsRef<Path>) -> Result<FridayArtifactSnapshot> {
        let root = root.as_ref();
        fs::create_dir_all(root)
            .with_context(|| format!("failed to create Friday artifact dir {}", root.display()))?;

        let artifacts_json = root.join("artifacts.json");
        let checkpoints_json = root.join("checkpoints.json");
        let diffs_json = root.join("diffs.json");
        let code_tasks_json = root.join("code-tasks.json");
        let multimodal_metadata_json = root.join("multimodal-metadata.json");
        let manifest_json = root.join("manifest.json");

        write_json(&artifacts_json, &self.artifacts)?;
        write_json(&checkpoints_json, &self.checkpoints)?;
        write_json(&diffs_json, &self.diffs)?;
        write_json(&code_tasks_json, &self.code_tasks)?;
        write_json(&multimodal_metadata_json, &self.multimodal_metadata)?;

        let manifest = FridayArtifactManifest {
            version: self.version,
            generated_at_unix_ms: unix_ms(),
            artifacts_json: artifacts_json.to_string_lossy().into_owned(),
            checkpoints_json: checkpoints_json.to_string_lossy().into_owned(),
            diffs_json: diffs_json.to_string_lossy().into_owned(),
            code_tasks_json: code_tasks_json.to_string_lossy().into_owned(),
            multimodal_metadata_json: multimodal_metadata_json.to_string_lossy().into_owned(),
            artifact_count: self.artifacts.len(),
            checkpoint_count: self.checkpoints.len(),
            diff_count: self.diffs.len(),
            code_task_count: self.code_tasks.len(),
            multimodal_metadata_count: self.multimodal_metadata.len(),
            findings: self.findings(),
        };
        write_json(&manifest_json, &manifest)?;

        Ok(FridayArtifactSnapshot {
            root_dir: root.to_path_buf(),
            artifacts_json,
            checkpoints_json,
            diffs_json,
            code_tasks_json,
            multimodal_metadata_json,
            manifest_json,
            manifest,
        })
    }

    pub fn read_from_dir(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        Ok(Self {
            version: 1,
            artifacts: read_json(&root.join("artifacts.json"))?,
            checkpoints: read_json(&root.join("checkpoints.json"))?,
            diffs: read_json(&root.join("diffs.json"))?,
            code_tasks: read_json(&root.join("code-tasks.json"))?,
            multimodal_metadata: read_json_if_exists(&root.join("multimodal-metadata.json"))?
                .unwrap_or_default(),
        })
    }
}

fn upsert_by_id<T, F>(items: &mut Vec<T>, id: String, value: T, id_of: F)
where
    F: Fn(&T) -> &String,
{
    if let Some(existing) = items.iter_mut().find(|item| id_of(item) == &id) {
        *existing = value;
    } else {
        items.push(value);
    }
}

fn finding(subject_id: &str, message: impl Into<String>) -> FridayArtifactFinding {
    FridayArtifactFinding {
        subject_id: subject_id.to_string(),
        message: message.into(),
    }
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

fn read_json<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let json = fs::read_to_string(path)
        .with_context(|| format!("failed to read Friday artifact file {}", path.display()))?;
    serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display()))
}

fn read_json_if_exists<T>(path: &Path) -> Result<Option<T>>
where
    T: for<'de> Deserialize<'de>,
{
    if path.exists() {
        read_json(path).map(Some)
    } else {
        Ok(None)
    }
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
    fn seed_store_links_artifacts_checkpoints_diffs_and_tasks() {
        let store = FridayArtifactStore::seed_for_local_workspace();
        assert_eq!(store.artifacts.len(), 2);
        assert_eq!(store.checkpoints.len(), 2);
        assert_eq!(store.diffs.len(), 1);
        assert_eq!(store.code_tasks.len(), 1);
        assert!(store.multimodal_metadata.is_empty());
        assert!(store.findings().is_empty());

        let ui = store.artifact("ui-prototype").unwrap();
        assert_eq!(ui.preview_runner, FridayPreviewRunner::Html);
        assert_eq!(store.checkpoints_for_artifact(&ui.id).len(), 1);
        assert_eq!(store.diffs_for_artifact(&ui.id).len(), 1);
        assert_eq!(store.task_artifacts("artifact-canvas-checkpoint").len(), 2);
    }

    #[test]
    fn artifact_store_round_trips_as_separate_json_files() {
        let root = std::env::temp_dir().join(format!(
            "friday-artifacts-{}-{}",
            std::process::id(),
            unix_ms()
        ));
        let _ = fs::remove_dir_all(&root);

        let store = FridayArtifactStore::seed_for_local_workspace();
        let snapshot = store.write_to_dir(&root).unwrap();
        assert!(snapshot.artifacts_json.exists());
        assert!(snapshot.checkpoints_json.exists());
        assert!(snapshot.diffs_json.exists());
        assert!(snapshot.code_tasks_json.exists());
        assert!(snapshot.multimodal_metadata_json.exists());
        assert!(snapshot.manifest_json.exists());

        let restored = FridayArtifactStore::read_from_dir(&root).unwrap();
        assert_eq!(restored, store);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn artifact_store_indexes_multimodal_metadata() {
        let mut store = FridayArtifactStore::seed_for_local_workspace();
        let metadata = FridayMultimodalArtifactMetadata {
            artifact_id: "research-answer-draft".to_string(),
            request_kind: super::super::FridayMultimodalRequestKind::Ocr,
            source_uri: "fixture://ocr".to_string(),
            source_mime: Some("image/png".to_string()),
            model_key: "glm-ocr-q4km".to_string(),
            prompt: None,
            output_format: "markdown".to_string(),
            local_only: true,
            model_execution: false,
            duration_ms: 1,
            confidence_percent: None,
            created_at_unix_ms: unix_ms(),
        };

        assert!(store.index_multimodal_metadata(metadata.clone()));
        assert!(!store.index_multimodal_metadata(metadata));
        assert_eq!(
            store
                .multimodal_metadata_for_artifact("research-answer-draft")
                .len(),
            1
        );
        assert!(store.findings().is_empty());
    }
}
