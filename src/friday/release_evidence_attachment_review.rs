use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseCheckpointEvidenceVault,
    FridayReleaseCheckpointEvidenceVaultEntry, FridayReleaseCheckpointEvidenceVaultEntryKind,
    read_friday_release_checkpoint_evidence_vault,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseEvidenceAttachmentState {
    Ready,
    Missing,
    InlineOnly,
    ChecksumMissing,
    Blocked,
}

impl FridayReleaseEvidenceAttachmentState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Missing => "missing",
            Self::InlineOnly => "inline-only",
            Self::ChecksumMissing => "checksum-missing",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseEvidenceAttachmentReviewItem {
    pub id: String,
    pub vault_entry_id: String,
    pub label: String,
    pub kind: FridayReleaseCheckpointEvidenceVaultEntryKind,
    pub path: String,
    pub state: FridayReleaseEvidenceAttachmentState,
    pub required: bool,
    pub present: bool,
    pub attachable: bool,
    pub bytes: u64,
    pub sha256: Option<String>,
    pub source_id: String,
    pub release_gate_blocking: bool,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseEvidenceAttachmentReview {
    pub review_id: String,
    pub review_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub ready_for_handoff: bool,
    pub vault_id: String,
    pub vault_json: String,
    pub manifest_sha256: String,
    pub item_count: usize,
    pub attachable_count: usize,
    pub missing_count: usize,
    pub inline_only_count: usize,
    pub checksum_missing_count: usize,
    pub blocked_count: usize,
    pub release_gate_blocking_count: usize,
    pub first_blocker: Option<String>,
    pub items: Vec<FridayReleaseEvidenceAttachmentReviewItem>,
    pub handoff_notes_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseEvidenceAttachmentReview {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_evidence_attachment_review_report(
    review_path: impl AsRef<Path>,
    vault_path: impl AsRef<Path>,
) -> FridayReleaseEvidenceAttachmentReview {
    let review_path = review_path.as_ref();
    let vault_path = vault_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let vault = read_friday_release_checkpoint_evidence_vault(vault_path).ok();
    let fallback = fallback_vault(vault_path);
    let vault = vault.as_ref().unwrap_or(&fallback);
    let mut items = vault
        .entries
        .iter()
        .map(|entry| attachment_item(entry, vault))
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        state_rank(left.state)
            .cmp(&state_rank(right.state))
            .then_with(|| left.kind.label().cmp(right.kind.label()))
            .then_with(|| left.id.cmp(&right.id))
    });

    let attachable_count = items.iter().filter(|item| item.attachable).count();
    let missing_count = items
        .iter()
        .filter(|item| {
            matches!(
                item.state,
                FridayReleaseEvidenceAttachmentState::Missing
                    | FridayReleaseEvidenceAttachmentState::Blocked
            )
        })
        .count();
    let inline_only_count = state_count(&items, FridayReleaseEvidenceAttachmentState::InlineOnly);
    let checksum_missing_count = state_count(
        &items,
        FridayReleaseEvidenceAttachmentState::ChecksumMissing,
    );
    let blocked_count = state_count(&items, FridayReleaseEvidenceAttachmentState::Blocked);
    let release_gate_blocking_count = items
        .iter()
        .filter(|item| item.release_gate_blocking)
        .count()
        .max(vault.release_gate_blocking_count);
    let first_blocker = items
        .iter()
        .find(|item| item.release_gate_blocking || !item.attachable && item.required)
        .map(|item| item.next_action.clone());
    let ready_for_handoff = blocked_count == 0
        && missing_count == 0
        && checksum_missing_count == 0
        && release_gate_blocking_count == 0;
    let status = if blocked_count > 0 || missing_count > 0 || release_gate_blocking_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if inline_only_count > 0 || checksum_missing_count > 0 {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };
    let review_json = path_string(review_path);

    FridayReleaseEvidenceAttachmentReview {
        review_id: format!("friday-release-evidence-attachment-review-{generated_at_unix_ms}"),
        review_json: review_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        ready_for_handoff,
        vault_id: vault.vault_id.clone(),
        vault_json: path_string(vault_path),
        manifest_sha256: vault.manifest_sha256.clone(),
        item_count: items.len(),
        attachable_count,
        missing_count,
        inline_only_count,
        checksum_missing_count,
        blocked_count,
        release_gate_blocking_count,
        first_blocker,
        handoff_notes_copy: handoff_notes_copy(
            &review_json,
            ready_for_handoff,
            &vault.manifest_sha256,
            &items,
        ),
        summary: format!(
            "Friday release evidence attachment review has {} item(s), {} attachable, {} missing, {} inline-only, {} checksum-missing, and {} blocked.",
            items.len(),
            attachable_count,
            missing_count,
            inline_only_count,
            checksum_missing_count,
            blocked_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-evidence-attachment-review --output {} --vault {}",
                review_json,
                path_string(vault_path)
            ),
            format!(
                "flow --friday-release-evidence-attachment-review-json --output {} --vault {}",
                review_json,
                path_string(vault_path)
            ),
        ],
        items,
    }
}

