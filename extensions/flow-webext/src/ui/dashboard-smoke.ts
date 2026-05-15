import {
  defaultFridayDashboardBinding,
  normalizeFridayDashboardBinding,
} from "../runtime/dashboard-binding";
import {
  buildTrustedHostRunnerCancellationUx,
  dispatchDashboardCommand,
  normalizeReleaseCandidateArchive,
  normalizeReleaseCheckpointEvidenceVault,
  normalizeReleaseCheckpointReview,
  normalizeReleaseCheckpointSignoffLedger,
  normalizeReleaseDeploymentGate,
  normalizeReleaseEscalationLedger,
  normalizeReleaseEvidenceAttachmentReview,
  normalizeReleaseEvidenceSlaMonitor,
  normalizeReleaseEvidenceExportKit,
  normalizeReleaseHandoffAuditTrail,
  normalizeReleaseHandoffDispatchChecklist,
  normalizeReleaseHandoffGovernanceReview,
  normalizeReleaseHandoffPacket,
  normalizeReleaseIncidentArchive,
  normalizeReleaseOperatorChecklist,
  normalizeReleaseOwnerFollowUpBoard,
  normalizeReleasePostPromotionMonitor,
  normalizeReleasePreventionPlan,
  normalizeReleasePromotionLedger,
  normalizeReleaseQaCommandCenter,
  normalizeReleaseRecoveryRunbook,
  normalizeReleaseRollbackDrill,
  normalizeReleaseStabilityBoard,
  normalizeDashboardHostCommandResults,
  normalizeTrustedHostLiveRunnerState,
  normalizeTrustedHostRunnerCancellationUx,
  normalizeTrustedHostRunnerOperatorReview,
  normalizeTrustedHostRunnerApprovalUi,
  normalizeTrustedHostRunnerResults,
  normalizeTrustedHostRunnerUx,
  normalizeTrustedRunnerReleasePackage,
  normalizeTrustedRunnerReleaseTimeline,
} from "../runtime/dashboard-actions";
import type { FlowDashboardProductUiBinding } from "../runtime/protocol";

type DashboardSmokeCheck = {
  id: string;
  passed: boolean;
  evidence: string;
};

type DashboardSmokeReport = {
  scoreOutOf100: number;
  checks: DashboardSmokeCheck[];
};

function check(id: string, passed: boolean, evidence: string): DashboardSmokeCheck {
  return { id, passed, evidence };
}

function score(checks: DashboardSmokeCheck[]) {
  if (checks.length === 0) {
    return 0;
  }

  const passed = checks.filter((item) => item.passed).length;
  return Math.round((passed / checks.length) * 100);
}

