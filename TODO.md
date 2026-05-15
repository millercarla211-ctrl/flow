# Flow TODO

Flow is not finished. Treat this file as the active product loop, not as a parking lot. When a set reaches 100/100, open the next set and keep moving.

## Completed Set: Completion Control Loop

Status: 100/100

Goal: make Flow's progress honest, visible, exportable, and hard to overstate while we continue building the actual product.

### Done

- [x] Create a professional git checkpoint before starting the new loop.
- [x] Replace static "release-ready" TODO language with this active completion loop.
- [x] Add `flow --completion` / `flow --progress` / `flow --next-100` so operators can see the current set and next actions.
- [x] Add `flow --completion-json` / `flow --progress-json` / `flow --next-100-json` for Friday/DX dashboard consumers.
- [x] Add completion-loop data to `flow --release-summary` and exported release handoff files.
- [x] Include `TODO.md` and `CHANGELOG.md` snapshots in exported production bundles.
- [x] Add tests that prevent accidental 100/100 claims while planned items remain.
- [x] Update `CHANGELOG.md` with the current loop progress.

### Completed Control Rule

This set is complete. The next loop should open `Host Autonomy Core` as the new active set and keep the same commit, typecheck, changelog, and TODO discipline.

## Completed Set: Host Autonomy Core

Status: 100/100

Goal: make Flow safer and more reliable as a host-level assistant that can read selections, rewrite text, pause itself, diagnose accessibility, and leave an audit trail.

### Done

- [x] Record host audit entries when automation reads and replaces the active selection.
- [x] Add operator-facing accessibility diagnostics for host automation readiness.
- [x] Add reliable pause/resume/snooze controls for always-on runtime hosts.
- [x] Persist and expose compact host audit logs for operator review.
- [x] Harden Windows global dictation host behavior beyond CLI/demo paths.

### Completed Control Rule

This set is complete. The next loop should open `Writing Quality Core` as the new active set and keep host autonomy stable while improving Flow's writing intelligence.

## Paused Set: Writing Quality Core

Status: 55/100

Goal: make Flow's writing assistance excellent enough to feel useful every day, with deeper explanations, citations, style policies, multilingual support, and host-friendly review surfaces.

### Done

- [x] Deepen grammar, clarity, and rewrite explanations beyond the current Harper-backed baseline.
- [x] Add citation, fact-checking, and academic assistance paths.

### Remaining To Reach 100/100

- [ ] Add style-guide and brand-tone policy enforcement.
- [ ] Add multilingual writing assistance.
- [ ] Add host-facing writing review summaries with accept/reject handoff.

### Pause Rule

This set remains important, but the active product direction has moved to Friday's competitive AI workspace. Resume this set after the Friday assistant shell has real Ask, Search, Research, Projects, Memory, Connectors, Voice, Artifacts, Automations, Code, and Multimodal surfaces.

## Completed Set: Friday Competitive AI Workspace

Status: 100/100

Goal: make Friday a local-first AI workspace that can compete with the useful surfaces of ChatGPT, Gemini, Perplexity, Grok, and Claude without depending on Perplexity Computer. Search and research must use the adjacent Rust metasearch crate.

### Done

- [x] Add a Friday capability map covering ChatGPT, Gemini, Perplexity, Grok, and Claude feature targets.
- [x] Add Friday workspace route definitions for Ask, Search, Research, Agents, Canvas, Projects, Memory, Connectors, Voice, Artifacts, Automations, Code, and Multimodal.
- [x] Add metasearch-first answer search and deep research planning that explicitly forbids Perplexity Computer as a dependency.
- [x] Add a Friday Research workflow contract with metasearch targets, stage states, and export formats.
- [x] Add a local metasearch API client path so Friday can execute cited search through the adjacent Rust server when it is running.
- [x] Add source-group, citation-ledger, and markdown-report records for metasearch research output.
- [x] Add progress-event records and persisted research bundles for reports, citations, source groups, events, and manifests.
- [x] Add local-first research synthesis prompts, answer deltas, citation references, and a local model synthesis CLI.
- [x] Add durable Projects, Memory, and Connectors stores with permission-scoped local data boundaries.
- [x] Build Canvas, Artifacts, and Code workspaces with editable outputs, previews, diffs, and checkpoints.
- [x] Connect Voice, Multimodal, and Automations surfaces to STT/TTS/OCR/VLM planning, scheduler, and audit records.

### Completed Control Rule

This set is complete. The next loop should open `Friday Product UI Integration` and connect these Rust contracts to the desktop/Next.js UI, live execution flows, visual verification, and production-ready interaction states.

