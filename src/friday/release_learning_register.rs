use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayReleaseContinuityEntryKind, FridayReleaseContinuityJournal,
    read_friday_release_continuity_journal,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseLearningCategory {
    Lesson,
    PreventionExperiment,
    DecisionPattern,
    QualityGate,
    OwnerCommitment,
    RetiredLearning,
}

impl FridayReleaseLearningCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lesson => "lesson",
            Self::PreventionExperiment => "prevention-experiment",
            Self::DecisionPattern => "decision-pattern",
            Self::QualityGate => "quality-gate",
            Self::OwnerCommitment => "owner-commitment",
            Self::RetiredLearning => "retired-learning",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "lesson" | "learning" => Ok(Self::Lesson),
            "prevention-experiment" | "prevention" | "experiment" => Ok(Self::PreventionExperiment),
            "decision-pattern" | "decision" | "pattern" => Ok(Self::DecisionPattern),
            "quality-gate" | "gate" | "qa-gate" => Ok(Self::QualityGate),
            "owner-commitment" | "commitment" | "owner" => Ok(Self::OwnerCommitment),
            "retired-learning" | "retired" | "archive" => Ok(Self::RetiredLearning),
            other => anyhow::bail!(
                "Unknown Friday release learning category `{}`. Use lesson, prevention-experiment, decision-pattern, quality-gate, owner-commitment, or retired-learning.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseLearningRequest {
    pub category: FridayReleaseLearningCategory,
    pub operator: String,
    pub learning: String,
    pub owner: Option<String>,
    pub next_cycle_commitment: Option<String>,
    pub quality_gate: Option<String>,
    pub retires_learning_id: Option<String>,
}