export function dashboardSectionSmokeReport(
  binding: FlowDashboardProductUiBinding = defaultFridayDashboardBinding(),
): DashboardSmokeReport {
  const imported = normalizeFridayDashboardBinding(
    {
      product_name: "Friday",
      route: "/dashboard",
      title: "Imported dashboard",
      source_file: "extensions/flow-webext/src/ui/app.ts",
      export_dir: "tmp/friday-dashboard",
      status: "ready",
      score_out_of_100: 100,
      summary: "Imported local JSON.",
      panel_json_command: "flow --friday-dashboard-panel-json tmp/friday-dashboard",
      export_command: "flow --friday-dashboard-export tmp/friday-dashboard",
      card_count: 1,
      bound_card_count: 1,
      action_count: 1,
      warning_count: 0,
      blocking_count: 0,
      cards: [
        {
          card_id: "imported-card",
          title: "Imported card",
          status: "ready",
          score_out_of_100: 100,
          primary_metric: "Imported from local JSON.",
          source_json: "tmp/friday-dashboard/imported.json",
          action_count: 1,
        },
      ],
      action_bindings: [
        {
          card_id: "imported-card",
          action_id: "imported-action",
          label: "Open imported",
          kind: "open",
          command: "flow --completion",
          local_only: true,
          enabled: true,
        },
      ],
      history: {
        record_count: 1,
        score_delta_from_previous: 0,
        readiness_delta_from_previous: 0,
        latest_score_out_of_100: 100,
        previous_score_out_of_100: 100,
        trend_label: "steady",
      },
      release_links: [
        {
          id: "summary",
          label: "Summary",
          kind: "markdown",
          path: "tmp/friday-dashboard/summary.md",
          section: "release-notes",
          local_only: true,
        },
      ],
      screenshot_prompts: [
        {
          route: "/dashboard",
          title: "Dashboard desktop",
          viewport_id: "desktop",
          status: "missing",
          prompt: "Capture dashboard.",
          capture_command: "agent-browser screenshot --route /dashboard",
        },
      ],
      next_actions: ["Review imported dashboard."],
    },
    "smoke-import.json",
  );
  const actionCount = binding.cards.reduce((total, card) => total + card.actions.length, 0);
  const executableAction = imported.cards[0]?.actions[0];
  const preparedResult = executableAction
    ? dispatchDashboardCommand(executableAction, {
        confirmed: true,
        now: "2026-05-15T00:00:00.000Z",
      })
    : null;
  const confirmationResult = executableAction
    ? dispatchDashboardCommand(
        {
          ...executableAction,
          buttonState: {
            ...executableAction.buttonState,
            requiresConfirmation: true,
          },
        },
        { now: "2026-05-15T00:00:01.000Z" },
      )
    : null;
  const blockedResult = executableAction
    ? dispatchDashboardCommand(
        {
          ...executableAction,
          localOnly: false,
        },
        { confirmed: true, now: "2026-05-15T00:00:02.000Z" },
      )
    : null;
  const failedResult = executableAction
    ? dispatchDashboardCommand(
        {
          ...executableAction,
          command: "",
        },
        { confirmed: true, now: "2026-05-15T00:00:03.000Z" },
      )
    : null;
  const hostBridgeResults = normalizeDashboardHostCommandResults({
    records: [
      {
        action_id: "host-open",
        label: "Open host report",
        kind: "open",
        command: "flow --completion",
        status: "awaiting-approval",
        approval_state: "required",
        audit: {
          stdout_summary: "not executed; waiting for operator approval",
          stderr_summary: "",
          recorded_at_unix_ms: 1,
        },
      },
    ],
  });
  const trustedRunnerResults = normalizeTrustedHostRunnerResults({
    action_id: "host-open",
    label: "Open host report",
    command: "flow --completion",
    status: "succeeded",
    approved: true,
    stdout_summary: "Flow Completion Loop",
    stderr_summary: "",
    recorded_at_unix_ms: 2,
  });
  const trustedRunnerUx = normalizeTrustedHostRunnerUx({
    history_json: "tmp/friday-dashboard/trusted-host-runner-history.json",
    result_count: 3,
    latest_status: "timed-out",
    status_summaries: [
      {
        status: "succeeded",
        count: 1,
        title: "Succeeded",
        description: "Approved host commands completed successfully.",
        tone: "ready",
      },
      {
        status: "timed-out",
        count: 1,
        title: "Timed out",
        description: "The runner stopped the command after its timeout.",
        tone: "warn",
      },
      {
        status: "cancelled",
        count: 1,
        title: "Cancelled",
        description: "The operator cancelled this run.",
        tone: "muted",
      },
    ],
    affordances: [
      {
        id: "copy-command-host-open",
        kind: "copy-command",
        action_id: "host-open",
        status: "timed-out",
        label: "Copy command",
        command: "flow --completion",
        detail: "Copy the original local command without executing it.",
        requires_approval: false,
        disabled: false,
      },
      {
        id: "retry-with-approval-host-open",
        kind: "retry",
        action_id: "host-open",
        status: "timed-out",
        label: "Retry with approval",
        command:
          "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id host-open --approve --execute",
        detail: "Prepare the same runner action again.",
        requires_approval: true,
        disabled: false,
      },
    ],
    operator_notes: [
      {
        id: "release-review-link",
        label: "Release review",
        detail: "Attach history to release review.",
        release_review_path: "tmp/friday-dashboard/release-review.json",
      },
    ],
  });
  const trustedRunnerApprovalUi = normalizeTrustedHostRunnerApprovalUi({
    history_json: "tmp/friday-dashboard/trusted-host-runner-history.json",
    result_count: 1,
    modal_id: "trusted-runner-approval",
    latest_action_id: "host-open",
    title: "Approve trusted runner action",
    body: "Review the local command and write an audit reason.",
    command_preview: "flow --completion",
    reason_label: "Audit reason",
    reason_placeholder: "why this command is safe",
    audit_reason_required: true,
    controls: [
      {
        id: "approve",
        kind: "approve",
        label: "Approve and run",
        command:
          "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id host-open --approve --execute --reason \"<audit reason>\"",
        detail: "Approve and run.",
        aria_label: "Approve and run",
        keyboard_shortcut: {
          key: "Ctrl+Enter",
          label: "Approve",
          detail: "Approve and copy the approved runner command.",
        },
        requires_reason: true,
        requires_approval: true,
        disabled: false,
      },
      {
        id: "deny",
        kind: "deny",
        label: "Deny",
        command:
          "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id host-open --reason \"<denial reason>\"",
        detail: "Deny without execution.",
        aria_label: "Deny",
        keyboard_shortcut: {
          key: "Esc",
          label: "Deny",
          detail: "Deny this approval draft.",
        },
        requires_reason: true,
        requires_approval: false,
        disabled: false,
      },
      {
        id: "snooze",
        kind: "snooze",
        label: "Snooze",
        command: "",
        detail: "Temporarily hide this draft.",
        aria_label: "Snooze",
        requires_reason: false,
        requires_approval: false,
        disabled: false,
      },
      {
        id: "undo",
        kind: "undo",
        label: "Undo draft",
        command: "",
        detail: "Clear the current draft.",
        aria_label: "Undo draft",
        requires_reason: false,
        requires_approval: false,
        disabled: false,
      },
    ],
    snooze_options: [
      {
        id: "snooze-5m",
        label: "Snooze 5 minutes",
        duration_seconds: 300,
      },
    ],
    undo_note: "Undo only clears the draft.",
    release_review_path: "tmp/friday-dashboard/release-review.json",
  });
  const trustedLiveRunnerState = normalizeTrustedHostLiveRunnerState({
    state_json: "tmp/friday-dashboard/trusted-host-live-state.json",
    generated_at_unix_ms: 1,
    record_count: 3,
    pending_count: 1,
    running_count: 1,
    finished_count: 0,
    stale_count: 1,
    stale_recovery_copy: "Refresh live runner state before trusting this dashboard.",
    records: [
      {
        job_id: "runner-pending",
        action_id: "host-open",
        label: "Open host report",
        command: "flow --completion",
        status: "pending",
        message: "Waiting for approval.",
        local_only: true,
        approved: false,
        timeout_ms: 30000,
        stale_after_ms: 120000,
        created_at_unix_ms: 1,
        updated_at_unix_ms: 1,
        finished_at_unix_ms: null,
        history_json: null,
        recovery_command:
          "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id host-open --cancel",
        cleanup_command:
          "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json",
      },
      {
        job_id: "runner-running",
        action_id: "host-run",
        label: "Run host report",
        command: "flow --friday-readiness",
        status: "running",
        message: "Executing.",
        local_only: true,
        approved: true,
        timeout_ms: 30000,
        stale_after_ms: 120000,
        created_at_unix_ms: 1,
        updated_at_unix_ms: 1,
        finished_at_unix_ms: null,
        history_json: null,
        recovery_command:
          "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id host-run --cancel",
        cleanup_command:
          "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json",
      },
      {
        job_id: "runner-stale",
        action_id: "host-stale",
        label: "Stale host report",
        command: "flow --friday-route-visuals",
        status: "stale",
        message: "Refresh before trusting.",
        local_only: true,
        approved: true,
        timeout_ms: 30000,
        stale_after_ms: 120000,
        created_at_unix_ms: 1,
        updated_at_unix_ms: 1,
        finished_at_unix_ms: 1,
        history_json: null,
        recovery_command:
          "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id host-stale --cancel",
        cleanup_command:
          "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json",
      },
    ],
  });
  const trustedCancellationUx =
    (trustedLiveRunnerState
      ? buildTrustedHostRunnerCancellationUx(trustedLiveRunnerState)
      : null) ??
    normalizeTrustedHostRunnerCancellationUx({
      state_json: "tmp/friday-dashboard/trusted-host-live-state.json",
      record_count: 0,
      active_count: 0,
      stale_count: 0,
      denial_count: 0,
      controls: [],
    });
  const trustedDenialRecoveryUx = normalizeTrustedHostRunnerCancellationUx({
    state_json: "tmp/friday-dashboard/trusted-host-live-state.json",
    record_count: 1,
    active_count: 0,
    stale_count: 0,
    denial_count: 1,
    draft: {
      storage_key: "flow.dashboard.runnerCancellationDrafts",
      default_reason: "Operator corrected approval decision",
      autosave_hint: "Denial recovery reasons are remembered locally.",
    },
    guidance: ["Recover denied runner records only with a clear audit reason."],
    controls: [
      {
        id: "recover-denied-runner",
        job_id: "runner-denied",
        action_id: "host-denied",
        kind: "denial-recovery",
        label: "Recover denied host report",
        command:
          'flow --friday-trusted-host-bridge-runner tmp/friday-dashboard --action-id host-denied --approve --execute --reason "<denial recovery reason>"',
        detail: "Approves and reruns the denied command.",
        requires_reason: true,
        disabled: false,
      },
    ],
  });
  const trustedOperatorReview = normalizeTrustedHostRunnerOperatorReview({
    history_json: "tmp/friday-dashboard/trusted-host-runner-history.json",
    review_id: "trusted-runner-review-smoke",
    generated_at_unix_ms: 1,
    record_count: 3,
    matched_count: 3,
    ready_count: 1,
    blocked_count: 2,
    release_gate_status: "blocked",
    filters: {
      status: null,
      action_id: "host",
      since_unix_ms: null,
      until_unix_ms: null,
      limit: 50,
    },
    release_gate_summaries: [
      {
        id: "succeeded",
        title: "Succeeded",
        severity: "ready",
        count: 1,
        detail: "Successful commands are ready for release review.",
        next_action: "Attach successful checks to release review.",
      },
      {
        id: "failed",
        title: "Failed",
        severity: "blocked",
        count: 1,
        detail: "Failed command blocks release until reviewed.",
        next_action: "Export incident notes and rerun.",
      },
      {
        id: "stale-live-state",
        title: "Stale live state",
        severity: "watch",
        count: 0,
        detail: "Import live state for stale runner records.",
        next_action: "Open the live runner state card.",
      },
    ],
    incident_notes: [
      {
        id: "incident-host-run",
        action_id: "host-run",
        status: "failed",
        severity: "blocked",
        title: "Failed: Run host report",
        body: "The command failed and blocks release.",
        export_markdown: "### Failed: Run host report",
        recorded_at_unix_ms: 1,
      },
    ],
    records: [
      {
        result_id: "host-run-1",
        action_id: "host-run",
        label: "Run host report",
        status: "failed",
        severity: "blocked",
        command: "flow --friday-readiness",
        summary: "Failed in 6ms: boom",
        release_gate: "This command failed and blocks release.",
        operator_reason: "reviewed locally",
        recorded_at_unix_ms: 1,
        duration_ms: 6,
        exit_code: 2,
      },
    ],
  });
  const trustedReleasePackage = normalizeTrustedRunnerReleasePackage({
    summary: "Trusted runner release package needs review.",
    ready_to_ship: false,
    warnings: ["1 trusted runner record still blocks release review."],
    manifest: {
      package_id: "trusted-runner-release-smoke",
      generated_at_unix_ms: 1,
      product_name: "Friday",
      local_only: true,
      package_json: "tmp/friday-dashboard/trusted-runner-release-package.json",
      dashboard_export_dir: "tmp/friday-dashboard",
      history_json: "tmp/friday-dashboard/trusted-host-runner-history.json",
      live_state_json: "tmp/friday-dashboard/trusted-host-live-state.json",
      release_review_json: "tmp/friday-dashboard/release-review.json",
      dashboard_index_json: "tmp/friday-dashboard/dashboard-index.json",
      evidence_count: 3,
      missing_count: 1,
      warning_count: 1,
      package_signature: "abc123signature",
      commands: ["flow --friday-trusted-host-runner-release-package tmp/friday-dashboard"],
      files: [
        {
          id: "runner-history",
          label: "Trusted runner history",
          kind: "runner-history-json",
          path: "tmp/friday-dashboard/trusted-host-runner-history.json",
          required: true,
          present: true,
          bytes: 120,
          sha256: "sha-history",
          warning: null,
        },
        {
          id: "runner-live-state",
          label: "Trusted runner live state",
          kind: "runner-live-state-json",
          path: "tmp/friday-dashboard/trusted-host-live-state.json",
          required: true,
          present: false,
          bytes: 0,
          sha256: null,
          warning: "Required evidence is missing.",
        },
        {
          id: "incident-notes",
          label: "Trusted runner incident notes",
          kind: "incident-markdown",
          path: "trusted-runner-incidents.md",
          required: false,
          present: true,
          bytes: 64,
          sha256: "sha-incidents",
          warning: null,
        },
      ],
    },
    operator_review: trustedOperatorReview,
    cancellation_ux: trustedCancellationUx,
    live_state: trustedLiveRunnerState,
    incident_markdown: "### Failed: Run host report",
  });
  const trustedReleaseTimeline = normalizeTrustedRunnerReleaseTimeline({
    timeline_id: "trusted-runner-release-timeline-smoke",
    timeline_json: "tmp/friday-dashboard/trusted-runner-release-timeline.json",
    generated_at_unix_ms: 3,
    local_only: true,
    package_count: 2,
    ready_count: 1,
    blocked_count: 1,
    latest_package_id: "trusted-runner-release-2",
    latest_package_json: "tmp/friday-dashboard/trusted-runner-release-package-2.json",
    missing_evidence_regressions: 1,
    warning_regressions: 1,
    signature_changes: 1,
    warnings: ["1 package comparison introduced new missing evidence."],
    entries: [
      {
        package_id: "trusted-runner-release-1",
        package_json: "tmp/friday-dashboard/trusted-runner-release-package-1.json",
        generated_at_unix_ms: 1,
        ready_to_ship: true,
        evidence_count: 6,
        missing_count: 0,
        warning_count: 0,
        stale_warning_count: 0,
        package_signature: "sig-1",
        missing_evidence_ids: [],
        summary: "Ready.",
      },
      {
        package_id: "trusted-runner-release-2",
        package_json: "tmp/friday-dashboard/trusted-runner-release-package-2.json",
        generated_at_unix_ms: 2,
        ready_to_ship: false,
        evidence_count: 6,
        missing_count: 1,
        warning_count: 2,
        stale_warning_count: 1,
        package_signature: "sig-2",
        missing_evidence_ids: ["runner-live-state"],
        summary: "Needs review.",
      },
    ],
    diffs: [
      {
        from_package_id: "trusted-runner-release-1",
        to_package_id: "trusted-runner-release-2",
        evidence_delta: 0,
        missing_delta: 1,
        warning_delta: 2,
        stale_warning_delta: 1,
        signature_changed: true,
        new_missing_evidence_ids: ["runner-live-state"],
        resolved_missing_evidence_ids: [],
        regression: true,
        summary: "trusted-runner-release-2 regressed from trusted-runner-release-1.",
      },
    ],
  });
  const releaseChecklist = normalizeReleaseOperatorChecklist({
    checklist_id: "friday-release-checklist-smoke",
    checklist_json: "tmp/friday-dashboard/release-operator-checklist.json",
    generated_at_unix_ms: 4,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    ready_to_ship: false,
    summary: "5/7 release checklist items ready; 1 warning, 1 blocking issue.",
    package_json: "tmp/friday-dashboard/trusted-runner-release-package.json",
    timeline_json: "tmp/friday-dashboard/trusted-runner-release-timeline.json",
    dashboard_export_dir: "tmp/friday-dashboard",
    todo_path: "TODO.md",
    changelog_path: "CHANGELOG.md",
    signoff_json: "tmp/friday-dashboard/release-signoffs.json",
    ready_count: 5,
    total_count: 7,
    warning_count: 1,
    blocking_count: 1,
    signoff_required: true,
    signoff_count: 0,
    latest_signoff: null,
    blockers: [
      {
        id: "active-loop-not-complete",
        category: "unreviewed-changes",
        severity: "blocking",
        title: "Active TODO loop is not complete",
        detail: "Friday Release Operator Checklist is still at 0 / 100.",
        source_path: "TODO.md",
        next_action: "Complete the current TODO loop.",
      },
    ],
    checklist: [
      {
        id: "release-package",
        title: "Trusted runner release package",
        ready: true,
        detail: "3 evidence item(s), 0 missing, 0 warning(s).",
        source_path: "tmp/friday-dashboard/trusted-runner-release-package.json",
      },
      {
        id: "operator-signoff",
        title: "Operator signoff",
        ready: false,
        detail: "No operator signoff has been recorded yet.",
        source_path: "tmp/friday-dashboard/release-signoffs.json",
      },
    ],
    signoffs: [],
    commands: [
      "flow --friday-release-checklist tmp/friday-dashboard",
      'flow --friday-release-signoff tmp/friday-dashboard/release-operator-checklist.json --signoffs tmp/friday-dashboard/release-signoffs.json --operator "<operator>" --decision approved --reason "<signoff reason>"',
    ],
  });
  const releaseQa = normalizeReleaseQaCommandCenter({
    report_id: "friday-release-qa-smoke",
    report_json: "tmp/friday-dashboard/release-qa-command-center.json",
    generated_at_unix_ms: 5,
    product_name: "Friday",
    local_only: true,
    status: "warning",
    score_out_of_100: 83,
    ready_to_ship: false,
    summary: "Friday release QA is 83/100 with 1 stale result.",
    checklist_json: "tmp/friday-dashboard/release-operator-checklist.json",
    package_json: "tmp/friday-dashboard/trusted-runner-release-package.json",
    timeline_json: "tmp/friday-dashboard/trusted-runner-release-timeline.json",
    warning_count: 1,
    blocking_count: 0,
    stale_count: 1,
    missing_count: 0,
    checks: [
      {
        id: "rust-cargo-check",
        label: "Rust cargo check",
        command: "cargo check",
        result_path: "tmp/friday-dashboard/cargo-check.txt",
        required: true,
        present: true,
        stale: false,
        bytes: 42,
        status: "passed",
        summary: "cargo check passed",
        next_action: "Refresh after code changes.",
      },
      {
        id: "dashboard-smoke",
        label: "Dashboard smoke",
        command: "npm run smoke:dashboard",
        result_path: "tmp/friday-dashboard/dashboard-smoke.txt",
        required: true,
        present: true,
        stale: true,
        bytes: 84,
        status: "stale",
        summary: "Dashboard smoke result is stale.",
        next_action: "Rerun dashboard smoke.",
      },
    ],
    commands: ["flow --friday-release-qa tmp/friday-dashboard"],
  });
  const releaseExportKit = normalizeReleaseEvidenceExportKit({
    summary: "Friday release evidence kit has 8 file(s), 0 missing, 1 stale, and 2 warning(s).",
    ready_to_attach: false,
    status: "warning",
    checklist_ready: false,
    qa_score_out_of_100: 83,
    qa_ready_to_ship: false,
    package_ready_to_ship: false,
    timeline_package_count: 2,
    signoff_count: 1,
    warnings: [
      "Release QA is not ready.",
      "Dashboard smoke result is older than 24 hours.",
    ],
    operator_copy:
      "Friday release evidence kit: tmp/friday-dashboard/release-evidence-export-kit.json\nStatus: needs review\nManifest checksum: kit-sha",
    manifest: {
      kit_id: "friday-release-export-kit-smoke",
      generated_at_unix_ms: 6,
      product_name: "Friday",
      local_only: true,
      kit_json: "tmp/friday-dashboard/release-evidence-export-kit.json",
      export_dir: "tmp/friday-dashboard",
      file_count: 8,
      required_count: 8,
      missing_count: 0,
      stale_count: 1,
      warning_count: 2,
      manifest_sha256: "kit-sha",
      commands: [
        "cargo check > tmp/friday-dashboard/cargo-check.txt",
        "flow --friday-release-export-kit tmp/friday-dashboard",
      ],
      files: [
        {
          id: "release-checklist",
          label: "Release operator checklist",
          kind: "release-checklist-json",
          path: "tmp/friday-dashboard/release-operator-checklist.json",
          required: true,
          present: true,
          stale: false,
          bytes: 120,
          sha256: "sha-checklist",
          warning: null,
        },
        {
          id: "dashboard-smoke-result",
          label: "Dashboard smoke result",
          kind: "check-result",
          path: "tmp/friday-dashboard/dashboard-smoke.txt",
          required: true,
          present: true,
          stale: true,
          bytes: 84,
          sha256: "sha-smoke",
          warning: "Dashboard smoke result is older than 24 hours.",
        },
      ],
    },
  });
  const releaseDeploymentGate = normalizeReleaseDeploymentGate({
    gate_id: "friday-release-deployment-gate-smoke",
    gate_json: "tmp/friday-dashboard/release-deployment-gate.json",
    generated_at_unix_ms: 7,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    decision: "no-go",
    ready_to_deploy: false,
    score_out_of_100: 71,
    summary:
      "Friday deployment gate is no-go at 71 / 100 with 2 blocking reason(s) and 1 warning(s).",
    target: {
      id: "local-friday-checkpoint",
      label: "Local Friday checkpoint",
      environment: "local",
      provider: "local",
      url: null,
      local_only_required: true,
      requires_vercel: false,
      expected_product_name: "Friday",
      rollback_note: "Keep previous evidence attached.",
    },
    export_kit_json: "tmp/friday-dashboard/release-evidence-export-kit.json",
    qa_json: "tmp/friday-dashboard/release-qa-command-center.json",
    checklist_json: "tmp/friday-dashboard/release-operator-checklist.json",
    package_json: "tmp/friday-dashboard/trusted-runner-release-package.json",
    timeline_json: "tmp/friday-dashboard/trusted-runner-release-timeline.json",
    dashboard_export_dir: "tmp/friday-dashboard",
    no_deploy_reason_count: 2,
    warning_count: 1,
    ready_count: 5,
    total_count: 7,
    reasons: [
      {
        id: "qa-not-ready",
        category: "blocked-qa",
        severity: "blocking",
        title: "Release QA is blocked",
        detail: "Dashboard smoke is stale.",
        source_path: "tmp/friday-dashboard/release-qa-command-center.json",
        next_action: "Refresh dashboard smoke.",
      },
    ],
    checklist: [
      {
        id: "release-qa",
        title: "Release QA command center",
        ready: false,
        detail: "Score 83 / 100 with stale dashboard smoke.",
        source_path: "tmp/friday-dashboard/release-qa-command-center.json",
      },
    ],
    deploy_checklist: ["Do not deploy yet.", "Resolve every blocking deployment-gate reason first."],
    rollback_note: "Keep previous evidence attached.",
    operator_copy:
      "Friday deployment gate: no-go\nScore: 71 / 100\nTarget: Local Friday checkpoint",
    commands: ["flow --friday-release-deployment-gate tmp/friday-dashboard"],
  });
  const releaseCandidateArchive = normalizeReleaseCandidateArchive({
    archive_id: "friday-release-candidate-archive-smoke",
    archive_json: "tmp/friday-dashboard/release-candidate-archive.json",
    generated_at_unix_ms: 8,
    local_only: true,
    candidate_count: 2,
    latest_candidate_id: "candidate-regressed",
    latest_decision: "no-go",
    latest_score_out_of_100: 71,
    go_count: 0,
    no_go_count: 2,
    draft_count: 0,
    regression_count: 1,
    entries: [
      {
        candidate_id: "candidate-initial",
        gate_id: "gate-initial",
        gate_json: "tmp/friday-dashboard/release-deployment-gate-1.json",
        export_kit_json: "tmp/friday-dashboard/release-evidence-export-kit.json",
        generated_at_unix_ms: 7,
        product_name: "Friday",
        decision: "no-go",
        score_out_of_100: 81,
        ready_to_deploy: false,
        target: {
          id: "local-friday-checkpoint",
          label: "Local Friday checkpoint",
          environment: "local",
          provider: "local",
          url: null,
          local_only_required: true,
          requires_vercel: false,
          expected_product_name: "Friday",
          rollback_note: "Keep previous evidence attached.",
        },
        no_deploy_reason_count: 1,
        warning_count: 1,
        reason_ids: ["qa-not-ready"],
        export_kit_manifest_sha256: "kit-sha",
        rollback_note: "Keep previous evidence attached.",
        summary: "Initial no-go candidate.",
      },
      {
        candidate_id: "candidate-regressed",
        gate_id: "gate-regressed",
        gate_json: "tmp/friday-dashboard/release-deployment-gate-2.json",
        export_kit_json: "tmp/friday-dashboard/release-evidence-export-kit.json",
        generated_at_unix_ms: 8,
        product_name: "Friday",
        decision: "no-go",
        score_out_of_100: 71,
        ready_to_deploy: false,
        target: {
          id: "local-friday-checkpoint",
          label: "Local Friday checkpoint",
          environment: "local",
          provider: "local",
          url: null,
          local_only_required: true,
          requires_vercel: false,
          expected_product_name: "Friday",
          rollback_note: "Keep previous evidence attached.",
        },
        no_deploy_reason_count: 2,
        warning_count: 1,
        reason_ids: ["qa-not-ready", "new-deploy-blocker"],
        export_kit_manifest_sha256: "kit-sha-2",
        rollback_note: "Keep previous evidence attached.",
        summary: "Regressed no-go candidate.",
      },
    ],
    diffs: [
      {
        from_candidate_id: "candidate-initial",
        to_candidate_id: "candidate-regressed",
        score_delta: -10,
        decision_changed: false,
        target_changed: false,
        evidence_checksum_changed: true,
        new_blocker_ids: ["new-deploy-blocker"],
        resolved_blocker_ids: [],
        regression: true,
        summary: "Candidate score changed by -10.",
      },
    ],
    commands: [
      "flow --friday-release-candidate-archive --archive tmp/friday-dashboard/release-candidate-archive.json --gate <deployment-gate.json>",
    ],
  });
  const releasePromotionLedger = normalizeReleasePromotionLedger({
    ledger_id: "friday-release-promotion-ledger-smoke",
    ledger_json: "tmp/friday-dashboard/release-promotion-ledger.json",
    generated_at_unix_ms: 9,
    local_only: true,
    record_count: 2,
    promoted_count: 1,
    held_count: 1,
    rolled_back_count: 0,
    superseded_count: 0,
    abandoned_count: 0,
    post_promotion_missing_count: 1,
    active_promotion_id: "promotion-candidate-promoted",
    active_candidate_id: "candidate-promoted",
    active_rollback_reference: "candidate-initial",
    latest_decision: "promoted",
    latest_deployment_note: "Promoted to the local Friday checkpoint after QA review.",
    warnings: ["Latest promoted candidate is missing 1 post-promotion check(s)."],
    records: [
      {
        promotion_id: "promotion-candidate-held",
        candidate_id: "candidate-regressed",
        archive_json: "tmp/friday-dashboard/release-candidate-archive.json",
        gate_json: "tmp/friday-dashboard/release-deployment-gate-2.json",
        export_kit_json: "tmp/friday-dashboard/release-evidence-export-kit.json",
        recorded_at_unix_ms: 8,
        product_name: "Friday",
        local_only: true,
        decision: "held",
        operator: "essencefromexistence",
        reason: "Held until dashboard smoke is fresh.",
        deployment_note: "No deployment performed.",
        target: {
          id: "local-friday-checkpoint",
          label: "Local Friday checkpoint",
          environment: "local",
          provider: "local",
          url: null,
          local_only_required: true,
          requires_vercel: false,
          expected_product_name: "Friday",
          rollback_note: "Keep previous evidence attached.",
        },
        rollback_reference: "candidate-initial",
        candidate_score_out_of_100: 71,
        candidate_ready_to_deploy: false,
        candidate_blocker_count: 2,
        post_promotion_required_count: 2,
        post_promotion_missing_count: 0,
        post_promotion_checks: [
          {
            id: "deployment-note",
            label: "Deployment note",
            result_path: "inline",
            required: true,
            present: true,
            bytes: 0,
            summary: "Deployment note is recorded.",
            next_action: "Attach note.",
          },
        ],
        summary: "Candidate was held.",
      },
      {
        promotion_id: "promotion-candidate-promoted",
        candidate_id: "candidate-promoted",
        archive_json: "tmp/friday-dashboard/release-candidate-archive.json",
        gate_json: "tmp/friday-dashboard/release-deployment-gate-3.json",
        export_kit_json: "tmp/friday-dashboard/release-evidence-export-kit.json",
        recorded_at_unix_ms: 9,
        product_name: "Friday",
        local_only: true,
        decision: "promoted",
        operator: "essencefromexistence",
        reason: "Promoted after lightweight checks were reviewed.",
        deployment_note: "Promoted to the local Friday checkpoint after QA review.",
        target: {
          id: "local-friday-checkpoint",
          label: "Local Friday checkpoint",
          environment: "local",
          provider: "local",
          url: null,
          local_only_required: true,
          requires_vercel: false,
          expected_product_name: "Friday",
          rollback_note: "Keep previous evidence attached.",
        },
        rollback_reference: "candidate-initial",
        candidate_score_out_of_100: 94,
        candidate_ready_to_deploy: true,
        candidate_blocker_count: 0,
        post_promotion_required_count: 3,
        post_promotion_missing_count: 1,
        post_promotion_checks: [
          {
            id: "deployment-note",
            label: "Deployment note",
            result_path: "inline",
            required: true,
            present: true,
            bytes: 0,
            summary: "Deployment note is recorded.",
            next_action: "Attach note.",
          },
          {
            id: "post-promotion-smoke",
            label: "post promotion smoke",
            result_path: "tmp/friday-dashboard/post-promotion-smoke.json",
            required: true,
            present: false,
            bytes: 0,
            summary: "Post-promotion check evidence is missing.",
            next_action: "Create the check-result file and record promotion again.",
          },
        ],
        summary: "Candidate was promoted with one missing post-promotion check.",
      },
    ],
    commands: [
      "flow --friday-release-promotion-ledger --ledger tmp/friday-dashboard/release-promotion-ledger.json --archive tmp/friday-dashboard/release-candidate-archive.json --decision held --reason \"<reason>\"",
    ],
  });
  const releasePostPromotionMonitor = normalizeReleasePostPromotionMonitor({
    monitor_id: "friday-release-post-promotion-monitor-smoke",
    monitor_json: "tmp/friday-dashboard/release-post-promotion-monitor.json",
    generated_at_unix_ms: 10,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    score_out_of_100: 72,
    ready_for_stable: false,
    promotion_ledger_json: "tmp/friday-dashboard/release-promotion-ledger.json",
    qa_json: "tmp/friday-dashboard/release-qa-command-center.json",
    dashboard_smoke_result_path: "tmp/friday-dashboard/dashboard-smoke.txt",
    active_candidate_id: "candidate-promoted",
    active_promotion_id: "promotion-candidate-promoted",
    active_rollback_reference: "candidate-initial",
    latest_decision: "promoted",
    promoted_count: 1,
    incident_note_count: 1,
    missing_evidence_count: 1,
    stale_count: 1,
    warning_count: 1,
    blocking_count: 1,
    checks: [
      {
        id: "promotion-ledger",
        label: "Release promotion ledger",
        source_path: "tmp/friday-dashboard/release-promotion-ledger.json",
        required: true,
        present: true,
        stale: false,
        bytes: 900,
        status: "passed",
        summary: "Two promotion records with one promoted candidate.",
        next_action: "Keep promotion ledger attached.",
      },
      {
        id: "post-promotion-smoke",
        label: "post promotion smoke",
        source_path: "tmp/friday-dashboard/post-promotion-smoke.json",
        required: true,
        present: false,
        stale: false,
        bytes: 0,
        status: "missing",
        summary: "Post-promotion check evidence is missing.",
        next_action: "Create the post-promotion smoke file.",
      },
      {
        id: "dashboard-smoke",
        label: "Dashboard smoke result",
        source_path: "tmp/friday-dashboard/dashboard-smoke.txt",
        required: true,
        present: true,
        stale: true,
        bytes: 1400,
        status: "stale",
        summary: "Dashboard smoke is stale.",
        next_action: "Refresh dashboard smoke.",
      },
    ],
    incident_notes: [
      {
        id: "voice-overlay-followup",
        path: "tmp/friday-dashboard/incidents/voice-overlay-followup.md",
        present: true,
        bytes: 240,
        summary: "Incident note evidence is present.",
      },
    ],
    warnings: ["1 post-promotion evidence item(s) are missing."],
    summary:
      "Friday post-promotion monitor is 72/100 with 1 blocking issue, 1 warning, and 1 stale check.",
    commands: [
      "flow --friday-release-post-promotion-monitor --output tmp/friday-dashboard/release-post-promotion-monitor.json --promotion-ledger tmp/friday-dashboard/release-promotion-ledger.json",
    ],
  });
  const releaseRollbackDrill = normalizeReleaseRollbackDrill({
    drill_id: "friday-release-rollback-drill-smoke",
    drill_json: "tmp/friday-dashboard/release-rollback-drill.json",
    generated_at_unix_ms: 11,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    score_out_of_100: 75,
    ready_to_rollback: false,
    ready_for_stable: false,
    active_candidate_id: "candidate-promoted",
    active_promotion_id: "promotion-candidate-promoted",
    active_rollback_reference: "candidate-initial",
    latest_promotion_decision: "promoted",
    deployment_gate_decision: "no-go",
    post_promotion_monitor_json: "tmp/friday-dashboard/release-post-promotion-monitor.json",
    promotion_ledger_json: "tmp/friday-dashboard/release-promotion-ledger.json",
    candidate_archive_json: "tmp/friday-dashboard/release-candidate-archive.json",
    deployment_gate_json: "tmp/friday-dashboard/release-deployment-gate.json",
    rollback_command:
      "flow --friday-release-rollback-drill-json --output tmp/friday-dashboard/release-rollback-drill.json",
    dry_run_command:
      "flow --friday-release-rollback-drill-json --output tmp/friday-dashboard/release-rollback-drill.json",
    operator: "essencefromexistence",
    reason: "Smoke rollback drill.",
    blocking_count: 1,
    warning_count: 1,
    stale_count: 1,
    missing_evidence_count: 0,
    checks: [
      {
        id: "rollback-reference",
        label: "Rollback reference",
        source_path: "inline",
        required: true,
        present: true,
        stale: false,
        bytes: 0,
        status: "passed",
        summary: "Active rollback reference is candidate-initial.",
        next_action: "Keep this reference attached.",
      },
      {
        id: "post-promotion-monitor",
        label: "Post-promotion monitor",
        source_path: "tmp/friday-dashboard/release-post-promotion-monitor.json",
        required: true,
        present: true,
        stale: false,
        bytes: 1500,
        status: "failed",
        summary: "Monitor has 1 blocking issue.",
        next_action: "Resolve post-promotion blockers.",
      },
      {
        id: "dashboard-smoke",
        label: "Dashboard smoke",
        source_path: "tmp/friday-dashboard/dashboard-smoke.txt",
        required: true,
        present: true,
        stale: true,
        bytes: 1400,
        status: "stale",
        summary: "Dashboard smoke is stale.",
        next_action: "Refresh dashboard smoke.",
      },
    ],
    blocked_reasons: [
      "Post-promotion monitor: Monitor has 1 blocking issue.",
      "Dashboard smoke: Dashboard smoke is stale.",
    ],
    summary:
      "Friday rollback drill is 75/100 with 1 blocking issue, 1 warning, and 1 stale check.",
    commands: [
      "flow --friday-release-rollback-drill --output tmp/friday-dashboard/release-rollback-drill.json",
      "flow --friday-release-rollback-drill-json --output tmp/friday-dashboard/release-rollback-drill.json",
    ],
  });
  const releaseStabilityBoard = normalizeReleaseStabilityBoard({
    board_id: "friday-release-stability-board-smoke",
    board_json: "tmp/friday-dashboard/release-stability-board.json",
    generated_at_unix_ms: 12,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    score_out_of_100: 58,
    ready_for_checkpoint: false,
    ready_to_deploy: false,
    stable_after_promotion: false,
    recoverable: false,
    active_candidate_id: "candidate-promoted",
    active_promotion_id: "promotion-candidate-promoted",
    active_rollback_reference: "candidate-initial",
    latest_promotion_decision: "promoted",
    deployment_gate_decision: "no-go",
    qa_json: "tmp/friday-dashboard/release-qa-command-center.json",
    candidate_archive_json: "tmp/friday-dashboard/release-candidate-archive.json",
    promotion_ledger_json: "tmp/friday-dashboard/release-promotion-ledger.json",
    post_promotion_monitor_json: "tmp/friday-dashboard/release-post-promotion-monitor.json",
    rollback_drill_json: "tmp/friday-dashboard/release-rollback-drill.json",
    deployment_gate_json: "tmp/friday-dashboard/release-deployment-gate.json",
    blocking_count: 3,
    warning_count: 1,
    stale_count: 1,
    missing_evidence_count: 0,
    checks: [
      {
        id: "deployment-gate",
        label: "Deployment readiness",
        category: "deployment-readiness",
        source_path: "tmp/friday-dashboard/release-deployment-gate.json",
        required: true,
        present: true,
        stale: false,
        bytes: 1600,
        status: "failed",
        summary: "Deployment gate is no-go with 2 no-deploy reasons.",
        next_action: "Resolve deployment gate blockers.",
      },
      {
        id: "rollback-recovery",
        label: "Rollback recovery",
        category: "rollback-recovery",
        source_path: "tmp/friday-dashboard/release-rollback-drill.json",
        required: true,
        present: true,
        stale: false,
        bytes: 1500,
        status: "failed",
        summary: "Rollback drill has 1 blocking issue.",
        next_action: "Run a clean rollback drill.",
      },
      {
        id: "post-promotion-freshness",
        label: "Post-promotion freshness",
        category: "post-promotion-freshness",
        source_path: "tmp/friday-dashboard/release-post-promotion-monitor.json",
        required: true,
        present: true,
        stale: true,
        bytes: 1500,
        status: "stale",
        summary: "Post-promotion monitor has stale evidence.",
        next_action: "Refresh post-promotion evidence.",
      },
    ],
    evidence_links: [
      {
        id: "release-qa",
        label: "Release QA",
        path: "tmp/friday-dashboard/release-qa-command-center.json",
        present: true,
      },
      {
        id: "rollback-drill",
        label: "Rollback drill",
        path: "tmp/friday-dashboard/release-rollback-drill.json",
        present: true,
      },
    ],
    active_risks: [
      "Deployment readiness: Deployment gate is no-go with 2 no-deploy reasons.",
      "Rollback recovery: Rollback drill has 1 blocking issue.",
    ],
    next_actions: ["Resolve deployment gate blockers.", "Run a clean rollback drill."],
    summary:
      "Friday stability board is 58/100 with 3 blocking issues, 1 warning, and 1 stale check.",
    commands: [
      "flow --friday-release-stability-board --output tmp/friday-dashboard/release-stability-board.json",
      "flow --friday-release-stability-board-json --output tmp/friday-dashboard/release-stability-board.json",
    ],
  });
  const releaseRecoveryRunbook = normalizeReleaseRecoveryRunbook({
    runbook_id: "friday-release-recovery-runbook-smoke",
    runbook_json: "tmp/friday-dashboard/release-recovery-runbook.json",
    generated_at_unix_ms: 13,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    score_out_of_100: 63,
    ready_for_operator_review: true,
    ready_to_execute_recovery: false,
    active_candidate_id: "candidate-promoted",
    active_promotion_id: "promotion-candidate-promoted",
    active_rollback_reference: "candidate-initial",
    latest_promotion_decision: "promoted",
    stability_board_json: "tmp/friday-dashboard/release-stability-board.json",
    rollback_drill_json: "tmp/friday-dashboard/release-rollback-drill.json",
    promotion_ledger_json: "tmp/friday-dashboard/release-promotion-ledger.json",
    post_promotion_monitor_json: "tmp/friday-dashboard/release-post-promotion-monitor.json",
    phase_count: 6,
    blocked_phase_count: 2,
    approval_gate_count: 3,
    unsatisfied_approval_gate_count: 3,
    command_count: 6,
    active_risks: ["Rollback recovery: Rollback drill has 1 blocking issue."],
    phases: [
      {
        kind: "pause",
        order: 1,
        label: "Pause Friday runtime",
        status: "requires-approval",
        approval_required: true,
        source_path: "tmp/friday-dashboard/release-stability-board.json",
        objective: "Pause live Friday automation.",
        command:
          "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json",
        verification: "Confirm no unsafe active action.",
        risks: ["Rollback recovery: Rollback drill has 1 blocking issue."],
        evidence_paths: ["tmp/friday-dashboard/release-stability-board.json"],
        next_action: "Get approval before pausing.",
      },
      {
        kind: "diagnose",
        order: 2,
        label: "Diagnose release risk",
        status: "ready",
        approval_required: false,
        source_path: "tmp/friday-dashboard/release-stability-board.json",
        objective: "Review release risk before rollback.",
        command:
          "flow --friday-release-stability-board-json --output tmp/friday-dashboard/release-stability-board.json",
        verification: "Stability board explains active risks.",
        risks: ["Deployment readiness: Deployment gate is no-go."],
        evidence_paths: ["tmp/friday-dashboard/release-stability-board.json"],
        next_action: "Refresh stale evidence.",
      },
      {
        kind: "rollback",
        order: 3,
        label: "Rollback dry run",
        status: "blocked",
        approval_required: true,
        source_path: "tmp/friday-dashboard/release-rollback-drill.json",
        objective: "Rehearse rollback locally.",
        command:
          "flow --friday-release-rollback-drill-json --output tmp/friday-dashboard/release-rollback-drill.json --dry-run",
        verification: "Dry run must not deploy or destroy data.",
        risks: ["Post-promotion monitor: Monitor has 1 blocking issue."],
        evidence_paths: ["tmp/friday-dashboard/release-rollback-drill.json"],
        next_action: "Resolve rollback blockers.",
      },
      {
        kind: "verify",
        order: 4,
        label: "Verify recovery state",
        status: "blocked",
        approval_required: false,
        source_path: "tmp/friday-dashboard/release-post-promotion-monitor.json",
        objective: "Verify post-promotion checks.",
        command:
          "flow --friday-release-post-promotion-monitor-json --output tmp/friday-dashboard/release-post-promotion-monitor.json",
        verification: "Monitor has no blocking, stale, or missing evidence.",
        risks: ["Dashboard smoke is stale."],
        evidence_paths: ["tmp/friday-dashboard/release-post-promotion-monitor.json"],
        next_action: "Refresh post-promotion evidence.",
      },
      {
        kind: "resume",
        order: 5,
        label: "Resume Friday runtime",
        status: "requires-approval",
        approval_required: true,
        source_path: "tmp/friday-dashboard/release-stability-board.json",
        objective: "Resume Friday only after recovery evidence is clean.",
        command:
          "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json --history tmp/friday-dashboard/trusted-host-runner-history.json",
        verification: "No recovery phase remains blocked.",
        risks: [],
        evidence_paths: ["tmp/friday-dashboard/release-stability-board.json"],
        next_action: "Get approval before resuming.",
      },
      {
        kind: "follow-up",
        order: 6,
        label: "Follow-up incident notes",
        status: "ready",
        approval_required: false,
        source_path: "tmp/friday-dashboard/release-promotion-ledger.json",
        objective: "Capture incident notes and prevention work.",
        command:
          "flow --friday-release-post-promotion-monitor --output tmp/friday-dashboard/release-post-promotion-monitor.json --promotion-ledger tmp/friday-dashboard/release-promotion-ledger.json --incident-note <incident-note.md>",
        verification: "Incident note explains trigger, action, verification, and prevention.",
        risks: [],
        evidence_paths: [
          "tmp/friday-dashboard/release-promotion-ledger.json",
          "tmp/friday-dashboard/release-post-promotion-monitor.json",
        ],
        next_action: "Attach an incident note.",
      },
    ],
    approval_gates: [
      {
        id: "approve-pause",
        label: "Approve Pause Friday runtime",
        phase: "pause",
        required: true,
        satisfied: false,
        summary: "Pause requires explicit approval.",
        operator_action: "Review pause.",
      },
      {
        id: "approve-rollback",
        label: "Approve Rollback dry run",
        phase: "rollback",
        required: true,
        satisfied: false,
        summary: "Rollback reference `candidate-initial` requires explicit operator approval.",
        operator_action: "Review rollback.",
      },
      {
        id: "approve-resume",
        label: "Approve Resume Friday runtime",
        phase: "resume",
        required: true,
        satisfied: false,
        summary: "Resume requires explicit approval.",
        operator_action: "Review resume.",
      },
    ],
    recovery_commands: [
      "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json",
      "flow --friday-release-stability-board-json --output tmp/friday-dashboard/release-stability-board.json",
      "flow --friday-release-rollback-drill-json --output tmp/friday-dashboard/release-rollback-drill.json --dry-run",
      "flow --friday-release-post-promotion-monitor-json --output tmp/friday-dashboard/release-post-promotion-monitor.json",
      "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json --history tmp/friday-dashboard/trusted-host-runner-history.json",
      "flow --friday-release-post-promotion-monitor --output tmp/friday-dashboard/release-post-promotion-monitor.json --promotion-ledger tmp/friday-dashboard/release-promotion-ledger.json --incident-note <incident-note.md>",
    ],
    summary:
      "Friday recovery runbook is 63/100 with 2 blocked phases and 3 approval gates still unsatisfied.",
    commands: [
      "flow --friday-release-recovery-runbook --output tmp/friday-dashboard/release-recovery-runbook.json",
      "flow --friday-release-recovery-runbook-json --output tmp/friday-dashboard/release-recovery-runbook.json",
    ],
  });
  const releaseIncidentArchive = normalizeReleaseIncidentArchive({
    archive_id: "friday-release-incident-archive-smoke",
    archive_json: "tmp/friday-dashboard/release-incident-archive.json",
    generated_at_unix_ms: 14,
    product_name: "Friday",
    local_only: true,
    incident_count: 1,
    open_count: 1,
    monitoring_count: 0,
    resolved_count: 0,
    rolled_back_count: 0,
    prevented_count: 0,
    critical_count: 1,
    blocking_count: 1,
    follow_up_count: 2,
    latest_incident_id: "friday-release-incident-smoke",
    latest_severity: "critical",
    latest_outcome: "open",
    latest_rollback_reference: "candidate-initial",
    entries: [
      {
        incident_id: "friday-release-incident-smoke",
        recorded_at_unix_ms: 14,
        product_name: "Friday",
        local_only: true,
        severity: "critical",
        outcome: "open",
        title: "Release recovery review for candidate-initial",
        summary:
          "Friday release incident is critical with outcome open; 2 blocked recovery phases, 1 active risk.",
        recovery_runbook_id: "friday-release-recovery-runbook-smoke",
        recovery_runbook_json: "tmp/friday-dashboard/release-recovery-runbook.json",
        stability_board_json: "tmp/friday-dashboard/release-stability-board.json",
        rollback_drill_json: "tmp/friday-dashboard/release-rollback-drill.json",
        post_promotion_monitor_json: "tmp/friday-dashboard/release-post-promotion-monitor.json",
        active_candidate_id: "candidate-promoted",
        active_promotion_id: "promotion-candidate-promoted",
        active_rollback_reference: "candidate-initial",
        blocked_phase_count: 2,
        active_risk_count: 1,
        incident_notes: [
          {
            id: "voice-overlay-followup",
            path: "tmp/friday-dashboard/incidents/voice-overlay-followup.md",
            present: true,
            bytes: 240,
            summary: "Incident note evidence is present.",
          },
        ],
        follow_up_actions: [
          "Rollback dry run: Resolve rollback blockers.",
          "Verify recovery state: Refresh post-promotion evidence.",
        ],
        prevention_items: [
          "Prevent recurrence of: Rollback recovery: Rollback drill has 1 blocking issue.",
        ],
        evidence_paths: [
          "tmp/friday-dashboard/release-recovery-runbook.json",
          "tmp/friday-dashboard/release-stability-board.json",
          "tmp/friday-dashboard/release-rollback-drill.json",
          "tmp/friday-dashboard/release-post-promotion-monitor.json",
          "tmp/friday-dashboard/incidents/voice-overlay-followup.md",
        ],
      },
    ],
    commands: [
      "flow --friday-release-incident-archive --archive tmp/friday-dashboard/release-incident-archive.json --runbook <release-recovery-runbook.json> --incident-note <incident-note.md>",
      "flow --friday-release-incident-archive-list --archive tmp/friday-dashboard/release-incident-archive.json",
      "flow --friday-release-incident-archive-export --archive tmp/friday-dashboard/release-incident-archive.json --output tmp/friday-dashboard/release-incident-archive.json",
      "flow --friday-release-incident-archive-json --archive tmp/friday-dashboard/release-incident-archive.json",
    ],
  });
  const releasePreventionPlan = normalizeReleasePreventionPlan({
    plan_id: "friday-release-prevention-plan-smoke",
    plan_json: "tmp/friday-dashboard/release-prevention-plan.json",
    generated_at_unix_ms: 15,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    score_out_of_100: 42,
    ready_for_next_checkpoint: false,
    incident_archive_json: "tmp/friday-dashboard/release-incident-archive.json",
    stability_board_json: "tmp/friday-dashboard/release-stability-board.json",
    incident_count: 1,
    finding_count: 4,
    recurring_issue_count: 1,
    action_count: 4,
    owner_ready_count: 2,
    blocker_count: 4,
    evidence_missing_count: 1,
    gate_blocking_count: 2,
    latest_incident_id: "friday-release-incident-smoke",
    active_rollback_reference: "candidate-initial",
    findings: [
      {
        id: "critical-incidents",
        kind: "critical-incident",
        severity: "critical",
        title: "Critical incidents are still in the archive",
        recurrence_count: 1,
        source_paths: ["tmp/friday-dashboard/release-recovery-runbook.json"],
        summary: "Critical incident history needs explicit prevention evidence.",
        next_action: "Assign prevention owners for every critical incident.",
        release_gate_blocking: true,
      },
      {
        id: "repeated-rollback",
        kind: "repeated-failure-class",
        severity: "blocking",
        title: "Repeated rollback release failure class",
        recurrence_count: 2,
        source_paths: ["tmp/friday-dashboard/release-rollback-drill.json"],
        summary: "The archive shows repeated rollback prevention signals.",
        next_action: "Create one owner-ready prevention action for rollback.",
        release_gate_blocking: true,
      },
    ],
    actions: [
      {
        id: "prevent-critical-incidents",
        kind: "resolve-recurrence",
        status: "owner-ready",
        owner: "release-operator",
        title: "Assign prevention owner",
        summary: "Critical incident history needs explicit prevention evidence.",
        source_path: "tmp/friday-dashboard/release-recovery-runbook.json",
        evidence_path: "tmp/friday-dashboard/release-prevention-plan.json",
        command:
          "flow --friday-release-prevention-plan --output tmp/friday-dashboard/release-prevention-plan.json --incident-archive tmp/friday-dashboard/release-incident-archive.json --stability-board tmp/friday-dashboard/release-stability-board.json",
        required: true,
        release_gate_blocking: false,
        next_action: "Assign prevention owners for every critical incident.",
      },
      {
        id: "prevent-rollback-recovery-gap",
        kind: "harden-rollback",
        status: "needs-evidence",
        owner: "release-operator",
        title: "Harden rollback drill evidence",
        summary: "Friday needs a clean rollback recovery path.",
        source_path: "tmp/friday-dashboard/release-rollback-drill.json",
        evidence_path: "tmp/friday-dashboard/release-rollback-drill.json",
        command:
          "flow --friday-release-rollback-drill-json --output tmp/friday-dashboard/release-rollback-drill.json --dry-run",
        required: true,
        release_gate_blocking: true,
        next_action: "Run a clean rollback drill and attach the result.",
      },
    ],
    evidence_links: [
      {
        id: "incident-archive",
        label: "Incident archive",
        path: "tmp/friday-dashboard/release-incident-archive.json",
        present: true,
      },
      {
        id: "stability-board",
        label: "Stability board",
        path: "tmp/friday-dashboard/release-stability-board.json",
        present: true,
      },
      {
        id: "missing-prevention-evidence",
        label: "Missing prevention evidence",
        path: "tmp/friday-dashboard/missing-prevention-evidence.json",
        present: false,
      },
    ],
    owner_ready_copy:
      "Friday release prevention plan\n- [owner-ready] Assign prevention owner -> Assign prevention owners for every critical incident.",
    summary:
      "Friday prevention plan is 42/100 with 4 blockers, 1 recurring issue class, and 2 release gate blockers.",
    commands: [
      "flow --friday-release-prevention-plan --output tmp/friday-dashboard/release-prevention-plan.json --incident-archive tmp/friday-dashboard/release-incident-archive.json --stability-board tmp/friday-dashboard/release-stability-board.json",
      "flow --friday-release-prevention-plan-json --output tmp/friday-dashboard/release-prevention-plan.json --incident-archive tmp/friday-dashboard/release-incident-archive.json --stability-board tmp/friday-dashboard/release-stability-board.json",
    ],
  });
  const releaseOwnerFollowUpBoard = normalizeReleaseOwnerFollowUpBoard({
    board_id: "friday-release-owner-followup-board-smoke",
    board_json: "tmp/friday-dashboard/release-owner-followup-board.json",
    generated_at_unix_ms: 16,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    score_out_of_100: 50,
    ready_for_next_checkpoint: false,
    prevention_plan_json: "tmp/friday-dashboard/release-prevention-plan.json",
    incident_archive_json: "tmp/friday-dashboard/release-incident-archive.json",
    stability_board_json: "tmp/friday-dashboard/release-stability-board.json",
    record_count: 2,
    owner_count: 1,
    ready_count: 1,
    waiting_count: 0,
    blocked_count: 0,
    overdue_count: 1,
    complete_count: 0,
    evidence_missing_count: 1,
    gate_blocking_count: 1,
    owner_groups: [
      {
        owner: "release-operator",
        record_count: 2,
        ready_count: 1,
        waiting_count: 0,
        blocked_count: 0,
        overdue_count: 1,
        complete_count: 0,
        evidence_missing_count: 1,
        records: ["followup-prevent-critical-incidents", "followup-prevent-rollback-recovery-gap"],
      },
    ],
    records: [
      {
        id: "followup-prevent-critical-incidents",
        action_id: "prevent-critical-incidents",
        owner: "release-operator",
        title: "Assign prevention owner",
        summary: "Critical incident history needs explicit prevention evidence.",
        completion_state: "ready",
        evidence_state: "present",
        source_path: "tmp/friday-dashboard/release-recovery-runbook.json",
        evidence_path: "tmp/friday-dashboard/release-prevention-plan.json",
        evidence_request:
          "Review the attached evidence at tmp/friday-dashboard/release-prevention-plan.json and confirm the owner follow-up is still current.",
        due_after_unix_ms: 16,
        due_before_unix_ms: 86400016,
        overdue: false,
        required: true,
        release_gate_blocking: false,
        command:
          "flow --friday-release-prevention-plan --output tmp/friday-dashboard/release-prevention-plan.json --incident-archive tmp/friday-dashboard/release-incident-archive.json --stability-board tmp/friday-dashboard/release-stability-board.json",
        assignment_copy:
          "@release-operator - Assign prevention owner\nState: ready\nEvidence: Review the attached evidence.",
        next_action: "Assign prevention owners for every critical incident.",
      },
      {
        id: "followup-prevent-rollback-recovery-gap",
        action_id: "prevent-rollback-recovery-gap",
        owner: "release-operator",
        title: "Harden rollback drill evidence",
        summary: "Friday needs a clean rollback recovery path.",
        completion_state: "overdue",
        evidence_state: "missing",
        source_path: "tmp/friday-dashboard/release-rollback-drill.json",
        evidence_path: "tmp/friday-dashboard/release-rollback-drill.json",
        evidence_request:
          "Attach evidence at tmp/friday-dashboard/release-rollback-drill.json before the next checkpoint.",
        due_after_unix_ms: 16,
        due_before_unix_ms: 16,
        overdue: true,
        required: true,
        release_gate_blocking: true,
        command:
          "flow --friday-release-rollback-drill-json --output tmp/friday-dashboard/release-rollback-drill.json --dry-run",
        assignment_copy:
          "@release-operator - Harden rollback drill evidence\nState: overdue\nCommand: flow --friday-release-rollback-drill-json --output tmp/friday-dashboard/release-rollback-drill.json --dry-run",
        next_action: "Run a clean rollback drill and attach the result.",
      },
    ],
    assignment_copy:
      "Friday release owner follow-up board\n- @release-operator [ready] Assign prevention owner -> Assign prevention owners for every critical incident.\n- @release-operator [overdue] Harden rollback drill evidence -> Run a clean rollback drill and attach the result.",
    summary:
      "Friday owner follow-up board is 50/100 with 1 owner, 2 assignments, 1 overdue item, and 1 release gate blocker.",
    commands: [
      "flow --friday-release-owner-followup-board --output tmp/friday-dashboard/release-owner-followup-board.json --prevention-plan tmp/friday-dashboard/release-prevention-plan.json",
      "flow --friday-release-owner-followup-board-json --output tmp/friday-dashboard/release-owner-followup-board.json --prevention-plan tmp/friday-dashboard/release-prevention-plan.json",
    ],
  });
  const releaseEvidenceSlaMonitor = normalizeReleaseEvidenceSlaMonitor({
    monitor_id: "friday-release-evidence-sla-monitor-smoke",
    monitor_json: "tmp/friday-dashboard/release-evidence-sla-monitor.json",
    generated_at_unix_ms: 17,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    score_out_of_100: 44,
    ready_for_next_checkpoint: false,
    owner_followup_board_json: "tmp/friday-dashboard/release-owner-followup-board.json",
    prevention_plan_json: "tmp/friday-dashboard/release-prevention-plan.json",
    stability_board_json: "tmp/friday-dashboard/release-stability-board.json",
    requirement_count: 3,
    owner_count: 1,
    fresh_count: 1,
    due_soon_count: 0,
    overdue_count: 1,
    missing_count: 1,
    blocked_count: 0,
    acknowledged_count: 0,
    escalation_count: 2,
    gate_blocking_count: 2,
    owner_groups: [
      {
        owner: "release-operator",
        requirement_count: 3,
        fresh_count: 1,
        due_soon_count: 0,
        overdue_count: 1,
        missing_count: 1,
        blocked_count: 0,
        acknowledged_count: 0,
        escalation_count: 2,
        release_gate_blocking_count: 2,
        requirements: [
          "sla-followup-prevent-rollback-recovery-gap",
          "sla-prevention-missing-prevention-evidence",
          "sla-stability-rollback-recovery",
        ],
      },
    ],
    requirements: [
      {
        id: "sla-followup-prevent-rollback-recovery-gap",
        source: "owner-follow-up",
        owner: "release-operator",
        title: "Harden rollback drill evidence",
        state: "overdue",
        escalation_level: "checkpoint",
        evidence_path: "tmp/friday-dashboard/release-rollback-drill.json",
        evidence_present: false,
        due_after_unix_ms: 16,
        due_before_unix_ms: 16,
        sla_window_ms: 0,
        age_ms: 1,
        acknowledgement_required: false,
        release_gate_blocking: true,
        escalation_copy:
          "@release-operator - Harden rollback drill evidence\nSLA: overdue\nEscalation: checkpoint",
        next_action: "Escalate this overdue owner evidence before the next checkpoint.",
      },
      {
        id: "sla-prevention-missing-prevention-evidence",
        source: "prevention-plan",
        owner: "release-operator",
        title: "Attach prevention evidence: Missing prevention evidence",
        state: "missing",
        escalation_level: "release-gate",
        evidence_path: "tmp/friday-dashboard/missing-prevention-evidence.json",
        evidence_present: false,
        due_after_unix_ms: 17,
        due_before_unix_ms: 17,
        sla_window_ms: 0,
        age_ms: 0,
        acknowledgement_required: false,
        release_gate_blocking: true,
        escalation_copy:
          "@release-operator - Attach prevention evidence\nSLA: missing\nEscalation: release-gate",
        next_action: "Attach the missing prevention evidence before checkpoint review.",
      },
      {
        id: "sla-stability-rollback-recovery",
        source: "stability-board",
        owner: "release-operator",
        title: "Rollback recovery",
        state: "fresh",
        escalation_level: "none",
        evidence_path: "tmp/friday-dashboard/release-rollback-drill.json",
        evidence_present: true,
        due_after_unix_ms: 17,
        due_before_unix_ms: 7200017,
        sla_window_ms: 7200000,
        age_ms: 0,
        acknowledgement_required: true,
        release_gate_blocking: false,
        escalation_copy:
          "@release-operator - Rollback recovery\nSLA: fresh\nEscalation: none",
        next_action: "Keep the rollback recovery evidence current.",
      },
    ],
    escalation_copy:
      "Friday release evidence SLA monitor\n- @release-operator [overdue / checkpoint] Harden rollback drill evidence -> Escalate this overdue owner evidence before the next checkpoint.\n- @release-operator [missing / release-gate] Attach prevention evidence: Missing prevention evidence -> Attach the missing prevention evidence before checkpoint review.",
    summary:
      "Friday release evidence SLA monitor is 44/100 with 3 requirements, 1 overdue, 1 missing, and 2 escalations.",
    commands: [
      "flow --friday-release-evidence-sla-monitor --output tmp/friday-dashboard/release-evidence-sla-monitor.json --owner-followup-board tmp/friday-dashboard/release-owner-followup-board.json --prevention-plan tmp/friday-dashboard/release-prevention-plan.json --stability-board tmp/friday-dashboard/release-stability-board.json",
      "flow --friday-release-evidence-sla-monitor-json --output tmp/friday-dashboard/release-evidence-sla-monitor.json --owner-followup-board tmp/friday-dashboard/release-owner-followup-board.json --prevention-plan tmp/friday-dashboard/release-prevention-plan.json --stability-board tmp/friday-dashboard/release-stability-board.json",
    ],
  });
  const releaseEscalationLedger = normalizeReleaseEscalationLedger({
    ledger_id: "friday-release-escalation-ledger-smoke",
    ledger_json: "tmp/friday-dashboard/release-escalation-ledger.json",
    generated_at_unix_ms: 18,
    product_name: "Friday",
    local_only: true,
    entry_count: 2,
    active_count: 2,
    acknowledged_count: 0,
    response_pending_count: 2,
    rejected_count: 0,
    resolved_count: 0,
    carryover_count: 2,
    release_gate_blocking_count: 2,
    acknowledgement_blocker_count: 2,
    owner_count: 1,
    latest_escalation_id: "friday-release-escalation-sla-prevention-missing-prevention-evidence-18",
    latest_gate_outcome: "carry-over",
    owner_groups: [
      {
        owner: "release-operator",
        entry_count: 2,
        active_count: 2,
        acknowledged_count: 0,
        acknowledgement_blocker_count: 2,
        carryover_count: 2,
        release_gate_blocking_count: 2,
        entries: [
          "friday-release-escalation-sla-followup-prevent-rollback-recovery-gap-18",
          "friday-release-escalation-sla-prevention-missing-prevention-evidence-18",
        ],
      },
    ],
    entries: [
      {
        escalation_id: "friday-release-escalation-sla-followup-prevent-rollback-recovery-gap-18",
        recorded_at_unix_ms: 18,
        product_name: "Friday",
        local_only: true,
        monitor_id: "friday-release-evidence-sla-monitor-smoke",
        monitor_json: "tmp/friday-dashboard/release-evidence-sla-monitor.json",
        requirement_id: "sla-followup-prevent-rollback-recovery-gap",
        source: "owner-follow-up",
        owner: "release-operator",
        title: "Harden rollback drill evidence",
        sla_state: "overdue",
        escalation_level: "checkpoint",
        owner_response: "pending",
        gate_outcome: "carry-over",
        acknowledgement_required: true,
        acknowledged: false,
        active_carryover: true,
        release_gate_blocking: true,
        evidence_path: "tmp/friday-dashboard/release-rollback-drill.json",
        escalation_copy:
          "@release-operator - Harden rollback drill evidence\nSLA: overdue\nEscalation: checkpoint",
        owner_response_copy:
          "@release-operator - Harden rollback drill evidence\nResponse: pending\nGate: carry-over",
        next_action: "Escalate this overdue owner evidence before the next checkpoint.",
      },
      {
        escalation_id: "friday-release-escalation-sla-prevention-missing-prevention-evidence-18",
        recorded_at_unix_ms: 18,
        product_name: "Friday",
        local_only: true,
        monitor_id: "friday-release-evidence-sla-monitor-smoke",
        monitor_json: "tmp/friday-dashboard/release-evidence-sla-monitor.json",
        requirement_id: "sla-prevention-missing-prevention-evidence",
        source: "prevention-plan",
        owner: "release-operator",
        title: "Attach prevention evidence: Missing prevention evidence",
        sla_state: "missing",
        escalation_level: "release-gate",
        owner_response: "pending",
        gate_outcome: "carry-over",
        acknowledgement_required: true,
        acknowledged: false,
        active_carryover: true,
        release_gate_blocking: true,
        evidence_path: "tmp/friday-dashboard/missing-prevention-evidence.json",
        escalation_copy:
          "@release-operator - Attach prevention evidence\nSLA: missing\nEscalation: release-gate",
        owner_response_copy:
          "@release-operator - Attach prevention evidence: Missing prevention evidence\nResponse: pending\nGate: carry-over",
        next_action: "Attach the missing prevention evidence before checkpoint review.",
      },
    ],
    owner_response_copy:
      "Friday release escalation ledger\n- @release-operator [pending / carry-over] Harden rollback drill evidence -> Escalate this overdue owner evidence before the next checkpoint.\n- @release-operator [pending / carry-over] Attach prevention evidence: Missing prevention evidence -> Attach the missing prevention evidence before checkpoint review.",
    summary:
      "Friday release escalation ledger has 2 records, 2 active carryovers, 2 acknowledgement blockers, and 2 release-gate blockers.",
    commands: [
      "flow --friday-release-escalation-ledger --ledger tmp/friday-dashboard/release-escalation-ledger.json --monitor tmp/friday-dashboard/release-evidence-sla-monitor.json --response pending --outcome carry-over",
      "flow --friday-release-escalation-ledger-list --ledger tmp/friday-dashboard/release-escalation-ledger.json",
    ],
  });
  const releaseCheckpointReview = normalizeReleaseCheckpointReview({
    review_id: "friday-release-checkpoint-review-smoke",
    review_json: "tmp/friday-dashboard/release-checkpoint-review.json",
    generated_at_unix_ms: 19,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    score_out_of_100: 0,
    decision: "hold",
    ready_for_checkpoint: false,
    item_count: 3,
    owner_count: 1,
    hold_count: 3,
    carryover_count: 1,
    review_required_count: 0,
    acknowledgement_required_count: 3,
    acknowledgement_blocker_count: 3,
    active_escalation_count: 2,
    release_gate_blocking_count: 3,
    escalation_ledger_json: "tmp/friday-dashboard/release-escalation-ledger.json",
    sla_monitor_json: "tmp/friday-dashboard/release-evidence-sla-monitor.json",
    owner_followup_board_json: "tmp/friday-dashboard/release-owner-followup-board.json",
    prevention_plan_json: "tmp/friday-dashboard/release-prevention-plan.json",
    stability_board_json: "tmp/friday-dashboard/release-stability-board.json",
    owner_groups: [
      {
        owner: "release-operator",
        item_count: 3,
        hold_count: 3,
        carryover_count: 1,
        review_required_count: 0,
        acknowledgement_blocker_count: 3,
        release_gate_blocking_count: 3,
        items: [
          "checkpoint-ledger-friday-release-escalation-sla-followup-prevent-rollback-recovery-gap-18",
          "checkpoint-sla-sla-followup-prevent-rollback-recovery-gap",
          "checkpoint-owner-owner-followup-prevent-rollback-recovery-gap",
        ],
      },
    ],
    items: [
      {
        id: "checkpoint-ledger-friday-release-escalation-sla-followup-prevent-rollback-recovery-gap-18",
        source: "escalation-ledger",
        owner: "release-operator",
        title: "Harden rollback drill evidence",
        state: "hold",
        decision: "hold",
        acknowledgement_required: true,
        acknowledged: false,
        active_carryover: true,
        release_gate_blocking: true,
        evidence_path: "tmp/friday-dashboard/release-rollback-drill.json",
        summary: "Owner response is pending and gate outcome is carry-over.",
        next_action: "Collect @release-operator acknowledgement or hold the checkpoint.",
      },
      {
        id: "checkpoint-sla-sla-followup-prevent-rollback-recovery-gap",
        source: "sla-monitor",
        owner: "release-operator",
        title: "Harden rollback drill evidence",
        state: "hold",
        decision: "hold",
        acknowledgement_required: true,
        acknowledged: false,
        active_carryover: false,
        release_gate_blocking: true,
        evidence_path: "tmp/friday-dashboard/release-rollback-drill.json",
        summary: "SLA state is overdue with escalation level checkpoint.",
        next_action: "Escalate this overdue owner evidence before the next checkpoint.",
      },
      {
        id: "checkpoint-owner-owner-followup-prevent-rollback-recovery-gap",
        source: "owner-follow-up",
        owner: "release-operator",
        title: "Harden rollback drill evidence",
        state: "hold",
        decision: "hold",
        acknowledgement_required: true,
        acknowledged: false,
        active_carryover: true,
        release_gate_blocking: true,
        evidence_path: "tmp/friday-dashboard/release-rollback-drill.json",
        summary: "Owner follow-up is overdue with evidence request.",
        next_action: "Run a clean rollback drill and attach the result.",
      },
    ],
    review_notes_copy:
      "Friday release checkpoint review: hold\n- @release-operator [escalation-ledger / hold] Harden rollback drill evidence -> Collect @release-operator acknowledgement or hold the checkpoint.",
    summary:
      "Friday checkpoint review is hold at 0/100 with 3 review items, 3 holds, 1 carryover, and 3 acknowledgement blockers.",
    commands: [
      "flow --friday-release-checkpoint-review --output tmp/friday-dashboard/release-checkpoint-review.json --ledger tmp/friday-dashboard/release-escalation-ledger.json --monitor tmp/friday-dashboard/release-evidence-sla-monitor.json --owner-followup-board tmp/friday-dashboard/release-owner-followup-board.json --prevention-plan tmp/friday-dashboard/release-prevention-plan.json --stability-board tmp/friday-dashboard/release-stability-board.json",
      "flow --friday-release-checkpoint-review-json --output tmp/friday-dashboard/release-checkpoint-review.json --ledger tmp/friday-dashboard/release-escalation-ledger.json --monitor tmp/friday-dashboard/release-evidence-sla-monitor.json --owner-followup-board tmp/friday-dashboard/release-owner-followup-board.json --prevention-plan tmp/friday-dashboard/release-prevention-plan.json --stability-board tmp/friday-dashboard/release-stability-board.json",
    ],
  });
  const releaseCheckpointSignoffLedger = normalizeReleaseCheckpointSignoffLedger({
    ledger_id: "friday-release-checkpoint-signoff-ledger-smoke",
    ledger_json: "tmp/friday-dashboard/release-checkpoint-signoff-ledger.json",
    generated_at_unix_ms: 20,
    product_name: "Friday",
    local_only: true,
    record_count: 1,
    signed_off_count: 0,
    held_count: 1,
    carried_over_count: 0,
    superseded_count: 0,
    revoked_count: 0,
    active_signoff_id: "friday-release-checkpoint-signoff-friday-release-checkpoint-review-smoke-20",
    active_review_id: "friday-release-checkpoint-review-smoke",
    active_decision: "held",
    active_hold_count: 1,
    active_carryover_count: 1,
    acknowledgement_evidence_missing_count: 0,
    release_gate_blocking_count: 1,
    records: [
      {
        signoff_id: "friday-release-checkpoint-signoff-friday-release-checkpoint-review-smoke-20",
        review_id: "friday-release-checkpoint-review-smoke",
        review_json: "tmp/friday-dashboard/release-checkpoint-review.json",
        recorded_at_unix_ms: 20,
        product_name: "Friday",
        local_only: true,
        decision: "held",
        operator: "release-operator",
        reason: "Hold checkpoint until acknowledgement blockers are cleared.",
        acknowledgement_evidence_path: "",
        acknowledgement_evidence_present: false,
        acknowledgement_evidence_bytes: 0,
        carryover_commitment:
          "Carry rollback evidence and prevention acknowledgement into the next release loop.",
        review_decision: "hold",
        review_score_out_of_100: 0,
        review_ready_for_checkpoint: false,
        review_hold_count: 3,
        review_carryover_count: 1,
        review_acknowledgement_blocker_count: 3,
        release_gate_blocking_count: 3,
        active_hold: true,
        active_carryover: true,
        release_notes:
          "Friday checkpoint signoff: held\nOperator: release-operator\nReason: Hold checkpoint until acknowledgement blockers are cleared.",
        summary:
          "release-operator held checkpoint friday-release-checkpoint-review-smoke with 3 hold(s), 1 carryover(s), and 3 acknowledgement blocker(s).",
      },
    ],
    release_notes_copy:
      "Friday checkpoint signoff ledger\n- release-operator [held] friday-release-checkpoint-review-smoke -> Hold checkpoint until acknowledgement blockers are cleared.",
    summary:
      "Friday checkpoint signoff ledger has 1 record, 0 signed off, 1 held, 0 carried over, and 0 missing acknowledgement evidence item(s).",
    commands: [
      "flow --friday-release-checkpoint-signoff --ledger tmp/friday-dashboard/release-checkpoint-signoff-ledger.json --review tmp/friday-dashboard/release-checkpoint-review.json --decision held --operator release-operator --reason \"Hold checkpoint until acknowledgement blockers are cleared.\"",
      "flow --friday-release-checkpoint-signoff-list --ledger tmp/friday-dashboard/release-checkpoint-signoff-ledger.json",
    ],
  });
  const releaseCheckpointEvidenceVault = normalizeReleaseCheckpointEvidenceVault({
    vault_id: "friday-release-checkpoint-evidence-vault-smoke",
    vault_json: "tmp/friday-dashboard/release-checkpoint-evidence-vault.json",
    generated_at_unix_ms: 21,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    ready_to_archive: false,
    review_id: "friday-release-checkpoint-review-smoke",
    review_decision: "hold",
    review_score_out_of_100: 0,
    signoff_ledger_id: "friday-release-checkpoint-signoff-ledger-smoke",
    active_signoff_id: "friday-release-checkpoint-signoff-friday-release-checkpoint-review-smoke-20",
    active_decision: "held",
    entry_count: 4,
    required_count: 4,
    present_count: 3,
    missing_count: 1,
    checksum_count: 3,
    acknowledgement_evidence_missing_count: 1,
    active_hold_count: 1,
    active_carryover_count: 1,
    release_gate_blocking_count: 1,
    manifest_sha256: "smoke-vault-checksum",
    entries: [
      {
        id: "checkpoint-review-json",
        label: "Checkpoint review JSON",
        kind: "checkpoint-review-json",
        path: "tmp/friday-dashboard/release-checkpoint-review.json",
        required: true,
        present: true,
        bytes: 300,
        sha256: "review-checksum",
        source_id: "friday-release-checkpoint-review-smoke",
        summary: "Release checkpoint review board used for this vault.",
        warning: null,
      },
      {
        id: "checkpoint-signoff-ledger-json",
        label: "Checkpoint signoff ledger JSON",
        kind: "checkpoint-signoff-ledger-json",
        path: "tmp/friday-dashboard/release-checkpoint-signoff-ledger.json",
        required: true,
        present: true,
        bytes: 220,
        sha256: "signoff-checksum",
        source_id: "friday-release-checkpoint-signoff-ledger-smoke",
        summary: "Checkpoint signoff history used for this vault.",
        warning: null,
      },
      {
        id: "acknowledgement-evidence-smoke",
        label: "Acknowledgement evidence",
        kind: "acknowledgement-evidence",
        path: "",
        required: true,
        present: false,
        bytes: 0,
        sha256: null,
        source_id: "friday-release-checkpoint-signoff-friday-release-checkpoint-review-smoke-20",
        summary: "Operator acknowledgement evidence attached to a checkpoint signoff.",
        warning: "Required evidence is missing: Acknowledgement evidence.",
      },
      {
        id: "checkpoint-release-notes",
        label: "Checkpoint release notes",
        kind: "release-notes",
        path: "inline://release-notes/checkpoint-signoff-ledger",
        required: true,
        present: true,
        bytes: 160,
        sha256: "release-notes-checksum",
        source_id: "friday-release-checkpoint-signoff-ledger-smoke",
        summary: "Copyable release notes generated from checkpoint signoff history.",
        warning: null,
      },
    ],
    attachment_notes_copy:
      "Friday checkpoint evidence vault: tmp/friday-dashboard/release-checkpoint-evidence-vault.json\nManifest checksum: smoke-vault-checksum\nMissing required evidence: 1",
    summary:
      "Friday checkpoint evidence vault has 4 entries, 1 missing required item(s), 3 checksum(s), and 1 release gate block(s).",
    commands: [
      "flow --friday-release-checkpoint-evidence-vault --output tmp/friday-dashboard/release-checkpoint-evidence-vault.json --review tmp/friday-dashboard/release-checkpoint-review.json --signoff-ledger tmp/friday-dashboard/release-checkpoint-signoff-ledger.json",
      "flow --friday-release-checkpoint-evidence-vault-json --output tmp/friday-dashboard/release-checkpoint-evidence-vault.json --review tmp/friday-dashboard/release-checkpoint-review.json --signoff-ledger tmp/friday-dashboard/release-checkpoint-signoff-ledger.json",
    ],
  });
  const releaseEvidenceAttachmentReview = normalizeReleaseEvidenceAttachmentReview({
    review_id: "friday-release-evidence-attachment-review-smoke",
    review_json: "tmp/friday-dashboard/release-evidence-attachment-review.json",
    generated_at_unix_ms: 22,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    ready_for_handoff: false,
    vault_id: "friday-release-checkpoint-evidence-vault-smoke",
    vault_json: "tmp/friday-dashboard/release-checkpoint-evidence-vault.json",
    manifest_sha256: "smoke-vault-checksum",
    item_count: 4,
    attachable_count: 2,
    missing_count: 1,
    inline_only_count: 1,
    checksum_missing_count: 0,
    blocked_count: 1,
    release_gate_blocking_count: 1,
    first_blocker: "Resolve blocking evidence for Acknowledgement evidence before release handoff.",
    items: [
      {
        id: "attachment-review-acknowledgement-evidence-smoke",
        vault_entry_id: "acknowledgement-evidence-smoke",
        label: "Acknowledgement evidence",
        kind: "acknowledgement-evidence",
        path: "",
        state: "blocked",
        required: true,
        present: false,
        attachable: false,
        bytes: 0,
        sha256: null,
        source_id: "friday-release-checkpoint-signoff-friday-release-checkpoint-review-smoke-20",
        release_gate_blocking: true,
        summary: "Required evidence is missing: Acknowledgement evidence.",
        next_action: "Resolve blocking evidence for Acknowledgement evidence before release handoff.",
      },
      {
        id: "attachment-review-checkpoint-release-notes",
        vault_entry_id: "checkpoint-release-notes",
        label: "Checkpoint release notes",
        kind: "release-notes",
        path: "inline://release-notes/checkpoint-signoff-ledger",
        state: "inline-only",
        required: true,
        present: true,
        attachable: false,
        bytes: 160,
        sha256: "release-notes-checksum",
        source_id: "friday-release-checkpoint-signoff-ledger-smoke",
        release_gate_blocking: false,
        summary: "Copyable release notes generated from checkpoint signoff history.",
        next_action: "Review inline note Checkpoint release notes and paste it into the handoff.",
      },
      {
        id: "attachment-review-checkpoint-review-json",
        vault_entry_id: "checkpoint-review-json",
        label: "Checkpoint review JSON",
        kind: "checkpoint-review-json",
        path: "tmp/friday-dashboard/release-checkpoint-review.json",
        state: "ready",
        required: true,
        present: true,
        attachable: true,
        bytes: 300,
        sha256: "review-checksum",
        source_id: "friday-release-checkpoint-review-smoke",
        release_gate_blocking: false,
        summary: "Release checkpoint review board used for this vault.",
        next_action: "Attach tmp/friday-dashboard/release-checkpoint-review.json to the release handoff.",
      },
      {
        id: "attachment-review-checkpoint-signoff-ledger-json",
        vault_entry_id: "checkpoint-signoff-ledger-json",
        label: "Checkpoint signoff ledger JSON",
        kind: "checkpoint-signoff-ledger-json",
        path: "tmp/friday-dashboard/release-checkpoint-signoff-ledger.json",
        state: "ready",
        required: true,
        present: true,
        attachable: true,
        bytes: 220,
        sha256: "signoff-checksum",
        source_id: "friday-release-checkpoint-signoff-ledger-smoke",
        release_gate_blocking: false,
        summary: "Checkpoint signoff history used for this vault.",
        next_action: "Attach tmp/friday-dashboard/release-checkpoint-signoff-ledger.json to the release handoff.",
      },
    ],
    handoff_notes_copy:
      "Friday release evidence attachment review: tmp/friday-dashboard/release-evidence-attachment-review.json\nStatus: needs attachment review\nManifest checksum: smoke-vault-checksum",
    summary:
      "Friday release evidence attachment review has 4 item(s), 2 attachable, 1 missing, 1 inline-only, 0 checksum-missing, and 1 blocked.",
    commands: [
      "flow --friday-release-evidence-attachment-review --output tmp/friday-dashboard/release-evidence-attachment-review.json --vault tmp/friday-dashboard/release-checkpoint-evidence-vault.json",
      "flow --friday-release-evidence-attachment-review-json --output tmp/friday-dashboard/release-evidence-attachment-review.json --vault tmp/friday-dashboard/release-checkpoint-evidence-vault.json",
    ],
  });
  const releaseHandoffPacket = normalizeReleaseHandoffPacket({
    packet_id: "friday-release-handoff-packet-smoke",
    packet_json: "tmp/friday-dashboard/release-handoff-packet.json",
    generated_at_unix_ms: 23,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    ready_to_send: false,
    attachment_review_id: "friday-release-evidence-attachment-review-smoke",
    attachment_review_json: "tmp/friday-dashboard/release-evidence-attachment-review.json",
    manifest_sha256: "smoke-vault-checksum",
    section_count: 5,
    included_count: 4,
    attachable_file_count: 2,
    inline_note_count: 1,
    unresolved_blocker_count: 1,
    checksum_count: 1,
    missing_count: 1,
    first_blocker: "Resolve blocking evidence for Acknowledgement evidence before release handoff.",
    sections: [
      {
        id: "operator-summary",
        kind: "operator-summary",
        title: "Operator summary",
        body: "Friday release evidence attachment review has 4 item(s), 2 attachable, 1 missing, 1 inline-only, 0 checksum-missing, and 1 blocked.",
        path: "tmp/friday-dashboard/release-evidence-attachment-review.json",
        source_id: "friday-release-evidence-attachment-review-smoke",
        required: true,
        included: true,
        checksum: null,
        next_action: "Resolve unresolved blockers before sending this handoff packet.",
      },
      {
        id: "manifest-checksum",
        kind: "manifest-checksum",
        title: "Manifest checksum",
        body: "smoke-vault-checksum",
        path: "tmp/friday-dashboard/release-checkpoint-evidence-vault.json",
        source_id: "friday-release-checkpoint-evidence-vault-smoke",
        required: true,
        included: true,
        checksum: "smoke-vault-checksum",
        next_action: "Include this checksum in the operator handoff note.",
      },
      {
        id: "unresolved-blocker-attachment-review-acknowledgement-evidence-smoke",
        kind: "unresolved-blocker",
        title: "Acknowledgement evidence",
        body: "Required evidence is missing: Acknowledgement evidence.",
        path: "",
        source_id: "friday-release-checkpoint-signoff-friday-release-checkpoint-review-smoke-20",
        required: true,
        included: false,
        checksum: null,
        next_action: "Resolve blocking evidence for Acknowledgement evidence before release handoff.",
      },
      {
        id: "attachable-file-attachment-review-checkpoint-review-json",
        kind: "attachable-file",
        title: "Checkpoint review JSON",
        body: "Release checkpoint review board used for this vault.",
        path: "tmp/friday-dashboard/release-checkpoint-review.json",
        source_id: "friday-release-checkpoint-review-smoke",
        required: true,
        included: true,
        checksum: "review-checksum",
        next_action: "Attach tmp/friday-dashboard/release-checkpoint-review.json to the release handoff.",
      },
      {
        id: "inline-note-attachment-review-checkpoint-release-notes",
        kind: "inline-note",
        title: "Checkpoint release notes",
        body: "Copyable release notes generated from checkpoint signoff history.",
        path: "inline://release-notes/checkpoint-signoff-ledger",
        source_id: "friday-release-checkpoint-signoff-ledger-smoke",
        required: true,
        included: true,
        checksum: "release-notes-checksum",
        next_action: "Review inline note Checkpoint release notes and paste it into the handoff.",
      },
    ],
    file_checklist_copy:
      "Friday release handoff file checklist\n- [ ] Checkpoint review JSON -> tmp/friday-dashboard/release-checkpoint-review.json (review-checksum)",
    handoff_packet_copy:
      "Friday release handoff packet: tmp/friday-dashboard/release-handoff-packet.json\nStatus: blocked before send\nManifest checksum: smoke-vault-checksum\nUnresolved blockers:\n- Acknowledgement evidence -> Resolve blocking evidence for Acknowledgement evidence before release handoff.",
    summary:
      "Friday release handoff packet has 5 section(s), 2 attachable file(s), 1 inline note(s), 1 unresolved blocker(s), and 1 checksum section(s).",
    commands: [
      "flow --friday-release-handoff-packet --output tmp/friday-dashboard/release-handoff-packet.json --attachment-review tmp/friday-dashboard/release-evidence-attachment-review.json",
      "flow --friday-release-handoff-packet-json --output tmp/friday-dashboard/release-handoff-packet.json --attachment-review tmp/friday-dashboard/release-evidence-attachment-review.json",
    ],
  });
  const releaseHandoffAuditTrail = normalizeReleaseHandoffAuditTrail({
    trail_id: "friday-release-handoff-audit-trail-smoke",
    trail_json: "tmp/friday-dashboard/release-handoff-audit-trail.json",
    generated_at_unix_ms: 24,
    product_name: "Friday",
    local_only: true,
    record_count: 1,
    draft_count: 0,
    ready_count: 0,
    sent_count: 0,
    superseded_count: 0,
    revoked_count: 0,
    blocked_count: 1,
    active_audit_id: "friday-release-handoff-audit-friday-release-handoff-packet-smoke-24",
    active_packet_id: "friday-release-handoff-packet-smoke",
    latest_audit_id: "friday-release-handoff-audit-friday-release-handoff-packet-smoke-24",
    latest_packet_id: "friday-release-handoff-packet-smoke",
    latest_state: "blocked",
    latest_ready_to_send: false,
    unresolved_blocker_count: 2,
    blocker_carryover_count: 1,
    acknowledgement_count: 1,
    records: [
      {
        audit_id: "friday-release-handoff-audit-friday-release-handoff-packet-smoke-24",
        packet_id: "friday-release-handoff-packet-smoke",
        packet_json: "tmp/friday-dashboard/release-handoff-packet.json",
        recorded_at_unix_ms: 24,
        product_name: "Friday",
        local_only: true,
        state: "blocked",
        operator: "operator",
        acknowledgement_note: "Blocked packet stays local until acknowledgement evidence is attached.",
        supersedes_packet_id: "previous-packet",
        packet_ready_to_send: false,
        packet_status: "blocked",
        packet_section_count: 5,
        attachable_file_count: 2,
        inline_note_count: 1,
        unresolved_blocker_count: 1,
        missing_count: 1,
        manifest_sha256: "smoke-vault-checksum",
        active: true,
        blocker_carryover: 2,
        audit_notes:
          "Friday handoff audit: blocked\nOperator: operator\nPacket: friday-release-handoff-packet-smoke\nAcknowledgement: Blocked packet stays local until acknowledgement evidence is attached.\nBlocker carryover: 2",
        summary:
          "operator recorded packet friday-release-handoff-packet-smoke as blocked with 1 unresolved blocker(s) and 1 missing item(s).",
      },
    ],
    audit_summary_copy:
      "Friday release handoff audit trail\n- operator [blocked] friday-release-handoff-packet-smoke -> Blocked packet stays local until acknowledgement evidence is attached.\n  carryover blockers: 2",
    summary:
      "Friday release handoff audit trail has 1 record(s), 0 ready, 0 sent, 1 blocked, and 1 blocker carryover record(s).",
    commands: [
      "flow --friday-release-handoff-audit --trail tmp/friday-dashboard/release-handoff-audit-trail.json --packet <release-handoff-packet.json> --state draft --operator <name>",
      "flow --friday-release-handoff-audit-list --trail tmp/friday-dashboard/release-handoff-audit-trail.json",
      "flow --friday-release-handoff-audit-export --trail tmp/friday-dashboard/release-handoff-audit-trail.json --output tmp/friday-dashboard/release-handoff-audit-trail.json",
      "flow --friday-release-handoff-audit-json --trail tmp/friday-dashboard/release-handoff-audit-trail.json --packet <release-handoff-packet.json>",
    ],
  });
  const releaseHandoffGovernanceReview = normalizeReleaseHandoffGovernanceReview({
    review_id: "friday-release-handoff-governance-review-smoke",
    review_json: "tmp/friday-dashboard/release-handoff-governance-review.json",
    generated_at_unix_ms: 25,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    score_out_of_100: 0,
    state: "blocked-carryover",
    approved_for_external_handoff: false,
    trail_id: "friday-release-handoff-audit-trail-smoke",
    trail_json: "tmp/friday-dashboard/release-handoff-audit-trail.json",
    latest_audit_id: "friday-release-handoff-audit-friday-release-handoff-packet-smoke-24",
    latest_packet_id: "friday-release-handoff-packet-smoke",
    active_audit_id: "friday-release-handoff-audit-friday-release-handoff-packet-smoke-24",
    active_packet_id: "friday-release-handoff-packet-smoke",
    latest_state: "blocked",
    record_count: 1,
    finding_count: 2,
    acknowledgement_gap_count: 0,
    stale_active_packet_count: 0,
    blocked_carryover_count: 1,
    held_count: 1,
    release_gate_blocking_count: 2,
    unresolved_blocker_count: 2,
    findings: [
      {
        id: "active-blocker-carryover",
        source: "blocker-carryover",
        state: "blocked-carryover",
        release_gate_blocking: true,
        audit_id: "friday-release-handoff-audit-friday-release-handoff-packet-smoke-24",
        packet_id: "friday-release-handoff-packet-smoke",
        title: "Active packet has blocker carryover",
        evidence_path: "tmp/friday-dashboard/release-handoff-packet.json",
        summary: "Active packet still carries 2 unresolved blocker(s).",
        next_action: "Resolve or explicitly carry the blockers before external handoff.",
      },
      {
        id: "latest-packet-held",
        source: "latest-packet",
        state: "held",
        release_gate_blocking: true,
        audit_id: "friday-release-handoff-audit-friday-release-handoff-packet-smoke-24",
        packet_id: "friday-release-handoff-packet-smoke",
        title: "Latest packet is not ready or sent",
        evidence_path: "tmp/friday-dashboard/release-handoff-packet.json",
        summary: "Latest handoff packet is blocked.",
        next_action:
          "Move the latest packet to ready or sent only after blockers and acknowledgements are resolved.",
      },
    ],
    governance_notes_copy:
      "Friday release handoff governance review\nStatus: hold external handoff\n- [blocked-carryover] Active packet has blocker carryover -> Resolve or explicitly carry the blockers before external handoff.",
    summary:
      "Friday release handoff governance review is blocked-carryover with score 0/100, 2 finding(s), 0 acknowledgement gap(s), 0 stale packet warning(s), and 1 blocker carryover issue(s).",
    commands: [
      "flow --friday-release-handoff-governance-review --output tmp/friday-dashboard/release-handoff-governance-review.json --trail tmp/friday-dashboard/release-handoff-audit-trail.json",
      "flow --friday-release-handoff-governance-review-json --output tmp/friday-dashboard/release-handoff-governance-review.json --trail tmp/friday-dashboard/release-handoff-audit-trail.json",
    ],
  });
  const releaseHandoffDispatchChecklist = normalizeReleaseHandoffDispatchChecklist({
    checklist_id: "friday-release-handoff-dispatch-checklist-smoke",
    checklist_json: "tmp/friday-dashboard/release-handoff-dispatch-checklist.json",
    generated_at_unix_ms: 26,
    product_name: "Friday",
    local_only: true,
    status: "blocked",
    state: "blocked",
    ready_to_dispatch: false,
    governance_review_id: "friday-release-handoff-governance-review-smoke",
    governance_review_json: "tmp/friday-dashboard/release-handoff-governance-review.json",
    governance_state: "blocked-carryover",
    approved_for_external_handoff: false,
    latest_packet_id: "friday-release-handoff-packet-smoke",
    active_packet_id: "friday-release-handoff-packet-smoke",
    recipient_count: 1,
    attachment_count: 1,
    dispatch_note_count: 1,
    privacy_boundary_count: 0,
    no_send_safeguard_count: 1,
    item_count: 6,
    ready_count: 3,
    missing_recipient_count: 0,
    missing_attachment_count: 1,
    privacy_review_count: 1,
    held_count: 0,
    blocked_count: 1,
    release_gate_blocking_count: 3,
    items: [
      {
        id: "governance-review",
        source: "governance-review",
        state: "blocked",
        required: true,
        ready: false,
        release_gate_blocking: true,
        title: "Governance review approval",
        detail: "Governance review is blocked-carryover.",
        evidence_path: "tmp/friday-dashboard/release-handoff-governance-review.json",
        next_action: "Resolve governance findings before preparing external dispatch.",
      },
      {
        id: "recipient-1",
        source: "recipient",
        state: "ready",
        required: true,
        ready: true,
        release_gate_blocking: false,
        title: "Dispatch recipient",
        detail: "release-operator@example.com",
        evidence_path: "",
        next_action: "Confirm this recipient is allowed to receive the handoff.",
      },
      {
        id: "attachment-1",
        source: "attachment",
        state: "missing-attachment",
        required: true,
        ready: false,
        release_gate_blocking: true,
        title: "Dispatch attachment",
        detail: "Attachment path is missing or unreadable.",
        evidence_path: "tmp/friday-dashboard/missing-handoff-packet.txt",
        next_action: "Create or correct this attachment path before dispatch.",
      },
      {
        id: "dispatch-note",
        source: "dispatch-note",
        state: "ready",
        required: true,
        ready: true,
        release_gate_blocking: false,
        title: "Dispatch note",
        detail: "No external send is performed by this checklist.",
        evidence_path: "",
        next_action: "Keep the dispatch note in the operator handoff record.",
      },
      {
        id: "privacy-boundary",
        source: "privacy-boundary",
        state: "privacy-review",
        required: true,
        ready: false,
        release_gate_blocking: true,
        title: "Privacy boundary",
        detail: "No privacy boundary note was supplied.",
        evidence_path: "",
        next_action:
          "Record what may and may not leave the local machine before external handoff.",
      },
      {
        id: "no-send-safeguard",
        source: "no-send-safeguard",
        state: "ready",
        required: true,
        ready: true,
        release_gate_blocking: false,
        title: "No-send safeguard",
        detail:
          "This checklist command only writes local JSON and never sends, uploads, deploys, or mutates external systems.",
        evidence_path: "",
        next_action:
          "Use a separate explicit operator action if external sending is ever approved.",
      },
    ],
    dispatch_checklist_copy:
      "Friday release handoff dispatch checklist\nStatus: hold dispatch\nRecipients:\n- release-operator@example.com\nAttachments:\n- tmp/friday-dashboard/missing-handoff-packet.txt",
    summary:
      "Friday release handoff dispatch checklist is blocked with 6 item(s), 1 recipient(s), 1 attachment(s), 0 privacy note(s), and 3 blocking issue(s).",
    commands: [
      "flow --friday-release-handoff-dispatch-checklist --output tmp/friday-dashboard/release-handoff-dispatch-checklist.json --governance-review tmp/friday-dashboard/release-handoff-governance-review.json --recipient <recipient> --attachment <file>",
      "flow --friday-release-handoff-dispatch-checklist-json --output tmp/friday-dashboard/release-handoff-dispatch-checklist.json --governance-review tmp/friday-dashboard/release-handoff-governance-review.json --recipient <recipient> --attachment <file>",
    ],
  });
  const trustedBridgeLiveRunnerState = normalizeTrustedHostLiveRunnerState({
    dashboard_import_guidance:
      "Import live-state JSON for current work; import runner history JSON only for audit history.",
    live_state: {
      state_json: "tmp/friday-dashboard/trusted-host-live-state.json",
      generated_at_unix_ms: 2,
      record_count: 1,
      pending_count: 0,
      running_count: 0,
      finished_count: 1,
      stale_count: 0,
      stale_recovery_copy: "No stale records.",
      records: [
        {
          job_id: "runner-finished",
          action_id: "host-open",
          label: "Open host report",
          command: "flow --completion",
          status: "succeeded",
          message: "Completed.",
          local_only: true,
          approved: true,
          timeout_ms: 30000,
          stale_after_ms: 120000,
          created_at_unix_ms: 2,
          updated_at_unix_ms: 2,
          finished_at_unix_ms: 2,
          history_json: "tmp/friday-dashboard/trusted-host-runner-history.json",
          recovery_command:
            "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id host-open --cancel",
          cleanup_command:
            "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json",
        },
      ],
    },
  });
  const checks = [
    check(
      "local-fallback-labelled",
      binding.localOnly && binding.fallback && binding.sourceKind === "embedded-snapshot",
      `${binding.sourceKind}: ${binding.sourceLabel}`,
    ),
    check(
      "local-json-importable",
      imported.localOnly &&
        !imported.fallback &&
        imported.sourceKind === "imported-json" &&
        imported.cards[0]?.actions.length === 1,
      `${imported.sourceKind}: ${imported.sourceLabel}`,
    ),
    check(
      "cards-renderable",
      binding.cards.length === binding.cardCount && binding.boundCardCount === binding.cardCount,
      `${binding.boundCardCount}/${binding.cardCount} cards bound`,
    ),
    check(
      "action-buttons-renderable",
      actionCount === binding.actionCount &&
        binding.cards.every((card) =>
          card.actions.every(
            (action) =>
              action.localOnly &&
              action.command.trim().length > 0 &&
              action.buttonState.idleLabel.trim().length > 0 &&
              action.buttonState.loadingLabel.trim().length > 0 &&
              action.buttonState.successLabel.trim().length > 0 &&
              action.buttonState.errorLabel.trim().length > 0 &&
              action.buttonState.ariaLabel.trim().length > 0,
          ),
        ),
      `${actionCount}/${binding.actionCount} actions renderable`,
    ),
    check(
      "command-dispatch-success",
      preparedResult?.status === "prepared" && preparedResult.permission === "allowed",
      preparedResult?.message ?? "missing dispatch result",
    ),
    check(
      "command-dispatch-confirmation",
      confirmationResult?.status === "blocked" &&
        confirmationResult.permission === "confirmation-required",
      confirmationResult?.message ?? "missing confirmation result",
    ),
    check(
      "command-dispatch-blocked",
      blockedResult?.status === "blocked" && blockedResult.permission === "blocked",
      blockedResult?.message ?? "missing blocked result",
    ),
    check(
      "command-dispatch-failure",
      failedResult?.status === "failed" && failedResult.permission === "blocked",
      failedResult?.message ?? "missing failed result",
    ),
    check(
      "host-bridge-importable",
      hostBridgeResults.length === 1 &&
        hostBridgeResults[0]?.permission === "confirmation-required" &&
        hostBridgeResults[0]?.status === "prepared",
      `${hostBridgeResults.length} host bridge record(s) normalized`,
    ),
    check(
      "trusted-runner-importable",
      trustedRunnerResults.length === 1 &&
        trustedRunnerResults[0]?.permission === "allowed" &&
        trustedRunnerResults[0]?.status === "succeeded",
      `${trustedRunnerResults.length} trusted runner result(s) normalized`,
    ),
    check(
      "trusted-runner-ux-grouped",
      trustedRunnerUx?.statusSummaries.some(
        (summary) => summary.status === "timed-out" && summary.count === 1,
      ) === true &&
        trustedRunnerUx?.statusSummaries.some(
          (summary) => summary.status === "cancelled" && summary.count === 1,
        ) === true,
      `${trustedRunnerUx?.statusSummaries.length ?? 0} trusted runner status groups`,
    ),
    check(
      "trusted-runner-ux-affordances",
      trustedRunnerUx?.affordances.some(
        (affordance) => affordance.kind === "retry" && affordance.requiresApproval,
      ) === true &&
        trustedRunnerUx?.affordances.some(
          (affordance) => affordance.kind === "copy-command" && !affordance.disabled,
        ) === true,
      `${trustedRunnerUx?.affordances.length ?? 0} trusted runner affordance(s)`,
    ),
    check(
      "trusted-runner-approval-modal",
      trustedRunnerApprovalUi?.modalId === "trusted-runner-approval" &&
        trustedRunnerApprovalUi.controls.some(
          (control) =>
            control.kind === "approve" &&
            control.requiresReason &&
            control.requiresApproval &&
            control.keyboardShortcut?.key === "Ctrl+Enter",
        ) &&
        trustedRunnerApprovalUi.controls.some(
          (control) => control.kind === "deny" && control.keyboardShortcut?.key === "Esc",
        ),
      `${trustedRunnerApprovalUi?.controls.length ?? 0} approval control(s)`,
    ),
    check(
      "trusted-runner-approval-draft-affordances",
      trustedRunnerApprovalUi?.controls.some((control) => control.kind === "snooze") === true &&
        trustedRunnerApprovalUi?.controls.some((control) => control.kind === "undo") === true &&
        trustedRunnerApprovalUi.auditReasonRequired,
      `${trustedRunnerApprovalUi?.snoozeOptions.length ?? 0} snooze option(s)`,
    ),
    check(
      "trusted-live-runner-state",
      trustedLiveRunnerState?.pendingCount === 1 &&
        trustedLiveRunnerState.runningCount === 1 &&
        trustedLiveRunnerState.staleCount === 1 &&
        trustedLiveRunnerState.records.some((record) => record.status === "stale"),
      `${trustedLiveRunnerState?.recordCount ?? 0} live runner state record(s)`,
    ),
    check(
      "trusted-live-runner-recovery",
      trustedLiveRunnerState?.staleRecoveryCopy.includes("Refresh") === true &&
        trustedLiveRunnerState.records.every((record) => record.cleanupCommand.trim().length > 0),
      trustedLiveRunnerState?.staleRecoveryCopy ?? "missing stale recovery copy",
    ),
    check(
      "trusted-runner-cancellation-controls",
      trustedCancellationUx?.controls.some(
        (control) => control.kind === "cancel" && control.requiresReason,
      ) === true &&
        trustedCancellationUx.controls.some((control) => control.kind === "cleanup-stale") &&
        trustedCancellationUx.controls.some(
          (control) => control.kind === "retry" && control.command.includes("--approve --execute"),
        ),
      `${trustedCancellationUx?.controls.length ?? 0} cancellation control(s)`,
    ),
    check(
      "trusted-runner-cancellation-drafts",
      trustedCancellationUx?.draft.storageKey === "flow.dashboard.runnerCancellationDrafts" &&
        trustedCancellationUx.draft.autosaveHint.includes("remembered"),
      trustedCancellationUx?.draft.autosaveHint ?? "missing draft hint",
    ),
    check(
      "trusted-runner-denial-recovery",
      trustedDenialRecoveryUx?.denialCount === 1 &&
        trustedDenialRecoveryUx.controls.some(
          (control) =>
            control.kind === "denial-recovery" &&
            control.requiresReason &&
            control.command.includes("denial recovery reason"),
      ),
      `${trustedDenialRecoveryUx?.controls.length ?? 0} denial recovery control(s)`,
    ),
    check(
      "trusted-runner-operator-review-filters",
      trustedOperatorReview?.filters.actionId === "host" &&
        trustedOperatorReview.releaseGateStatus === "blocked" &&
        trustedOperatorReview.blockedCount === 2,
      `${trustedOperatorReview?.matchedCount ?? 0} review record(s)`,
    ),
    check(
      "trusted-runner-operator-review-release-gate",
      trustedOperatorReview?.releaseGateSummaries.some(
        (summary) => summary.id === "failed" && summary.severity === "blocked",
      ) === true &&
        trustedOperatorReview.releaseGateSummaries.some(
          (summary) => summary.id === "stale-live-state",
        ),
      `${trustedOperatorReview?.releaseGateSummaries.length ?? 0} release gate summaries`,
    ),
    check(
      "trusted-runner-operator-review-incidents",
      trustedOperatorReview?.incidentNotes.some(
        (note) =>
          note.severity === "blocked" &&
          note.exportMarkdown.includes("Failed: Run host report"),
      ) === true,
      `${trustedOperatorReview?.incidentNotes.length ?? 0} incident note(s)`,
    ),
    check(
      "trusted-runner-release-package-evidence",
      trustedReleasePackage?.manifest.localOnly === true &&
        trustedReleasePackage.manifest.files.some(
          (file) => file.id === "runner-history" && file.sha256 === "sha-history",
        ) &&
        trustedReleasePackage.manifest.files.some(
          (file) => file.id === "runner-live-state" && !file.present,
        ),
      `${trustedReleasePackage?.manifest.evidenceCount ?? 0} release evidence item(s)`,
    ),
    check(
      "trusted-runner-release-package-warnings",
      trustedReleasePackage?.readyToShip === false &&
        trustedReleasePackage.manifest.missingCount === 1 &&
        trustedReleasePackage.warnings.some((warning) => warning.includes("blocks release")),
      `${trustedReleasePackage?.manifest.warningCount ?? 0} release package warning(s)`,
    ),
    check(
      "trusted-runner-release-timeline-importable",
      trustedReleaseTimeline?.localOnly === true &&
        trustedReleaseTimeline.packageCount === 2 &&
        trustedReleaseTimeline.latestPackageId === "trusted-runner-release-2",
      `${trustedReleaseTimeline?.packageCount ?? 0} release package(s)`,
    ),
    check(
      "trusted-runner-release-timeline-regression",
      trustedReleaseTimeline?.missingEvidenceRegressions === 1 &&
        trustedReleaseTimeline.diffs.some(
          (diff) => diff.regression && diff.newMissingEvidenceIds.includes("runner-live-state"),
        ),
      `${trustedReleaseTimeline?.missingEvidenceRegressions ?? 0} missing evidence regression(s)`,
    ),
    check(
      "release-checklist-blockers",
      releaseChecklist?.localOnly === true &&
        releaseChecklist.blockingCount === 1 &&
        releaseChecklist.blockers.some((blocker) => blocker.category === "unreviewed-changes"),
      `${releaseChecklist?.blockingCount ?? 0} release checklist blocker(s)`,
    ),
    check(
      "release-checklist-signoff-command",
      releaseChecklist?.signoffRequired === true &&
        releaseChecklist.commands.some((command) => command.includes("--friday-release-signoff")),
      `${releaseChecklist?.commands.length ?? 0} release checklist command(s)`,
    ),
    check(
      "release-qa-importable",
      releaseQa?.localOnly === true &&
        releaseQa.scoreOutOf100 === 83 &&
        releaseQa.checks.some((check) => check.id === "rust-cargo-check" && check.status === "passed"),
      `${releaseQa?.scoreOutOf100 ?? 0}/100 release QA score`,
    ),
    check(
      "release-qa-stale-warning",
      releaseQa?.staleCount === 1 &&
        releaseQa.checks.some((check) => check.id === "dashboard-smoke" && check.stale),
      `${releaseQa?.staleCount ?? 0} stale release QA result(s)`,
    ),
    check(
      "release-export-kit-importable",
      releaseExportKit?.manifest.localOnly === true &&
        releaseExportKit.manifest.fileCount === 8 &&
        releaseExportKit.manifest.files.some(
          (file) => file.id === "release-checklist" && file.sha256 === "sha-checklist",
        ),
      `${releaseExportKit?.manifest.fileCount ?? 0} release export file(s)`,
    ),
    check(
      "release-export-kit-stale-and-copy",
      releaseExportKit?.manifest.staleCount === 1 &&
        releaseExportKit.operatorCopy.includes("Manifest checksum") &&
        releaseExportKit.manifest.commands.some((command) =>
          command.includes("--friday-release-export-kit"),
      ),
      `${releaseExportKit?.manifest.staleCount ?? 0} stale export-kit file(s)`,
    ),
    check(
      "release-deployment-gate-importable",
      releaseDeploymentGate?.decision === "no-go" &&
        releaseDeploymentGate.target.localOnlyRequired === true &&
        releaseDeploymentGate.noDeployReasonCount === 2,
      `${releaseDeploymentGate?.decision ?? "missing"} deployment decision`,
    ),
    check(
      "release-deployment-gate-copy",
      releaseDeploymentGate?.operatorCopy.includes("Friday deployment gate") === true &&
        releaseDeploymentGate.commands.some((command) =>
          command.includes("--friday-release-deployment-gate"),
        ),
      `${releaseDeploymentGate?.commands.length ?? 0} deployment gate command(s)`,
    ),
    check(
      "release-candidate-archive-importable",
      releaseCandidateArchive?.candidateCount === 2 &&
        releaseCandidateArchive.latestDecision === "no-go" &&
        releaseCandidateArchive.entries.some(
          (entry) => entry.exportKitManifestSha256 === "kit-sha-2",
        ),
      `${releaseCandidateArchive?.candidateCount ?? 0} release candidate(s)`,
    ),
    check(
      "release-candidate-archive-regression",
      releaseCandidateArchive?.regressionCount === 1 &&
        releaseCandidateArchive.diffs.some(
          (diff) => diff.regression && diff.newBlockerIds.includes("new-deploy-blocker"),
        ) &&
        releaseCandidateArchive.commands.some((command) =>
          command.includes("--friday-release-candidate-archive"),
      ),
      `${releaseCandidateArchive?.regressionCount ?? 0} candidate archive regression(s)`,
    ),
    check(
      "release-promotion-ledger-importable",
      releasePromotionLedger?.recordCount === 2 &&
        releasePromotionLedger.promotedCount === 1 &&
        releasePromotionLedger.heldCount === 1 &&
        releasePromotionLedger.activeRollbackReference === "candidate-initial",
      `${releasePromotionLedger?.recordCount ?? 0} release promotion record(s)`,
    ),
    check(
      "release-promotion-ledger-checks",
      releasePromotionLedger?.postPromotionMissingCount === 1 &&
        releasePromotionLedger.records.some(
          (record) =>
            record.decision === "promoted" &&
            record.postPromotionChecks.some((item) => item.id === "post-promotion-smoke"),
        ) &&
        releasePromotionLedger.commands.some((command) =>
          command.includes("--friday-release-promotion-ledger"),
      ),
      `${releasePromotionLedger?.postPromotionMissingCount ?? 0} missing promotion check(s)`,
    ),
    check(
      "release-post-promotion-monitor-importable",
      releasePostPromotionMonitor?.scoreOutOf100 === 72 &&
        releasePostPromotionMonitor.blockingCount === 1 &&
        releasePostPromotionMonitor.activeRollbackReference === "candidate-initial",
      `${releasePostPromotionMonitor?.scoreOutOf100 ?? 0}/100 post-promotion monitor score`,
    ),
    check(
      "release-post-promotion-monitor-evidence",
      releasePostPromotionMonitor?.checks.some(
        (item) => item.status === "missing" && item.id === "post-promotion-smoke",
      ) === true &&
        releasePostPromotionMonitor.incidentNotes.some((note) => note.present) &&
        releasePostPromotionMonitor.commands.some((command) =>
          command.includes("--friday-release-post-promotion-monitor"),
      ),
      `${releasePostPromotionMonitor?.incidentNoteCount ?? 0} incident note(s)`,
    ),
    check(
      "release-rollback-drill-importable",
      releaseRollbackDrill?.scoreOutOf100 === 75 &&
        releaseRollbackDrill.activeRollbackReference === "candidate-initial" &&
        releaseRollbackDrill.dryRunCommand.includes("--friday-release-rollback-drill-json"),
      `${releaseRollbackDrill?.scoreOutOf100 ?? 0}/100 rollback drill score`,
    ),
    check(
      "release-rollback-drill-blockers",
      releaseRollbackDrill?.blockedReasons.length === 2 &&
        releaseRollbackDrill.checks.some(
          (item) => item.status === "failed" && item.id === "post-promotion-monitor",
        ) &&
        releaseRollbackDrill.commands.some((command) =>
          command.includes("--friday-release-rollback-drill"),
      ),
      `${releaseRollbackDrill?.blockedReasons.length ?? 0} rollback drill blocker(s)`,
    ),
    check(
      "release-stability-board-importable",
      releaseStabilityBoard?.scoreOutOf100 === 58 &&
        releaseStabilityBoard.readyForCheckpoint === false &&
        releaseStabilityBoard.activeRollbackReference === "candidate-initial",
      `${releaseStabilityBoard?.scoreOutOf100 ?? 0}/100 stability board score`,
    ),
    check(
      "release-stability-board-risks",
      releaseStabilityBoard?.activeRisks.length === 2 &&
        releaseStabilityBoard.checks.some(
          (item) => item.category === "rollback-recovery" && item.status === "failed",
        ) &&
        releaseStabilityBoard.evidenceLinks.some((link) => link.id === "rollback-drill") &&
        releaseStabilityBoard.commands.some((command) =>
          command.includes("--friday-release-stability-board"),
      ),
      `${releaseStabilityBoard?.activeRisks.length ?? 0} stability risk(s)`,
    ),
    check(
      "release-recovery-runbook-importable",
      releaseRecoveryRunbook?.scoreOutOf100 === 63 &&
        releaseRecoveryRunbook.phaseCount === 6 &&
        releaseRecoveryRunbook.activeRollbackReference === "candidate-initial",
      `${releaseRecoveryRunbook?.scoreOutOf100 ?? 0}/100 recovery runbook score`,
    ),
    check(
      "release-recovery-runbook-phases",
      releaseRecoveryRunbook !== null &&
        releaseRecoveryRunbook.phases
          .slice()
          .sort((left, right) => left.order - right.order)
          .map((phase) => phase.kind)
          .join(",") === "pause,diagnose,rollback,verify,resume,follow-up" &&
        releaseRecoveryRunbook.phases.some(
          (phase) =>
            phase.kind === "rollback" &&
            phase.status === "blocked" &&
            phase.command.includes("--dry-run"),
        ) &&
        releaseRecoveryRunbook.approvalGates.length === 3 &&
        releaseRecoveryRunbook.commands.some((command) =>
          command.includes("--friday-release-recovery-runbook"),
      ),
      `${releaseRecoveryRunbook?.approvalGates.length ?? 0} recovery approval gate(s)`,
    ),
    check(
      "release-incident-archive-importable",
      releaseIncidentArchive?.incidentCount === 1 &&
        releaseIncidentArchive.latestSeverity === "critical" &&
        releaseIncidentArchive.latestRollbackReference === "candidate-initial",
      `${releaseIncidentArchive?.incidentCount ?? 0} archived incident(s)`,
    ),
    check(
      "release-incident-archive-actions",
      releaseIncidentArchive !== null &&
        releaseIncidentArchive.entries.some(
          (entry) =>
            entry.followUpActions.length === 2 &&
            entry.preventionItems.some((item) => item.includes("Prevent recurrence")),
        ) &&
        releaseIncidentArchive.commands.some((command) =>
          command.includes("--friday-release-incident-archive"),
      ),
      `${releaseIncidentArchive?.followUpCount ?? 0} incident follow-up action(s)`,
    ),
    check(
      "release-prevention-plan-importable",
      releasePreventionPlan?.scoreOutOf100 === 42 &&
        releasePreventionPlan.readyForNextCheckpoint === false &&
        releasePreventionPlan.activeRollbackReference === "candidate-initial",
      `${releasePreventionPlan?.scoreOutOf100 ?? 0}/100 prevention plan score`,
    ),
    check(
      "release-prevention-plan-actions",
      releasePreventionPlan !== null &&
        releasePreventionPlan.recurringIssueCount === 1 &&
        releasePreventionPlan.actions.some(
          (action) =>
            action.kind === "harden-rollback" &&
            action.command.includes("--dry-run") &&
            action.releaseGateBlocking,
        ) &&
        releasePreventionPlan.ownerReadyCopy.includes("Friday release prevention plan") &&
        releasePreventionPlan.commands.some((command) =>
          command.includes("--friday-release-prevention-plan"),
        ),
      `${releasePreventionPlan?.actionCount ?? 0} prevention action(s)`,
    ),
    check(
      "release-owner-followup-board-importable",
      releaseOwnerFollowUpBoard?.scoreOutOf100 === 50 &&
        releaseOwnerFollowUpBoard.ownerCount === 1 &&
        releaseOwnerFollowUpBoard.overdueCount === 1,
      `${releaseOwnerFollowUpBoard?.recordCount ?? 0} owner follow-up record(s)`,
    ),
    check(
      "release-owner-followup-board-assignments",
      releaseOwnerFollowUpBoard !== null &&
        releaseOwnerFollowUpBoard.ownerGroups.some(
          (group) => group.owner === "release-operator" && group.evidenceMissingCount === 1,
        ) &&
        releaseOwnerFollowUpBoard.records.some(
          (record) =>
            record.completionState === "overdue" &&
            record.command.includes("--dry-run") &&
            record.assignmentCopy.includes("@release-operator"),
        ) &&
        releaseOwnerFollowUpBoard.commands.some((command) =>
          command.includes("--friday-release-owner-followup-board"),
        ),
      `${releaseOwnerFollowUpBoard?.evidenceMissingCount ?? 0} missing owner evidence request(s)`,
    ),
    check(
      "release-evidence-sla-monitor-importable",
      releaseEvidenceSlaMonitor?.scoreOutOf100 === 44 &&
        releaseEvidenceSlaMonitor.overdueCount === 1 &&
        releaseEvidenceSlaMonitor.escalationCount === 2,
      `${releaseEvidenceSlaMonitor?.requirementCount ?? 0} SLA requirement(s)`,
    ),
    check(
      "release-evidence-sla-monitor-escalations",
      releaseEvidenceSlaMonitor !== null &&
        releaseEvidenceSlaMonitor.ownerGroups.some(
          (group) => group.owner === "release-operator" && group.releaseGateBlockingCount === 2,
        ) &&
        releaseEvidenceSlaMonitor.requirements.some(
          (requirement) =>
            requirement.state === "overdue" &&
            requirement.escalationLevel === "checkpoint" &&
            requirement.escalationCopy.includes("@release-operator"),
        ) &&
        releaseEvidenceSlaMonitor.escalationCopy.includes(
          "Friday release evidence SLA monitor",
        ) &&
        releaseEvidenceSlaMonitor.commands.some((command) =>
          command.includes("--friday-release-evidence-sla-monitor"),
        ),
      `${releaseEvidenceSlaMonitor?.gateBlockingCount ?? 0} SLA gate blocker(s)`,
    ),
    check(
      "release-escalation-ledger-importable",
      releaseEscalationLedger?.entryCount === 2 &&
        releaseEscalationLedger.activeCount === 2 &&
        releaseEscalationLedger.acknowledgementBlockerCount === 2,
      `${releaseEscalationLedger?.entryCount ?? 0} escalation record(s)`,
    ),
    check(
      "release-escalation-ledger-carryovers",
      releaseEscalationLedger !== null &&
        releaseEscalationLedger.ownerGroups.some(
          (group) =>
            group.owner === "release-operator" &&
            group.releaseGateBlockingCount === 2,
        ) &&
        releaseEscalationLedger.entries.some(
          (entry) =>
            entry.activeCarryover &&
            entry.ownerResponse === "pending" &&
            entry.ownerResponseCopy.includes("Harden rollback drill evidence"),
        ) &&
        releaseEscalationLedger.ownerResponseCopy.includes(
          "Friday release escalation ledger",
        ) &&
        releaseEscalationLedger.commands.some((command) =>
          command.includes("--friday-release-escalation-ledger"),
        ),
      `${releaseEscalationLedger?.carryoverCount ?? 0} escalation carryover(s)`,
    ),
    check(
      "release-checkpoint-review-importable",
      releaseCheckpointReview?.decision === "hold" &&
        releaseCheckpointReview.scoreOutOf100 === 0 &&
        releaseCheckpointReview.acknowledgementBlockerCount === 3,
      `${releaseCheckpointReview?.itemCount ?? 0} checkpoint review item(s)`,
    ),
    check(
      "release-checkpoint-review-gates",
      releaseCheckpointReview !== null &&
        releaseCheckpointReview.ownerGroups.some(
          (group) =>
            group.owner === "release-operator" &&
            group.releaseGateBlockingCount === 3,
        ) &&
        releaseCheckpointReview.items.some(
          (item) =>
            item.source === "escalation-ledger" &&
            item.state === "hold" &&
            item.nextAction.includes("@release-operator"),
        ) &&
        releaseCheckpointReview.reviewNotesCopy.includes(
          "Friday release checkpoint review",
        ) &&
        releaseCheckpointReview.commands.some((command) =>
          command.includes("--friday-release-checkpoint-review"),
        ),
      `${releaseCheckpointReview?.releaseGateBlockingCount ?? 0} checkpoint gate blocker(s)`,
    ),
    check(
      "release-checkpoint-signoff-importable",
      releaseCheckpointSignoffLedger?.recordCount === 1 &&
        releaseCheckpointSignoffLedger.heldCount === 1 &&
        releaseCheckpointSignoffLedger.activeDecision === "held",
      `${releaseCheckpointSignoffLedger?.recordCount ?? 0} checkpoint signoff record(s)`,
    ),
    check(
      "release-checkpoint-signoff-history",
      releaseCheckpointSignoffLedger !== null &&
        releaseCheckpointSignoffLedger.records.some(
          (record) =>
            record.operator === "release-operator" &&
            record.activeHold &&
            record.activeCarryover &&
            record.reason.includes("acknowledgement blockers"),
        ) &&
        releaseCheckpointSignoffLedger.releaseNotesCopy.includes(
          "Friday checkpoint signoff ledger",
        ) &&
        releaseCheckpointSignoffLedger.commands.some((command) =>
          command.includes("--friday-release-checkpoint-signoff"),
        ),
      `${releaseCheckpointSignoffLedger?.activeHoldCount ?? 0} active signoff hold(s)`,
    ),
    check(
      "release-checkpoint-evidence-vault-importable",
      releaseCheckpointEvidenceVault?.entryCount === 4 &&
        releaseCheckpointEvidenceVault.missingCount === 1 &&
        releaseCheckpointEvidenceVault.manifestSha256 === "smoke-vault-checksum",
      `${releaseCheckpointEvidenceVault?.entryCount ?? 0} checkpoint vault entries`,
    ),
    check(
      "release-checkpoint-evidence-vault-attachments",
      releaseCheckpointEvidenceVault !== null &&
        releaseCheckpointEvidenceVault.entries.some(
          (entry) =>
            entry.kind === "acknowledgement-evidence" &&
            entry.required &&
            !entry.present &&
            entry.warning?.includes("Required evidence"),
        ) &&
        releaseCheckpointEvidenceVault.attachmentNotesCopy.includes(
          "Friday checkpoint evidence vault",
        ) &&
        releaseCheckpointEvidenceVault.commands.some((command) =>
          command.includes("--friday-release-checkpoint-evidence-vault"),
        ),
      `${releaseCheckpointEvidenceVault?.missingCount ?? 0} vault missing evidence item(s)`,
    ),
    check(
      "release-evidence-attachment-review-importable",
      releaseEvidenceAttachmentReview?.itemCount === 4 &&
        releaseEvidenceAttachmentReview.attachableCount === 2 &&
        releaseEvidenceAttachmentReview.blockedCount === 1,
      `${releaseEvidenceAttachmentReview?.itemCount ?? 0} attachment review item(s)`,
    ),
    check(
      "release-evidence-attachment-review-handoff",
      releaseEvidenceAttachmentReview !== null &&
        releaseEvidenceAttachmentReview.items.some(
          (item) =>
            item.state === "inline-only" &&
            item.nextAction.includes("paste it into the handoff"),
        ) &&
        releaseEvidenceAttachmentReview.items.some(
          (item) =>
            item.state === "blocked" &&
            item.releaseGateBlocking &&
            item.nextAction.includes("Resolve blocking evidence"),
        ) &&
        releaseEvidenceAttachmentReview.handoffNotesCopy.includes(
          "Friday release evidence attachment review",
        ) &&
        releaseEvidenceAttachmentReview.commands.some((command) =>
          command.includes("--friday-release-evidence-attachment-review"),
        ),
      `${releaseEvidenceAttachmentReview?.releaseGateBlockingCount ?? 0} attachment gate blocker(s)`,
    ),
    check(
      "release-handoff-packet-importable",
      releaseHandoffPacket?.sectionCount === 5 &&
        releaseHandoffPacket.attachableFileCount === 2 &&
        releaseHandoffPacket.unresolvedBlockerCount === 1,
      `${releaseHandoffPacket?.sectionCount ?? 0} handoff packet section(s)`,
    ),
    check(
      "release-handoff-packet-copy",
      releaseHandoffPacket !== null &&
        releaseHandoffPacket.sections.some(
          (section) => section.kind === "attachable-file" && section.included,
        ) &&
        releaseHandoffPacket.sections.some(
          (section) =>
            section.kind === "unresolved-blocker" &&
            section.nextAction.includes("Resolve blocking evidence"),
        ) &&
        releaseHandoffPacket.fileChecklistCopy.includes(
          "Friday release handoff file checklist",
        ) &&
        releaseHandoffPacket.handoffPacketCopy.includes(
          "Friday release handoff packet",
        ) &&
        releaseHandoffPacket.commands.some((command) =>
          command.includes("--friday-release-handoff-packet"),
        ),
      `${releaseHandoffPacket?.unresolvedBlockerCount ?? 0} handoff packet blocker(s)`,
    ),
    check(
      "release-handoff-audit-trail-importable",
      releaseHandoffAuditTrail?.recordCount === 1 &&
        releaseHandoffAuditTrail.blockedCount === 1 &&
        releaseHandoffAuditTrail.activePacketId === "friday-release-handoff-packet-smoke",
      `${releaseHandoffAuditTrail?.recordCount ?? 0} handoff audit record(s)`,
    ),
    check(
      "release-handoff-audit-trail-copy",
      releaseHandoffAuditTrail !== null &&
        releaseHandoffAuditTrail.records.some(
          (record) =>
            record.state === "blocked" &&
            record.blockerCarryover === 2 &&
            record.acknowledgementNote.includes("Blocked packet stays local"),
        ) &&
        releaseHandoffAuditTrail.auditSummaryCopy.includes(
          "Friday release handoff audit trail",
        ) &&
        releaseHandoffAuditTrail.commands.some((command) =>
          command.includes("--friday-release-handoff-audit"),
        ),
      `${releaseHandoffAuditTrail?.blockerCarryoverCount ?? 0} audit carryover record(s)`,
    ),
    check(
      "release-handoff-governance-review-importable",
      releaseHandoffGovernanceReview?.state === "blocked-carryover" &&
        releaseHandoffGovernanceReview.findingCount === 2 &&
        releaseHandoffGovernanceReview.releaseGateBlockingCount === 2,
      `${releaseHandoffGovernanceReview?.findingCount ?? 0} handoff governance finding(s)`,
    ),
    check(
      "release-handoff-governance-review-copy",
      releaseHandoffGovernanceReview !== null &&
        !releaseHandoffGovernanceReview.approvedForExternalHandoff &&
        releaseHandoffGovernanceReview.findings.some(
          (finding) =>
            finding.state === "blocked-carryover" &&
            finding.nextAction.includes("before external handoff"),
        ) &&
        releaseHandoffGovernanceReview.governanceNotesCopy.includes(
          "Friday release handoff governance review",
        ) &&
        releaseHandoffGovernanceReview.commands.some((command) =>
          command.includes("--friday-release-handoff-governance-review"),
      ),
      `${releaseHandoffGovernanceReview?.blockedCarryoverCount ?? 0} governance carryover blocker(s)`,
    ),
    check(
      "release-handoff-dispatch-checklist-importable",
      releaseHandoffDispatchChecklist?.state === "blocked" &&
        releaseHandoffDispatchChecklist.itemCount === 6 &&
        releaseHandoffDispatchChecklist.releaseGateBlockingCount === 3,
      `${releaseHandoffDispatchChecklist?.itemCount ?? 0} dispatch checklist item(s)`,
    ),
    check(
      "release-handoff-dispatch-checklist-copy",
      releaseHandoffDispatchChecklist !== null &&
        !releaseHandoffDispatchChecklist.readyToDispatch &&
        releaseHandoffDispatchChecklist.items.some(
          (item) =>
            item.source === "privacy-boundary" &&
            item.state === "privacy-review" &&
            item.releaseGateBlocking,
        ) &&
        releaseHandoffDispatchChecklist.dispatchChecklistCopy.includes(
          "Friday release handoff dispatch checklist",
        ) &&
        releaseHandoffDispatchChecklist.commands.some((command) =>
          command.includes("--friday-release-handoff-dispatch-checklist"),
        ),
      `${releaseHandoffDispatchChecklist?.privacyReviewCount ?? 0} dispatch privacy review item(s)`,
    ),
    check(
      "trusted-bridge-live-runner-importable",
      trustedBridgeLiveRunnerState?.finishedCount === 1 &&
        trustedBridgeLiveRunnerState.records[0]?.historyJson?.endsWith(
          "trusted-host-runner-history.json",
        ) === true,
      `${trustedBridgeLiveRunnerState?.recordCount ?? 0} bridge live state record(s)`,
    ),
    check(
      "history-rail-renderable",
      binding.history.recordCount > 0 &&
        binding.history.latestScoreOutOf100 != null &&
        binding.history.previousScoreOutOf100 != null &&
        binding.history.trendLabel.trim().length > 0,
      `${binding.history.recordCount} history records`,
    ),
    check(
      "release-links-renderable",
      binding.releaseLinks.length > 0 &&
        binding.releaseLinks.every(
          (link) =>
            link.localOnly &&
            link.path.trim().length > 0 &&
            link.buttonState.ariaLabel.trim().length > 0,
        ),
      `${binding.releaseLinks.length} release links`,
    ),
    check(
      "screenshot-prompts-renderable",
      binding.screenshotPrompts.length > 0 &&
        binding.screenshotPrompts.every(
          (prompt) =>
            prompt.route.trim().length > 0 &&
            prompt.viewportId.trim().length > 0 &&
            prompt.captureCommand.trim().length > 0,
        ),
      `${binding.screenshotPrompts.length} screenshot prompts`,
    ),
  ];

  return {
    scoreOutOf100: score(checks),
    checks,
  };
}
