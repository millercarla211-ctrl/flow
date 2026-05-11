<div align="center">
  <h1>Flow</h1>
  <p>Local-first voice dictation for Windows and macOS, built for unlimited private speech-to-text.</p>
  <img src="./public/brand/flow-mark.svg" width="128" height="128" alt="Flow logo" />
</div>

---

Flow is a Tauri desktop app for private dictation, local model management, audio/video transcription, dictionary support, and optional AI cleanup. The product direction is simple: keep the core speech loop local and unlimited, then add cloud providers only as explicit optional fallbacks.

## Current Focus

- Fast local dictation with warmed STT models.
- G-drive-first model/data storage on Windows lab machines.
- Local ASR model choices including Parakeet, Nemotron, and Whisper-family models.
- A compact always-on recording overlay inspired by modern voice tools.
- A Vercel-like black/white/zinc UI system with reusable Flow brand assets.

## Features

- **Local transcription.** Core dictation runs on-device by default.
- **Model manager.** Download and validate local STT models from the app.
- **Dictionary and replacements.** Teach Flow names, jargon, acronyms, and direct replacements.
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
- `src/assets/brand/` and `public/brand/`: reusable Flow SVG marks.
- `src/components/ui/`: shadcn/ElevenLabs-inspired UI primitives used by the app.

## Privacy

Flow is local-first. Audio, transcripts, prompts, model files, and API keys remain on the device by default. Optional provider features should be treated as explicit network boundaries: when enabled, only the text needed for that operation is sent to the configured provider.

## License

Flow is based on an AGPL-3.0 desktop dictation codebase and remains AGPL-3.0. Distributions or network services based on this code must provide the corresponding source code under the same license.
