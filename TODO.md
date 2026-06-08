# Flow TODO

## June 6 DX/Zed Dictation Host Handoff

Status: source-aligned, runtime proofs partial

This section supersedes older completed-set language for current Zed STT/TTS proof claims: live Zed microphone capture, governed Nemotron smoke proof, and audible Kokoro playback remain unclaimed until explicitly rerun.

- [x] Keep `flow-dictate` as the focused STT host for Zed's Agent composer.
- [x] Document Parakeet as the default STT model and Nemotron / Whisper Tiny GGML as explicit opt-in models.
- [x] Document Whisper binary, model, and language flag/env aliases used by Zed and the focused host.
- [ ] Install or point Nemotron model files before claiming Nemotron runtime readiness.
- [x] Install or point a whisper.cpp binary plus GGML model before claiming Whisper runtime readiness.
- [ ] Run governed non-live Nemotron file-smoke proof before widening Nemotron readiness claims.
- [x] Run governed non-live Whisper file-smoke proof through `flow-dictate --file ... --model whisper-tiny-ggml`.
- [ ] Run governed live Zed microphone and Kokoro audible playback proof from the editor when authorized.

## May 22 DX Launch Token / RLM / Serializer Lane

Status: 0/100

- [ ] Inspect `G:\Dx\token`, `G:\Workspaces\flow\trash\token`, and `G:\Dx\inspirations\openclaw\extensions\tokenjuice`.
- [ ] Extract useful token budget/live-prune ideas into a clean DX receipt contract.
- [ ] Connect serializer/RLM estimates to Zed-facing fields without broad JSON parser rewrites.
- [ ] Preserve source, models, and existing Flow crates.
- [ ] Ask before full builds, local servers, or heavy validation.

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

## Completed Set: Friday Release Evidence Export Kit

Status: 100/100

Goal: bundle the release checklist, QA command center, package, timeline, signoffs, and lightweight check outputs into one local-only review kit with manifests, checksums, and dashboard import guidance.

### Completed To Reach 100/100

- [x] Add a typed release evidence export-kit model with manifest checksums for checklist, QA, package, timeline, signoffs, and check-result files.
- [x] Add CLI and JSON commands for generating the export kit without running host commands or full builds.
- [x] Add dashboard import/rendering for export-kit completeness, stale result warnings, and missing artifact guidance.
- [x] Add operator copy for attaching the export kit to major checkpoints and deployment notes.
- [x] Add focused Rust/TypeScript coverage for export-kit completeness, checksum copy, stale warnings, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Deployment Gate` and turn checklist, QA, export-kit, dashboard, and deployment target evidence into an explicit deploy/no-deploy decision surface.

## Completed Set: Friday Release Deployment Gate

Status: 100/100

Goal: make Friday's major-checkpoint deployment decision explicit by consuming the release evidence export kit, QA report, checklist, dashboard state, and deployment target into one local-first go/no-go gate.

### Completed To Reach 100/100

- [x] Add a typed deployment-gate model that consumes export-kit, QA, checklist, release package, timeline, dashboard readiness, and optional target metadata.
- [x] Add no-deploy reason categories for missing evidence, stale checks, blocked QA, unsigned releases, unreviewed dashboard state, and target mismatch.
- [x] Add CLI and JSON commands for generating deployment-gate reports without running builds or deployments.
- [x] Add dashboard import/rendering for go/no-go status, target profile, rollback note, and copyable deploy checklist.
- [x] Add focused Rust/TypeScript coverage for deployment-gate scoring, target mismatch warnings, and visible dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Candidate Archive` and preserve deployment gates, evidence kits, target metadata, and rollback notes as comparable candidate records before any major deploy.

## Completed Set: Friday Release Candidate Archive

Status: 100/100

Goal: preserve every Friday major-checkpoint candidate as a comparable local record so operators can see deployment gates, evidence kits, targets, rollback notes, and promotion history without digging through loose JSON files.

