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

## Active Set: Friday Dashboard Product UI Wiring

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

## Next Set: Friday Dashboard Visible UI Execution

Status: 0/100

Goal: wire the visible Friday dashboard to render the typed dashboard binding and execute local-only actions from the UI layer.

### Remaining To Reach 100/100

- [ ] Render dashboard cards from the product UI binding in `extensions/flow-webext/src/ui/app.ts`.
- [ ] Add visible disabled/loading/success/error button states for dashboard actions.
- [ ] Render export history, release-review links, and screenshot prompts in the visible dashboard.
- [ ] Add a small TypeScript smoke/typecheck path for dashboard section rendering.
- [ ] Keep local-only fallback behavior and remove any dummy product copy from this dashboard surface.