## Completed Set: Friday Product UI Integration

Status: 100/100

Goal: turn the completed Friday runtime contracts into a polished product UI and verified live workflows without weakening the local-first, low-resource defaults.

### Done

- [x] Wire Ask/Search/Research to visible streaming UI, citations, saved reports, and source controls.
- [x] Wire Projects, Memory, Connectors, Canvas, Artifacts, Code, Voice, Multimodal, and Automations pages to the new stores.
- [x] Add end-to-end local execution checks for STT/TTS/OCR/metasearch/artifact preview flows.
- [x] Add production-ready empty/loading/error/permission states for every Friday route.
- [x] Run targeted browser verification and only deploy after a major user-visible feature ships.

### Completed Control Rule

This set is complete. The next loop should open `Multimodal Local Core` and finish local OCR, VLM, image, audio, and video foundations before adding more UI polish.

## Completed Set: Multimodal Local Core

Status: 100/100

Goal: finish Friday's local OCR, VLM, image, audio, and video foundations with explicit low-resource routing and artifact metadata before adding heavier UI polish.

### Done

- [x] Add a local multimodal readiness gate covering OCR artifacts, speech readiness, metasearch wiring, and artifact previews.
- [x] Add a bounded OCR fixture/screenshot smoke path that writes an artifact record.
- [x] Wire a low-resource screenshot/VLM execution contract with explicit model and artifact boundaries.
- [x] Add model-role routing rules for image, audio, video, OCR, and VLM requests.
- [x] Extend artifact records for OCR/VLM/media outputs and add tests for metadata round trips.

### Completed Control Rule

This set is complete. The next loop should open `Multimodal Product Execution` and connect these contracts to UI capture, real VLM execution, image/video install actions, and persisted user-facing artifacts without changing the low-resource idle defaults.

## Completed Set: Multimodal Product Execution

Status: 100/100

Goal: connect Friday's multimodal contracts to UI capture, install/run affordances, persisted artifacts, and visual checks without changing the low-resource idle defaults.

### Done

- [x] Connect OCR smoke reports to the Multimodal UI and OCR page diagnostics.
- [x] Add a screenshot capture command that feeds the VLM contract from a real local image path.
- [x] Add explicit install/run affordances for image and video model candidates.
- [x] Persist multimodal artifact metadata in the Friday artifact store.
- [x] Add browser or desktop visual checks for the Multimodal UI route.

### Completed Control Rule

This set is complete. The next loop should open `Browser And Extension Core` and connect these verification gates to live desktop/browser screenshot checks, packaged-extension smoke tests, offline browser model reuse, and quota recovery.

## Completed Set: Browser And Extension Core

Status: 100/100

Goal: harden Friday's browser extension release path with packaged smoke checks, installed-browser launch validation, offline browser-pack reuse, recovery handling, and optional Chromium acceleration without weakening local-first defaults.

### Done

- [x] Add a packaged extension smoke matrix for Chromium, Edge, Firefox, and Safari targets with installed-browser detection.
- [x] Smoke test packaged extensions with isolated temporary-profile launches in installed Chrome, Edge, Firefox, and Safari targets.
- [x] Verify offline browser-pack reuse after first download.
- [x] Verify partial-download resume, hash rejection, and quota recovery.
- [x] Add optional Chromium WebLLM acceleration after Qwen browser packs are validated.

### Completed Control Rule

This set is complete. The next loop should open `Friday Live UI Execution` and connect these contracts to tracked desktop/web route files, screenshot-backed visual verification, and operator-facing readiness summaries.

## Completed Set: Friday Live UI Execution

Status: 100/100

Goal: connect Friday's Rust contracts to tracked desktop/web UI route files, screenshot-backed visual verification, and operator-facing readiness summaries.

### Done

- [x] Connect the Friday Rust contracts to tracked desktop/web UI route files instead of contract-only CLI output.
- [x] Add an operator-facing readiness summary for local model, extension, desktop host, route, multimodal, and release-loop readiness.
- [x] Add screenshot target verification for the most-used Friday routes across desktop/mobile viewports with tracked artifact paths.
- [x] Add desktop/web execution handoff contracts for launching live flows from UI surfaces.
- [x] Export readiness summaries for Friday/DX dashboards.

### Completed Control Rule

This set is complete. The next loop should open `Friday Dashboard Runtime Wiring` and consume the exported readiness bundle from the product UI/DX dashboard instead of scraping CLI text.

## Completed Set: Friday Dashboard Runtime Wiring

Status: 100/100