### Completed Toward 100/100

- [x] Add a typed release-candidate archive model that stores deployment gate JSON, export kit JSON, target profile, score, decision, and rollback note.
- [x] Add candidate diff summaries for score changes, new blockers, resolved blockers, target changes, and evidence checksum changes.
- [x] Add CLI and JSON commands for appending candidates and printing candidate history without running builds or deployments.

### Remaining To Reach 100/100

- [x] Add dashboard import/rendering for candidate history, latest-candidate status, and compare-to-previous guidance.
- [x] Add focused Rust/TypeScript coverage for candidate archive writes, diff warnings, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Promotion Ledger` and make go/no-go candidate promotion, deployment notes, rollback references, and post-promotion verification auditable before any major deploy.

## Completed Set: Friday Release Promotion Ledger

Status: 100/100

Goal: track which candidate was promoted, why it was promoted, what deployment note was attached, which rollback reference is active, and what post-promotion checks still need evidence.

### Completed To Reach 100/100

- [x] Add a typed promotion ledger model that links a release candidate to an operator decision, deployment note, target, and rollback reference.
- [x] Add promotion decision categories for promoted, held, rolled back, superseded, and abandoned candidates.
- [x] Add CLI and JSON commands for recording promotions without running deployments.
- [x] Add dashboard import/rendering for promotion history, active rollback reference, and post-promotion checks.
- [x] Add focused Rust/TypeScript coverage for promotion writes, rollback references, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Post-Promotion Monitor` and track post-promotion verification evidence, stale checks, rollback readiness, and incident notes after a candidate is promoted.

## Completed Set: Friday Release Post-Promotion Monitor

Status: 100/100

Goal: make Friday's post-promotion state visible after a candidate is promoted, including verification freshness, incident notes, rollback readiness, and follow-up checks.

### Completed To Reach 100/100

- [x] Add a typed post-promotion monitor model that consumes the promotion ledger, QA command center, dashboard smoke, and incident-note evidence.
- [x] Add stale-check and missing-evidence warnings for promoted candidates.
- [x] Add CLI and JSON commands for generating monitor reports without running deployments.
- [x] Add dashboard import/rendering for post-promotion status, rollback readiness, and incident notes.
- [x] Add focused Rust/TypeScript coverage for monitor scoring, stale warnings, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Rollback Drill` and prove that rollback references can produce a local-only drill report before any promoted candidate is treated as stable.

## Completed Set: Friday Release Rollback Drill

Status: 100/100

Goal: make rollback readiness testable by turning active rollback references, promotion records, deployment gates, and post-promotion monitor evidence into a dry-run drill report.

### Completed To Reach 100/100

- [x] Add a typed rollback drill model that consumes the post-promotion monitor, promotion ledger, candidate archive, and deployment gate.
- [x] Add rollback readiness checks for missing rollback references, stale monitor evidence, and unresolved post-promotion blockers.
- [x] Add CLI and JSON commands for generating rollback drill reports without executing rollback commands.
- [x] Add dashboard import/rendering for rollback drill status, dry-run commands, and blocked rollback reasons.
- [x] Add focused Rust/TypeScript coverage for rollback drill scoring, blocked reasons, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Stability Evidence Board` and consolidate release QA, candidate archive, promotion ledger, post-promotion monitor, rollback drill, and deployment gate evidence into one operator-facing stability surface.

## Completed Set: Friday Release Stability Evidence Board

Status: 100/100

Goal: consolidate all major Friday release evidence into one local-first stability board that tells the operator whether the current candidate is deployable, stable, recoverable, and ready for the next major checkpoint.

### Completed Toward 100/100

