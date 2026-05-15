use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    FridayDashboardPanelStatus, FridayReleaseCheckpointDecision,
    FridayReleaseCheckpointSignoffDecision, read_friday_release_checkpoint_review_board_report,
    read_friday_release_checkpoint_signoff_ledger,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseCheckpointEvidenceVaultEntryKind {
    CheckpointReviewJson,
    CheckpointSignoffLedgerJson,
    AcknowledgementEvidence,
    CarryoverCommitment,
    ReleaseNotes,
}

impl FridayReleaseCheckpointEvidenceVaultEntryKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::CheckpointReviewJson => "checkpoint-review-json",
            Self::CheckpointSignoffLedgerJson => "checkpoint-signoff-ledger-json",
            Self::AcknowledgementEvidence => "acknowledgement-evidence",
            Self::CarryoverCommitment => "carryover-commitment",
            Self::ReleaseNotes => "release-notes",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseCheckpointEvidenceVaultEntry {
    pub id: String,
    pub label: String,
    pub kind: FridayReleaseCheckpointEvidenceVaultEntryKind,
    pub path: String,
    pub required: bool,
    pub present: bool,
    pub bytes: u64,
    pub sha256: Option<String>,
    pub source_id: String,
    pub summary: String,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseCheckpointEvidenceVault {
    pub vault_id: String,
    pub vault_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub ready_to_archive: bool,
    pub review_id: Option<String>,
    pub review_decision: Option<FridayReleaseCheckpointDecision>,
    pub review_score_out_of_100: Option<u8>,
    pub signoff_ledger_id: Option<String>,
    pub active_signoff_id: Option<String>,
    pub active_decision: Option<FridayReleaseCheckpointSignoffDecision>,
    pub entry_count: usize,
    pub required_count: usize,
    pub present_count: usize,
    pub missing_count: usize,
    pub checksum_count: usize,
    pub acknowledgement_evidence_missing_count: usize,
    pub active_hold_count: usize,
    pub active_carryover_count: usize,
    pub release_gate_blocking_count: usize,
    pub manifest_sha256: String,
    pub entries: Vec<FridayReleaseCheckpointEvidenceVaultEntry>,
    pub attachment_notes_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseCheckpointEvidenceVault {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_checkpoint_evidence_vault_report(
    vault_path: impl AsRef<Path>,
    review_path: impl AsRef<Path>,
    signoff_ledger_path: impl AsRef<Path>,
) -> FridayReleaseCheckpointEvidenceVault {
    let vault_path = vault_path.as_ref();
    let review_path = review_path.as_ref();
    let signoff_ledger_path = signoff_ledger_path.as_ref();
    let generated_at_unix_ms = unix_ms();

    let review = read_friday_release_checkpoint_review_board_report(review_path).ok();
    let signoff_ledger = read_friday_release_checkpoint_signoff_ledger(signoff_ledger_path).ok();

    let mut entries = vec![
        file_entry(
            "checkpoint-review-json",
            "Checkpoint review JSON",
            FridayReleaseCheckpointEvidenceVaultEntryKind::CheckpointReviewJson,
            review_path,
            true,
            review
                .as_ref()
                .map(|report| report.review_id.clone())
                .unwrap_or_default(),
            "Release checkpoint review board used for this vault.",
        ),
        file_entry(
            "checkpoint-signoff-ledger-json",
            "Checkpoint signoff ledger JSON",
            FridayReleaseCheckpointEvidenceVaultEntryKind::CheckpointSignoffLedgerJson,
            signoff_ledger_path,
            true,
            signoff_ledger
                .as_ref()
                .map(|ledger| ledger.ledger_id.clone())
                .unwrap_or_default(),
            "Checkpoint signoff history used for this vault.",
        ),
    ];

    if let Some(ledger) = &signoff_ledger {
        for record in &ledger.records {
            let evidence_path = record.acknowledgement_evidence_path.trim();
            let requires_ack_evidence = record.review_acknowledgement_blocker_count > 0
                || record.decision == FridayReleaseCheckpointSignoffDecision::SignedOff;
            if requires_ack_evidence || !evidence_path.is_empty() {
                entries.push(file_entry_from_text_path(
                    &format!("acknowledgement-evidence-{}", record.signoff_id),
                    "Acknowledgement evidence",
                    FridayReleaseCheckpointEvidenceVaultEntryKind::AcknowledgementEvidence,
                    evidence_path,
                    requires_ack_evidence,
                    record.signoff_id.clone(),
                    "Operator acknowledgement evidence attached to a checkpoint signoff.",
                ));
            }

            if record.active_carryover || !record.carryover_commitment.trim().is_empty() {
                entries.push(inline_entry(
                    &format!("carryover-commitment-{}", record.signoff_id),
                    "Carryover commitment",
                    FridayReleaseCheckpointEvidenceVaultEntryKind::CarryoverCommitment,
                    &format!("inline://carryover-commitments/{}", record.signoff_id),
                    record.active_carryover,
                    record.signoff_id.clone(),
                    record.carryover_commitment.trim(),
                    "Carryover commitment captured with the checkpoint signoff.",
                ));
            }
        }

        entries.push(inline_entry(
            "checkpoint-release-notes",
            "Checkpoint release notes",
            FridayReleaseCheckpointEvidenceVaultEntryKind::ReleaseNotes,
            "inline://release-notes/checkpoint-signoff-ledger",
            true,
            ledger.ledger_id.clone(),
            ledger.release_notes_copy.trim(),
            "Copyable release notes generated from checkpoint signoff history.",
        ));
    } else {
        entries.push(inline_entry(
            "checkpoint-release-notes",
            "Checkpoint release notes",
            FridayReleaseCheckpointEvidenceVaultEntryKind::ReleaseNotes,
            "inline://release-notes/checkpoint-signoff-ledger",
            true,
            String::new(),
            "",
            "Copyable release notes generated from checkpoint signoff history.",
        ));
    }

    entries.sort_by(|left, right| {
        left.kind
            .label()
            .cmp(right.kind.label())
            .then_with(|| left.id.cmp(&right.id))
    });
    entries.dedup_by(|left, right| left.id == right.id);

    let required_count = entries.iter().filter(|entry| entry.required).count();
    let present_count = entries.iter().filter(|entry| entry.present).count();
    let missing_count = entries
        .iter()
        .filter(|entry| entry.required && !entry.present)
        .count();
    let checksum_count = entries
        .iter()
        .filter(|entry| entry.sha256.is_some())
        .count();
    let acknowledgement_evidence_missing_count = signoff_ledger
        .as_ref()
        .map(|ledger| ledger.acknowledgement_evidence_missing_count)
        .unwrap_or_else(|| {
            entries
                .iter()
                .filter(|entry| {
                    entry.kind
                        == FridayReleaseCheckpointEvidenceVaultEntryKind::AcknowledgementEvidence
                        && entry.required
                        && !entry.present
                })
                .count()
        });
    let active_hold_count = signoff_ledger
        .as_ref()
        .map(|ledger| ledger.active_hold_count)
        .unwrap_or(0);
    let active_carryover_count = signoff_ledger
        .as_ref()
        .map(|ledger| ledger.active_carryover_count)
        .unwrap_or(0);
    let release_gate_blocking_count = signoff_ledger
        .as_ref()
        .map(|ledger| ledger.release_gate_blocking_count)
        .unwrap_or_else(|| {
            review
                .as_ref()
                .map(|report| report.release_gate_blocking_count)
                .unwrap_or(0)
        });
    let ready_to_archive = missing_count == 0 && acknowledgement_evidence_missing_count == 0;
    let status = if !ready_to_archive || release_gate_blocking_count > 0 || active_hold_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if active_carryover_count > 0 {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };
    let manifest_sha256 = manifest_signature(&entries);
    let vault_json = path_string(vault_path);
    let review_id = review.as_ref().map(|report| report.review_id.clone());
    let signoff_ledger_id = signoff_ledger
        .as_ref()
        .map(|ledger| ledger.ledger_id.clone());
    let active_signoff_id = signoff_ledger
        .as_ref()
        .and_then(|ledger| ledger.active_signoff_id.clone());
    let active_decision = signoff_ledger
        .as_ref()
        .and_then(|ledger| ledger.active_decision);

    let commands = vec![
        format!(
            "flow --friday-release-checkpoint-evidence-vault --output {} --review {} --signoff-ledger {}",
            vault_json,
            path_string(review_path),
            path_string(signoff_ledger_path)
        ),
        format!(
            "flow --friday-release-checkpoint-evidence-vault-json --output {} --review {} --signoff-ledger {}",
            vault_json,
            path_string(review_path),
            path_string(signoff_ledger_path)
        ),
    ];
    let attachment_notes_copy = attachment_notes_copy(
        &vault_json,
        ready_to_archive,
        &manifest_sha256,
        &entries,
        review_id.as_deref(),
        active_decision,
        missing_count,
        release_gate_blocking_count,
    );

    FridayReleaseCheckpointEvidenceVault {
        vault_id: format!("friday-release-checkpoint-evidence-vault-{generated_at_unix_ms}"),
        vault_json,
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        ready_to_archive,
        review_id,
        review_decision: review.as_ref().map(|report| report.decision),
        review_score_out_of_100: review.as_ref().map(|report| report.score_out_of_100),
        signoff_ledger_id,
        active_signoff_id,
        active_decision,
        entry_count: entries.len(),
        required_count,
        present_count,
        missing_count,
        checksum_count,
        acknowledgement_evidence_missing_count,
        active_hold_count,
        active_carryover_count,
        release_gate_blocking_count,
        manifest_sha256,
        summary: format!(
            "Friday checkpoint evidence vault has {} entries, {} missing required item(s), {} checksum(s), and {} release gate block(s).",
            entries.len(),
            missing_count,
            checksum_count,
            release_gate_blocking_count
        ),
        entries,
        attachment_notes_copy,
        commands,
    }
}

pub fn write_friday_release_checkpoint_evidence_vault(
    vault_path: impl AsRef<Path>,
    vault: &FridayReleaseCheckpointEvidenceVault,
) -> Result<()> {
    let vault_path = vault_path.as_ref();
    if let Some(parent) = vault_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday checkpoint evidence vault directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(vault_path, vault.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday checkpoint evidence vault {}",
            vault_path.display()
        )
    })
}

pub fn read_friday_release_checkpoint_evidence_vault(
    vault_path: impl AsRef<Path>,
) -> Result<FridayReleaseCheckpointEvidenceVault> {
    let vault_path = vault_path.as_ref();
    let bytes = fs::read(vault_path).with_context(|| {
        format!(
            "Could not read Friday checkpoint evidence vault {}",
            vault_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday checkpoint evidence vault {}",
            vault_path.display()
        )
    })
}

fn file_entry(
    id: &str,
    label: &str,
    kind: FridayReleaseCheckpointEvidenceVaultEntryKind,
    path: &Path,
    required: bool,
    source_id: String,
    summary: &str,
) -> FridayReleaseCheckpointEvidenceVaultEntry {
    file_entry_from_text_path(
        id,
        label,
        kind,
        &path_string(path),
        required,
        source_id,
        summary,
    )
}

fn file_entry_from_text_path(
    id: &str,
    label: &str,
    kind: FridayReleaseCheckpointEvidenceVaultEntryKind,
    path: &str,
    required: bool,
    source_id: String,
    summary: &str,
) -> FridayReleaseCheckpointEvidenceVaultEntry {
    let trimmed_path = path.trim();
    if trimmed_path.is_empty() {
        return FridayReleaseCheckpointEvidenceVaultEntry {
            id: id.to_string(),
            label: label.to_string(),
            kind,
            path: String::new(),
            required,
            present: false,
            bytes: 0,
            sha256: None,
            source_id,
            summary: summary.to_string(),
            warning: required.then(|| format!("Required evidence is missing: {label}.")),
        };
    }

    match fs::read(trimmed_path) {
        Ok(bytes) => FridayReleaseCheckpointEvidenceVaultEntry {
            id: id.to_string(),
            label: label.to_string(),
            kind,
            path: path_string(Path::new(trimmed_path)),
            required,
            present: true,
            bytes: bytes.len() as u64,
            sha256: Some(sha256_hex(&bytes)),
            source_id,
            summary: summary.to_string(),
            warning: None,
        },
        Err(_) => FridayReleaseCheckpointEvidenceVaultEntry {
            id: id.to_string(),
            label: label.to_string(),
            kind,
            path: trimmed_path.replace('\\', "/"),
            required,
            present: false,
            bytes: 0,
            sha256: None,
            source_id,
            summary: summary.to_string(),
            warning: required.then(|| format!("Required evidence is missing: {label}.")),
        },
    }
}

fn inline_entry(
    id: &str,
    label: &str,
    kind: FridayReleaseCheckpointEvidenceVaultEntryKind,
    path: &str,
    required: bool,
    source_id: String,
    content: &str,
    summary: &str,
) -> FridayReleaseCheckpointEvidenceVaultEntry {
    let bytes = content.as_bytes();
    let present = !content.trim().is_empty();
    FridayReleaseCheckpointEvidenceVaultEntry {
        id: id.to_string(),
        label: label.to_string(),
        kind,
        path: path.to_string(),
        required,
        present,
        bytes: bytes.len() as u64,
        sha256: present.then(|| sha256_hex(bytes)),
        source_id,
        summary: summary.to_string(),
        warning: (required && !present).then(|| format!("Required evidence is missing: {label}.")),
    }
}

fn manifest_signature(entries: &[FridayReleaseCheckpointEvidenceVaultEntry]) -> String {
    let mut input = entries
        .iter()
        .map(|entry| {
            format!(
                "{}:{}:{}:{}:{}:{}",
                entry.id,
                entry.kind.label(),
                entry.path,
                entry.present,
                entry.bytes,
                entry.sha256.as_deref().unwrap_or("missing")
            )
        })
        .collect::<Vec<_>>();
    input.sort();
    sha256_hex(input.join("\n").as_bytes())
}

fn attachment_notes_copy(
    vault_json: &str,
    ready_to_archive: bool,
    manifest_sha256: &str,
    entries: &[FridayReleaseCheckpointEvidenceVaultEntry],
    review_id: Option<&str>,
    active_decision: Option<FridayReleaseCheckpointSignoffDecision>,
    missing_count: usize,
    release_gate_blocking_count: usize,
) -> String {
    let mut lines = vec![
        format!("Friday checkpoint evidence vault: {vault_json}"),
        format!(
            "Status: {}",
            if ready_to_archive {
                "ready to archive"
            } else {
                "needs evidence"
            }
        ),
        format!("Manifest checksum: {manifest_sha256}"),
        format!("Review: {}", review_id.unwrap_or("missing")),
        format!(
            "Active signoff: {}",
            active_decision
                .map(|decision| decision.label())
                .unwrap_or("none")
        ),
        format!("Missing required evidence: {missing_count}"),
        format!("Release gate blocks: {release_gate_blocking_count}"),
        "Attach these evidence paths:".to_string(),
    ];

    for entry in entries.iter().filter(|entry| entry.present).take(10) {
        lines.push(format!(
            "- {} [{}] {}",
            entry.label,
            entry.kind.label(),
            entry.path
        ));
    }
    lines.join("\n")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
