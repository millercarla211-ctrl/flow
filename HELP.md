# Kokoro TTS Runtime Notes

**Date:** 2026-06-08
**Status:** Resolved for the focused `flow-tts` adapter.

---

## Current Working Path

Use the focused file-output adapter:

```powershell
.\target\debug\flow-tts.exe --text "hello from flow" --output .\tmp\speech.wav
```

The adapter:

- loads `models/tts/kokoro-v1.0.int8.onnx`
- loads `models/tts/voices-v1.0.bin`
- loads `models/tts/config.json`
- synthesizes with the real `KokoroTTS` runtime
- writes WAV output only
- rejects invalid, empty, and all-silent WAV files

This is the path Zed should use for read-aloud. Avoid routing read-aloud through the older interactive `flow --speak` path.

---

## Required Local Files

```text
models/tts/kokoro-v1.0.int8.onnx
models/tts/voices-v1.0.bin
models/tts/config.json
```

If any of these files are missing or empty, `flow-tts` should fail readiness instead of pretending TTS is available.

---

## Historical Issue

Older Kokoro experiments produced silent audio because tokenization did not match Kokoro's phoneme-token expectations. That note is no longer the current production path. The maintained adapter validates the final WAV so this class of failure is caught before success is reported.