pub fn write_friday_release_evidence_attachment_review(
    review_path: impl AsRef<Path>,
    review: &FridayReleaseEvidenceAttachmentReview,
) -> Result<()> {
    let review_path = review_path.as_ref();
    if let Some(parent) = review_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release evidence attachment review directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(review_path, review.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release evidence attachment review {}",
            review_path.display()
        )
    })
}

pub fn read_friday_release_evidence_attachment_review(
    review_path: impl AsRef<Path>,
) -> Result<FridayReleaseEvidenceAttachmentReview> {
    let review_path = review_path.as_ref();
    let bytes = fs::read(review_path).with_context(|| {
        format!(
            "Could not read Friday release evidence attachment review {}",
            review_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release evidence attachment review {}",
            review_path.display()
        )
    })
}

fn attachment_item(
    entry: &FridayReleaseCheckpointEvidenceVaultEntry,
    vault: &FridayReleaseCheckpointEvidenceVault,
) -> FridayReleaseEvidenceAttachmentReviewItem {
    let release_gate_blocking = entry.required
        && !entry.present
        && (vault.release_gate_blocking_count > 0 || vault.active_hold_count > 0);
    let state = attachment_state(entry, release_gate_blocking);
    let attachable = matches!(state, FridayReleaseEvidenceAttachmentState::Ready);
    let next_action = match state {
        FridayReleaseEvidenceAttachmentState::Ready => {
            format!("Attach {} to the release handoff.", entry.path)
        }
        FridayReleaseEvidenceAttachmentState::Missing => {
            format!(
                "Attach missing evidence for {} before handoff.",
                entry.label
            )
        }
        FridayReleaseEvidenceAttachmentState::InlineOnly => {
            format!(
                "Review inline note {} and paste it into the handoff.",
                entry.label
            )
        }
        FridayReleaseEvidenceAttachmentState::ChecksumMissing => {
            format!(
                "Regenerate {} so the attachment has a checksum.",
                entry.label
            )
        }
        FridayReleaseEvidenceAttachmentState::Blocked => {
            format!(
                "Resolve blocking evidence for {} before release handoff.",
                entry.label
            )
        }
    };

    FridayReleaseEvidenceAttachmentReviewItem {
        id: format!("attachment-review-{}", entry.id),
        vault_entry_id: entry.id.clone(),
        label: entry.label.clone(),
        kind: entry.kind,
        path: entry.path.clone(),
        state,
        required: entry.required,
        present: entry.present,
        attachable,
        bytes: entry.bytes,
        sha256: entry.sha256.clone(),
        source_id: entry.source_id.clone(),
        release_gate_blocking,
        summary: entry
            .warning
            .clone()
            .unwrap_or_else(|| entry.summary.clone()),
        next_action,
    }
}

