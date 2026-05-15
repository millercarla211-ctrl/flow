use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayReleaseDeploymentGateDecision, FridayReleaseDeploymentGateReport,
    FridayReleaseDeploymentTarget, read_friday_release_deployment_gate,
    read_friday_release_evidence_export_kit,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseCandidateArchiveEntry {
    pub candidate_id: String,
    pub gate_id: String,
    pub gate_json: String,
    pub export_kit_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub decision: FridayReleaseDeploymentGateDecision,
    pub score_out_of_100: u8,
    pub ready_to_deploy: bool,
    pub target: FridayReleaseDeploymentTarget,
    pub no_deploy_reason_count: usize,
    pub warning_count: usize,
    pub reason_ids: Vec<String>,
    pub export_kit_manifest_sha256: Option<String>,
    pub rollback_note: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseCandidateArchiveDiff {
    pub from_candidate_id: String,
    pub to_candidate_id: String,
    pub score_delta: i16,
    pub decision_changed: bool,
    pub target_changed: bool,
    pub evidence_checksum_changed: bool,
    pub new_blocker_ids: Vec<String>,
    pub resolved_blocker_ids: Vec<String>,
    pub regression: bool,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseCandidateArchive {
    pub archive_id: String,
    pub archive_json: String,
    pub generated_at_unix_ms: u128,
    pub local_only: bool,
    pub candidate_count: usize,
    pub latest_candidate_id: Option<String>,
    pub latest_decision: Option<FridayReleaseDeploymentGateDecision>,
    pub latest_score_out_of_100: Option<u8>,
    pub go_count: usize,
    pub no_go_count: usize,
    pub draft_count: usize,
    pub regression_count: usize,
    pub entries: Vec<FridayReleaseCandidateArchiveEntry>,
    pub diffs: Vec<FridayReleaseCandidateArchiveDiff>,
    pub commands: Vec<String>,
}

impl FridayReleaseCandidateArchive {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_candidate_archive_report(
    archive_path: impl AsRef<Path>,
    entries: Vec<FridayReleaseCandidateArchiveEntry>,
) -> FridayReleaseCandidateArchive {
    let archive_path = archive_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let mut entries = entries;
    entries.sort_by_key(|entry| entry.generated_at_unix_ms);
    entries.dedup_by(|left, right| left.gate_json == right.gate_json);
    let diffs = entries
        .windows(2)
        .map(|window| candidate_diff(&window[0], &window[1]))
        .collect::<Vec<_>>();
    let latest = entries.last();

    FridayReleaseCandidateArchive {
        archive_id: format!("friday-release-candidate-archive-{generated_at_unix_ms}"),
        archive_json: path_string(archive_path),
        generated_at_unix_ms,
        local_only: true,
        candidate_count: entries.len(),
        latest_candidate_id: latest.map(|entry| entry.candidate_id.clone()),
        latest_decision: latest.map(|entry| entry.decision),
        latest_score_out_of_100: latest.map(|entry| entry.score_out_of_100),
        go_count: entries
            .iter()
            .filter(|entry| entry.decision == FridayReleaseDeploymentGateDecision::Go)
            .count(),
        no_go_count: entries
            .iter()
            .filter(|entry| entry.decision == FridayReleaseDeploymentGateDecision::NoGo)
            .count(),
        draft_count: entries
            .iter()
            .filter(|entry| entry.decision == FridayReleaseDeploymentGateDecision::Draft)
            .count(),
        regression_count: diffs.iter().filter(|diff| diff.regression).count(),
        commands: vec![
            format!(
                "flow --friday-release-candidate-archive --archive {} --gate <deployment-gate.json>",
                path_string(archive_path)
            ),
            format!(
                "flow --friday-release-candidate-archive-json --archive {}",
                path_string(archive_path)
            ),
        ],
        entries,
        diffs,
    }
}

pub fn friday_release_candidate_entry_from_gate(
    gate_path: impl AsRef<Path>,
) -> Result<FridayReleaseCandidateArchiveEntry> {
    let gate_path = gate_path.as_ref();
    let gate = read_friday_release_deployment_gate(gate_path)?;
    let export_kit_manifest_sha256 = read_friday_release_evidence_export_kit(
        resolve_candidate_path(gate_path, &gate.export_kit_json),
    )
    .ok()
    .map(|kit| kit.manifest.manifest_sha256);

    Ok(candidate_entry(
        gate_path,
        &gate,
        export_kit_manifest_sha256,
    ))
}

pub fn append_friday_release_candidate_to_archive(
    archive_path: impl AsRef<Path>,
    gate_path: impl AsRef<Path>,
) -> Result<FridayReleaseCandidateArchive> {
    let archive_path = archive_path.as_ref();
    let mut entries = read_friday_release_candidate_archive(archive_path)
        .map(|archive| archive.entries)
        .unwrap_or_default();
    entries.push(friday_release_candidate_entry_from_gate(gate_path)?);
    let archive = friday_release_candidate_archive_report(archive_path, entries);
    write_friday_release_candidate_archive(archive_path, &archive)?;
    Ok(archive)
}

pub fn write_friday_release_candidate_archive(
    archive_path: impl AsRef<Path>,
    archive: &FridayReleaseCandidateArchive,
) -> Result<()> {
    let archive_path = archive_path.as_ref();
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release candidate archive directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(archive_path, archive.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release candidate archive {}",
            archive_path.display()
        )
    })
}

pub fn read_friday_release_candidate_archive(
    archive_path: impl AsRef<Path>,
) -> Result<FridayReleaseCandidateArchive> {
    let archive_path = archive_path.as_ref();
    let bytes = fs::read(archive_path).with_context(|| {
        format!(
            "Could not read Friday release candidate archive {}",
            archive_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release candidate archive {}",
            archive_path.display()
        )
    })
}

fn candidate_entry(
    gate_path: &Path,
    gate: &FridayReleaseDeploymentGateReport,
    export_kit_manifest_sha256: Option<String>,
) -> FridayReleaseCandidateArchiveEntry {
    FridayReleaseCandidateArchiveEntry {
        candidate_id: format!("candidate-{}", gate.gate_id),
        gate_id: gate.gate_id.clone(),
        gate_json: path_string(gate_path),
        export_kit_json: gate.export_kit_json.clone(),
        generated_at_unix_ms: gate.generated_at_unix_ms,
        product_name: gate.product_name.clone(),
        decision: gate.decision,
        score_out_of_100: gate.score_out_of_100,
        ready_to_deploy: gate.ready_to_deploy,
        target: gate.target.clone(),
        no_deploy_reason_count: gate.no_deploy_reason_count,
        warning_count: gate.warning_count,
        reason_ids: gate
            .reasons
            .iter()
            .map(|reason| reason.id.clone())
            .collect(),
        export_kit_manifest_sha256,
        rollback_note: gate.rollback_note.clone(),
        summary: gate.summary.clone(),
    }
}

fn candidate_diff(
    from: &FridayReleaseCandidateArchiveEntry,
    to: &FridayReleaseCandidateArchiveEntry,
) -> FridayReleaseCandidateArchiveDiff {
    let from_reasons = from.reason_ids.iter().cloned().collect::<BTreeSet<_>>();
    let to_reasons = to.reason_ids.iter().cloned().collect::<BTreeSet<_>>();
    let new_blocker_ids = to_reasons
        .difference(&from_reasons)
        .cloned()
        .collect::<Vec<_>>();
    let resolved_blocker_ids = from_reasons
        .difference(&to_reasons)
        .cloned()
        .collect::<Vec<_>>();
    let score_delta = to.score_out_of_100 as i16 - from.score_out_of_100 as i16;
    let decision_changed = from.decision != to.decision;
    let target_changed = from.target.id != to.target.id
        || from.target.provider != to.target.provider
        || from.target.environment != to.target.environment;
    let evidence_checksum_changed =
        from.export_kit_manifest_sha256 != to.export_kit_manifest_sha256;
    let regression = score_delta < 0
        || to.no_deploy_reason_count > from.no_deploy_reason_count
        || !new_blocker_ids.is_empty();

    FridayReleaseCandidateArchiveDiff {
        from_candidate_id: from.candidate_id.clone(),
        to_candidate_id: to.candidate_id.clone(),
        score_delta,
        decision_changed,
        target_changed,
        evidence_checksum_changed,
        new_blocker_ids,
        resolved_blocker_ids,
        regression,
        summary: format!(
            "Candidate score changed by {score_delta}; decision changed={}, target changed={}, evidence checksum changed={}.",
            yes_no(decision_changed),
            yes_no(target_changed),
            yes_no(evidence_checksum_changed)
        ),
    }
}

fn resolve_candidate_path(gate_path: &Path, candidate_path: &str) -> PathBuf {
    let path = PathBuf::from(candidate_path);
    if path.is_absolute() || path.exists() {
        return path;
    }
    gate_path
        .parent()
        .map(|parent| parent.join(path.file_name().unwrap_or_default()))
        .unwrap_or(path)
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
