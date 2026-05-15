use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayReleaseCandidateArchive, FridayReleaseCandidateArchiveEntry,
    FridayReleaseDeploymentTarget, read_friday_release_candidate_archive,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleasePromotionDecision {
    Promoted,
    Held,
    RolledBack,
    Superseded,
    Abandoned,
}

impl FridayReleasePromotionDecision {
    pub fn label(self) -> &'static str {
        match self {
            Self::Promoted => "promoted",
            Self::Held => "held",
            Self::RolledBack => "rolled-back",
            Self::Superseded => "superseded",
            Self::Abandoned => "abandoned",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "promoted" | "promote" | "go" => Ok(Self::Promoted),
            "held" | "hold" | "no-go" | "blocked" => Ok(Self::Held),
            "rolled-back" | "rolled_back" | "rollback" | "rolledback" => Ok(Self::RolledBack),
            "superseded" | "supersede" => Ok(Self::Superseded),
            "abandoned" | "abandon" => Ok(Self::Abandoned),
            other => anyhow::bail!(
                "Unknown Friday promotion decision `{}`. Use promoted, held, rolled-back, superseded, or abandoned.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePromotionPostCheck {
    pub id: String,
    pub label: String,
    pub result_path: String,
    pub required: bool,
    pub present: bool,
    pub bytes: u64,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePromotionRecordRequest {
    pub candidate_id: Option<String>,
    pub decision: FridayReleasePromotionDecision,
    pub operator: String,
    pub reason: String,
    pub deployment_note: String,
    pub rollback_reference: String,
    pub post_check_files: Vec<String>,
}

impl Default for FridayReleasePromotionRecordRequest {
    fn default() -> Self {
        Self {
            candidate_id: None,
            decision: FridayReleasePromotionDecision::Held,
            operator: "operator".to_string(),
            reason: "Reviewed release candidate and kept promotion local-only.".to_string(),
            deployment_note: "No deployment executed by this command.".to_string(),
            rollback_reference: "Previous release candidate remains active.".to_string(),
            post_check_files: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePromotionRecord {
    pub promotion_id: String,
    pub candidate_id: String,
    pub archive_json: String,
    pub gate_json: String,
    pub export_kit_json: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub decision: FridayReleasePromotionDecision,
    pub operator: String,
    pub reason: String,
    pub deployment_note: String,
    pub target: FridayReleaseDeploymentTarget,
    pub rollback_reference: String,
    pub candidate_score_out_of_100: u8,
    pub candidate_ready_to_deploy: bool,
    pub candidate_blocker_count: usize,
    pub post_promotion_required_count: usize,
    pub post_promotion_missing_count: usize,
    pub post_promotion_checks: Vec<FridayReleasePromotionPostCheck>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePromotionLedger {
    pub ledger_id: String,
    pub ledger_json: String,
    pub generated_at_unix_ms: u128,
    pub local_only: bool,
    pub record_count: usize,
    pub promoted_count: usize,
    pub held_count: usize,
    pub rolled_back_count: usize,
    pub superseded_count: usize,
    pub abandoned_count: usize,
    pub post_promotion_missing_count: usize,
    pub active_promotion_id: Option<String>,
    pub active_candidate_id: Option<String>,
    pub active_rollback_reference: Option<String>,
    pub latest_decision: Option<FridayReleasePromotionDecision>,
    pub latest_deployment_note: Option<String>,
    pub warnings: Vec<String>,
    pub records: Vec<FridayReleasePromotionRecord>,
    pub commands: Vec<String>,
}

impl FridayReleasePromotionLedger {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_promotion_ledger_report(
    ledger_path: impl AsRef<Path>,
    mut records: Vec<FridayReleasePromotionRecord>,
) -> FridayReleasePromotionLedger {
    let ledger_path = ledger_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    records.sort_by_key(|record| record.recorded_at_unix_ms);
    records.dedup_by(|left, right| left.promotion_id == right.promotion_id);
    let latest = records.last();
    let promoted = records
        .iter()
        .rev()
        .find(|record| record.decision == FridayReleasePromotionDecision::Promoted);
    let active = promoted.or(latest);
    let post_promotion_missing_count = records
        .iter()
        .map(|record| record.post_promotion_missing_count)
        .sum::<usize>();
    let mut warnings = Vec::new();

    if let Some(record) = latest {
        if record.decision == FridayReleasePromotionDecision::Promoted
            && record.post_promotion_missing_count > 0
        {
            warnings.push(format!(
                "Latest promoted candidate is missing {} post-promotion check(s).",
                record.post_promotion_missing_count
            ));
        }
        if record.decision == FridayReleasePromotionDecision::Promoted
            && !record.candidate_ready_to_deploy
        {
            warnings.push("Latest promoted candidate was not marked ready to deploy.".to_string());
        }
    }

    FridayReleasePromotionLedger {
        ledger_id: format!("friday-release-promotion-ledger-{generated_at_unix_ms}"),
        ledger_json: path_string(ledger_path),
        generated_at_unix_ms,
        local_only: true,
        record_count: records.len(),
        promoted_count: decision_count(&records, FridayReleasePromotionDecision::Promoted),
        held_count: decision_count(&records, FridayReleasePromotionDecision::Held),
        rolled_back_count: decision_count(&records, FridayReleasePromotionDecision::RolledBack),
        superseded_count: decision_count(&records, FridayReleasePromotionDecision::Superseded),
        abandoned_count: decision_count(&records, FridayReleasePromotionDecision::Abandoned),
        post_promotion_missing_count,
        active_promotion_id: active.map(|record| record.promotion_id.clone()),
        active_candidate_id: active.map(|record| record.candidate_id.clone()),
        active_rollback_reference: active.map(|record| record.rollback_reference.clone()),
        latest_decision: latest.map(|record| record.decision),
        latest_deployment_note: latest.map(|record| record.deployment_note.clone()),
        warnings,
        commands: vec![
            format!(
                "flow --friday-release-promotion-ledger --ledger {} --archive <candidate-archive.json> --decision held --reason \"<reason>\"",
                path_string(ledger_path)
            ),
            format!(
                "flow --friday-release-promotion-ledger-json --ledger {}",
                path_string(ledger_path)
            ),
        ],
        records,
    }
}

pub fn append_friday_release_promotion_to_ledger(
    ledger_path: impl AsRef<Path>,
    archive_path: impl AsRef<Path>,
    request: FridayReleasePromotionRecordRequest,
) -> Result<FridayReleasePromotionLedger> {
    let ledger_path = ledger_path.as_ref();
    let archive_path = archive_path.as_ref();
    let archive = read_friday_release_candidate_archive(archive_path)?;
    let candidate = select_candidate(&archive, request.candidate_id.as_deref())?;
    let mut records = read_friday_release_promotion_ledger(ledger_path)
        .map(|ledger| ledger.records)
        .unwrap_or_default();
    records.push(promotion_record(archive_path, candidate, request));
    let ledger = friday_release_promotion_ledger_report(ledger_path, records);
    write_friday_release_promotion_ledger(ledger_path, &ledger)?;
    Ok(ledger)
}

pub fn write_friday_release_promotion_ledger(
    ledger_path: impl AsRef<Path>,
    ledger: &FridayReleasePromotionLedger,
) -> Result<()> {
    let ledger_path = ledger_path.as_ref();
    if let Some(parent) = ledger_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release promotion ledger directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(ledger_path, ledger.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release promotion ledger {}",
            ledger_path.display()
        )
    })
}

pub fn read_friday_release_promotion_ledger(
    ledger_path: impl AsRef<Path>,
) -> Result<FridayReleasePromotionLedger> {
    let ledger_path = ledger_path.as_ref();
    let bytes = fs::read(ledger_path).with_context(|| {
        format!(
            "Could not read Friday release promotion ledger {}",
            ledger_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release promotion ledger {}",
            ledger_path.display()
        )
    })
}

fn select_candidate<'a>(
    archive: &'a FridayReleaseCandidateArchive,
    candidate_id: Option<&str>,
) -> Result<&'a FridayReleaseCandidateArchiveEntry> {
    if let Some(candidate_id) = candidate_id {
        return archive
            .entries
            .iter()
            .find(|entry| entry.candidate_id == candidate_id)
            .with_context(|| format!("No release candidate `{candidate_id}` found in archive"));
    }

    let latest = archive
        .latest_candidate_id
        .as_deref()
        .context("Release candidate archive has no latest candidate")?;
    archive
        .entries
        .iter()
        .find(|entry| entry.candidate_id == latest)
        .with_context(|| format!("Latest release candidate `{latest}` was not found in archive"))
}

fn promotion_record(
    archive_path: &Path,
    candidate: &FridayReleaseCandidateArchiveEntry,
    request: FridayReleasePromotionRecordRequest,
) -> FridayReleasePromotionRecord {
    let recorded_at_unix_ms = unix_ms();
    let checks = promotion_checks(&request);
    let post_promotion_required_count = checks.iter().filter(|check| check.required).count();
    let post_promotion_missing_count = checks
        .iter()
        .filter(|check| check.required && !check.present)
        .count();
    let summary = format!(
        "Candidate {} was {} by {} with {} post-promotion check(s) missing.",
        candidate.candidate_id,
        request.decision.label(),
        request.operator,
        post_promotion_missing_count
    );

    FridayReleasePromotionRecord {
        promotion_id: format!(
            "promotion-{}-{}-{recorded_at_unix_ms}",
            candidate.candidate_id,
            request.decision.label()
        ),
        candidate_id: candidate.candidate_id.clone(),
        archive_json: path_string(archive_path),
        gate_json: candidate.gate_json.clone(),
        export_kit_json: candidate.export_kit_json.clone(),
        recorded_at_unix_ms,
        product_name: candidate.product_name.clone(),
        local_only: true,
        decision: request.decision,
        operator: request.operator,
        reason: request.reason,
        deployment_note: request.deployment_note,
        target: candidate.target.clone(),
        rollback_reference: request.rollback_reference,
        candidate_score_out_of_100: candidate.score_out_of_100,
        candidate_ready_to_deploy: candidate.ready_to_deploy,
        candidate_blocker_count: candidate.no_deploy_reason_count,
        post_promotion_required_count,
        post_promotion_missing_count,
        post_promotion_checks: checks,
        summary,
    }
}

fn promotion_checks(
    request: &FridayReleasePromotionRecordRequest,
) -> Vec<FridayReleasePromotionPostCheck> {
    let mut checks = vec![
        inline_check(
            "deployment-note",
            "Deployment note",
            !request.deployment_note.trim().is_empty(),
            "Attach a deployment note before promotion review.",
        ),
        inline_check(
            "rollback-reference",
            "Rollback reference",
            !request.rollback_reference.trim().is_empty(),
            "Attach a rollback reference before promotion review.",
        ),
    ];

    checks.extend(
        request
            .post_check_files
            .iter()
            .map(|value| file_check(value)),
    );
    checks
}

fn inline_check(
    id: &str,
    label: &str,
    present: bool,
    next_action: &str,
) -> FridayReleasePromotionPostCheck {
    FridayReleasePromotionPostCheck {
        id: id.to_string(),
        label: label.to_string(),
        result_path: "inline".to_string(),
        required: true,
        present,
        bytes: 0,
        summary: if present {
            format!("{label} is recorded.")
        } else {
            format!("{label} is missing.")
        },
        next_action: next_action.to_string(),
    }
}

fn file_check(value: &str) -> FridayReleasePromotionPostCheck {
    let (id, path) = value
        .split_once('=')
        .map(|(id, path)| (id.trim(), path.trim()))
        .unwrap_or_else(|| ("post-promotion-check", value.trim()));
    match fs::metadata(path) {
        Ok(metadata) => FridayReleasePromotionPostCheck {
            id: id.to_string(),
            label: id.replace('-', " "),
            result_path: path_string(Path::new(path)),
            required: true,
            present: true,
            bytes: metadata.len(),
            summary: "Post-promotion check evidence is present.".to_string(),
            next_action: "Attach this evidence to the promotion ledger entry.".to_string(),
        },
        Err(_) => FridayReleasePromotionPostCheck {
            id: id.to_string(),
            label: id.replace('-', " "),
            result_path: path.to_string(),
            required: true,
            present: false,
            bytes: 0,
            summary: "Post-promotion check evidence is missing.".to_string(),
            next_action: "Create the check-result file and record promotion again.".to_string(),
        },
    }
}

fn decision_count(
    records: &[FridayReleasePromotionRecord],
    decision: FridayReleasePromotionDecision,
) -> usize {
    records
        .iter()
        .filter(|record| record.decision == decision)
        .count()
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