impl Default for FridayReleaseLearningRequest {
    fn default() -> Self {
        Self {
            category: FridayReleaseLearningCategory::Lesson,
            operator: "operator".to_string(),
            learning: "Recorded local release learning.".to_string(),
            owner: None,
            next_cycle_commitment: None,
            quality_gate: None,
            retires_learning_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseLearningRecord {
    pub learning_id: String,
    pub continuity_journal_id: String,
    pub continuity_journal_json: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub category: FridayReleaseLearningCategory,
    pub operator: String,
    pub learning: String,
    pub owner: Option<String>,
    pub next_cycle_commitment: Option<String>,
    pub quality_gate: Option<String>,
    pub retires_learning_id: Option<String>,
    pub latest_continuity_entry_id: Option<String>,
    pub latest_continuity_entry_kind: Option<FridayReleaseContinuityEntryKind>,
    pub continuity_entry_count: usize,
    pub outcome_entry_count: usize,
    pub carryover_entry_count: usize,
    pub blocker_pattern_count: usize,
    pub next_release_note_count: usize,
    pub operator_decision_count: usize,
    pub recurring_blocker_count: usize,
    pub carryover_commitment_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub repeated_lesson_count: usize,
    pub owner_commitment_count: usize,
    pub quality_gate_count: usize,
    pub active: bool,
    pub externally_mutated_by_friday: bool,
    pub learning_notes_copy: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseLearningRegister {
    pub register_id: String,
    pub register_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub record_count: usize,
    pub lesson_count: usize,
    pub prevention_experiment_count: usize,
    pub decision_pattern_count: usize,
    pub quality_gate_count: usize,
    pub owner_commitment_count: usize,
    pub retired_learning_count: usize,
    pub active_learning_id: Option<String>,
    pub latest_learning_id: Option<String>,
    pub latest_category: Option<FridayReleaseLearningCategory>,
    pub latest_continuity_journal_id: Option<String>,
    pub repeated_lesson_count: usize,
    pub next_cycle_commitment_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub records: Vec<FridayReleaseLearningRecord>,
    pub next_cycle_commitments_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseLearningRegister {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_learning_register_report(
    register_path: impl AsRef<Path>,
    mut records: Vec<FridayReleaseLearningRecord>,
) -> FridayReleaseLearningRegister {
    let register_path = register_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    records.sort_by(|left, right| {
        left.recorded_at_unix_ms
            .cmp(&right.recorded_at_unix_ms)
            .then_with(|| left.learning_id.cmp(&right.learning_id))
    });
    records.dedup_by(|left, right| left.learning_id == right.learning_id);
    let latest = records.last();
    let active = records.iter().rev().find(|record| record.active).or(latest);
    let register_json = path_string(register_path);

    FridayReleaseLearningRegister {
        register_id: format!("friday-release-learning-register-{generated_at_unix_ms}"),
        register_json: register_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: records.len(),
        lesson_count: category_count(&records, FridayReleaseLearningCategory::Lesson),
        prevention_experiment_count: category_count(
            &records,
            FridayReleaseLearningCategory::PreventionExperiment,
        ),
        decision_pattern_count: category_count(
            &records,
            FridayReleaseLearningCategory::DecisionPattern,
        ),
        quality_gate_count: category_count(&records, FridayReleaseLearningCategory::QualityGate),
        owner_commitment_count: category_count(
            &records,
            FridayReleaseLearningCategory::OwnerCommitment,
        ),
        retired_learning_count: category_count(
            &records,
            FridayReleaseLearningCategory::RetiredLearning,
        ),
        active_learning_id: active.map(|record| record.learning_id.clone()),
        latest_learning_id: latest.map(|record| record.learning_id.clone()),
        latest_category: latest.map(|record| record.category),
        latest_continuity_journal_id: latest.map(|record| record.continuity_journal_id.clone()),
        repeated_lesson_count: records
            .iter()
            .map(|record| record.repeated_lesson_count)
            .sum(),
        next_cycle_commitment_count: records
            .iter()
            .filter(|record| record.next_cycle_commitment.is_some())
            .count(),
        release_gate_blocking_count: active
            .map(|record| record.release_gate_blocking_count)
            .unwrap_or(0),
        unresolved_blocker_count: active
            .map(|record| record.unresolved_blocker_count)
            .unwrap_or(0),
        next_cycle_commitments_copy: next_cycle_commitments_copy(&records),
        summary: format!(
            "Friday release learning register has {} record(s), {} lesson(s), {} prevention experiment(s), {} decision pattern(s), {} quality gate(s), and {} owner commitment(s).",
            records.len(),
            category_count(&records, FridayReleaseLearningCategory::Lesson),
            category_count(
                &records,
                FridayReleaseLearningCategory::PreventionExperiment
            ),
            category_count(&records, FridayReleaseLearningCategory::DecisionPattern),
            category_count(&records, FridayReleaseLearningCategory::QualityGate),
            category_count(&records, FridayReleaseLearningCategory::OwnerCommitment)
        ),
        commands: vec![
            format!(
                "flow --friday-release-learning --register {} --continuity-journal <release-continuity-journal.json> --category lesson --operator <name>",
                register_json
            ),
            format!(
                "flow --friday-release-learning-list --register {}",
                register_json
            ),
            format!(
                "flow --friday-release-learning-export --register {} --output {}",
                register_json, register_json
            ),
            format!(
                "flow --friday-release-learning-json --register {} --continuity-journal <release-continuity-journal.json>",
                register_json
            ),
        ],
        records,
    }
}

pub fn append_friday_release_learning_to_register(
    register_path: impl AsRef<Path>,
    continuity_journal_path: impl AsRef<Path>,
    request: FridayReleaseLearningRequest,
) -> Result<FridayReleaseLearningRegister> {
    let register_path = register_path.as_ref();
    let continuity_journal_path = continuity_journal_path.as_ref();
    let mut records = read_friday_release_learning_register(register_path)
        .map(|register| register.records)
        .unwrap_or_default();
    records.push(friday_release_learning_record_from_continuity_journal(
        continuity_journal_path,
        request,
    )?);
    let register = friday_release_learning_register_report(register_path, records);
    write_friday_release_learning_register(register_path, &register)?;
    Ok(register)
}

pub fn friday_release_learning_record_from_continuity_journal(
    continuity_journal_path: impl AsRef<Path>,
    request: FridayReleaseLearningRequest,
) -> Result<FridayReleaseLearningRecord> {
    let continuity_journal_path = continuity_journal_path.as_ref();
    let journal = read_friday_release_continuity_journal(continuity_journal_path)?;
    Ok(learning_record(continuity_journal_path, &journal, request))
}

pub fn write_friday_release_learning_register(
    register_path: impl AsRef<Path>,
    register: &FridayReleaseLearningRegister,
) -> Result<()> {
    let register_path = register_path.as_ref();
    if let Some(parent) = register_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release learning register directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(register_path, register.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release learning register {}",
            register_path.display()
        )
    })
}

pub fn read_friday_release_learning_register(
    register_path: impl AsRef<Path>,
) -> Result<FridayReleaseLearningRegister> {
    let register_path = register_path.as_ref();
    let bytes = fs::read(register_path).with_context(|| {
        format!(
            "Could not read Friday release learning register {}",
            register_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release learning register {}",
            register_path.display()
        )
    })
}

fn learning_record(
    continuity_journal_path: &Path,
    journal: &FridayReleaseContinuityJournal,
    request: FridayReleaseLearningRequest,
) -> FridayReleaseLearningRecord {
    let recorded_at_unix_ms = unix_ms();
    let continuity_journal_json = path_string(continuity_journal_path);
    let repeated_lesson_count = repeated_lesson_count(journal, request.category);
    let owner_commitment_count = owner_commitment_count(journal, &request);
    let quality_gate_count = usize::from(
        request.quality_gate.is_some()
            || request.category == FridayReleaseLearningCategory::QualityGate,
    );
    let active = request.category != FridayReleaseLearningCategory::RetiredLearning;
    let learning_notes_copy = format!(
        "Friday release learning register\nCategory: {}\nOperator: {}\nOwner: {}\nLearning: {}\nQuality gate: {}\nNext cycle commitment: {}\nContinuity journal: {}\nRepeated lessons: {}\nOwner commitments: {}\nFriday did not fetch, send, publish, deploy, upload, or email.\nNo external mutation by Friday: true",
        request.category.label(),
        request.operator,
        request.owner.as_deref().unwrap_or("not-assigned"),
        request.learning,
        request.quality_gate.as_deref().unwrap_or("not-recorded"),
        request
            .next_cycle_commitment
            .as_deref()
            .unwrap_or("not-recorded"),
        journal.journal_id,
        repeated_lesson_count,
        owner_commitment_count
    );
    let summary = format!(
        "{} recorded {} learning for {} with {} repeated lesson signal(s) and {} owner commitment(s).",
        request.operator,
        request.category.label(),
        journal.journal_id,
        repeated_lesson_count,
        owner_commitment_count
    );

    FridayReleaseLearningRecord {
        learning_id: format!(
            "friday-release-learning-{}-{recorded_at_unix_ms}",
            journal.journal_id
        ),
        continuity_journal_id: journal.journal_id.clone(),
        continuity_journal_json,
        recorded_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        category: request.category,
        operator: request.operator,
        learning: request.learning,
        owner: request.owner,
        next_cycle_commitment: request.next_cycle_commitment,
        quality_gate: request.quality_gate,
        retires_learning_id: request.retires_learning_id,
        latest_continuity_entry_id: journal.latest_entry_id.clone(),
        latest_continuity_entry_kind: journal.latest_entry_kind,
        continuity_entry_count: journal.entry_count,
        outcome_entry_count: journal.outcome_entry_count,
        carryover_entry_count: journal.carryover_entry_count,
        blocker_pattern_count: journal.blocker_pattern_count,
        next_release_note_count: journal.next_release_note_count,
        operator_decision_count: journal.operator_decision_count,
        recurring_blocker_count: journal.recurring_blocker_count,
        carryover_commitment_count: journal.carryover_commitment_count,
        release_gate_blocking_count: journal.release_gate_blocking_count,
        unresolved_blocker_count: journal.unresolved_blocker_count,
        repeated_lesson_count,
        owner_commitment_count,
        quality_gate_count,
        active,
        externally_mutated_by_friday: false,
        learning_notes_copy,
        summary,
    }
}

fn repeated_lesson_count(
    journal: &FridayReleaseContinuityJournal,
    category: FridayReleaseLearningCategory,
) -> usize {
    if journal.recurring_blocker_count > 0 {
        journal.recurring_blocker_count
    } else if category == FridayReleaseLearningCategory::DecisionPattern
        && journal.operator_decision_count > 0
    {
        journal.operator_decision_count
    } else {
        0
    }
}

fn owner_commitment_count(
    journal: &FridayReleaseContinuityJournal,
    request: &FridayReleaseLearningRequest,
) -> usize {
    let requested = usize::from(
        request.owner.is_some()
            || request.next_cycle_commitment.is_some()
            || request.category == FridayReleaseLearningCategory::OwnerCommitment,
    );
    requested + journal.carryover_commitment_count
}

fn category_count(
    records: &[FridayReleaseLearningRecord],
    category: FridayReleaseLearningCategory,
) -> usize {
    records
        .iter()
        .filter(|record| record.category == category)
        .count()
}

fn next_cycle_commitments_copy(records: &[FridayReleaseLearningRecord]) -> String {
    let mut lines = vec!["Friday release learning register".to_string()];
    for record in records.iter().rev().take(10) {
        lines.push(format!(
            "- [{}] {} -> {}",
            record.category.label(),
            record.operator,
            record.learning
        ));
        if let Some(owner) = &record.owner {
            lines.push(format!("  owner: {owner}"));
        }
        if let Some(gate) = &record.quality_gate {
            lines.push(format!("  quality gate: {gate}"));
        }
        if let Some(commitment) = &record.next_cycle_commitment {
            lines.push(format!("  next cycle: {commitment}"));
        }
        if record.repeated_lesson_count > 0 || record.owner_commitment_count > 0 {
            lines.push(format!(
                "  repeated lessons: {}, owner commitments: {}",
                record.repeated_lesson_count, record.owner_commitment_count
            ));
        }
    }
    if lines.len() == 1 {
        lines.push("No release learning records are recorded.".to_string());
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

    #[test]
    fn learning_register_preserves_lessons_gates_and_owner_commitments() {
        let journal = FridayReleaseContinuityJournal {
            journal_id: "continuity-journal".to_string(),
            journal_json: "tmp/release-continuity-journal.json".to_string(),
            generated_at_unix_ms: 1,
            product_name: "Friday".to_string(),
            local_only: true,
            entry_count: 1,
            outcome_entry_count: 0,
            carryover_entry_count: 0,
            blocker_pattern_count: 1,
            next_release_note_count: 0,
            operator_decision_count: 0,
            superseded_history_count: 0,
            active_entry_id: Some("entry".to_string()),
            latest_entry_id: Some("entry".to_string()),
            latest_entry_kind: Some(FridayReleaseContinuityEntryKind::BlockerPattern),
            latest_closure_ledger_id: Some("closure-ledger".to_string()),
            latest_closure_state: None,
            closed_outcome_count: 0,
            carryover_commitment_count: 1,
            recurring_blocker_count: 2,
            release_gate_blocking_count: 1,
            unresolved_blocker_count: 1,
            records: Vec::new(),
            next_release_notes_copy: "copy".to_string(),
            summary: "summary".to_string(),
            commands: Vec::new(),
        };
        let record = learning_record(
            Path::new("tmp/release-continuity-journal.json"),
            &journal,
            FridayReleaseLearningRequest {
                category: FridayReleaseLearningCategory::QualityGate,
                operator: "release-operator".to_string(),
                learning: "Blocker patterns need a release gate before closure.".to_string(),
                owner: Some("platform".to_string()),
                next_cycle_commitment: Some(
                    "Add blocker-pattern review to the next loop.".to_string(),
                ),
                quality_gate: Some("No closure without continuity review.".to_string()),
                retires_learning_id: None,
            },
        );
        let register =
            friday_release_learning_register_report("tmp/release-learning.json", vec![record]);

        assert_eq!(register.record_count, 1);
        assert_eq!(register.quality_gate_count, 1);
        assert_eq!(register.repeated_lesson_count, 2);
        assert!(register.next_cycle_commitments_copy.contains("platform"));
        assert!(
            register
                .next_cycle_commitments_copy
                .contains("Friday did not fetch, send, publish, deploy, upload, or email")
        );
    }
}
