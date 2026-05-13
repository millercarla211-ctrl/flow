<div align="center">
  <h1>Friday</h1>
  <p>Local-first AI workspace for Windows and macOS, with unlimited private voice built in.</p>
  <img src="./public/brand/flow-mark.svg" width="128" height="128" alt="Friday logo" />
</div>

---

Friday is a Tauri desktop app for local-first AI work: assistant chat, research shells, agent planning, canvas/artifact spaces, model routing, and the existing private dictation stack. The product direction is simple: keep core work local and unlimited, then add cloud providers only as explicit optional fallbacks.

## Current Focus

- Friday assistant workspace with Ask, Research, Agents, Canvas, Projects, Memory, Connectors, Artifacts, and Automations surfaces.
- WhisperFlow Beater voice area with warmed STT models.
- G-drive-first model/data storage on Windows lab machines.
- Local ASR model choices including Parakeet, Nemotron, and Whisper-family models.
- A compact always-on recording overlay inspired by modern voice tools.
- A Vercel-like black/white/zinc UI system with reusable Friday brand assets.

## Features

- **Ask Friday.** Streaming AI SDK chat shell with local model routing and optional Gateway models.
- **Research workspace.** Plan-first cited research UI with source controls and export-ready report structure.
- **Local transcription.** Core dictation runs on-device by default in the Voice area.
- **Model manager.** Download and validate local STT models from the app.
- **Dictionary and replacements.** Teach Friday names, jargon, acronyms, and direct replacements.
- **Library transcription.** Import audio/video, transcribe, retry, export, and keep transcripts organized.
- **Optional AI cleanup.** Route cleanup/edit tasks through configured providers when enabled.
- **Privacy-first storage.** Transcripts, recordings, model files, and API keys stay under local app storage unless a feature explicitly sends text to a provider.
- **Desktop overlay.** A small always-visible pill expands into recording controls only when needed.

## Development

### Prerequisites

- Windows 10/11 or macOS
- [Bun](https://bun.sh/) 1.3+
- [Rust](https://rustup.rs/) with the MSVC toolchain on Windows
- Microsoft Edge WebView2 Runtime on Windows

### Install

```powershell
bun install
```

### Run

```powershell
bun run tauri dev
```

### Verify

```powershell
bun run format
bun run lint
bun run typecheck
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

On Windows, the Tauri wrapper stores Cargo build artifacts in `C:\.flow-cargo-target` unless `CARGO_TARGET_DIR` or `FLOW_CARGO_TARGET_DIR` is set.

## Project Layout

- `src-tauri/`: Rust backend, native windows, hotkeys, audio capture, model management, storage, tray/menu, and transcription orchestration.
- `src/`: React/TypeScript frontend for the settings hub, overlay, onboarding, library, dictionary, and shared UI.
- `src/assets/brand/` and `public/brand/`: reusable Friday SVG marks.
- `src/components/ui/`: shadcn/ElevenLabs-inspired UI primitives used by the app.

## Privacy

Friday is local-first. Audio, transcripts, prompts, model files, and API keys remain on the device by default. Optional provider features should be treated as explicit network boundaries: when enabled, only the text needed for that operation is sent to the configured provider.

## License

Friday is based on an AGPL-3.0 desktop dictation codebase and remains AGPL-3.0. Distributions or network services based on this code must provide the corresponding source code under the same license.
