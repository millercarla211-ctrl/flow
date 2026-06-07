# Flow TTS Host

Focused build host for `src/bin/flow-tts.rs`.

Run from the Flow repo root:

```powershell
cargo build --manifest-path tools\flow-tts-host\Cargo.toml --target-dir G:\Dx\flow\target -j1
.\target\debug\flow-tts.exe --text "hello" --output tmp\flow-tts.wav
```

The adapter writes a WAV file only. It does not play audio itself, so host apps such as Zed can keep playback, cancellation, and cleanup under their own control.