fn attachment_state(
    entry: &FridayReleaseCheckpointEvidenceVaultEntry,
    release_gate_blocking: bool,
) -> FridayReleaseEvidenceAttachmentState {
    if release_gate_blocking {
        return FridayReleaseEvidenceAttachmentState::Blocked;
    }
    if !entry.present {
        return FridayReleaseEvidenceAttachmentState::Missing;
    }
    if entry.path.starts_with("inline://") {
        return FridayReleaseEvidenceAttachmentState::InlineOnly;
    }
    if entry.sha256.is_none() {
        return FridayReleaseEvidenceAttachmentState::ChecksumMissing;
    }
    FridayReleaseEvidenceAttachmentState::Ready
}

fn state_count(
    items: &[FridayReleaseEvidenceAttachmentReviewItem],
    state: FridayReleaseEvidenceAttachmentState,
) -> usize {
    items.iter().filter(|item| item.state == state).count()
}

fn state_rank(state: FridayReleaseEvidenceAttachmentState) -> u8 {
    match state {
        FridayReleaseEvidenceAttachmentState::Blocked => 0,
        FridayReleaseEvidenceAttachmentState::Missing => 1,
        FridayReleaseEvidenceAttachmentState::ChecksumMissing => 2,
        FridayReleaseEvidenceAttachmentState::InlineOnly => 3,
        FridayReleaseEvidenceAttachmentState::Ready => 4,
    }
}

fn handoff_notes_copy(
    review_json: &str,
    ready_for_handoff: bool,
    manifest_sha256: &str,
    items: &[FridayReleaseEvidenceAttachmentReviewItem],
) -> String {
    let mut lines = vec![
        format!("Friday release evidence attachment review: {review_json}"),
        format!(
            "Status: {}",
            if ready_for_handoff {
                "ready for handoff"
            } else {
                "needs attachment review"
            }
        ),
        format!("Manifest checksum: {manifest_sha256}"),
        "Attachment actions:".to_string(),
    ];

    for item in items.iter().take(10) {
        lines.push(format!(
            "- {} [{}] {}",
            item.label,
            item.state.label(),
            item.next_action
        ));
    }
    lines.join("\n")
}

fn fallback_vault(vault_path: &Path) -> FridayReleaseCheckpointEvidenceVault {
    FridayReleaseCheckpointEvidenceVault {
        vault_id: "missing-checkpoint-evidence-vault".to_string(),
        vault_json: path_string(vault_path),
        generated_at_unix_ms: unix_ms(),
        product_name: "Friday".to_string(),
        local_only: true,
        status: FridayDashboardPanelStatus::Blocked,
        ready_to_archive: false,
        review_id: None,
        review_decision: None,
        review_score_out_of_100: None,
        signoff_ledger_id: None,
        active_signoff_id: None,
        active_decision: None,
        entry_count: 0,
        required_count: 0,
        present_count: 0,
        missing_count: 1,
        checksum_count: 0,
        acknowledgement_evidence_missing_count: 1,
        active_hold_count: 1,
        active_carryover_count: 0,
        release_gate_blocking_count: 1,
        manifest_sha256: "missing".to_string(),
        entries: vec![FridayReleaseCheckpointEvidenceVaultEntry {
            id: "checkpoint-evidence-vault-json".to_string(),
            label: "Checkpoint evidence vault JSON".to_string(),
            kind: FridayReleaseCheckpointEvidenceVaultEntryKind::CheckpointReviewJson,
            path: path_string(vault_path),
            required: true,
            present: false,
            bytes: 0,
            sha256: None,
            source_id: String::new(),
            summary: "Checkpoint evidence vault JSON could not be loaded.".to_string(),
            warning: Some(
                "Required evidence is missing: checkpoint evidence vault JSON.".to_string(),
            ),
        }],
        attachment_notes_copy: String::new(),
        summary: "Checkpoint evidence vault JSON could not be loaded.".to_string(),
        commands: Vec::new(),
    }
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
