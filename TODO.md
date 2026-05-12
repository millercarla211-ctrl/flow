# Flow TODO

The current repository scope is validated and release-ready. The items below are next-wave expansion work, not blockers for the present build, test, or packaging baseline.

## Host Expansion

- add deeper Windows native automation and accessibility adapters
- add deeper macOS native automation and accessibility adapters
- add deeper Linux native automation and accessibility adapters
- add Android host-bridge implementations
- add iOS host-bridge implementations
- add richer Tauri embed helpers
- add richer Flutter embed helpers

## Product Uplift

- improve global desktop dictation polish
- improve global desktop rewrite overlays
- deepen command mode for editing, apps, navigation, and search
- improve live grammar and rewrite-on-paste behavior
- improve cross-app snippets and personal dictionary sync
- deepen privacy controls, local-only mode, and audit-log UX

## Competitive Uplift

- improve Wispr-grade variable recognition quality
- improve Wispr-grade polished file tagging across editors and docs
- improve Grammarly-grade academic assistance
- improve Grammarly-grade citation and source workflows
- improve Grammarly-grade plagiarism review paths
- improve tone and clarity coaching quality

## Multimodal Runtime

- deepen STT, TTS, OCR, VLM, image, and video runtime adapters
- deepen low-end routing defaults and promotion logic
- deepen model conversion and validation flows
- deepen publish-ready local artifact metadata

## Browser Extension

- run manual smoke tests in real Chrome, Edge, Firefox, and Safari installs against the packaged extension artifacts
- verify offline reuse after first browser-pack download on each target browser
- verify partial-download resume, hash rejection, and quota-recovery flows on each target browser
- add optional Chromium-only WebLLM acceleration once the Qwen browser packs are validated there

## Release Follow-Up

- refresh the competitive scorecard after the next product-quality pass
- keep checked-in `configs/production` exports refreshed when production defaults change
- keep checked-in `release/` handoff exports refreshed when release metadata changes