- [x] Add a typed stability evidence board model that consumes release QA, candidate archive, promotion ledger, post-promotion monitor, rollback drill, and deployment gate reports.
- [x] Add stability score categories for deployment readiness, post-promotion freshness, rollback recovery, candidate regression, and QA health.
- [x] Add CLI and JSON commands for generating the stability board without running builds, deployments, or rollback commands.
- [x] Add dashboard import/rendering for stability score, active risks, evidence links, and next operator actions.
- [x] Add focused TypeScript dashboard coverage for board import/rendering after the visible board is wired.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Recovery Runbook` and turn stability-board risks, rollback drill output, and release evidence links into a local-only operator runbook with explicit approval gates.

## Completed Set: Friday Release Recovery Runbook

Status: 100/100

Goal: convert the stability evidence board into a practical local-only recovery runbook that tells the operator exactly how to pause, investigate, rollback, verify, and resume Friday after a blocked release or failed promotion.

### Completed To Reach 100/100

- [x] Add a typed recovery runbook model that consumes the stability board, rollback drill, promotion ledger, and post-promotion monitor.
- [x] Add runbook phases for pause, diagnose, rollback, verify, resume, and follow-up incident notes.
- [x] Add CLI and JSON commands for generating runbooks without executing recovery commands.
- [x] Add dashboard import/rendering for runbook phases, approval gates, and copyable recovery commands.
- [x] Add focused Rust/TypeScript coverage for blocked-risk mapping, command safety, phase ordering, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Incident Archive` and preserve recovery runbooks, incident notes, stability snapshots, rollback drills, and operator outcomes as searchable local release history.

## Completed Set: Friday Release Incident Archive

Status: 100/100

Goal: preserve Friday recovery decisions as searchable local release history that connects runbooks, incident notes, stability-board snapshots, rollback drills, and operator outcomes.

### Completed To Reach 100/100

- [x] Add a typed incident archive model that consumes recovery runbooks, stability boards, post-promotion monitors, rollback drills, and incident-note files.
- [x] Add severity, outcome, follow-up action, and prevention taxonomies for release incidents.
- [x] Add CLI and JSON commands for append, list, and export workflows without executing recovery commands.
- [x] Add dashboard import/rendering for incident history, severity, outcomes, follow-ups, and source evidence.
- [x] Add focused Rust/TypeScript coverage for archive append/list behavior, severity mapping, stale evidence, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Prevention Planner` and turn incident history into concrete prevention work, owner-ready actions, and readiness gates for future release loops.

## Completed Set: Friday Release Prevention Planner

Status: 100/100

Goal: convert archived release incidents into a prevention plan that identifies recurring failure classes, assigns owner-ready follow-up actions, and blocks the next checkpoint until prevention evidence is attached.

### Completed To Reach 100/100

- [x] Add a typed prevention planner model that consumes the release incident archive and current stability board.
- [x] Add recurrence detection for repeated blocker categories, stale evidence, missing incident notes, and unresolved rollback gaps.
- [x] Add CLI and JSON commands for generating prevention plans without executing remediation commands.
- [x] Add dashboard import/rendering for prevention actions, owner-ready copy, blockers, and evidence links.
- [x] Add focused Rust/TypeScript coverage for recurrence scoring, blocked-plan states, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Owner Follow-up Board` and turn prevention actions into reviewable owner assignments, due windows, evidence requests, and completion gates.

## Completed Set: Friday Release Owner Follow-up Board

Status: 100/100

Goal: turn prevention-plan actions into owner-ready follow-up records with due windows, evidence requirements, completion states, and dashboard review controls.

### Completed To Reach 100/100