Goal: make Friday's dashboard and DX surfaces consume live readiness exports, display recent visual captures, and guide the next release loop without manual CLI copying.

### Done

- [x] Wire the dashboard export bundle into a real Friday/DX UI panel.
- [x] Add recent screenshot capture history and missing-capture prompts for top Friday routes.
- [x] Add one-click local command launch/recovery actions from dashboard readiness cards.
- [x] Persist export history so operators can compare readiness between checkpoints.
- [x] Add a release-review handoff that links completion, changelog, TODO, visual targets, and dashboard export files.

### Completed Control Rule

This set is complete. The next loop should open `Friday Dashboard Product UI Wiring` and consume these typed cards, actions, history records, and release-review handoffs from the visible Friday dashboard without scraping CLI text.

## Completed Set: Friday Dashboard Product UI Wiring

Status: 100/100

Goal: make the visible Friday desktop/web dashboard consume the live Rust dashboard panel contract, execute safe local actions, and show release readiness without dummy product copy.

### Done

- [x] Bind the dashboard panel JSON to the visible Friday dashboard surface.
- [x] Wire typed dashboard actions to safe UI buttons with disabled/loading/error states.
- [x] Show export history deltas and screenshot prompts in the product UI.
- [x] Render release-review handoff links for TODO, changelog, visual targets, and export artifacts.
- [x] Add a small UI smoke contract proving the dashboard can load the exported panel.

### Completed Control Rule

This set is complete. The next loop should open `Friday Dashboard Visible UI Execution` and render the typed dashboard binding inside the visible web extension/desktop UI without dummy product copy.

## Completed Set: Friday Dashboard Visible UI Execution

Status: 100/100

Goal: wire the visible Friday dashboard to render the typed dashboard binding and execute local-only actions from the UI layer.

### Done

- [x] Render dashboard cards from the product UI binding in `extensions/flow-webext/src/ui/app.ts`.
- [x] Add visible disabled/loading/success/error button states for dashboard actions.
- [x] Render export history, release-review links, and screenshot prompts in the visible dashboard.
- [x] Add a small TypeScript smoke/typecheck path for dashboard section rendering.
- [x] Keep local-only fallback behavior and remove any dummy product copy from this dashboard surface.

### Completed Control Rule

This set is complete. The next loop should open `Friday Dashboard Command Execution` and turn the visible dashboard action buttons from safe local command labels into a permission-aware command dispatch surface.

## Completed Set: Friday Dashboard Command Execution

Status: 100/100

Goal: execute dashboard actions through explicit local command handoffs while preserving user control, permissions, and low-resource behavior.

### Done

- [x] Add a local command dispatcher contract for dashboard actions.
- [x] Add confirmation and permission states for risky dashboard commands.
- [x] Persist recent dashboard action results for operator review.
- [x] Add focused tests for command dispatch success, failure, and blocked states.
- [x] Surface command execution results in the visible dashboard without auto-running anything silently.

### Completed Control Rule

This set is complete. The next loop should open `Friday Dashboard Host Command Bridge` and connect trusted desktop/Tauri command execution to the dispatcher without allowing silent background commands.

## Completed Set: Friday Dashboard Host Command Bridge

Status: 100/100

Goal: bridge prepared dashboard command handoffs into trusted desktop execution with operator approval, audit logs, and safe failure recovery.

### Done

- [x] Add a trusted host command bridge contract for prepared dashboard command results.
- [x] Require explicit operator approval before desktop/Tauri command execution.
- [x] Write command execution audit records with stdout/stderr summaries and duration.
- [x] Add blocked-command tests for remote, destructive, and malformed commands.
- [x] Surface trusted host execution results in the dashboard without freezing the UI.

### Completed Control Rule

This set is complete. The next loop should open `Friday Trusted Host Runner` and implement the approved desktop/Tauri execution worker with bounded stdout/stderr capture, cancellation, and persistent audit history.

## Completed Set: Friday Trusted Host Runner

Status: 100/100

Goal: execute approved dashboard host commands through a bounded trusted runner while preserving cancellation, auditability, and local-only safety.

### Done

- [x] Add a bounded trusted command runner that accepts only approved bridge records.
- [x] Add timeout, cancellation, stdout/stderr size limits, and process error mapping.
- [x] Persist host execution audit history separately from prepared handoff history.
- [x] Add tests for approved success, timeout, cancellation, and denied commands.
- [x] Surface trusted runner status in the dashboard with non-blocking progress updates.

### Completed Control Rule

This set is complete. The next loop should open `Friday Dashboard Runner UX` and make approved runner progress, history, retry, and cancellation states feel excellent in the product UI.

