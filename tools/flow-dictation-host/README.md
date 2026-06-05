# Flow Dictation Host

This package is a focused build host for `src/bin/flow-dictate.rs`.
It keeps Zed's Agent composer STT helper build small by depending only on the audio, WAV, and Sherpa crates used by the dictation binary.

Run commands from the Flow repo root:

```powershell
cargo build --manifest-path tools\flow-dictation-host\Cargo.toml --target-dir G:\Dx\flow\target -j1
.\target\debug\flow-dictate.exe --devices
.\target\debug\flow-dictate.exe --meter
.\target\debug\flow-dictate.exe --file <16k-mono-wav>
.\target\debug\flow-dictate.exe --file <16k-mono-wav> --model parakeet-tdt-0.6b-v3-int8
.\target\debug\flow-dictate.exe --file <16k-mono-wav> --model nemotron-speech-streaming-en-0.6b-int8
```

The focused host supports Sherpa Parakeet and Nemotron model bundles:

- Parakeet TDT 0.6B v3 INT8, the default, under `models\stt\parakeet-tdt-0.6b-v3-int8`.
- Nemotron Speech Streaming EN 0.6B INT8 under `models\stt\nemotron-speech-streaming-en-0.6b-int8`.

Install or repair the default Parakeet bundle with `scripts\download_sherpa_parakeet_stt.ps1` before live dictation proof.
Nemotron requires its own Sherpa transducer bundle in the path above.