- [x] Add a typed owner follow-up board model that consumes the release prevention plan.
- [x] Add owner assignment, due-window, evidence-request, and completion-state fields for each prevention action.
- [x] Add CLI and JSON commands for generating follow-up boards without executing remediation commands.
- [x] Add dashboard import/rendering for owners, due windows, evidence requests, completion blockers, and copyable assignment text.
- [x] Add focused Rust/TypeScript coverage for owner grouping, overdue detection, evidence gates, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Evidence SLA Monitor` and watch owner follow-up records, prevention-plan evidence, and stability artifacts for freshness, due-window breaches, and escalation-ready release blockers.

## Completed Set: Friday Release Evidence SLA Monitor

Status: 100/100

Goal: monitor owner follow-up boards and release evidence freshness against explicit SLA windows so Friday can escalate stale or overdue release blockers before the next checkpoint.

### Completed To Reach 100/100

- [x] Add a typed release evidence SLA monitor model that consumes owner follow-up boards, prevention plans, and stability evidence.
- [x] Add freshness, due-window, escalation, and acknowledgement states for each release evidence requirement.
- [x] Add CLI and JSON commands for generating SLA monitor reports without running builds, deployments, or remediation commands.
- [x] Add dashboard import/rendering for SLA status, overdue owners, escalation copy, and acknowledgement blockers.
- [x] Add focused Rust/TypeScript coverage for SLA scoring, stale evidence, owner escalation grouping, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Escalation Ledger` and preserve SLA escalations, owner responses, acknowledgement decisions, and release-gate outcomes as searchable local release-control history.

## Completed Set: Friday Release Escalation Ledger

Status: 100/100

Goal: turn SLA monitor escalations into auditable release-control history with owner responses, acknowledgement decisions, release-gate outcomes, and next-checkpoint carryover.

### Completed To Reach 100/100

- [x] Add a typed escalation ledger model that consumes release evidence SLA monitor reports.
- [x] Add owner response, acknowledgement, release-gate outcome, and carryover states for each escalation.
- [x] Add append/list/export CLI and JSON commands without executing remediation commands.
- [x] Add dashboard import/rendering for escalation history, active carryovers, acknowledgement blockers, and copyable owner response text.
- [x] Add focused Rust/TypeScript coverage for append/list behavior, acknowledgement scoring, release-gate carryover, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Checkpoint Review Board` and consolidate escalation ledgers, SLA monitors, owner follow-ups, prevention evidence, and stability evidence into a signed checkpoint decision surface.

## Completed Set: Friday Release Checkpoint Review Board

Status: 100/100

Goal: consolidate release escalations, SLA state, owner follow-ups, prevention evidence, and stability artifacts into one signed checkpoint review board that decides whether Friday can move forward.

### Completed To Reach 100/100

- [x] Add a typed checkpoint review board model that consumes escalation ledgers, SLA monitors, owner follow-up boards, prevention plans, and stability evidence.
- [x] Add readiness, hold, carryover, and review-decision states with explicit owner acknowledgement requirements.
- [x] Add CLI and JSON commands for generating checkpoint review boards without running deployments, builds, or remediation commands.
- [x] Add dashboard import/rendering for checkpoint decisions, active escalations, carryover blockers, and copyable review notes.
- [x] Add focused Rust/TypeScript coverage for decision scoring, carryover blockers, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Checkpoint Signoff Ledger` and preserve checkpoint decisions, operator signoffs, acknowledgement evidence, and carryover commitments as searchable local release history.

## Completed Set: Friday Release Checkpoint Signoff Ledger

Status: 100/100

Goal: turn checkpoint review decisions into auditable signoff records that capture who approved, held, or carried over the release checkpoint and which acknowledgement evidence was attached.

### Completed To Reach 100/100