## Completed Set: Friday Dashboard Runner UX

Status: 100/100

Goal: make trusted runner history and live progress easy to understand, retry, cancel, and audit from the dashboard without clutter or blocking the UI.

### Done

- [x] Add grouped runner history summaries for success, failure, timeout, cancelled, and denied states.
- [x] Add retry and copy-command affordances that preserve explicit approval requirements.
- [x] Add cancellation and timeout status copy that explains what happened clearly.
- [x] Add focused UI smoke checks for runner history rendering and status grouping.
- [x] Add compact operator export notes linking runner history to release review.

### Completed Control Rule

This set is complete. The next loop should open `Friday Runner Approval UI` and make approve, deny, retry, copy, and cancel interactions feel native in the visible dashboard without weakening local-only safety.

## Completed Set: Friday Runner Approval UI

Status: 100/100

Goal: turn trusted runner UX metadata into a polished dashboard approval surface with keyboard-accessible controls, audit reasons, and live-safe affordances.

### Done

- [x] Add a dedicated approval modal contract for trusted runner actions.
- [x] Add keyboard-accessible approve, deny, copy, retry, and cancel controls.
- [x] Capture operator approval or denial reasons in the runner audit trail.
- [x] Add undo/snooze affordances for pending dashboard execution handoffs.
- [x] Add focused UI smoke checks for approval-modal rendering and audit reason persistence.

### Completed Control Rule

This set is complete. The next loop should open `Friday Live Runner State` and make pending, running, cancelled, timed-out, failed, and completed runner states update from a live desktop host without polling brittle CLI text.

## Completed Set: Friday Live Runner State

Status: 100/100

Goal: connect trusted runner approval and history UX to live state transitions so the dashboard can show pending, running, completed, failed, timed-out, cancelled, and denied work without confusing stale imports for live execution.

### Done

- [x] Add a typed live runner state record separate from immutable history.
- [x] Add a local-only state writer/reader for pending, running, finished, and stale runner jobs.
- [x] Add dashboard rendering for live runner progress without blocking the UI.
- [x] Add stale-state recovery copy and cleanup affordances.
- [x] Add focused tests for state transitions and stale import handling.

### Completed Control Rule

This set is complete. The next loop should open `Friday Desktop Runner Bridge` and connect live runner state updates to the trusted desktop host process instead of only projected/imported JSON.

## Completed Set: Friday Desktop Runner Bridge

Status: 100/100

Goal: make the trusted desktop host produce live runner state updates during real approved command execution while keeping all host execution local-only, auditable, cancellable, and bounded.

### Done

- [x] Add a desktop-host runner bridge interface that emits pending, running, and finished state updates.
- [x] Write live state before execution starts and after completion, denial, timeout, or cancellation.
- [x] Add cancellation token plumbing for live host commands.
- [x] Add dashboard import guidance that distinguishes live host state from static history exports.
- [x] Add focused tests for bridge state emission and cancellation boundaries.

### Completed Control Rule

This set is complete. The next loop should open `Friday Desktop Runner Cancellation UX` and make cancellation, stale cleanup, retry, and denial recovery feel excellent from the visible dashboard.

## Next Set: Friday Desktop Runner Cancellation UX

Status: 100/100

Goal: make live trusted-runner cancellation and recovery obvious in the dashboard so operators can stop, clean up, retry, or deny work without guessing which JSON import is authoritative.

### Completed To Reach 100/100

- [x] Add visible cancellation controls for active live runner records.
- [x] Add stale cleanup and retry guidance linked to bridge events.
- [x] Add denial recovery copy for approval mistakes.
- [x] Persist dashboard-side cancellation draft state.
- [x] Add focused smoke/tests for cancellation and stale cleanup UI.

### Completed Control Rule

This set is complete. The next loop should open `Friday Trusted Runner Operator Review` and make local command audit review, filtering, export, and release gating feel production-ready.

## Next Set: Friday Trusted Runner Operator Review

Status: 100/100

Goal: make trusted runner audit review useful after many local commands by giving operators filterable history, export-ready incident notes, release-gate summaries, and clear escalation paths.

### Completed To Reach 100/100

- [x] Add typed trusted-runner review filters for status, action, and time window.
- [x] Add release-gate summaries that highlight blocked, failed, timed-out, cancelled, denied, and stale runner work.
- [x] Add export-ready incident notes for failed or unsafe command attempts.
- [x] Add dashboard rendering for the review filters and release-gate summaries.
- [x] Add focused Rust/TypeScript coverage for trusted-runner review filtering and release-gate copy.

