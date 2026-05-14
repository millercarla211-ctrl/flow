use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayConnectorKind {
    LocalFiles,
    Metasearch,
    ProviderCatalog,
    Calendar,
    Mail,
    Drive,
    Github,
    CustomMcp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayConnectorAuthState {
    LocalOnly,
    NotConfigured,
    Configured,
    Disabled,
}

impl FridayConnectorAuthState {
    pub fn label(self) -> &'static str {
        match self {
            Self::LocalOnly => "local-only",
            Self::NotConfigured => "not-configured",
            Self::Configured => "configured",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayMemoryState {
    Active,
    PendingReview,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayPermissionScope {
    LocalProject,
    LocalFiles,
    Metasearch,
    Connector,
    Memory,
    RemoteProvider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayBoundarySeverity {
    Info,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayProjectFile {
    pub path: String,
    pub purpose: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayProjectRecord {
    pub id: String,
    pub name: String,
    pub instructions: String,
    pub files: Vec<FridayProjectFile>,
    pub memory_ids: Vec<String>,
    pub connector_ids: Vec<String>,
    pub created_at_unix_ms: u128,
    pub updated_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayMemoryRecord {
    pub id: String,
    pub project_id: String,
    pub summary: String,
    pub source: String,
    pub state: FridayMemoryState,
    pub created_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayConnectorRecord {
    pub id: String,
    pub kind: FridayConnectorKind,
    pub name: String,
    pub auth_state: FridayConnectorAuthState,
    pub permission_scopes: Vec<FridayPermissionScope>,
    pub local_boundary: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayPermissionFinding {
    pub severity: FridayBoundarySeverity,
    pub subject_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayWorkspaceStore {
    pub version: u16,
    pub projects: Vec<FridayProjectRecord>,
    pub memories: Vec<FridayMemoryRecord>,
    pub connectors: Vec<FridayConnectorRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayWorkspaceManifest {
    pub version: u16,
    pub generated_at_unix_ms: u128,
    pub projects_json: String,
    pub memories_json: String,
    pub connectors_json: String,
    pub project_count: usize,
    pub memory_count: usize,
    pub connector_count: usize,
    pub findings: Vec<FridayPermissionFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayWorkspaceSnapshot {
    pub root_dir: PathBuf,
    pub projects_json: PathBuf,
    pub memories_json: PathBuf,
    pub connectors_json: PathBuf,
    pub manifest_json: PathBuf,
    pub manifest: FridayWorkspaceManifest,
}

impl FridayWorkspaceStore {
    pub fn seed_local_first() -> Self {
        let now = unix_ms();
        let project_id = "friday-local".to_string();
        let memory_id = "local-first-memory".to_string();

        Self {
            version: 1,
            projects: vec![FridayProjectRecord {
                id: project_id.clone(),
                name: "Friday Local Workspace".to_string(),
                instructions:
                    "Use local models and local metasearch first. Ask before using remote providers."
                        .to_string(),
                files: vec![
                    FridayProjectFile {
                        path: "TODO.md".to_string(),
                        purpose: "Active product loop and score evidence.".to_string(),
                        required: true,
                    },
                    FridayProjectFile {
                        path: "CHANGELOG.md".to_string(),
                        purpose: "Operator-visible progress history.".to_string(),
                        required: true,
                    },
                ],
                memory_ids: vec![memory_id.clone()],
                connector_ids: vec![
                    "local-files".to_string(),
                    "metasearch".to_string(),
                    "provider-catalog".to_string(),
                ],
                created_at_unix_ms: now,
                updated_at_unix_ms: now,
            }],
            memories: vec![FridayMemoryRecord {
                id: memory_id,
                project_id,
                summary:
                    "Friday should preserve a local-first default, keep cloud providers explicit, and use metasearch for cited search."
                        .to_string(),
                source: "project-policy".to_string(),
                state: FridayMemoryState::Active,
                created_at_unix_ms: now,
            }],
            connectors: vec![
                FridayConnectorRecord {
                    id: "local-files".to_string(),
                    kind: FridayConnectorKind::LocalFiles,
                    name: "Local files".to_string(),
                    auth_state: FridayConnectorAuthState::LocalOnly,
                    permission_scopes: vec![
                        FridayPermissionScope::LocalProject,
                        FridayPermissionScope::LocalFiles,
                    ],
                    local_boundary:
                        "Read/write stays inside user-approved local project paths.".to_string(),
                    enabled: true,
                },
                FridayConnectorRecord {
                    id: "metasearch".to_string(),
                    kind: FridayConnectorKind::Metasearch,
                    name: "Local metasearch".to_string(),
                    auth_state: FridayConnectorAuthState::LocalOnly,
                    permission_scopes: vec![FridayPermissionScope::Metasearch],
                    local_boundary:
                        "Search runs through the adjacent Rust metasearch service when available."
                            .to_string(),
                    enabled: true,
                },
                FridayConnectorRecord {
                    id: "provider-catalog".to_string(),
                    kind: FridayConnectorKind::ProviderCatalog,
                    name: "Provider catalog".to_string(),
                    auth_state: FridayConnectorAuthState::NotConfigured,
                    permission_scopes: vec![
                        FridayPermissionScope::Connector,
                        FridayPermissionScope::RemoteProvider,
                    ],
                    local_boundary:
                        "Remote providers remain disabled until a user configures credentials and policy."
                            .to_string(),
                    enabled: false,
                },
            ],
        }
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn connector(&self, id: &str) -> Option<&FridayConnectorRecord> {
        self.connectors.iter().find(|connector| connector.id == id)
    }

    pub fn project_memory(&self, project_id: &str) -> Vec<&FridayMemoryRecord> {
        self.memories
            .iter()
            .filter(|memory| memory.project_id == project_id)
            .collect()
    }

    pub fn connectors_for_project(&self, project_id: &str) -> Vec<&FridayConnectorRecord> {
        let Some(project) = self
            .projects
            .iter()
            .find(|project| project.id == project_id)
        else {
            return Vec::new();
        };

        project
            .connector_ids
            .iter()
            .filter_map(|connector_id| self.connector(connector_id))
            .collect()
    }

    pub fn permission_findings(&self) -> Vec<FridayPermissionFinding> {
        let project_ids = self
            .projects
            .iter()
            .map(|project| project.id.as_str())
            .collect::<BTreeSet<_>>();
        let connector_ids = self
            .connectors
            .iter()
            .map(|connector| connector.id.as_str())
            .collect::<BTreeSet<_>>();

        let mut findings = Vec::new();

        for project in &self.projects {
            for connector_id in &project.connector_ids {
                if !connector_ids.contains(connector_id.as_str()) {
                    findings.push(Finding::warning(
                        &project.id,
                        format!("Project references missing connector `{connector_id}`."),
                    ));
                }
            }
        }

        for memory in &self.memories {
            if !project_ids.contains(memory.project_id.as_str()) {
                findings.push(Finding::warning(
                    &memory.id,
                    format!("Memory references missing project `{}`.", memory.project_id),
                ));
            }
            if memory.state == FridayMemoryState::PendingReview {
                findings.push(Finding::info(
                    &memory.id,
                    "Memory is waiting for user review before active use.",
                ));
            }
        }

        for connector in &self.connectors {
            if connector.enabled && connector.auth_state == FridayConnectorAuthState::NotConfigured
            {
                findings.push(Finding::blocked(
                    &connector.id,
                    "Connector is enabled without completed configuration.",
                ));
            }
            if connector
                .permission_scopes
                .contains(&FridayPermissionScope::RemoteProvider)
                && connector.auth_state != FridayConnectorAuthState::Configured
                && connector.enabled
            {
                findings.push(Finding::blocked(
                    &connector.id,
                    "Remote provider connector cannot run without explicit user configuration.",
                ));
            }
        }

        findings
    }

    pub fn write_to_dir(&self, root: impl AsRef<Path>) -> Result<FridayWorkspaceSnapshot> {
        let root = root.as_ref();
        fs::create_dir_all(root)
            .with_context(|| format!("failed to create Friday workspace dir {}", root.display()))?;

        let projects_json = root.join("projects.json");
        let memories_json = root.join("memories.json");
        let connectors_json = root.join("connectors.json");
        let manifest_json = root.join("manifest.json");

        write_json(&projects_json, &self.projects)?;
        write_json(&memories_json, &self.memories)?;
        write_json(&connectors_json, &self.connectors)?;

        let manifest = FridayWorkspaceManifest {
            version: self.version,
            generated_at_unix_ms: unix_ms(),
            projects_json: projects_json.to_string_lossy().into_owned(),
            memories_json: memories_json.to_string_lossy().into_owned(),
            connectors_json: connectors_json.to_string_lossy().into_owned(),
            project_count: self.projects.len(),
            memory_count: self.memories.len(),
            connector_count: self.connectors.len(),
            findings: self.permission_findings(),
        };
        write_json(&manifest_json, &manifest)?;

        Ok(FridayWorkspaceSnapshot {
            root_dir: root.to_path_buf(),
            projects_json,
            memories_json,
            connectors_json,
            manifest_json,
            manifest,
        })
    }

    pub fn read_from_dir(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        Ok(Self {
            version: 1,
            projects: read_json(&root.join("projects.json"))?,
            memories: read_json(&root.join("memories.json"))?,
            connectors: read_json(&root.join("connectors.json"))?,
        })
    }
}

struct Finding;

impl Finding {
    fn info(subject_id: &str, message: impl Into<String>) -> FridayPermissionFinding {
        FridayPermissionFinding {
            severity: FridayBoundarySeverity::Info,
            subject_id: subject_id.to_string(),
            message: message.into(),
        }
    }

    fn warning(subject_id: &str, message: impl Into<String>) -> FridayPermissionFinding {
        FridayPermissionFinding {
            severity: FridayBoundarySeverity::Warning,
            subject_id: subject_id.to_string(),
            message: message.into(),
        }
    }

    fn blocked(subject_id: &str, message: impl Into<String>) -> FridayPermissionFinding {
        FridayPermissionFinding {
            severity: FridayBoundarySeverity::Blocked,
            subject_id: subject_id.to_string(),
            message: message.into(),
        }
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
        .with_context(|| format!("failed to read Friday workspace file {}", path.display()))?;
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
    fn seed_store_preserves_local_first_boundaries() {
        let store = FridayWorkspaceStore::seed_local_first();
        assert_eq!(store.projects.len(), 1);
        assert_eq!(store.memories.len(), 1);
        assert_eq!(store.connectors.len(), 3);
        assert!(store.permission_findings().is_empty());

        let metasearch = store.connector("metasearch").unwrap();
        assert!(metasearch.enabled);
        assert_eq!(metasearch.auth_state, FridayConnectorAuthState::LocalOnly);
        assert!(
            metasearch
                .permission_scopes
                .contains(&FridayPermissionScope::Metasearch)
        );

        let provider_catalog = store.connector("provider-catalog").unwrap();
        assert!(!provider_catalog.enabled);
        assert!(
            provider_catalog
                .permission_scopes
                .contains(&FridayPermissionScope::RemoteProvider)
        );
    }

    #[test]
    fn workspace_store_round_trips_as_separate_json_files() {
        let root = std::env::temp_dir().join(format!("friday-workspace-{}", unix_ms()));
        let _ = fs::remove_dir_all(&root);

        let store = FridayWorkspaceStore::seed_local_first();
        let snapshot = store.write_to_dir(&root).unwrap();
        assert!(snapshot.projects_json.exists());
        assert!(snapshot.memories_json.exists());
        assert!(snapshot.connectors_json.exists());
        assert!(snapshot.manifest_json.exists());

        let restored = FridayWorkspaceStore::read_from_dir(&root).unwrap();
        assert_eq!(restored, store);

        let _ = fs::remove_dir_all(&root);
    }
}
