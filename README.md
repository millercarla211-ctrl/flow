# Flow

Flow is a Rust-first, local-first AI runtime and input layer for `dx`.

The current goal is simple: make Flow the fastest way to use AI on low-end and high-end devices at the same time. It should feel like a system-wide dictation app, a grammar assistant, a local model runtime, and a safe device-control layer in one library.

Flow is being built as a reusable Rust crate first so it can later plug into desktop apps, mobile shells, Tauri, Flutter, browser/WASM surfaces, editor forks, and other host software without forcing one UI or one transport model.

## Current Direction

Flow is now focused on the standalone core. Adjacent sibling projects are intentionally decoupled from the active Flow surface until they are mature enough to reintroduce without adding instability.

The active product scope is:

- always-on local activation with wake word and keyboard shortcut entry
- low-latency local dictation and text rewriting
- grammar, clarity, tone, and proofing assistance
- low-end friendly local model routing and warm-cache behavior
- safe OS control plans so AI can operate the host with explicit permission gates
- library-first APIs that other Rust hosts can embed

## Product Targets

Flow is being designed to beat the combination of Wispr Flow and Grammarly while also adding local AI runtime control:

- system-wide voice entry with wake words, hold-to-dictate, and overlay access
- live grammar correction, rewriting, tone shaping, snippet expansion, and dictionary support
- command-style editing for email, docs, chat, coding, and notes
- file tagging and symbol-aware editor assistance
- 24/7 low-end runtime plans that keep only the cheapest services resident
- local model execution without internal HTTP by default
- AI-assisted OS control for text insertion, app launch, file navigation, URLs, windows, media, and other host actions behind safety policies
- future multimodal support for STT, TTS, OCR, VLM, image, and video workflows

## AI-First Local UI Runtime

Flow uses a local role policy on this Windows machine:

- `qwen3-0.6b` is the instant helper. Use it for prompt cleanup, text cleanup, tiny rewrites, text conversion, labels, and low-latency support tasks.
- `xlam2-3b-fc-r-q4km` is the dedicated local tool-agent research model. Use it for strict JSON tool-routing/function-call decisions; because it is CC-BY-NC-4.0, keep commercial defaults on Apache/MIT candidates such as Ministral or Granite.
- `qwen35-4b-revised-q4km` is the daily smart model. Use it for DX UI registry work, shadcn-style component edits, React/TypeScript/Tailwind prompts, Rust edits, normal useful answers, and general coding help.
- `qwen35-9b-q4km` is the slow backup when the 4B model fails and latency is acceptable.
- `gemma4-e4b-frontend-text-q4km` is no longer a daily quality-chat role on this OS. Keep Gemma for UI-generation, vision, and benchmark experiments only.

Fast local model role commands:

```powershell
cargo run --release --bin flow -- --model-roles
cargo run --release --bin flow -- --tool-model-candidates
cargo run --release --bin flow -- --install-model qwen35-4b-revised-q4km
cargo run --release --bin flow -- --install-model xlam2-3b-fc-r-q4km
cargo run --release --bin flow -- --install-model qwen3-0.6b
cargo run --release --bin flow -- --install-model qwen35-9b-q4km
cargo run --release --bin flow -- --models chat
cargo run --release --bin flow -- --plan chat qwen35-4b-revised-q4km
cargo run --release --bin flow -- --chat qwen35-4b-revised-q4km
cargo run --release --bin flow -- --tool-agent "choose a tool for this request"
cargo run --release --bin flow -- --tool-agent-tools examples/tool-agent/weather-tools.json "weather in Dhaka tomorrow"
cargo run --release --bin flow -- --chat qwen3-0.6b
```

Flow can run small UI-specialized GGUF models locally through Rust/llama.cpp. The default UI model is `Qwendean-4B` `Q4_K_M`, stored at `models/llm/Qwendean-4B.Q4_K_M.gguf`; `WEBGEN-4B-Preview` i1 is also installed as the website-generation candidate at `models/llm/WEBGEN-4B-Preview.i1-Q4_K_M.gguf`.

