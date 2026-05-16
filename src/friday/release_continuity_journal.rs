use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayReleaseClosureLedger, FridayReleaseClosureState, FridayReleaseReceiptReviewDecision,
    read_friday_release_closure_ledger,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseContinuityEntryKind {
    Outcome,
    Carryover,
    BlockerPattern,
    NextReleaseNote,
    OperatorDecision,
    SupersededHistory,
}

impl FridayReleaseContinuityEntryKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Outcome => "outcome",
            Self::Carryover => "carryover",
            Self::BlockerPattern => "blocker-pattern",
            Self::NextReleaseNote => "next-release-note",
            Self::OperatorDecision => "operator-decision",
            Self::SupersededHistory => "superseded-history",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "outcome" | "result" | "release-outcome" => Ok(Self::Outcome),
            "carryover" | "carry-over" | "carried-over" => Ok(Self::Carryover),
            "blocker-pattern" | "blocker" | "blockers" | "pattern" => Ok(Self::BlockerPattern),
            "next-release-note" | "next-release" | "next" | "note" => Ok(Self::NextReleaseNote),
            "operator-decision" | "decision" | "operator" => Ok(Self::OperatorDecision),
            "superseded-history" | "superseded" | "history" => Ok(Self::SupersededHistory),
            other => anyhow::bail!(
                "Unknown Friday release continuity entry kind `{}`. Use outcome, carryover, blocker-pattern, next-release-note, operator-decision, or superseded-history.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseContinuityRequest {
    pub entry_kind: FridayReleaseContinuityEntryKind,
    pub operator: String,
    pub note: String,
    pub owner: Option<String>,
    pub next_release_target: Option<String>,
    pub supersedes_entry_id: Option<String>,
}

