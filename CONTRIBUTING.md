# Contributing to Flow

Flow is a local-first desktop dictation app. Contributions should protect the core loop: capture audio quickly, transcribe locally when possible, paste reliably, and keep user data private.

## Principles

- Keep the product local/unlimited first.
- Preserve native desktop behavior and accessibility.
- Prefer existing owners and patterns over new layers.
- Do not log transcripts, audio, prompts, API keys, or sensitive paths.
- Keep model artifacts out of git.

## Development Setup

```powershell
bun install
bun run tauri dev
```

On Windows, set these environment variables when working from the lab machine if you want all heavy artifacts on G drive:

```powershell
$env:FLOW_DATA_DIR = "G:\Workspaces\flow-stt-lab\appdata\Flow\data"
$env:FLOW_CONFIG_DIR = "G:\Workspaces\flow-stt-lab\appdata\Flow\config"
$env:FLOW_CARGO_TARGET_DIR = "G:\Workspaces\flow-stt-lab\target\flow"
$env:CARGO_TARGET_DIR = $env:FLOW_CARGO_TARGET_DIR
```

## Checks

Run these before handing off a change:

```powershell
bun run format
bun run lint
bun run typecheck
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

## Pull Request Expectations

- Explain the user-facing change.
- List affected runtime surfaces: overlay, settings, tray, model manager, library, or storage.
- Include the verification commands you ran.
- Call out any model files, generated assets, or migrations that are intentionally excluded from git.