For screenshot cloning, use the separate vision path first. The best local visual candidate tested here is `Gemma-4-E4B-Frontend` with both `models/llm/gemma-4-E4B-it.Q4_K_M.gguf` and `models/llm/gemma-4-E4B-it.BF16-mmproj.gguf`, but the first Google clone eval failed the complete-HTML gate. Treat it as a wired local vision runtime, not a proven UI clone model.

Fast commands:

```powershell
cargo run --release --bin flow -- --ui-model-candidates
cargo run --release --bin flow -- --install-model gemma4-e4b-frontend-q4km
cargo run --release --bin flow -- --install-model webgen-4b-preview-i1-q4km
cargo run --release --bin flow -- --install-model qwendean-4b-q4km
cargo run --release --bin flow -- --models ui
cargo run --release --bin flow -- --models vlm
cargo run --release --bin flow -- --uigen tmp/uigen-output/index.html "make a shadcn-style Google homepage clone"
cargo run --release --bin flow -- --uigen-vision-google
cargo run --release --bin flow -- --uigen-google webgen-4b-preview-i1-q4km
cargo run --release --bin flow -- --uigen-google qwendean-4b-q4km
powershell -ExecutionPolicy Bypass -File scripts\uigen_vision_google_eval.ps1 -ForceGenerate
powershell -ExecutionPolicy Bypass -File scripts\uigen_google_eval.ps1 -ModelKey webgen-4b-preview-i1-q4km -ForceGenerate
powershell -ExecutionPolicy Bypass -File scripts\uigen_google_eval.ps1 -ModelKey qwendean-4b-q4km -ForceGenerate
```

`FLOW_LLAMA_GPU_LAYERS` can be set to opt into llama.cpp GPU offload. Leave it unset on low-VRAM Windows machines.

## Implemented Library Surfaces

The main product-facing module is [`src/experience`](./src/experience).

Key library APIs:

- `FlowLocalRuntime`
  - one embeddable local runtime for chat, STT, TTS, and transcript cleanup
  - uses broker-selected local models instead of HTTP wrappers
  - defaults low-end devices to `qwen3-0.6b` for text generation
  - lazy-loads Moonshine STT and Kokoro TTS on first use
  - exposes `generate_text`, `transcribe_file`, `clean_transcription`, `synthesize_text`, and `transcribe_clean_and_synthesize_to_file`
- `FlowLocalRuntimeSummary`
  - reports the selected local chat, STT, and TTS models for the current device
  - reports whether each selected model is present locally
- `ZedFlowAdapter`
  - wraps `FlowLocalRuntime` in shapes that map to Zed’s Agent Panel, Inline Assistant, edit prediction, and voice-input surfaces
  - exposes `agent_panel_reply`, `inline_assist`, `edit_prediction`, and `transcribe_voice_note`
  - maps Zed-style profiles (`Ask`, `Write`, `Minimal`) and tool-permission modes (`confirm`, `allow`, `deny`)
- `CodexFlowAdapter`
  - wraps `FlowLocalRuntime` in shapes that map to Codex-style local tasks, follow-ups, and PR review flows
  - exposes `run_task`, `follow_up`, and `review_pull_request`
  - maps Codex-style approval modes (`Suggest`, `Auto Edit`, `Full Auto`) and reasoning effort hints (`low`, `medium`, `high`, `xhigh`)
  - carries CLI, desktop, IDE, GitHub-review, browser-context, and background-task metadata without forcing HTTP or a sidecar daemon
- `ZeroClawFlowAdapter`
  - wraps `FlowLocalRuntime` in shapes that map to ZeroClaw-style agent, gateway, daemon, channel, and skill-runner flows
  - exposes `run_task` and `follow_up`
  - maps ZeroClaw-style autonomy levels (`readonly`, `supervised`, `full`) plus channel and tool-policy metadata
  - carries browser, terminal, memory, identity, and session context without forcing a local HTTP bridge
- `FlowProductionConfig`
  - provides one serializable production configuration surface for DX, browser, Zed, Codex, and ZeroClaw targets
  - carries low-end-safe defaults for local-only mode, warmup policy, candidate counts, context channels, and host-specific behavior
  - can be generated directly from the current machine through `DxFlowRuntime::production_config(...)`