- [x] Add a typed checkpoint signoff ledger model that consumes checkpoint review boards.
- [x] Add signed-off, held, carried-over, superseded, and revoked decision states with operator reason capture.
- [x] Add append/list/export CLI and JSON commands without running deployments, builds, or remediation commands.
- [x] Add dashboard import/rendering for signoff history, active holds, carryover commitments, and copyable release notes.
- [x] Add focused Rust/TypeScript coverage for signoff writes, decision history, carryover commitments, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Checkpoint Evidence Vault` and package checkpoint reviews, signoff ledgers, acknowledgement evidence, carryover commitments, and release notes into one local evidence bundle.

## Completed Set: Friday Release Checkpoint Evidence Vault

Status: 100/100

Goal: package checkpoint review and signoff evidence into a durable local vault so every release-control decision has attached evidence, checksums, and operator-ready export notes.

### Completed To Reach 100/100

- [x] Add a typed checkpoint evidence vault model that consumes checkpoint reviews and signoff ledgers.
- [x] Add manifest entries for review JSON, signoff ledger JSON, acknowledgement evidence files, carryover commitments, and release notes.
- [x] Add CLI and JSON commands for generating vault bundles without running deployments, builds, or remediation commands.
- [x] Add dashboard import/rendering for vault completeness, missing evidence, checksums, and copyable attachment notes.
- [x] Add focused Rust/TypeScript coverage for vault completeness, checksum copy, missing-evidence warnings, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Evidence Attachment Review` and verify that vault attachments are operator-ready before any release note, deployment note, or handoff.

## Completed Set: Friday Release Evidence Attachment Review

Status: 100/100

Goal: review checkpoint evidence vault attachments before handoff so Friday can show exactly which files are attachable, which inline notes need review, and which evidence blockers remain.

### Completed To Reach 100/100

- [x] Add a typed evidence attachment review model that consumes checkpoint evidence vaults.
- [x] Add attachability states for ready, missing, inline-only, checksum-missing, and blocked evidence.
- [x] Add CLI and JSON commands that review attachments without uploading, deploying, building, or mutating external systems.
- [x] Add dashboard import/rendering for attachment readiness, first missing blocker, manifest checksum, and copyable handoff notes.
- [x] Add focused Rust/TypeScript coverage for attachment readiness, warning copy, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Handoff Packet` and assemble attachment reviews into a final local handoff packet with operator summary, attachable files, inline notes, and unresolved blockers.

## Completed Set: Friday Release Handoff Packet

Status: 100/100

Goal: assemble the attachment review into one final local release handoff packet that can be copied into an operator note without losing attachable files, inline release notes, blockers, or manifest checksums.

### Completed To Reach 100/100

- [x] Add a typed release handoff packet model that consumes evidence attachment reviews.
- [x] Add packet sections for operator summary, attachable files, inline notes, unresolved blockers, and manifest checksums.
- [x] Add CLI and JSON commands that create handoff packets without uploading, deploying, building, or mutating external systems.
- [x] Add dashboard import/rendering for packet readiness, blocker summary, copyable handoff packet, and file checklist.
- [x] Add focused Rust/TypeScript coverage for packet assembly, blocker preservation, copy text, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Handoff Audit Trail` and preserve draft, ready, sent, superseded, revoked, and blocked handoff packet history without sending, deploying, or mutating external systems.

## Completed Set: Friday Release Handoff Audit Trail

Status: 100/100

Goal: preserve every generated release handoff packet as a local audit trail so Friday can show packet history, latest readiness, operator acknowledgement state, superseded packet lineage, and unresolved blocker carryover.

### Completed To Reach 100/100

- [x] Add a typed handoff audit trail model that consumes release handoff packets.
- [x] Add audit states for draft, ready, sent, superseded, revoked, and blocked packets.
- [x] Add append, list, export, and JSON commands that never send, deploy, build, or mutate external systems.
- [x] Add dashboard import/rendering for packet history, latest packet, unresolved blockers, acknowledgement notes, and copyable audit summary.
- [x] Add focused Rust/TypeScript coverage for trail append/list behavior, state preservation, blocker carryover, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Handoff Governance Review` and validate audit trails before any release note, public handoff, deployment note, or external send.

## Completed Set: Friday Release Handoff Governance Review

Status: 100/100

Goal: review release handoff audit trails before external communication so Friday can prove the latest packet is ready or sent, no blocker carryover remains, superseded/revoked packets are not active, and acknowledgement notes are present.

### Completed To Reach 100/100

- [x] Add a typed handoff governance review model that consumes handoff audit trails.
- [x] Add review states for approved, held, needs-acknowledgement, stale-active-packet, and blocked-carryover.
- [x] Add CLI and JSON commands that create governance reviews without sending, deploying, building, or mutating external systems.
- [x] Add dashboard import/rendering for approval state, latest packet, stale/superseded/revoked warnings, acknowledgement gaps, and copyable governance notes.
- [x] Add focused Rust/TypeScript coverage for review scoring, stale packet detection, acknowledgement enforcement, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Handoff Dispatch Checklist` and convert approved governance reviews into a final local operator checklist before any external send.

