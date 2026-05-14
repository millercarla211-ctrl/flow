# Friday Product TODO

This file tracks the active 100% feature set loop for Friday. The score is about product capability, not how recently the code compiled.

## Active Set: Friday Reliable Workspace Core

Status: 30/100

Goal: make Friday's local-first assistant workspace dependable enough for daily use before starting the next major product layer.

### Must Have

- [x] Preserve WhisperFlow Beater as a separate feature area instead of removing voice work.
- [x] Keep local-first/provider boundaries explicit in the Connectors workspace.
- [x] Add workspace backup export, import, restore checkpoint, and sync status feedback.
- [x] Harden backup, sync, provider health, provider research, web search, and web inspection blank-error states.
- [x] Harden automation runner failures so scheduled tasks never save blank errors.
- [x] Harden local file export failures so downloads/importable files never report blank errors.
- [ ] Add visible reliability status on the Friday dashboard for local mode, cloud mode, sync, and voice readiness.
- [ ] Add a recovery center page or panel for restore checkpoints, failed automations, sync status, and local backup export.
- [ ] Add actionable empty states for Ask, Research, Agents, Projects, Memory, Connectors, Artifacts, Automations, and Voice.
- [ ] Add end-to-end smoke coverage for every local workspace create/edit/delete/export path.
- [ ] Add type-safe shared status/result helpers for all user-facing async operations.
- [ ] Add a local-only acceptance checklist that can be run without building the desktop app.

### Should Have

- [ ] Add research source quality labels and citation confidence.
- [ ] Add project-scoped memories to Ask responses.
- [ ] Add connector readiness badges to the Friday sidebar.
- [ ] Add automation run history export.
- [ ] Add web inspection result caching with clear invalidation.
- [ ] Add local files source picker improvements.
- [ ] Add a single canonical changelog file and archive the duplicate case variants safely.

### Not In This Set

- Full release build verification.
- Heavy lint/format/build loops after every small change.
- Reworking the working STT path unless a voice bug is directly reported.

## Next Set Candidates

- Friday Competitive Assistant Core: streaming Ask polish, model routing, tool calls, citations, project memory, and Canvas artifacts.
- WhisperFlow Beater Core: wake robustness, overlay reliability, STT/TTS latency, dictation recovery, and focused-field paste reliability.
- Local Model Ops: model inventory, health checks, benchmark history, disk-location policy, and safe model cleanup.