- `FlowProductionBundleManifest`
  - captures the machine-specific handoff state for a production export
  - records selected models, missing local model paths, validated commands, browser artifacts, and exported config files
  - can be exported directly through `DxFlowRuntime::export_production_bundle(...)`
- `FlowReleaseSummary`
  - captures the repo-level release handoff state for client delivery
  - records production bundle files, browser release artifacts, validated commands, and external release tasks
  - can be exported through `DxFlowRuntime::export_release_summary(...)`
- `FlowTypingAssistant`
  - grammar correction
  - style rewrites
  - snippet expansion
  - dictionary normalization
  - spoken command rewrites
- `FlowDictationEngine`
  - filler cleanup
  - repeated-word cleanup
  - spoken formatting cleanup
  - numbered-list formatting
  - workspace-aware tagging hooks
- `FlowExperienceHub`
  - shared and personal dictionaries
  - shared and personal snippets
  - style presets
  - usage dashboard state
- `FlowActivationProfile`
  - wake commands
  - keyboard shortcuts
  - model lookup under `models/wake_words/`
  - debounce and idle policy
- `FlowAlwaysOnProfile`
  - low-end and balanced 24/7 runtime budgets
  - resident lanes
  - warm/cold model strategy
  - battery and thermal backoff guards
- `FlowControlPolicy`
  - cross-surface control capabilities
  - confirmation and consent gates
  - action planning for text insert, URL open, shortcut send, app launch, and shell-command consent
- `FlowControlAuditLog`
  - approval scopes
  - action history
  - host-facing audit trail for device control
- `HostAdapterDescriptor` and `FlowHostControlAdapter`
  - clean adapter boundary for desktop, mobile, browser, and editor hosts
- `FlowProofingPlanner`
  - grammar, clarity, tone, citation, fact-check, and plagiarism-screen planning
- `FlowEditorAssistPlanner`
  - variable recognition
  - file tagging
  - command-mode editor hints
- `FlowCommandRouter`
  - routes spoken or typed commands into safe host action plans
- `FlowModuleBootstrapper`
  - builds OS-aware automatic base-module install plans
  - keeps low-end devices on green-tier modules only
  - defers heavier multimodal modules until benchmarking promotes the device
- `FlowInstallerFacade`
  - tracks first-run install state
  - manages promotion and demotion between device tiers
- `FlowRuntimeTierPolicy`
  - interprets RAM, VRAM, latency, battery, and thermal signals
  - upgrades or downgrades the active Flow tier
- `FlowOnboardingBuilder`
  - creates first-run permission and setup steps per operating system
- `FlowPermissionPlanner`
  - defines the required and optional permission set per host platform
- `FlowAudioPlanner`
  - chooses the wake and dictation audio pipeline for the current OS and device tier
- `FlowOverlayController`
  - manages compact, dictation, command, rewrite, and proofing overlay states
- `FlowPersistentState`
  - captures installed modules, approvals, and benchmark history for reload
- `FlowSessionRuntime`
  - ties activation, installs, typing, proofing, commands, and control together
- `FlowEngine`
  - host-facing orchestration API for bootstrap, typing, commands, and runtime refresh
- `FlowHostBundle`
  - packages engine, permissions, onboarding, audio, overlay, and recovery into one embed surface
  - can bootstrap a host and advance lifecycle or recovery flows directly
  - can sync overlay/audio presenters and create a native executor for the current OS
  - can run selection rewrite and shortcut automation through a dedicated automation bridge
  - can create a clipboard-backed native automation bridge for desktop fallback behavior
- `FlowLifecycleController`
  - explicit state machine for cold boot, listening, overlay, dictation, command mode, timeout, pause, and resume
- host contracts
  - `FlowModuleInstaller`
  - `FlowStateStore`
  - `FlowPermissionGate`
  - `FlowControlExecutor`
  - `FlowOverlayPresenter`
  - `FlowAudioRuntime`
  - `FlowAutomationBridge`
  - plus in-memory and recording reference implementations
- `NativeControlExecutor`
  - provides OS-aware native execution for supported URL, app, file, search, notification, and basic media actions
- `ClipboardAutomationBridge`
  - provides a clipboard-backed native fallback for selection replacement and shortcut dispatch on desktop hosts