### Completed Control Rule

This set is complete. The next loop should open `Friday Trusted Runner Release Package` and turn runner review evidence into a single exportable release handoff that operators can attach to shipped builds.

## Completed Set: Friday Trusted Runner Release Package

Status: 100/100

Goal: package trusted-runner evidence into one signed, reviewable, local-only release handoff with manifest links, incident markdown, live-state freshness, and CLI/browser import guidance.

### Completed To Reach 100/100

- [x] Add a typed trusted-runner release package report that merges history review, cancellation UX, live state, and release review links.
- [x] Add a local-only package manifest with checksums for runner history, live state, incident notes, and dashboard JSON.
- [x] Add CLI and JSON commands for generating the release package without running new host commands.
- [x] Add dashboard import/rendering for the package summary and missing-evidence warnings.
- [x] Add focused Rust/TypeScript coverage for package completeness, checksum copy, and stale evidence warnings.

### Completed Control Rule

This set is complete. The next loop should open `Friday Trusted Runner Evidence Timeline` and make release packages comparable over time with diff warnings, archive commands, and visible package-history review.

## Completed Set: Friday Trusted Runner Evidence Timeline

Status: 100/100

Goal: make trusted-runner release packages comparable over time so operators can spot regressions, stale evidence, missing artifacts, and recurring command failures before shipping.

### Completed To Reach 100/100

- [x] Add a typed trusted-runner package timeline/history model that can load multiple release package JSON files.
- [x] Add package-to-package diff summaries for missing evidence, warning count changes, stale runner changes, and signature changes.
- [x] Add CLI and JSON archive commands for appending release packages to a local evidence timeline without running host commands.
- [x] Add dashboard timeline rendering with compare controls, regression warnings, and latest-package freshness copy.
- [x] Add focused Rust/TypeScript coverage for timeline loading, diff warnings, archive writes, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Operator Checklist` and turn package/timeline evidence into a signed checklist that operators can review before shipping.

## Completed Set: Friday Release Operator Checklist

Status: 100/100

Goal: turn release package and evidence-timeline data into a concise operator checklist with explicit signoff, unresolved-blocker copy, and local-only audit history.

### Completed To Reach 100/100

- [x] Add a typed release checklist model that consumes release packages, timelines, TODO, changelog, and dashboard readiness.
- [x] Add blocker categorization for missing evidence, warning regressions, stale live state, pending runner work, and unreviewed changes.
- [x] Add CLI and JSON commands for creating checklist reports and appending local signoff records.
- [x] Add dashboard checklist rendering with signoff controls, reason capture, and latest-check freshness copy.
- [x] Add focused Rust/TypeScript coverage for checklist generation, signoff persistence, blocker copy, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release QA Command Center` and consolidate lightweight checks, checklist status, package/timeline status, and dashboard smoke into one local command center.

## Completed Set: Friday Release QA Command Center

Status: 100/100

Goal: provide a single local-first QA command center that runs or imports lightweight checks, summarizes release risk, and tells the operator exactly what must pass before a major Friday checkpoint.

### Completed Toward 100/100

- [x] Add a typed QA command-center report that consumes checklist, package, timeline, dashboard smoke, Rust check status, and extension typecheck status.
- [x] Add local-only command descriptors for each lightweight check without silently running expensive builds.
- [x] Add CLI and JSON commands for creating QA command-center reports and importing check-result files.
- [x] Add dashboard QA rendering with pass/fail badges, copyable commands, and stale-result warnings.
- [x] Add focused Rust/TypeScript coverage for QA report scoring, stale result detection, command copy, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Evidence Export Kit` and make the release checklist, QA command center, package, timeline, signoffs, and lightweight check outputs exportable as one review bundle.

## Next Set: Friday Release Evidence Export Kit

Status: 0/100

Goal: bundle the release checklist, QA command center, package, timeline, signoffs, and lightweight check outputs into one local-only review kit with manifests, checksums, and dashboard import guidance.

### Remaining To Reach 100/100

- [ ] Add a typed release evidence export-kit model with manifest checksums for checklist, QA, package, timeline, signoffs, and check-result files.
- [ ] Add CLI and JSON commands for generating the export kit without running host commands or full builds.
- [ ] Add dashboard import/rendering for export-kit completeness, stale result warnings, and missing artifact guidance.
- [ ] Add operator copy for attaching the export kit to major checkpoints and deployment notes.
- [ ] Add focused Rust/TypeScript coverage for export-kit completeness, checksum copy, stale warnings, and dashboard rendering.