## Completed Set: Friday Release Handoff Dispatch Checklist

Status: 100/100

Goal: turn approved handoff governance reviews into a final local dispatch checklist so Friday can verify recipients, attachments, notes, privacy boundaries, and no-send safeguards before any external communication.

### Completed To Reach 100/100

- [x] Add a typed handoff dispatch checklist model that consumes governance reviews.
- [x] Add checklist states for ready, held, missing-recipient, missing-attachment, privacy-review, and blocked.
- [x] Add CLI and JSON commands that generate dispatch checklists without sending, deploying, building, or mutating external systems.
- [x] Add dashboard import/rendering for recipients, attachments, privacy notes, no-send warnings, and copyable dispatch checklist.
- [x] Add focused Rust/TypeScript coverage for checklist readiness, privacy boundary enforcement, missing attachment detection, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Handoff Dispatch Audit` and preserve generated dispatch checklists plus operator final decisions as local history before any external send.

## Completed Set: Friday Release Handoff Dispatch Audit

Status: 100/100

Goal: preserve every generated release handoff dispatch checklist and operator decision as a local audit history before any external communication happens.

### Completed To Reach 100/100

- [x] Add a typed handoff dispatch audit model that consumes dispatch checklists.
- [x] Add audit states for draft, ready, held, approved, sent-manually, revoked, and blocked.
- [x] Add append, list, export, and JSON commands without sending, deploying, building, uploading, or mutating external systems.
- [x] Add dashboard import/rendering for dispatch history, latest checklist, operator final decision, blockers, and copyable audit summary.
- [x] Add focused Rust/TypeScript coverage for audit append/list behavior, final decision preservation, blocker carryover, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Handoff Dispatch Governance` and validate dispatch audit trails before any external handoff is considered complete.

## Completed Set: Friday Release Handoff Dispatch Governance

Status: 100/100

Goal: validate handoff dispatch audit trails so Friday can prove the latest dispatch decision is approved or sent manually, blockers are resolved, revoked decisions are inactive, and final decision notes are present.

### Completed To Reach 100/100

- [x] Add a typed handoff dispatch governance model that consumes dispatch audit trails.
- [x] Add governance states for approved, held, needs-final-decision, stale-checklist, revoked-active-decision, and blocked-carryover.
- [x] Add CLI and JSON commands that create governance reviews without sending, deploying, building, uploading, or mutating external systems.
- [x] Add dashboard import/rendering for latest dispatch decision, revoked/stale warnings, final decision gaps, blocker carryover, and copyable governance notes.
- [x] Add focused Rust/TypeScript coverage for governance scoring, revoked decision detection, final decision enforcement, blocker carryover, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Handoff Completion Ledger` and preserve dispatch governance approvals, manual-send confirmations, unresolved blockers, and final handoff outcomes as local history.

## Completed Set: Friday Release Handoff Completion Ledger

Status: 100/100

Goal: preserve governed handoff completion outcomes so Friday can show which external handoff was approved, manually sent, held, revoked, or blocked without performing any external send itself.

### Completed To Reach 100/100

- [x] Add a typed handoff completion ledger model that consumes dispatch governance reviews.
- [x] Add completion states for draft, completed, manually-sent, held, revoked, superseded, and blocked.
- [x] Add append, list, export, and JSON commands without sending, deploying, building, uploading, or mutating external systems.
- [x] Add dashboard import/rendering for completion history, latest governance review, operator outcome, blocker carryover, and copyable completion notes.
- [x] Add focused Rust/TypeScript coverage for completion append/list behavior, manual-send wording, revoked/superseded handling, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Publication Control` and keep final release notes, deployment notes, public announcements, and external send instructions local-only until an operator explicitly marks them ready.