- `FlowFileStateStore`
  - persists module state, approvals, and benchmark history to disk without requiring host-specific serialization first
- `NativeOverlayPresenter` and `ManagedAudioRuntime`
  - provide concrete default presenter/runtime adapters for the default host kit
- `ManagedWakeRuntime` and `FlowHealthReport`
  - keep the wake runtime armed/listening in sync with lifecycle and report whether the embedded host is actually ready
- `FlowConsentPlanner`
  - builds the first-run approval plan for live native execution
- `ManagedMicrophoneService`, `NativeSelectionBridge`, and `FlowRuntimeSupervisor`
  - provide a managed microphone lifecycle, stronger native selection automation, and an operational supervisor loop for the embedded host runtime
- `CpalCaptureWorker` and `FlowAccessibilityRuntime`
  - provide a low-level capture worker path with per-frame RMS/speech/clipping reporting and an explicit probed accessibility/runtime mode for desktop automation
- `OpenWakeInferenceWorker`
  - validates wake detections against configured aliases and can also use the real local ONNX wake-word detector path when local wake models are present
- `FlowEmbeddedHost`
  - wraps the default host kit, supervisor, and session context into a single operational runtime object for embedders
  - can also ingest live PCM frames to drive wake detection without going through the old phrase-only path
- `FlowDefaultHostKit`
  - bundles the default store, permission gate, executor, presenters, automation bridge, and Flow host bundle into one ready-to-embed integration surface
  - keeps wake runtime state synced and can emit a host readiness report
  - supports both dry-run and live host construction paths
  - can emit a first-run consent plan for live native execution
- `FlowProductSurface`
  - packaged low-end desktop, balanced desktop, and mobile-oriented Flow profiles
  - can also choose a baseline profile from host OS and hardware class

## Activation

Flow currently treats activation as a first-class product surface.

- wake commands: `dx`, `friday`, `hello`, `aladdin`, `arise`
- keyboard shortcuts:
  - `Ctrl+Alt+Space` for toggle
  - `Ctrl+Shift+Space` for hold-to-dictate
  - `Alt+Backquote` for a quick overlay
- wake models are expected under `models/wake_words/{dx,friday,hello,aladdin,arise}.onnx`
- custom wake-word training templates live under `configs/wakewords/`; see `docs/WAKEWORD_TRAINING.md`

## 24/7 Local Runtime Strategy

Flow is optimized for bad hardware first.

Low-end 24/7 defaults:

- `qwen3-0.6b` for fast local rewrites
- `moonshine-tiny` as the smallest on-demand speech recognition fallback
- `parakeet-tdt-0.6b-v3-int8` and `nemotron-speech-streaming-en-0.6b-int8` as optional sherpa-onnx STT upgrades when local artifacts exist
- `kokoro-onnx-int8` for lightweight voice confirmation
- tight RAM budgets
- aggressive unload-on-idle
- always-hot wake/VAD lane with STT loaded on demand

Low-end module bootstrap defaults:

- install green-tier modules only
- install OS-specific host bridges automatically on first run
- defer balanced, creator, and workstation modules until the machine proves it can sustain them
- keep a lightweight `green-3d-assist-preview` module available instead of heavier creative stacks

Balanced desktop defaults:

- `smollm3-3b` for better rewrites and command mode
- `parakeet-tdt-0.6b-v3-int8` for higher-quality local STT when installed
- `gemma-4-e2b` for richer overlay help
- wider context and cache budgets

## OS Control

Flow is not only an input layer. It is also becoming a safe AI control layer for the host device.

The current policy model supports:

- clipboard reads and writes
- reading and replacing the active selection
- sending shortcuts
- opening URLs
- launching applications
- opening and revealing files
- creating draft files
- focusing windows
- media playback
- volume and brightness changes
- system search
- notifications

Anything shell-like is treated as explicit-consent only.

Flow also now includes an approval and audit surface so host apps can remember granted capabilities and record every control action.

## Platform Direction

Flow is being built so the same core can be embedded into:

- Windows, macOS, Linux
- Android and iOS host shells
- Tauri and Flutter apps
- browser/WASM entry points where feasible
- editor integrations
- VPS and headless agents
- Raspberry Pi and other edge devices
- future tablet, TV, watch, and other constrained form factors

