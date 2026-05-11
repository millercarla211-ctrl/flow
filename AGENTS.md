Flow is a Tauri desktop app with a Rust backend and a React/TypeScript frontend. It is a multi-window desktop app, not a routed SPA.

## What Matters

- Keep invoke -> record -> transcribe -> insert low-latency.
- Keep local transcription and local storage as the default path.
- Keep Windows lab data/model artifacts on G drive when scripts are involved.
- Preserve native window, shortcut, permission, focus, overlay, tray, and menu behavior.
- Extend existing owners instead of adding duplicate service layers.

## Runtime Surfaces

- `main`: always-on pill overlay window.
- `toast`: transient toast window.
- `settings`: main settings/history/library window.

Window labels and behavior must stay aligned across:

- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/*.json`
- `src-tauri/src/lib.rs`
- `src-tauri/src/platform/**/*`
- `src/app/App.tsx`

## Backend Ownership

- `src-tauri/src/lib.rs`: composition root, `AppState`, plugin setup, command registration, startup loops.
- `src-tauri/src/pill.rs`: shortcut-driven recording lifecycle, overlay state, selected-text capture, permission warnings, media pause/resume.
- `src-tauri/src/recorder.rs`: recorder thread, audio preprocessing, validation inputs, WAV persistence helpers.
- `src-tauri/src/transcribe.rs`: transcription orchestration, chunking/dedupe, completion/error events, storage writes.
- `src-tauri/src/local_transcription.rs`: loaded ASR engine lifecycle. Do not duplicate warm/load/unload logic elsewhere.
- `src-tauri/src/model_manager.rs` and `src-tauri/src/downloader.rs`: model catalog, artifact layout, install state, downloads.
- `src-tauri/src/assistive.rs`: text insertion and selected-text access.
- `src-tauri/src/mode_context.rs` and `src-tauri/src/accessibility_context.rs`: active app/site context for edit and personalization behavior.
- `src-tauri/src/llm_cleanup.rs`: optional second-stage LLM cleanup/edit, provider routing, preflight cache.
- `src-tauri/src/settings.rs`: settings schema and persistence in `settings.db`.
- `src-tauri/src/core/settings.rs`: settings validation and post-save side effects.
- `src-tauri/src/storage.rs`: transcription/history storage and DB migrations in `transcriptions.db`.
- `src-tauri/src/library/*`: library SQL, filesystem/import/export, queueing, and Tauri commands.
- `src-tauri/src/dictionary.rs`: dictionary and replacements domain logic.
- `src-tauri/src/personalization.rs`: personalities plus app/site icon discovery.
- `src-tauri/src/toast.rs`: toast payloads and toast window positioning.
- `src-tauri/src/tray.rs` and `src-tauri/src/platform/macos/menu.rs`: tray/app menu ownership and settings-window accessory behavior.
- `src-tauri/src/update_checker.rs`: background checks, idle gating, download/install flow, restart marker.
- `src-tauri/src/crypto.rs`: API-key encryption/decryption.

## Frontend Ownership

- `src/app/App.tsx`: routes by Tauri window label, not URL.
- `src/Home.tsx`: settings-window shell and sidebar navigation.
- `src/features/settings/useSettingsForm.ts`: editable settings modal state and autosave.
- `src/features/onboarding/*`: onboarding step flow and initial settings write.
- `src/features/pill/*`: overlay rendering and frontend state machine.
- `src/features/toast/ToastOverlay.tsx`: toast rendering and actions.
- `src/features/transcriptions/*`: transcription history UI.
- `src/features/dictionary/components/DictionaryView.tsx`: dictionary and replacements UI.
- `src/features/personalization/*`: personalization UI.
- `src/features/library/*`: library frontend owners.
- `src/shared/lib/*`: static metadata/formatting only.
- `src/shared/ui/*`: small reusable UI primitives.
- `src/types/*`: shared frontend type layer.

## Change Map

- If you change a persisted setting, update Rust settings, validation, settings form state, and onboarding if first-run behavior changes.
- If you change mode/model/microphone behavior, keep tray and macOS app menu behavior in sync.
- If you change transcription payloads or events, update Rust emitters, frontend consumers, and `src/types/*` together.
- If you change permissions or plugin access, update Tauri config, capabilities, plist, and entitlements.
- If you change library storage or status semantics, keep storage, repo, queue, processing, and frontend queries aligned.
- If you change overlay/toast/settings window behavior, keep Rust window config, native platform code, and frontend label-based routing aligned.

## Validation

Run the relevant targeted check plus the baseline commands before handoff:

```powershell
bun run format
bun run lint
bun run typecheck
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```
