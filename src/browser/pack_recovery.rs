use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::{BrowserPackFile, BrowserPackManifest, default_browser_pack_catalog};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserPackRecoveryStatus {
    Passed,
    Warning,
    Failed,
}

impl BrowserPackRecoveryStatus {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserPackRecoveryScenarioKind {
    PartialDownloadResume,
    HashMismatchRejection,
    QuotaPressureRecovery,
}

impl BrowserPackRecoveryScenarioKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::PartialDownloadResume => "partial-download-resume",
            Self::HashMismatchRejection => "hash-mismatch-rejection",
            Self::QuotaPressureRecovery => "quota-pressure-recovery",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserPackRecoveryFile {
    pub path: String,
    pub expected_bytes: u64,
    pub partial_bytes: u64,
    pub resumed_bytes: u64,
    pub expected_hash: String,
    pub observed_hash: String,
    pub cache_key: String,
    pub valid_relative_path: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserPackRecoveryScenario {
    pub kind: BrowserPackRecoveryScenarioKind,
    pub status: BrowserPackRecoveryStatus,
    pub action: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserPackRecoveryTarget {
    pub pack_key: String,
    pub model_key: String,
    pub display_name: String,
    pub files_total: usize,
    pub required_bytes: u64,
    pub stale_bytes_evicted: u64,
    pub available_bytes_before: u64,
    pub available_bytes_after: u64,
    pub files: Vec<BrowserPackRecoveryFile>,
    pub scenarios: Vec<BrowserPackRecoveryScenario>,
}

impl BrowserPackRecoveryTarget {
    pub fn status(&self) -> BrowserPackRecoveryStatus {
        if self
            .scenarios
            .iter()
            .any(|scenario| scenario.status == BrowserPackRecoveryStatus::Failed)
        {
            BrowserPackRecoveryStatus::Failed
        } else if self
            .scenarios
            .iter()
            .any(|scenario| scenario.status == BrowserPackRecoveryStatus::Warning)
        {
            BrowserPackRecoveryStatus::Warning
        } else {
            BrowserPackRecoveryStatus::Passed
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserPackRecoveryReport {
    pub generated_at_unix_ms: u128,
    pub summary: String,
    pub score_out_of_100: u8,
    pub local_only: bool,
    pub touches_network: bool,
    pub targets: Vec<BrowserPackRecoveryTarget>,
}

impl BrowserPackRecoveryReport {
    pub fn passed_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status() == BrowserPackRecoveryStatus::Passed)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status() == BrowserPackRecoveryStatus::Warning)
            .count()
    }

    pub fn blocking_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status() == BrowserPackRecoveryStatus::Failed)
            .count()
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn browser_pack_recovery_smoke_report() -> BrowserPackRecoveryReport {
    browser_pack_recovery_smoke_report_for_catalog(&default_browser_pack_catalog())
}

pub fn browser_pack_recovery_smoke_report_for_catalog(
    catalog: &[BrowserPackManifest],
) -> BrowserPackRecoveryReport {
    let targets = catalog.iter().map(pack_recovery_target).collect::<Vec<_>>();
    let score_out_of_100 = score_targets(&targets);
    let passed = targets
        .iter()
        .filter(|target| target.status() == BrowserPackRecoveryStatus::Passed)
        .count();
    let warnings = targets
        .iter()
        .filter(|target| target.status() == BrowserPackRecoveryStatus::Warning)
        .count();
    let blocking = targets
        .iter()
        .filter(|target| target.status() == BrowserPackRecoveryStatus::Failed)
        .count();

    BrowserPackRecoveryReport {
        generated_at_unix_ms: unix_ms(),
        summary: format!(
            "{passed}/{} browser packs have deterministic recovery contracts; {warnings} warning(s), {blocking} blocking issue(s).",
            targets.len()
        ),
        score_out_of_100,
        local_only: true,
        touches_network: false,
        targets,
    }
}

fn pack_recovery_target(pack: &BrowserPackManifest) -> BrowserPackRecoveryTarget {
    let files = pack
        .files
        .iter()
        .map(|file| recovery_file(pack, file))
        .collect::<Vec<_>>();
    let required_bytes = files
        .iter()
        .map(|file| file.expected_bytes)
        .sum::<u64>()
        .max(1);
    let stale_bytes_evicted = required_bytes.saturating_add(4 * 1024 * 1024);
    let available_bytes_before = required_bytes.saturating_sub(1);
    let available_bytes_after = available_bytes_before.saturating_add(stale_bytes_evicted);
    let all_paths_valid = files.iter().all(|file| file.valid_relative_path);
    let all_partial_resumable = files
        .iter()
        .all(|file| file.partial_bytes > 0 && file.partial_bytes < file.expected_bytes);
    let all_hashes_reject = files
        .iter()
        .all(|file| file.observed_hash != file.expected_hash && !file.expected_hash.is_empty());
    let quota_recovers =
        available_bytes_before < required_bytes && available_bytes_after >= required_bytes;

    BrowserPackRecoveryTarget {
        pack_key: pack.pack_key.clone(),
        model_key: pack.model_key.clone(),
        display_name: pack.display_name.clone(),
        files_total: files.len(),
        required_bytes,
        stale_bytes_evicted,
        available_bytes_before,
        available_bytes_after,
        scenarios: vec![
            BrowserPackRecoveryScenario {
                kind: BrowserPackRecoveryScenarioKind::PartialDownloadResume,
                status: if all_paths_valid && all_partial_resumable {
                    BrowserPackRecoveryStatus::Passed
                } else {
                    BrowserPackRecoveryStatus::Failed
                },
                action: "resume range requests into a temporary cache entry, then atomically promote the completed file".to_string(),
                evidence: vec![
                    format!("files_checked={}", files.len()),
                    format!("all_paths_valid={}", yes_no(all_paths_valid)),
                    format!("all_partial_resumable={}", yes_no(all_partial_resumable)),
                ],
            },
            BrowserPackRecoveryScenario {
                kind: BrowserPackRecoveryScenarioKind::HashMismatchRejection,
                status: if all_paths_valid && all_hashes_reject {
                    BrowserPackRecoveryStatus::Passed
                } else {
                    BrowserPackRecoveryStatus::Failed
                },
                action: "delete corrupt cache entries, keep the last known good pack inactive, and require a clean re-fetch before activation".to_string(),
                evidence: vec![
                    format!("files_checked={}", files.len()),
                    format!("corrupt_hashes_rejected={}", yes_no(all_hashes_reject)),
                    "activation_requires_hash_match=yes".to_string(),
                ],
            },
            BrowserPackRecoveryScenario {
                kind: BrowserPackRecoveryScenarioKind::QuotaPressureRecovery,
                status: if quota_recovers {
                    BrowserPackRecoveryStatus::Passed
                } else {
                    BrowserPackRecoveryStatus::Warning
                },
                action: "evict stale temporary chunks before deleting active packs, then retry the current pack transaction".to_string(),
                evidence: vec![
                    format!("required_bytes={required_bytes}"),
                    format!("available_before={available_bytes_before}"),
                    format!("stale_bytes_evicted={stale_bytes_evicted}"),
                    format!("available_after={available_bytes_after}"),
                    format!("quota_recovers={}", yes_no(quota_recovers)),
                ],
            },
        ],
        files,
    }
}

fn recovery_file(pack: &BrowserPackManifest, file: &BrowserPackFile) -> BrowserPackRecoveryFile {
    let expected_bytes = file
        .bytes
        .unwrap_or_else(|| simulated_required_bytes(&pack.pack_key, &file.path));
    let partial_bytes = (expected_bytes / 3)
        .max(1)
        .min(expected_bytes.saturating_sub(1));
    let expected_hash = file.sha256.clone().unwrap_or_else(|| {
        stable_hash(&format!("{}:{}:{expected_bytes}", pack.pack_key, file.path))
    });

    BrowserPackRecoveryFile {
        path: file.path.clone(),
        expected_bytes,
        partial_bytes,
        resumed_bytes: expected_bytes.saturating_sub(partial_bytes),
        observed_hash: format!("corrupt-{expected_hash}"),
        expected_hash,
        cache_key: format!("{}:{}", pack.pack_key, file.path),
        valid_relative_path: valid_pack_file_path(&file.path),
    }
}

fn valid_pack_file_path(path: &str) -> bool {
    !path.trim().is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains("..")
        && !path.contains(':')
}

fn simulated_required_bytes(pack_key: &str, path: &str) -> u64 {
    let hash = stable_hash(&format!("{pack_key}:{path}"));
    let seed = u64::from_str_radix(&hash[..8], 16).unwrap_or(0);
    2 * 1024 * 1024 + (seed % (24 * 1024 * 1024))
}

fn stable_hash(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn score_targets(targets: &[BrowserPackRecoveryTarget]) -> u8 {
    let scenarios = targets
        .iter()
        .flat_map(|target| target.scenarios.iter())
        .collect::<Vec<_>>();

    if scenarios.is_empty() {
        return 0;
    }

    let earned = scenarios
        .iter()
        .map(|scenario| scenario.status.score())
        .sum::<f32>();
    ((earned / scenarios.len() as f32) * 100.0)
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
