import {
  defaultFridayDashboardBinding,
  normalizeFridayDashboardBinding,
} from "../runtime/dashboard-binding";
import {
  buildTrustedHostRunnerCancellationUx,
  dispatchDashboardCommand,
  normalizeReleaseCandidateArchive,
  normalizeReleaseDeploymentGate,
  normalizeReleaseEvidenceExportKit,
  normalizeReleaseOperatorChecklist,
  normalizeReleasePostPromotionMonitor,
  normalizeReleasePromotionLedger,
  normalizeReleaseQaCommandCenter,
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