## Completed Set: Friday Release Publication Control

Status: 100/100

Goal: turn completed handoff ledgers into local-only publication controls so Friday can prepare release notes, deployment notes, announcement copy, and external-send instructions without publishing, deploying, uploading, emailing, or mutating external systems.

### Completed To Reach 100/100

- [x] Add a typed publication-control model that consumes completion ledgers and marks publish readiness.
- [x] Add publication states for draft, ready, held, blocked, published-manually, revoked, and superseded.
- [x] Add CLI/JSON commands that generate publication controls and local copy without performing external publication.
- [x] Add dashboard import/rendering for latest completion, publication blockers, manual-publish wording, and copyable release notes.
- [x] Add focused Rust/TypeScript coverage for publication readiness, blocked ledgers, manual-publish safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Outbound Review` and make final operator review explicit before any copied release note, deployment note, announcement, or external-send instruction can be treated as ready.

## Completed Set: Friday Release Outbound Review

Status: 100/100

Goal: add a final local review surface for publication controls so Friday can distinguish drafted copy, operator-reviewed copy, held copy, and manually published outcomes without sending, publishing, deploying, uploading, or emailing.

### Completed To Reach 100/100

- [x] Add a typed outbound-review model that consumes publication controls and records final operator review decisions.
- [x] Add review states for draft, reviewed, changes-requested, held, blocked, manually-published, revoked, and superseded.
- [x] Add append/list/export/JSON commands for outbound reviews without performing external mutation.
- [x] Add dashboard import/rendering for review history, active publication control, copy safety, manual-publish references, and final operator notes.
- [x] Add focused Rust/TypeScript coverage for reviewed copy, blocked publication controls, manual-publish references, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release External Receipt Archive` and preserve human-owned external publication, send, deploy, upload, or announcement receipts as local evidence without fetching or mutating external systems.

## Completed Set: Friday Release External Receipt Archive

Status: 100/100

Goal: preserve human-owned external publication/send/deploy receipts as local evidence so Friday can connect outbound reviews to real-world operator outcomes without fetching, sending, deploying, uploading, emailing, or mutating external systems.

### Completed To Reach 100/100

- [x] Add a typed external receipt archive that consumes outbound review ledgers.
- [x] Add receipt states for draft, attached, verified, stale, missing, revoked, superseded, and blocked.
- [x] Add append/list/export/JSON commands that never fetch, send, deploy, upload, email, or mutate external systems.
- [x] Add dashboard import/rendering for receipt history, reviewed copy, manual references, evidence paths, and copyable audit notes.
- [x] Add focused Rust/TypeScript coverage for receipt attachability, blocked reviews, stale evidence, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Receipt Review Board` and consolidate outbound reviews, external receipts, blocker carryover, evidence freshness, and operator final decisions into one local review surface.

## Completed Set: Friday Release Receipt Review Board

Status: 100/100

Goal: consolidate outbound reviews and external receipt archives into a final local review board so Friday can distinguish verified release outcomes, stale evidence, missing receipts, blocked reviews, and operator carryover before any release is treated as externally complete.

### Completed To Reach 100/100

- [x] Add a typed receipt review board that consumes external receipt archives.
- [x] Add review decisions for verified, held, missing-receipt, stale-evidence, blocked-review, revoked-receipt, and carryover.
- [x] Add CLI/JSON commands that generate review boards without fetching, sending, deploying, uploading, emailing, or mutating external systems.
- [x] Add dashboard import/rendering for receipt decisions, active evidence, freshness warnings, blocker carryover, and copyable review notes.
- [x] Add focused Rust/TypeScript coverage for verified receipt decisions, stale/missing receipt handling, blocked outbound reviews, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Closure Ledger` and preserve reviewed receipt decisions, operator closure notes, carryover commitments, and final release outcome history without external mutation.

