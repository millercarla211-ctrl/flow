use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, friday_dashboard_product_ui_binding_from_export,
    read_friday_release_evidence_export_kit, read_friday_release_operator_checklist,
    read_friday_release_qa_command_center_report, read_friday_trusted_runner_release_package,
    read_friday_trusted_runner_release_timeline,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseDeploymentGateDecision {
    Go,
    NoGo,
    Draft,
}

impl FridayReleaseDeploymentGateDecision {
    pub fn label(self) -> &'static str {
        match self {
            Self::Go => "go",
            Self::NoGo => "no-go",
            Self::Draft => "draft",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseDeploymentGateReasonCategory {
    MissingEvidence,
    StaleChecks,
    BlockedQa,
    UnsignedRelease,
    DashboardState,
    TargetMismatch,
}

impl FridayReleaseDeploymentGateReasonCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::MissingEvidence => "missing-evidence",
            Self::StaleChecks => "stale-checks",
            Self::BlockedQa => "blocked-qa",
            Self::UnsignedRelease => "unsigned-release",
            Self::DashboardState => "dashboard-state",
            Self::TargetMismatch => "target-mismatch",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseDeploymentGateReasonSeverity {
    Blocking,
    Warning,
}

impl FridayReleaseDeploymentGateReasonSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Blocking => "blocking",
            Self::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseDeploymentTarget {
    pub id: String,
    pub label: String,
    pub environment: String,
    pub provider: String,
    pub url: Option<String>,
    pub local_only_required: bool,
    pub requires_vercel: bool,
    pub expected_product_name: String,
    pub rollback_note: String,
}

impl Default for FridayReleaseDeploymentTarget {
    fn default() -> Self {
        Self {
            id: "local-friday-checkpoint".to_string(),
            label: "Local Friday checkpoint".to_string(),
            environment: "local".to_string(),
            provider: "local".to_string(),
            url: None,
            local_only_required: true,
            requires_vercel: false,
            expected_product_name: "Friday".to_string(),
            rollback_note: "Keep the previous evidence kit and release package attached; do not promote until the deployment gate returns go.".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseDeploymentGateReason {
    pub id: String,
    pub category: FridayReleaseDeploymentGateReasonCategory,
    pub severity: FridayReleaseDeploymentGateReasonSeverity,
    pub title: String,
    pub detail: String,
    pub source_path: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseDeploymentGateChecklistItem {
    pub id: String,
    pub title: String,
    pub ready: bool,
    pub detail: String,
    pub source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseDeploymentGateReport {
    pub gate_id: String,
    pub gate_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub decision: FridayReleaseDeploymentGateDecision,
    pub ready_to_deploy: bool,
    pub score_out_of_100: u8,
    pub summary: String,
    pub target: FridayReleaseDeploymentTarget,
    pub export_kit_json: String,
    pub qa_json: String,
    pub checklist_json: String,
    pub package_json: String,
    pub timeline_json: String,
    pub dashboard_export_dir: String,
    pub no_deploy_reason_count: usize,
    pub warning_count: usize,
    pub ready_count: usize,
    pub total_count: usize,
    pub reasons: Vec<FridayReleaseDeploymentGateReason>,
    pub checklist: Vec<FridayReleaseDeploymentGateChecklistItem>,
    pub deploy_checklist: Vec<String>,
    pub rollback_note: String,
    pub operator_copy: String,
    pub commands: Vec<String>,
}

impl FridayReleaseDeploymentGateReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_deployment_gate_report(
    gate_path: impl AsRef<Path>,
    export_kit_path: impl AsRef<Path>,
    qa_path: impl AsRef<Path>,
    checklist_path: impl AsRef<Path>,
    package_path: impl AsRef<Path>,
    timeline_path: impl AsRef<Path>,
    dashboard_export_dir: impl AsRef<Path>,
    target: FridayReleaseDeploymentTarget,
) -> FridayReleaseDeploymentGateReport {
    let gate_path = gate_path.as_ref();
    let export_kit_path = export_kit_path.as_ref();
    let qa_path = qa_path.as_ref();
    let checklist_path = checklist_path.as_ref();
    let package_path = package_path.as_ref();
    let timeline_path = timeline_path.as_ref();
    let dashboard_export_dir = dashboard_export_dir.as_ref();
    let generated_at_unix_ms = unix_ms();

    let export_kit = read_friday_release_evidence_export_kit(export_kit_path).ok();
    let qa = read_friday_release_qa_command_center_report(qa_path).ok();
    let checklist_report = read_friday_release_operator_checklist(checklist_path).ok();
    let package = read_friday_trusted_runner_release_package(package_path).ok();
    let timeline = read_friday_trusted_runner_release_timeline(timeline_path).ok();
    let dashboard = friday_dashboard_product_ui_binding_from_export(dashboard_export_dir).ok();

    let mut reasons = Vec::new();
    let mut checklist = Vec::new();

    push_check(
        &mut checklist,
        "release-export-kit",
        "Release evidence export kit",
        export_kit
            .as_ref()
            .is_some_and(|report| report.ready_to_attach),
        export_kit.as_ref().map_or_else(
            || "Evidence export kit is missing or could not be parsed.".to_string(),
            |report| {
                format!(
                    "{} missing, {} stale, {} warning(s).",
                    report.manifest.missing_count,
                    report.manifest.stale_count,
                    report.manifest.warning_count
                )
            },
        ),
        export_kit_path,
    );
    if let Some(report) = &export_kit {
        if report.manifest.missing_count > 0 {
            push_reason(
                &mut reasons,
                "export-kit-missing-evidence",
                FridayReleaseDeploymentGateReasonCategory::MissingEvidence,
                FridayReleaseDeploymentGateReasonSeverity::Blocking,
                "Evidence kit has missing files",
                format!(
                    "{} required evidence file(s) are missing from the export kit.",
                    report.manifest.missing_count
                ),
                export_kit_path,
                "Regenerate the export kit after creating all required release evidence.",
            );
        }
        if report.manifest.stale_count > 0 {
            push_reason(
                &mut reasons,
                "export-kit-stale-checks",
                FridayReleaseDeploymentGateReasonCategory::StaleChecks,
                FridayReleaseDeploymentGateReasonSeverity::Blocking,
                "Evidence kit contains stale lightweight checks",
                format!(
                    "{} check result file(s) are older than the freshness window.",
                    report.manifest.stale_count
                ),
                export_kit_path,
                "Refresh cargo check, extension typecheck, dashboard smoke, then regenerate the export kit.",
            );
        }
        if !report.ready_to_attach {
            push_reason(
                &mut reasons,
                "export-kit-not-ready",
                FridayReleaseDeploymentGateReasonCategory::MissingEvidence,
                FridayReleaseDeploymentGateReasonSeverity::Warning,
                "Evidence kit is not ready to attach",
                report.summary.clone(),
                export_kit_path,
                "Resolve kit warnings before attaching it to a major checkpoint.",
            );
        }
    } else {
        push_reason(
            &mut reasons,
            "export-kit-unreadable",
            FridayReleaseDeploymentGateReasonCategory::MissingEvidence,
            FridayReleaseDeploymentGateReasonSeverity::Blocking,
            "Evidence kit is missing",
            "Friday could not read the release evidence export kit JSON.".to_string(),
            export_kit_path,
            "Generate it with flow --friday-release-export-kit before opening the deployment gate.",
        );
    }

    push_check(
        &mut checklist,
        "release-qa",
        "Release QA command center",
        qa.as_ref().is_some_and(|report| report.ready_to_ship),
        qa.as_ref().map_or_else(
            || "Release QA report is missing or could not be parsed.".to_string(),
            |report| {
                format!(
                    "Score {} / 100 with {} blocking, {} stale, and {} missing check(s).",
                    report.score_out_of_100,
                    report.blocking_count,
                    report.stale_count,
                    report.missing_count
                )
            },
        ),
        qa_path,
    );
    if let Some(report) = &qa {
        if !report.ready_to_ship {
            push_reason(
                &mut reasons,
                "qa-not-ready",
                FridayReleaseDeploymentGateReasonCategory::BlockedQa,
                FridayReleaseDeploymentGateReasonSeverity::Blocking,
                "Release QA is blocked",
                report.summary.clone(),
                qa_path,
                "Clear blocking QA checks and refresh stale check-result files.",
            );
        }
    } else {
        push_reason(
            &mut reasons,
            "qa-unreadable",
            FridayReleaseDeploymentGateReasonCategory::BlockedQa,
            FridayReleaseDeploymentGateReasonSeverity::Blocking,
            "Release QA report is missing",
            "Friday could not read the QA command-center JSON.".to_string(),
            qa_path,
            "Generate it with flow --friday-release-qa before opening the deployment gate.",
        );
    }

    push_check(
        &mut checklist,
        "operator-checklist",
        "Operator checklist and signoff",
        checklist_report
            .as_ref()
            .is_some_and(|report| report.ready_to_ship && report.signoff_count > 0),
        checklist_report.as_ref().map_or_else(
            || "Release checklist is missing or could not be parsed.".to_string(),
            |report| {
                format!(
                    "{} / {} checklist item(s) ready with {} signoff(s).",
                    report.ready_count, report.total_count, report.signoff_count
                )
            },
        ),
        checklist_path,
    );
    if let Some(report) = &checklist_report {
        if report.signoff_count == 0 {
            push_reason(
                &mut reasons,
                "operator-signoff-missing",
                FridayReleaseDeploymentGateReasonCategory::UnsignedRelease,
                FridayReleaseDeploymentGateReasonSeverity::Blocking,
                "Operator signoff is missing",
                "The release checklist has no local operator signoff.".to_string(),
                checklist_path,
                "Append a local signoff after reviewing checklist, QA, evidence, and deployment target.",
            );
        }
        if !report.ready_to_ship {
            push_reason(
                &mut reasons,
                "operator-checklist-blocked",
                FridayReleaseDeploymentGateReasonCategory::UnsignedRelease,
                FridayReleaseDeploymentGateReasonSeverity::Blocking,
                "Release checklist is blocked",
                report.summary.clone(),
                checklist_path,
                "Resolve checklist blockers before deploying.",
            );
        }
    } else {
        push_reason(
            &mut reasons,
            "operator-checklist-unreadable",
            FridayReleaseDeploymentGateReasonCategory::UnsignedRelease,
            FridayReleaseDeploymentGateReasonSeverity::Blocking,
            "Release checklist is missing",
            "Friday could not read the operator checklist JSON.".to_string(),
            checklist_path,
            "Generate it with flow --friday-release-checklist before opening the deployment gate.",
        );
    }

    push_check(
        &mut checklist,
        "release-package",
        "Trusted runner release package",
        package.as_ref().is_some_and(|report| report.ready_to_ship),
        package.as_ref().map_or_else(
            || "Release package is missing or could not be parsed.".to_string(),
            |report| {
                format!(
                    "{} evidence file(s), {} missing, {} warning(s).",
                    report.manifest.evidence_count,
                    report.manifest.missing_count,
                    report.manifest.warning_count
                )
            },
        ),
        package_path,
    );
    if let Some(report) = &package {
        if !report.ready_to_ship {
            push_reason(
                &mut reasons,
                "release-package-not-ready",
                FridayReleaseDeploymentGateReasonCategory::MissingEvidence,
                FridayReleaseDeploymentGateReasonSeverity::Blocking,
                "Trusted runner release package is not ready",
                report.summary.clone(),
                package_path,
                "Regenerate the release package after runner history and live-state evidence are complete.",
            );
        }
    } else {
        push_reason(
            &mut reasons,
            "release-package-unreadable",
            FridayReleaseDeploymentGateReasonCategory::MissingEvidence,
            FridayReleaseDeploymentGateReasonSeverity::Blocking,
            "Release package is missing",
            "Friday could not read the trusted runner release package JSON.".to_string(),
            package_path,
            "Generate it with flow --friday-trusted-host-runner-release-package.",
        );
    }

    push_check(
        &mut checklist,
        "release-timeline",
        "Release evidence timeline",
        timeline.as_ref().is_some_and(|report| {
            report.package_count > 0
                && report.missing_evidence_regressions == 0
                && report.warning_regressions == 0
        }),
        timeline.as_ref().map_or_else(
            || "Release timeline is missing or could not be parsed.".to_string(),
            |report| {
                format!(
                    "{} package(s), {} missing-evidence regression(s), {} warning regression(s).",
                    report.package_count,
                    report.missing_evidence_regressions,
                    report.warning_regressions
                )
            },
        ),
        timeline_path,
    );
    if let Some(report) = &timeline {
        if report.package_count == 0 {
            push_reason(
                &mut reasons,
                "release-timeline-empty",
                FridayReleaseDeploymentGateReasonCategory::MissingEvidence,
                FridayReleaseDeploymentGateReasonSeverity::Blocking,
                "Release timeline has no package history",
                "The deployment gate needs at least one package entry for comparison and rollback context.".to_string(),
                timeline_path,
                "Archive the current release package into the timeline.",
            );
        }
        if report.missing_evidence_regressions > 0 || report.warning_regressions > 0 {
            push_reason(
                &mut reasons,
                "release-timeline-regressed",
                FridayReleaseDeploymentGateReasonCategory::MissingEvidence,
                FridayReleaseDeploymentGateReasonSeverity::Blocking,
                "Release timeline shows regressions",
                format!(
                    "{} missing-evidence regression(s), {} warning regression(s).",
                    report.missing_evidence_regressions, report.warning_regressions
                ),
                timeline_path,
                "Compare the latest package against the previous package and resolve regressions.",
            );
        }
    } else {
        push_reason(
            &mut reasons,
            "release-timeline-unreadable",
            FridayReleaseDeploymentGateReasonCategory::MissingEvidence,
            FridayReleaseDeploymentGateReasonSeverity::Blocking,
            "Release timeline is missing",
            "Friday could not read the release timeline JSON.".to_string(),
            timeline_path,
            "Generate or archive a trusted-runner release timeline.",
        );
    }

    push_check(
        &mut checklist,
        "dashboard-state",
        "Dashboard product UI state",
        dashboard
            .as_ref()
            .is_some_and(|binding| binding.status == FridayDashboardPanelStatus::Ready),
        dashboard.as_ref().map_or_else(
            || "Dashboard product binding is missing or could not be parsed.".to_string(),
            |binding| {
                format!(
                    "Dashboard score {} / 100, {} blocking, {} warning(s).",
                    binding.score_out_of_100, binding.blocking_count, binding.warning_count
                )
            },
        ),
        dashboard_export_dir,
    );
    if let Some(binding) = &dashboard {
        if binding.status != FridayDashboardPanelStatus::Ready || binding.blocking_count > 0 {
            push_reason(
                &mut reasons,
                "dashboard-not-ready",
                FridayReleaseDeploymentGateReasonCategory::DashboardState,
                FridayReleaseDeploymentGateReasonSeverity::Blocking,
                "Dashboard state is not ready",
                binding.summary.clone(),
                dashboard_export_dir,
                "Regenerate the dashboard export and clear blocking cards before deploying.",
            );
        }
    } else {
        push_reason(
            &mut reasons,
            "dashboard-unreadable",
            FridayReleaseDeploymentGateReasonCategory::DashboardState,
            FridayReleaseDeploymentGateReasonSeverity::Blocking,
            "Dashboard state is missing",
            "Friday could not read the dashboard product UI binding from the export directory.".to_string(),
            dashboard_export_dir,
            "Run flow --friday-dashboard-export and import/check the resulting dashboard state.",
        );
    }

    let product_name = export_kit
        .as_ref()
        .map(|report| report.manifest.product_name.clone())
        .or_else(|| qa.as_ref().map(|report| report.product_name.clone()))
        .or_else(|| checklist_report.as_ref().map(|report| report.product_name.clone()))
        .or_else(|| package.as_ref().map(|report| report.manifest.product_name.clone()))
        .unwrap_or_else(|| "Friday".to_string());
    let target_product_match = product_name == target.expected_product_name;
    let target_local_match = !target.local_only_required
        || export_kit
            .as_ref()
            .is_some_and(|report| report.manifest.local_only)
            && qa.as_ref().is_some_and(|report| report.local_only)
            && checklist_report
                .as_ref()
                .is_some_and(|report| report.local_only)
            && package
                .as_ref()
                .is_some_and(|report| report.manifest.local_only)
            && timeline.as_ref().is_some_and(|report| report.local_only);
    let target_provider_match =
        !target.requires_vercel || target.provider.to_ascii_lowercase().contains("vercel");
    let target_ready = target_product_match && target_local_match && target_provider_match;

    push_check(
        &mut checklist,
        "deployment-target",
        "Deployment target profile",
        target_ready,
        format!(
            "Target {} via {} in {}; local-only required={}.",
            target.label, target.provider, target.environment, target.local_only_required
        ),
        gate_path,
    );
    if !target_product_match {
        push_reason(
            &mut reasons,
            "target-product-mismatch",
            FridayReleaseDeploymentGateReasonCategory::TargetMismatch,
            FridayReleaseDeploymentGateReasonSeverity::Blocking,
            "Deployment target product name does not match evidence",
            format!(
                "Target expects `{}`, but evidence identifies `{}`.",
                target.expected_product_name, product_name
            ),
            gate_path,
            "Use a Friday target or regenerate evidence for the intended product.",
        );
    }
    if !target_local_match {
        push_reason(
            &mut reasons,
            "target-local-policy-mismatch",
            FridayReleaseDeploymentGateReasonCategory::TargetMismatch,
            FridayReleaseDeploymentGateReasonSeverity::Blocking,
            "Deployment target violates local-only policy",
            "This target requires local-only release evidence, but one or more evidence reports are missing or not local-only.".to_string(),
            gate_path,
            "Regenerate local-only evidence or explicitly choose a remote-allowed target.",
        );
    }
    if !target_provider_match {
        push_reason(
            &mut reasons,
            "target-provider-mismatch",
            FridayReleaseDeploymentGateReasonCategory::TargetMismatch,
            FridayReleaseDeploymentGateReasonSeverity::Blocking,
            "Deployment target provider mismatch",
            format!("Target requires Vercel, but provider is `{}`.", target.provider),
            gate_path,
            "Set --provider vercel or remove --vercel for a local checkpoint gate.",
        );
    }

    reasons.sort_by(|left, right| {
        left.severity
            .label()
            .cmp(right.severity.label())
            .then_with(|| left.id.cmp(&right.id))
    });
    let no_deploy_reason_count = reasons
        .iter()
        .filter(|reason| reason.severity == FridayReleaseDeploymentGateReasonSeverity::Blocking)
        .count();
    let warning_count = reasons
        .iter()
        .filter(|reason| reason.severity == FridayReleaseDeploymentGateReasonSeverity::Warning)
        .count();
    let total_count = checklist.len();
    let ready_count = checklist.iter().filter(|item| item.ready).count();
    let ready_to_deploy = no_deploy_reason_count == 0 && ready_count == total_count;
    let decision = if ready_to_deploy {
        FridayReleaseDeploymentGateDecision::Go
    } else if export_kit.is_none()
        && qa.is_none()
        && checklist_report.is_none()
        && package.is_none()
        && timeline.is_none()
    {
        FridayReleaseDeploymentGateDecision::Draft
    } else {
        FridayReleaseDeploymentGateDecision::NoGo
    };
    let status = match decision {
        FridayReleaseDeploymentGateDecision::Go => FridayDashboardPanelStatus::Ready,
        FridayReleaseDeploymentGateDecision::Draft => FridayDashboardPanelStatus::Warning,
        FridayReleaseDeploymentGateDecision::NoGo => FridayDashboardPanelStatus::Blocked,
    };
    let score_out_of_100 = score_gate(ready_count, total_count, no_deploy_reason_count, warning_count);
    let gate_json = path_string(gate_path);
    let commands = vec![
        format!(
            "flow --friday-release-deployment-gate --export-dir {} --output {}",
            path_string(dashboard_export_dir),
            gate_json
        ),
        format!(
            "flow --friday-release-export-kit --export-dir {} --output {}",
            path_string(dashboard_export_dir),
            path_string(export_kit_path)
        ),
        "cargo check --manifest-path src-tauri/Cargo.toml".to_string(),
        "cd extensions/flow-webext && npm run typecheck".to_string(),
    ];
    let deploy_checklist = deploy_checklist(&target, ready_to_deploy);
    let rollback_note = target.rollback_note.clone();
    let operator_copy = operator_copy(
        decision,
        score_out_of_100,
        &target,
        no_deploy_reason_count,
        warning_count,
        &gate_json,
        &rollback_note,
    );

    FridayReleaseDeploymentGateReport {
        gate_id: format!("friday-release-deployment-gate-{generated_at_unix_ms}"),
        gate_json,
        generated_at_unix_ms,
        product_name,
        local_only: target.local_only_required,
        status,
        decision,
        ready_to_deploy,
        score_out_of_100,
        summary: format!(
            "Friday deployment gate is {} at {} / 100 with {} blocking reason(s) and {} warning(s).",
            decision.label(),
            score_out_of_100,
            no_deploy_reason_count,
            warning_count
        ),
        target,
        export_kit_json: path_string(export_kit_path),
        qa_json: path_string(qa_path),
        checklist_json: path_string(checklist_path),
        package_json: path_string(package_path),
        timeline_json: path_string(timeline_path),
        dashboard_export_dir: path_string(dashboard_export_dir),
        no_deploy_reason_count,
        warning_count,
        ready_count,
        total_count,
        reasons,
        checklist,
        deploy_checklist,
        rollback_note,
        operator_copy,
        commands,
    }
}

pub fn write_friday_release_deployment_gate(
    gate_path: impl AsRef<Path>,
    report: &FridayReleaseDeploymentGateReport,
) -> Result<()> {
    let gate_path = gate_path.as_ref();
    if let Some(parent) = gate_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release deployment-gate directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(gate_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release deployment gate {}",
            gate_path.display()
        )
    })
}

pub fn read_friday_release_deployment_gate(
    gate_path: impl AsRef<Path>,
) -> Result<FridayReleaseDeploymentGateReport> {
    let gate_path = gate_path.as_ref();
    let bytes = fs::read(gate_path).with_context(|| {
        format!(
            "Could not read Friday release deployment gate {}",
            gate_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release deployment gate {}",
            gate_path.display()
        )
    })
}

fn push_check(
    checklist: &mut Vec<FridayReleaseDeploymentGateChecklistItem>,
    id: &str,
    title: &str,
    ready: bool,
    detail: String,
    source_path: &Path,
) {
    checklist.push(FridayReleaseDeploymentGateChecklistItem {
        id: id.to_string(),
        title: title.to_string(),
        ready,
        detail,
        source_path: path_string(source_path),
    });
}

#[allow(clippy::too_many_arguments)]
fn push_reason(
    reasons: &mut Vec<FridayReleaseDeploymentGateReason>,
    id: &str,
    category: FridayReleaseDeploymentGateReasonCategory,
    severity: FridayReleaseDeploymentGateReasonSeverity,
    title: &str,
    detail: String,
    source_path: &Path,
    next_action: &str,
) {
    reasons.push(FridayReleaseDeploymentGateReason {
        id: id.to_string(),
        category,
        severity,
        title: title.to_string(),
        detail,
        source_path: path_string(source_path),
        next_action: next_action.to_string(),
    });
}

fn score_gate(
    ready_count: usize,
    total_count: usize,
    no_deploy_reason_count: usize,
    warning_count: usize,
) -> u8 {
    if total_count == 0 {
        return 0;
    }

    let readiness = (ready_count as f32 / total_count as f32) * 100.0;
    let penalty = no_deploy_reason_count as f32 * 8.0 + warning_count as f32 * 3.0;
    (readiness - penalty).round().clamp(0.0, 100.0) as u8
}

fn deploy_checklist(target: &FridayReleaseDeploymentTarget, ready: bool) -> Vec<String> {
    if ready {
        vec![
            format!("Deploy target: {} ({})", target.label, target.environment),
            "Attach release deployment gate JSON and evidence export kit to the checkpoint note."
                .to_string(),
            "Keep rollback evidence available before promoting the new Friday build.".to_string(),
            "Do not switch local-only policy unless the operator explicitly approves the target."
                .to_string(),
        ]
    } else {
        vec![
            "Do not deploy yet.".to_string(),
            "Resolve every blocking deployment-gate reason first.".to_string(),
            "Refresh lightweight checks and regenerate QA, export kit, and deployment gate JSON."
                .to_string(),
            format!("Rollback note to keep ready: {}", target.rollback_note),
        ]
    }
}

fn operator_copy(
    decision: FridayReleaseDeploymentGateDecision,
    score_out_of_100: u8,
    target: &FridayReleaseDeploymentTarget,
    blocking_count: usize,
    warning_count: usize,
    gate_json: &str,
    rollback_note: &str,
) -> String {
    format!(
        "Friday deployment gate: {}\nScore: {} / 100\nTarget: {} ({}, {})\nBlocking reasons: {}\nWarnings: {}\nGate JSON: {}\nRollback: {}",
        decision.label(),
        score_out_of_100,
        target.label,
        target.environment,
        target.provider,
        blocking_count,
        warning_count,
        gate_json,
        rollback_note
    )
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
