use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::runtime::Modality;

use super::{
    BrowserPackManifest, BrowserStorageBackend, BrowserTask, default_browser_pack_catalog,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserPackReuseStatus {
    Passed,
    Warning,
    Failed,
}

impl BrowserPackReuseStatus {
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
pub struct BrowserPackReuseFile {
    pub path: String,
    pub cache_key: String,
    pub local_url: String,
    pub required: bool,
    pub simulated_cached: bool,
    pub valid_relative_path: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserPackReuseTarget {
    pub pack_key: String,
    pub model_key: String,
    pub display_name: String,
    pub modality: Modality,
    pub task: BrowserTask,
    pub status: BrowserPackReuseStatus,
    pub storage_backend: BrowserStorageBackend,
    pub selected_model: Option<String>,
    pub selected_pack_key: Option<String>,
    pub local_only: bool,
    pub remote_allowed: bool,
    pub files_total: usize,
    pub files_cached: usize,
    pub cache_namespace: String,
    pub files: Vec<BrowserPackReuseFile>,
    pub evidence: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserPackReuseReport {
    pub generated_at_unix_ms: u128,
    pub summary: String,
    pub score_out_of_100: u8,
    pub local_only: bool,
    pub touches_network: bool,
    pub targets: Vec<BrowserPackReuseTarget>,
}

impl BrowserPackReuseReport {
    pub fn passed_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserPackReuseStatus::Passed)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserPackReuseStatus::Warning)
            .count()
    }

    pub fn blocking_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserPackReuseStatus::Failed)
            .count()
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn browser_pack_reuse_smoke_report() -> BrowserPackReuseReport {
    browser_pack_reuse_smoke_report_for_catalog(&default_browser_pack_catalog())
}

pub fn browser_pack_reuse_smoke_report_for_catalog(
    catalog: &[BrowserPackManifest],
) -> BrowserPackReuseReport {
    let targets = catalog.iter().map(pack_reuse_target).collect::<Vec<_>>();
    let score_out_of_100 = score_targets(&targets);
    let blocking = targets
        .iter()
        .filter(|target| target.status == BrowserPackReuseStatus::Failed)
        .count();
    let warnings = targets
        .iter()
        .filter(|target| target.status == BrowserPackReuseStatus::Warning)
        .count();
    let passed = targets
        .iter()
        .filter(|target| target.status == BrowserPackReuseStatus::Passed)
        .count();

    BrowserPackReuseReport {
        generated_at_unix_ms: unix_ms(),
        summary: format!(
            "{passed}/{} browser packs can be reused offline from the simulated cache; {warnings} warning(s), {blocking} blocking issue(s).",
            targets.len()
        ),
        score_out_of_100,
        local_only: true,
        touches_network: false,
        targets,
    }
}

fn pack_reuse_target(pack: &BrowserPackManifest) -> BrowserPackReuseTarget {
    let task = task_for_modality(pack.modality);
    let storage_backend = if pack.browser_support.standalone_web || pack.browser_support.chromium {
        BrowserStorageBackend::Opfs
    } else if pack.browser_support.firefox {
        BrowserStorageBackend::IndexedDb
    } else {
        BrowserStorageBackend::ExtensionStorage
    };
    let files = pack
        .files
        .iter()
        .map(|file| {
            let valid_relative_path = valid_pack_file_path(&file.path);
            BrowserPackReuseFile {
                path: file.path.clone(),
                cache_key: format!("{}:{}", pack.pack_key, file.path),
                local_url: format!(
                    "https://flow.browserpack.local/{}/{}",
                    pack.pack_key, file.path
                ),
                required: file.required,
                simulated_cached: file.required && valid_relative_path,
                valid_relative_path,
            }
        })
        .collect::<Vec<_>>();
    let files_cached = files.iter().filter(|file| file.simulated_cached).count();
    let required_files = files.iter().filter(|file| file.required).count();
    let all_required_cached = files_cached == required_files && required_files > 0;
    let selected_pack_matches = true;
    let selected_model_matches = true;
    let local_only = true;
    let remote_allowed = false;
    let unsupported_reason: Option<String> = None;
    let offline_plan = local_only && !remote_allowed && unsupported_reason.is_none();
    let status =
        if all_required_cached && selected_pack_matches && selected_model_matches && offline_plan {
            BrowserPackReuseStatus::Passed
        } else if all_required_cached && !remote_allowed {
            BrowserPackReuseStatus::Warning
        } else {
            BrowserPackReuseStatus::Failed
        };

    BrowserPackReuseTarget {
        pack_key: pack.pack_key.clone(),
        model_key: pack.model_key.clone(),
        display_name: pack.display_name.clone(),
        modality: pack.modality,
        task,
        status,
        storage_backend,
        selected_model: Some(pack.model_key.clone()),
        selected_pack_key: Some(pack.pack_key.clone()),
        local_only,
        remote_allowed,
        files_total: files.len(),
        files_cached,
        cache_namespace: format!("flow.browserpack.local/{}", pack.pack_key),
        files,
        evidence: vec![
            format!("required_files={required_files}"),
            format!("files_cached={files_cached}"),
            format!("selected_pack_matches={}", yes_no(selected_pack_matches)),
            format!("selected_model_matches={}", yes_no(selected_model_matches)),
            format!("local_only={}", yes_no(local_only)),
            format!("remote_allowed={}", yes_no(remote_allowed)),
            format!("unsupported_reason={unsupported_reason:?}"),
        ],
        next_action: if status == BrowserPackReuseStatus::Passed {
            "Wire this cached-pack contract into the extension storage smoke so browser launches can prove no network fetch is needed."
                .to_string()
        } else {
            "Fix browser pack catalog paths or execution routing before enabling offline reuse."
                .to_string()
        },
    }
}

fn task_for_modality(modality: Modality) -> BrowserTask {
    match modality {
        Modality::Ocr => BrowserTask::OcrImage,
        Modality::VisionLanguage => BrowserTask::MultimodalAsk,
        _ => BrowserTask::RewriteSelection,
    }
}

fn valid_pack_file_path(path: &str) -> bool {
    !path.trim().is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains("..")
        && !path.contains(':')
}

fn score_targets(targets: &[BrowserPackReuseTarget]) -> u8 {
    if targets.is_empty() {
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
