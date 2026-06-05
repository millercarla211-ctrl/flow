# Flow Dictation Host

This package is a focused build host for `src/bin/flow-dictate.rs`.
It keeps Zed's Agent composer STT helper build small by depending only on the audio, WAV, and Sherpa Parakeet crates used by the dictation binary.

Run commands from the Flow repo root:

```powershell
cargo build --manifest-path tools\flow-dictation-host\Cargo.toml --target-dir G:\Dx\flow\target -j1
.\target\debug\flow-dictate.exe --devices
.\target\debug\flow-dictate.exe --meter
.\target\debug\flow-dictate.exe --file <16k-mono-wav>
```

The binary expects Parakeet files under `models\stt\parakeet-tdt-0.6b-v3-int8`.
Install or repair them with `scripts\download_sherpa_parakeet_stt.ps1` before live dictation proof.