Flow now also includes an OS-aware base-module bootstrap layer. On first run, the host can select Windows, macOS, Linux, Android, iOS, browser/WASM, or server install plans and automatically provision the right base modules for that surface.

After first run, Flow can keep an install-state registry and promote or demote the active module set based on measured latency, throughput, battery, and thermal behavior.

The core also now has explicit onboarding, permissions, overlay, audio, recovery, and persistence surfaces so hosts can guide users through setup once and then reload module, approval, benchmark, and UX state later.

The newest layer is a host-facing engine API. A desktop shell, mobile container, editor embed, or future DX host can now bootstrap Flow, restore persisted state, process text, route commands, and refresh runtime tiers through one orchestration surface instead of stitching every subsystem together manually.

Flow also now has a runtime lifecycle state machine so hosts can reason about always-on behavior directly instead of inferring it from wake-word and overlay policies. Session lifecycle changes can now drive overlay state directly.

## Editor Integration Surface

Flow now includes a direct embeddable local-runtime API intended for hosts like your Zed fork:

- `FlowLocalRuntime::detect()`
  - builds a device-aware local runtime using the broker and the current machine profile
- `FlowLocalRuntime::for_device_profile(profile)`
  - lets another host supply its own detected hardware profile
- `FlowLocalRuntime::generate_text(prompt)`
  - runs local text generation through the selected local chat model
- `FlowLocalRuntime::transcribe_file(path)`
  - runs local STT through Moonshine
- `FlowLocalRuntime::clean_transcription(raw)`
  - runs local transcript cleanup through the selected local chat model
- `FlowLocalRuntime::synthesize_text(text)`
  - runs local TTS through Kokoro
- `FlowLocalRuntime::transcribe_clean_and_synthesize_to_file(input, output)`
  - runs the full local speech pipeline in one call

`DxFlowRuntime` also now exposes:

- `create_local_runtime()`
- `local_runtime_summary()`

This is the intended integration path for editor hosts that want direct local inference without routing through Ollama-style HTTP daemons.

## Zed Compatibility Surface

Flow now also includes a Zed-focused adapter layer under [`src/zed`](./src/zed):

- `ZedFlowAdapter::detect()`
  - creates a Zed-oriented adapter on top of the broker-selected local runtime
- `ZedFlowAdapter::local_model_status()`
  - reports whether local chat, STT, and TTS are ready for editor use
- `ZedFlowAdapter::agent_panel_reply(...)`
  - shapes Flow output for a Zed Agent Panel-style thread request
- `ZedFlowAdapter::inline_assist(...)`
  - shapes Flow output for a Zed Inline Assistant-style selection rewrite
- `ZedFlowAdapter::edit_prediction(...)`
  - shapes Flow output for a Zed-style local edit prediction request
- `ZedFlowAdapter::transcribe_voice_note(...)`
  - turns a recorded audio file into editor-ready cleaned text

The current compatibility target is the latest documented Zed nightly feature set around:

- Agent Panel profiles
- Inline Assistant transformations
- pluggable edit prediction providers
- external-agent style permission mapping

For a forked/native Zed integration, this direct Rust adapter is the recommended path before adding a separate ACP server process.

## Codex Compatibility Surface

Flow now also includes a Codex-focused adapter layer under [`src/codex`](./src/codex):

- `CodexFlowAdapter::detect()`
  - creates a Codex-oriented adapter on top of the broker-selected local runtime
- `CodexFlowAdapter::local_model_status()`
  - reports whether local chat is ready for Codex-style CLI, IDE, review, and follow-up work
- `CodexFlowAdapter::run_task(...)`
  - shapes Flow output for Codex-like local tasks across CLI, desktop, IDE, browser, and background surfaces
- `CodexFlowAdapter::follow_up(...)`
  - carries prior answer state plus fresh diff and terminal summaries into a follow-up pass
- `CodexFlowAdapter::review_pull_request(...)`
  - shapes Flow output for Codex-style PR review with a summary, parsed findings, and suggested tests

The current compatibility target is the latest official Codex feature direction as of **April 27, 2026**, including:

- suggest / auto-edit / full-auto approval semantics
- IDE, desktop, CLI, GitHub-review, and background-task surface metadata
- local best-of-N style multi-candidate responses
- browser and terminal context threading
- production-style PR review formatting

For a forked/native Codex integration, this direct Rust adapter is the recommended path before adding any external protocol or remote orchestration layer.

## ZeroClaw Compatibility Surface

Flow now also includes a ZeroClaw-focused adapter layer under [`src/zeroclaw`](./src/zeroclaw):

- `ZeroClawFlowAdapter::detect()`
  - creates a ZeroClaw-oriented adapter on top of the broker-selected local runtime
- `ZeroClawFlowAdapter::local_model_status()`
  - reports whether local chat is ready for agent CLI, gateway, daemon, channel, and skill-runner work
- `ZeroClawFlowAdapter::run_task(...)`
  - shapes Flow output for ZeroClaw-like local tasks across CLI, gateway, daemon, browser, and channel surfaces
- `ZeroClawFlowAdapter::follow_up(...)`
  - carries prior answer state plus fresh memory, browser, and terminal summaries into a follow-up pass

The current compatibility target is the latest public ZeroClaw feature direction as of **April 27, 2026**, including:

- readonly / supervised / full autonomy semantics
- agent, gateway, daemon, and multi-channel surface metadata
- browser, terminal, memory, and session context threading
- skill-runner style workspace guidance
- local best-of-N style multi-candidate responses

For a forked/native ZeroClaw integration, this direct Rust adapter is the recommended path before adding any external gateway or transport layer.

## Browser Extension

Flow now has a browser-first local inference stack for Chromium, Firefox, and Safari-class WebExtension hosts.

The browser implementation is split across four layers:

- [`src/browser`](./src/browser)
  - browser capability types
  - browser execution planning
  - browser-pack catalog metadata
- [`crates/flow-browser-core`](./crates/flow-browser-core)
  - Rust/WASM orchestration surface for capability detection and plan generation
- [`extensions/flow-webext`](./extensions/flow-webext)
  - shared WebExtension shell
  - Chromium MV3 background + side panel
  - Firefox background + sidebar
  - Safari Web Extension packaging assets from the same source tree
- [`browserpacks`](./browserpacks)
  - browser-ready manifest format for ONNX and future MLC packs

Current browser-local default packs:

- `onnx-community/Qwen3-0.6B-DQ-ONNX`
- `Xenova/trocr-small-printed`
- `onnx-community/Qwen3.5-0.8B-ONNX`

Current browser-local features:

- local text rewrite, summarize, compose, and explain flows
- local OCR for screenshots and dropped images
- WebGPU-gated multimodal image/document questioning on capable browsers
- local-only mode with no hidden inference API fallback
- distinct popup, side panel, sidebar, options, and in-page overlay surfaces
- quick-action overview, full workbench, model-pack management, settings, and delivery/handoff screens
- persisted browser-side settings and workbench drafts for local-only behavior, context capture, and preferred pack selection
- apply-back, clipboard copy, context refresh, and page overlay controls for real user flows
- OPFS-first pack storage with IndexedDB fallback
- extension-storage fallback when IndexedDB is unavailable
- pack-state verification so cached browser packs are checked file-by-file instead of trusting manifest presence alone
- release packaging into per-browser zip artifacts with SHA-256 checksum files
- on-demand browser pack download instead of shipping giant weights inside the extension bundle

Validated integration commands run on **April 27, 2026**:

- `cargo check`
- `cargo test`
- `cargo build`
- `cargo check -p flow-browser-core`
- `cargo check --features example-binaries --examples`
- `npm install` in [`extensions/flow-webext`](./extensions/flow-webext)
- `npm run typecheck` in [`extensions/flow-webext`](./extensions/flow-webext)
- `npm run build:all` in [`extensions/flow-webext`](./extensions/flow-webext)
- `npm run package:all` in [`extensions/flow-webext`](./extensions/flow-webext)

## Production Configuration

Flow now has a unified production-configuration surface under [`src/config`](./src/config).

Use `DxFlowRuntime::production_config(...)` or `DxFlowRuntime::production_config_json(...)` to generate
host-targeted production defaults for:

- `dx-desktop`
- `browser-extension`
- `zed-fork`
- `codex-fork`
- `zeroclaw-fork`

The CLI now also exposes:

- `cargo run --bin flow -- --production-config dx-desktop`
- `cargo run --bin flow -- --production-config browser-extension`
- `cargo run --bin flow -- --production-config zed-fork`
- `cargo run --bin flow -- --production-config codex-fork`
- `cargo run --bin flow -- --production-config zeroclaw-fork`
- `cargo run --bin flow -- --export-production-bundle configs/production`
- `cargo run --bin flow -- --release-summary`
- `cargo run --bin flow -- --export-release-summary release`

These generated configs are intentionally low-end-safe on this machine:

- local-only mode stays enabled by default
- `qwen3-0.6b` remains the default text model
- candidate counts stay conservative on weak hardware
- Codex defaults to `suggest` mode on low-end devices
- ZeroClaw defaults to `readonly` autonomy on low-end devices
- browser packs default to `qwen3-0.6b` required plus OCR and multimodal optional

Flow can now also export a full delivery bundle under `configs/production` or any chosen directory:

- one JSON config per supported target
- `manifest.json` with selected local models, readiness, validated command matrix, and browser artifact paths
- `README.txt` with a compact operator handoff summary

Flow can also export a higher-level release handoff under `release/`:

- `flow-release-summary.json`
- `FLOW_RELEASE_HANDOFF.md`

That release summary tracks browser artifact presence, production bundle presence, validated commands, and the remaining external tasks such as Firebase wiring and browser-store publication.

The repository now also includes a root CI workflow in [`.github/workflows/ci.yml`](./.github/workflows/ci.yml)
that validates the Rust crate and the browser extension with the same commands used locally for release readiness.

## Low-End Validation Defaults

Flow now ships with validation defaults aimed at weak Windows machines as well as stronger developer workstations:

- Cargo uses `jobs = 1` through [`.cargo/config.toml`](./.cargo/config.toml)
- `dev` and `test` profiles disable incremental compilation to avoid multi-gigabyte stale caches
- example binaries are opt-in through the `example-binaries` feature so default `cargo test` stays focused on the real crate/test surface

This was done after the repository hit real `link.exe` memory limits, paging-file pressure, and disk-space pressure during validation on this machine.

## Competitive Status

The last documented scorecard is in [`docs/COMPETITIVE_SCORECARD.md`](./docs/COMPETITIVE_SCORECARD.md).

Last validated competitive baseline:

- overall Flow completion: `51/100`
- Wispr replacement readiness: `52/100`
- Grammarly replacement readiness: `40/100`
- Flow-native runtime advantage: `57/100`

Verified repository-scope release snapshot after the latest validation pass:

- current repo scope readiness: `100/100`
- release validation complete: `yes`
- browser extension packaging complete: `yes`

This `100/100` score applies to the implemented scope inside this repository today. It does not mean every future platform ambition is already delivered.

Newly added but not yet re-scored:

- OS-aware automatic module bootstrap
- first-run install state and tier transitions
- unified Flow session orchestration
- approval and audit-aware host command routing

## Roadmap

The historical completion plan is documented in [`docs/FLOW_100_PLAN.md`](./docs/FLOW_100_PLAN.md). Ongoing expansion work is tracked in [`TODO.md`](./TODO.md).

The biggest next-wave expansion areas are:

- polished host integrations and real end-to-end system wiring
- production-grade wake detection and hotkey adapters across platforms
- deeper Grammarly-class proofing, citation, and academic checks
- richer editor intelligence and variable recognition quality
- completed multimodal local runtime execution and conversion flows
- validated low-end soak testing

Operational release docs now live in:

- [`docs/BROWSER_RELEASE.md`](./docs/BROWSER_RELEASE.md)
- [`docs/CLIENT_HANDOFF.md`](./docs/CLIENT_HANDOFF.md)

## Philosophy

- local first
- library first
- low-end first
- no internal HTTP by default
- safe host control with auditable permissions
- strong fallbacks instead of silent failure

## Status

Flow is production-ready for the current repository scope and still under active expansion for future platform depth, competitive uplift, and broader host integrations.
