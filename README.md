<div align="center">
  <h1>Glimpse</h1>
  <p>Voice dictation that stays on your device. Free, open-source, and built for people who actually care about privacy.</p>
  <img
    src="https://github.com/user-attachments/assets/c34a35a5-e2c9-469f-87c4-4c0d20c8082d"
    width="256"
    height="256"
    alt="Glimpse"
  />
  <p>
    <a href="https://github.com/LegendarySpy/Glimpse/releases/latest">Download</a> ·
    <a href="#roadmap">Roadmap</a> ·
    <a href="#privacy">Privacy</a>
  </p>
  <p><em>Currently in beta — macOS 14+ only</em></p>
</div>

---

Glimpse is a local-first voice dictation app for Mac. No subscription, no data leaving your machine. It's just fast, accurate transcription powered by models running entirely on-device.

Built as an open-source alternative to Superwhisper and WisprFlow, for people who left (or never started) because of pricing or privacy concerns.

## Download

Grab the latest `.dmg` from the [Releases page](https://github.com/LegendarySpy/Glimpse/releases/latest) and drag Glimpse to your Applications folder.

## Features

- **Local transcription** — Runs entirely on-device using Whisper or Parakeet. Nothing is sent to the cloud by default.
- **Assistive text insertion** — Transcribed text is inserted directly where you're typing, in any app.
- **Edit mode** — Highlight any text, speak your changes, and Glimpse rewrites it in context.
- **Personalization** — Set per-app writing styles so Glimpse knows the difference between an email and a Slack message.
- **Custom dictionary** — Add names, jargon, or domain-specific words for more accurate transcription.
- **Replacements** — Define shortcuts that automatically expand into words, phrases, or anything else.
- **Library** — Transcribe audio and video files with synced playback and multi-format export.

## Roadmap

- [x] Custom dictionary
- [x] Built-in updater
- [x] Per-app personalization
- [x] Edit mode
- [x] Audio & video transcription
- [ ] Temporary mode (transcribe without saving)
- [ ] App localization
- [ ] Ask mode — query what's on your screen
- [ ] Better customizable keybinds

## Privacy

The core app runs fully on-device. Your transcriptions, API keys, and prompts are never collected or transmitted.

Glimpse does collect **anonymous, non-identifying analytics** via [Aptabase](https://aptabase.com/) to help prioritize development:

- App launches/exits, uptime, recording count
- Which transcription engine is active and which keybind mode is used
- Model downloads and onboarding completion

Analytics are GDPR compliant and fully anonymous. For complete transparency, see [`src-tauri/src/analytics.rs`](src-tauri/src/analytics.rs).

## License

Glimpse is licensed under [AGPL-3.0](LICENSE). Free to self-host and modify — just keep changes open. An optional paid cloud tier is planned for users who want faster speeds or cloud-only features.

## Acknowledgments

- [Tauri](https://v2.tauri.app/) — app framework
- [Glimpse-Speech](https://github.com/LegendarySpy/Glimpse-Speech) (MIT) — local transcription engine
- [whisper-rs](https://github.com/tazz4843/whisper-rs) (Unlicense) — Rust bindings for Whisper
- [parakeet-rs](https://github.com/altunenes/parakeet-rs) (MIT OR Apache-2.0) — ONNX Runtime bindings for Parakeet

**Bundled models** (downloaded in-app from Hugging Face):
- Whisper GGML (MIT): `ggml-large-v3-turbo-q8_0.bin`, `ggml-small-q5_1.bin` via [`ggerganov/whisper.cpp`](https://huggingface.co/ggerganov/whisper.cpp)
- Parakeet TDT 0.6B v3 ONNX (CC-BY-4.0): Int8 variant via [`istupakov/parakeet-tdt-0.6b-v3-onnx`](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx)

## Building Locally

**Prerequisites:** macOS 14+, [Rust](https://rustup.rs/) 1.74+, [Bun](https://bun.sh/) 1.3+, Xcode Command Line Tools
```bash
xcode-select --install
git clone https://github.com/LegendarySpy/Glimpse.git
cd Glimpse
bun install
```
```bash
bun tauri dev       # Development with hot reload
bun tauri build     # Production build
```

> [!TIP]
> After a production build, you may need to re-enable accessibility permissions in System Settings for text insertion to work.