impl Default for FridayReleaseContinuityRequest {
    fn default() -> Self {
        Self {
            entry_kind: FridayReleaseContinuityEntryKind::Outcome,
            operator: "operator".to_string(),
            note: "Recorded local release continuity note.".to_string(),
            owner: None,
            next_release_target: None,
            supersedes_entry_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseContinuityEntry {
    pub entry_id: String,
    pub closure_ledger_id: String,
    pub closure_ledger_json: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub entry_kind: FridayReleaseContinuityEntryKind,
    pub operator: String,
    pub note: String,
    pub owner: Option<String>,
    pub next_release_target: Option<String>,
    pub supersedes_entry_id: Option<String>,
    pub latest_closure_id: Option<String>,
    pub latest_closure_state: Option<FridayReleaseClosureState>,
    pub latest_receipt_review_id: Option<String>,
    pub latest_review_decision: Option<FridayReleaseReceiptReviewDecision>,
    pub closure_record_count: usize,
    pub closed_outcome_count: usize,
    pub carryover_outcome_count: usize,
    pub blocked_outcome_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub recurring_blocker_count: usize,
    pub carryover_commitment_count: usize,
    pub active: bool,
    pub externally_mutated_by_friday: bool,
    pub entry_notes_copy: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseContinuityJournal {
    pub journal_id: String,
    pub journal_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub entry_count: usize,
    pub outcome_entry_count: usize,
    pub carryover_entry_count: usize,
    pub blocker_pattern_count: usize,
    pub next_release_note_count: usize,
    pub operator_decision_count: usize,
    pub superseded_history_count: usize,
    pub active_entry_id: Option<String>,
    pub latest_entry_id: Option<String>,
    pub latest_entry_kind: Option<FridayReleaseContinuityEntryKind>,
    pub latest_closure_ledger_id: Option<String>,
    pub latest_closure_state: Option<FridayReleaseClosureState>,
    pub closed_outcome_count: usize,
    pub carryover_commitment_count: usize,
    pub recurring_blocker_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub records: Vec<FridayReleaseContinuityEntry>,
    pub next_release_notes_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseContinuityJournal {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_continuity_journal_report(
    journal_path: impl AsRef<Path>,
    mut records: Vec<FridayReleaseContinuityEntry>,
) -> FridayReleaseContinuityJournal {
    let journal_path = journal_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    records.sort_by(|left, right| {
        left.recorded_at_unix_ms
            .cmp(&right.recorded_at_unix_ms)
            .then_with(|| left.entry_id.cmp(&right.entry_id))
    });
    records.dedup_by(|left, right| left.entry_id == right.entry_id);
    let latest = records.last();
    let active = records.iter().rev().find(|record| record.active).or(latest);
    let journal_json = path_string(journal_path);

    FridayReleaseContinuityJournal {
        journal_id: format!("friday-release-continuity-journal-{generated_at_unix_ms}"),
        journal_json: journal_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        entry_count: records.len(),
        outcome_entry_count: kind_count(&records, FridayReleaseContinuityEntryKind::Outcome),
        carryover_entry_count: kind_count(&records, FridayReleaseContinuityEntryKind::Carryover),
        blocker_pattern_count: kind_count(
            &records,
            FridayReleaseContinuityEntryKind::BlockerPattern,
        ),
        next_release_note_count: kind_count(
            &records,
            FridayReleaseContinuityEntryKind::NextReleaseNote,
        ),
        operator_decision_count: kind_count(
            &records,
            FridayReleaseContinuityEntryKind::OperatorDecision,
        ),
        superseded_history_count: kind_count(
            &records,
            FridayReleaseContinuityEntryKind::SupersededHistory,
        ),
        active_entry_id: active.map(|record| record.entry_id.clone()),
        latest_entry_id: latest.map(|record| record.entry_id.clone()),
        latest_entry_kind: latest.map(|record| record.entry_kind),
        latest_closure_ledger_id: latest.map(|record| record.closure_ledger_id.clone()),
        latest_closure_state: latest.and_then(|record| record.latest_closure_state),
        closed_outcome_count: records
            .iter()
            .map(|record| record.closed_outcome_count)
            .max()
            .unwrap_or(0),
        carryover_commitment_count: records
            .iter()
            .map(|record| record.carryover_commitment_count)
            .sum(),
        recurring_blocker_count: records
            .iter()
            .map(|record| record.recurring_blocker_count)
            .sum(),
        release_gate_blocking_count: active
            .map(|record| record.release_gate_blocking_count)
            .unwrap_or(0),
        unresolved_blocker_count: active
            .map(|record| record.unresolved_blocker_count)
            .unwrap_or(0),
        next_release_notes_copy: next_release_notes_copy(&records),
        summary: format!(
            "Friday release continuity journal has {} entry(s), {} outcome note(s), {} carryover note(s), {} blocker pattern(s), and {} next-release note(s).",
            records.len(),
            kind_count(&records, FridayReleaseContinuityEntryKind::Outcome),
            kind_count(&records, FridayReleaseContinuityEntryKind::Carryover),
            kind_count(&records, FridayReleaseContinuityEntryKind::BlockerPattern),
            kind_count(&records, FridayReleaseContinuityEntryKind::NextReleaseNote)
        ),
        commands: vec![
            format!(
                "flow --friday-release-continuity --journal {} --closure-ledger <release-closure-ledger.json> --kind outcome --operator <name>",
                journal_json
            ),
            format!(
                "flow --friday-release-continuity-list --journal {}",
                journal_json
            ),
            format!(
                "flow --friday-release-continuity-export --journal {} --output {}",
                journal_json, journal_json
            ),
            format!(
                "flow --friday-release-continuity-json --journal {} --closure-ledger <release-closure-ledger.json>",
                journal_json
            ),
        ],
        records,
    }
}

pub fn append_friday_release_continuity_to_journal(
    journal_path: impl AsRef<Path>,
    closure_ledger_path: impl AsRef<Path>,
    request: FridayReleaseContinuityRequest,
) -> Result<FridayReleaseContinuityJournal> {
    let journal_path = journal_path.as_ref();
    let closure_ledger_path = closure_ledger_path.as_ref();
    let mut records = read_friday_release_continuity_journal(journal_path)
        .map(|journal| journal.records)
        .unwrap_or_default();
    records.push(friday_release_continuity_entry_from_closure_ledger(
        closure_ledger_path,
        request,
    )?);
    let journal = friday_release_continuity_journal_report(journal_path, records);
    write_friday_release_continuity_journal(journal_path, &journal)?;
    Ok(journal)
}

pub fn friday_release_continuity_entry_from_closure_ledger(
    closure_ledger_path: impl AsRef<Path>,
    request: FridayReleaseContinuityRequest,
) -> Result<FridayReleaseContinuityEntry> {
    let closure_ledger_path = closure_ledger_path.as_ref();
    let ledger = read_friday_release_closure_ledger(closure_ledger_path)?;
    Ok(continuity_entry(closure_ledger_path, &ledger, request))
}

pub fn write_friday_release_continuity_journal(
    journal_path: impl AsRef<Path>,
    journal: &FridayReleaseContinuityJournal,
) -> Result<()> {
    let journal_path = journal_path.as_ref();
    if let Some(parent) = journal_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release continuity journal directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(journal_path, journal.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release continuity journal {}",
            journal_path.display()
        )
    })
}

pub fn read_friday_release_continuity_journal(
    journal_path: impl AsRef<Path>,
) -> Result<FridayReleaseContinuityJournal> {
    let journal_path = journal_path.as_ref();
    let bytes = fs::read(journal_path).with_context(|| {
        format!(
            "Could not read Friday release continuity journal {}",
            journal_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release continuity journal {}",
            journal_path.display()
        )
    })
}

fn continuity_entry(
    closure_ledger_path: &Path,
    ledger: &FridayReleaseClosureLedger,
    request: FridayReleaseContinuityRequest,
) -> FridayReleaseContinuityEntry {
    let recorded_at_unix_ms = unix_ms();
    let closure_ledger_json = path_string(closure_ledger_path);
    let recurring_blocker_count = recurring_blocker_count(ledger, request.entry_kind);
    let carryover_commitment_count = ledger
        .records
        .iter()
        .filter(|record| record.carryover_commitment.is_some())
        .count();
    let active = request.entry_kind != FridayReleaseContinuityEntryKind::SupersededHistory;
    let entry_notes_copy = format!(
        "Friday release continuity journal\nKind: {}\nOperator: {}\nOwner: {}\nNext release target: {}\nClosure ledger: {}\nLatest closure: {}\nNote: {}\nCarryover commitments: {}\nRecurring blockers: {}\nFriday did not fetch, send, publish, deploy, upload, or email.\nNo external mutation by Friday: true",
        request.entry_kind.label(),
        request.operator,
        request.owner.as_deref().unwrap_or("not-assigned"),
        request
            .next_release_target
            .as_deref()
            .unwrap_or("not-recorded"),
        ledger.ledger_id,
        ledger.latest_closure_id.as_deref().unwrap_or("none"),
        request.note,
        carryover_commitment_count,
        recurring_blocker_count
    );
    let summary = format!(
        "{} recorded {} continuity for {} with {} carryover commitment(s) and {} recurring blocker signal(s).",
        request.operator,
        request.entry_kind.label(),
        ledger.ledger_id,
        carryover_commitment_count,
        recurring_blocker_count
    );

    FridayReleaseContinuityEntry {
        entry_id: format!(
            "friday-release-continuity-{}-{recorded_at_unix_ms}",
            ledger.ledger_id
        ),
        closure_ledger_id: ledger.ledger_id.clone(),
        closure_ledger_json,
        recorded_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        entry_kind: request.entry_kind,
        operator: request.operator,
        note: request.note,
        owner: request.owner,
        next_release_target: request.next_release_target,
        supersedes_entry_id: request.supersedes_entry_id,
        latest_closure_id: ledger.latest_closure_id.clone(),
        latest_closure_state: ledger.latest_state,
        latest_receipt_review_id: ledger.latest_receipt_review_id.clone(),
        latest_review_decision: ledger.latest_review_decision,
        closure_record_count: ledger.record_count,
        closed_outcome_count: ledger.closed_outcome_count,
        carryover_outcome_count: ledger.carryover_outcome_count,
        blocked_outcome_count: ledger.blocked_outcome_count,
        release_gate_blocking_count: ledger.release_gate_blocking_count,
        unresolved_blocker_count: ledger.unresolved_blocker_count,
        recurring_blocker_count,
        carryover_commitment_count,
        active,
        externally_mutated_by_friday: false,
        entry_notes_copy,
        summary,
    }
}

fn recurring_blocker_count(
    ledger: &FridayReleaseClosureLedger,
    kind: FridayReleaseContinuityEntryKind,
) -> usize {
    let blocked_records = ledger
        .records
        .iter()
        .filter(|record| {
            record.state == FridayReleaseClosureState::Blocked
                || record.release_gate_blocking_count > 0
                || record.unresolved_blocker_count > 0
                || !record.ready_for_external_completion
        })
        .count();
    if blocked_records > 1 {
        blocked_records
    } else if kind == FridayReleaseContinuityEntryKind::BlockerPattern && blocked_records > 0 {
        1
    } else {
        0
    }
}

fn kind_count(
    records: &[FridayReleaseContinuityEntry],
    kind: FridayReleaseContinuityEntryKind,
) -> usize {
    records
        .iter()
        .filter(|record| record.entry_kind == kind)
        .count()
}

fn next_release_notes_copy(records: &[FridayReleaseContinuityEntry]) -> String {
    let mut lines = vec!["Friday release continuity journal".to_string()];
    for record in records.iter().rev().take(10) {
        lines.push(format!(
            "- [{}] {} -> {}",
            record.entry_kind.label(),
            record.operator,
            record.note
        ));
        if let Some(owner) = &record.owner {
            lines.push(format!("  owner: {owner}"));
        }
        if let Some(target) = &record.next_release_target {
            lines.push(format!("  next release: {target}"));
        }
        if record.carryover_commitment_count > 0 || record.recurring_blocker_count > 0 {
            lines.push(format!(
                "  carryover commitments: {}, recurring blocker signals: {}",
                record.carryover_commitment_count, record.recurring_blocker_count
            ));
        }
    }
    if lines.len() == 1 {
        lines.push("No release continuity entries are recorded.".to_string());
    }
    lines.push("Friday did not fetch, send, publish, deploy, upload, or email.".to_string());
    lines.join("\n")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::friday::{FridayReleaseClosureRecord, FridayReleaseClosureState};

    fn closure_record(id: &str, state: FridayReleaseClosureState) -> FridayReleaseClosureRecord {
        FridayReleaseClosureRecord {
            closure_id: id.to_string(),
            receipt_review_id: "review".to_string(),
            receipt_review_json: "tmp/review.json".to_string(),
            recorded_at_unix_ms: 1,
            product_name: "Friday".to_string(),
            local_only: true,
            state,
            operator: "operator".to_string(),
            closure_note: "note".to_string(),
            external_reference: None,
            carryover_commitment: Some("carryover".to_string()),
            supersedes_closure_id: None,
            review_decision: FridayReleaseReceiptReviewDecision::BlockedReview,
            review_status: super::super::FridayDashboardPanelStatus::Blocked,
            review_score_out_of_100: 0,
            ready_for_external_completion: false,
            archive_id: "archive".to_string(),
            active_receipt_id: None,
            latest_receipt_state: None,
            latest_outbound_review_id: None,
            latest_outbound_review_state: None,
            finding_count: 1,
            verified_receipt_count: 0,
            stale_or_missing_count: 0,
            release_gate_blocking_count: 1,
            unresolved_blocker_count: 1,
            active: true,
            externally_mutated_by_friday: false,
            closure_notes_copy: "copy".to_string(),
            summary: "summary".to_string(),
        }
    }

    #[test]
    fn continuity_journal_summarizes_blockers_and_carryover() {
        let ledger = FridayReleaseClosureLedger {
            ledger_id: "closure-ledger".to_string(),
            ledger_json: "tmp/release-closure-ledger.json".to_string(),
            generated_at_unix_ms: 1,
            product_name: "Friday".to_string(),
            local_only: true,
            record_count: 2,
            draft_count: 0,
            closed_count: 0,
            held_count: 0,
            carryover_count: 0,
            blocked_count: 2,
            revoked_count: 0,
            superseded_count: 0,
            active_closure_id: Some("closure-2".to_string()),
            latest_closure_id: Some("closure-2".to_string()),
            latest_state: Some(FridayReleaseClosureState::Blocked),
            latest_receipt_review_id: Some("review".to_string()),
            latest_review_decision: Some(FridayReleaseReceiptReviewDecision::BlockedReview),
            closed_outcome_count: 0,
            carryover_outcome_count: 2,
            blocked_outcome_count: 2,
            release_gate_blocking_count: 1,
            unresolved_blocker_count: 1,
            records: vec![
                closure_record("closure-1", FridayReleaseClosureState::Blocked),
                closure_record("closure-2", FridayReleaseClosureState::Blocked),
            ],
            closure_summary_copy: "copy".to_string(),
            summary: "summary".to_string(),
            commands: Vec::new(),
        };
        let entry = continuity_entry(
            Path::new("tmp/release-closure-ledger.json"),
            &ledger,
            FridayReleaseContinuityRequest {
                entry_kind: FridayReleaseContinuityEntryKind::BlockerPattern,
                operator: "release-operator".to_string(),
                note: "Carry blocker into next release planning.".to_string(),
                owner: Some("platform".to_string()),
                next_release_target: Some("next-100".to_string()),
                supersedes_entry_id: None,
            },
        );
        let journal =
            friday_release_continuity_journal_report("tmp/release-continuity.json", vec![entry]);

        assert_eq!(journal.entry_count, 1);
        assert_eq!(journal.blocker_pattern_count, 1);
        assert_eq!(journal.recurring_blocker_count, 2);
        assert!(journal.next_release_notes_copy.contains("platform"));
        assert!(
            journal
                .next_release_notes_copy
                .contains("Friday did not fetch, send, publish, deploy, upload, or email")
        );
    }
}