## Completed Set: Friday Release Closure Ledger

Status: 100/100

Goal: preserve receipt review board outcomes as a local closure history so Friday can show which releases are closed, held, blocked, carried over, revoked, or superseded without sending, deploying, uploading, emailing, or mutating external systems.

### Completed To Reach 100/100

- [x] Add a typed closure ledger that consumes release receipt review boards.
- [x] Add closure states for draft, closed, held, carryover, blocked, revoked, and superseded.
- [x] Add append/list/export/JSON commands that preserve closure records without external mutation.
- [x] Add dashboard import/rendering for closure history, active review board, operator closure notes, carryover, and copyable closure summaries.
- [x] Add focused Rust/TypeScript coverage for closed receipt decisions, blocked carryover, revoked/superseded records, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Continuity Journal` and turn closure ledgers into a searchable local continuity surface for next-release planning, carryover ownership, and historical release outcome review.

## Completed Set: Friday Release Continuity Journal

Status: 100/100

Goal: connect release closure ledgers into a local continuity journal so Friday can show historical release outcomes, recurring blockers, carryover commitments, and next-release planning notes without external mutation.

### Completed To Reach 100/100

- [x] Add a typed continuity journal that consumes release closure ledgers.
- [x] Add journal entry kinds for outcome, carryover, blocker-pattern, next-release-note, operator-decision, and superseded-history.
- [x] Add append/list/export/JSON commands that preserve continuity history without external mutation.
- [x] Add dashboard import/rendering for release outcome history, recurring blockers, carryover ownership, and copyable next-release notes.
- [x] Add focused Rust/TypeScript coverage for closure history ingestion, recurring blocker detection, carryover summaries, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Learning Register` and turn continuity journals into prevention lessons, decision patterns, and next-release commitments without external mutation.

## Completed Set: Friday Release Learning Register

Status: 100/100

Goal: convert release continuity journals into a local learning register so Friday can track repeated release lessons, prevention experiments, operator decisions, quality gates, and next-cycle commitments without changing external systems.

### Completed To Reach 100/100

- [x] Add a typed learning register that consumes release continuity journals.
- [x] Add learning categories for lesson, prevention-experiment, decision-pattern, quality-gate, owner-commitment, and retired-learning.
- [x] Add append/list/export/JSON commands that preserve learning history without external mutation.
- [x] Add dashboard import/rendering for release lessons, prevention experiments, decision patterns, quality gates, and next-cycle commitments.
- [x] Add focused Rust/TypeScript coverage for continuity ingestion, repeated lesson detection, owner commitments, command safety, and dashboard rendering.

### Completed Control Rule

This set is complete. The next loop should open `Friday Release Knowledge Index` and make release learning searchable, filterable, and comparable across local release cycles without external mutation.

## Next Set: Friday Release Knowledge Index

Status: 0/100

Goal: index release learning registers into a searchable local knowledge surface so Friday can compare lessons, owners, quality gates, blocker themes, and next-cycle commitments across release cycles without mutating external systems.

### Remaining To Reach 100/100

- [ ] Add a typed knowledge index that consumes release learning registers.
- [ ] Add index dimensions for lesson-theme, owner, quality-gate, blocker-theme, decision-pattern, and commitment-target.
- [ ] Add generate/list/export/JSON commands that preserve searchable learning history without external mutation.
- [ ] Add dashboard import/rendering for searchable lessons, owner filters, recurring blocker themes, quality gates, and commitment targets.
- [ ] Add focused Rust/TypeScript coverage for learning ingestion, repeated theme indexing, owner/gate filtering, command safety, and dashboard rendering.
