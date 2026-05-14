# Flow TODO

Flow is not finished. Treat this file as the active product loop, not as a parking lot. When a set reaches 100/100, open the next set and keep moving.

## Active Set: Completion Control Loop

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

## Next Set Candidates

### Host Autonomy Core

- [ ] Harden Windows global dictation host behavior beyond CLI/demo paths.
- [ ] Add production-grade desktop accessibility diagnostics.
- [ ] Add reliable pause/resume/snooze controls for always-on runtime hosts.
- [ ] Add host-level audit logs for native automation and text replacement actions.

### Writing Quality Core

- [ ] Deepen grammar, clarity, and rewrite explanations beyond the current Harper-backed baseline.
- [ ] Add citation, fact-checking, and academic assistance paths.
- [ ] Add style-guide and brand-tone policy enforcement.
- [ ] Add multilingual writing assistance.

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
