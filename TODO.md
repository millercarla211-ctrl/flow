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

## Next Active Set: Friday Product UI Integration

Status: 85/100

Goal: turn the completed Friday runtime contracts into a polished product UI and verified live workflows without weakening the local-first, low-resource defaults.

### Remaining To Reach 100/100

- [x] Wire Ask/Search/Research to visible streaming UI, citations, saved reports, and source controls.
- [x] Wire Projects, Memory, Connectors, Canvas, Artifacts, Code, Voice, Multimodal, and Automations pages to the new stores.
- [x] Add end-to-end local execution checks for STT/TTS/OCR/metasearch/artifact preview flows.
- [x] Add production-ready empty/loading/error/permission states for every Friday route.
- [ ] Run targeted browser verification and only deploy after a major user-visible feature ships.

## Next Set Candidates

### Multimodal Local Core

- [ ] Finish local OCR/VLM/image/video runtime execution paths.
- [ ] Add conversion and validation workflows for community model artifacts.
- [ ] Add publish-ready local artifact metadata.
- [ ] Add low-end routing promotion/demotion checks for every model role.

### Browser And Extension Core

- [ ] Smoke test packaged browser extensions in real Chrome, Edge, Firefox, and Safari installs.
- [ ] Verify offline browser-pack reuse after first download.
- [ ] Verify partial-download resume, hash rejection, and quota recovery.
- [ ] Add optional Chromium WebLLM acceleration after Qwen browser packs are validated.
