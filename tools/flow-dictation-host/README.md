# Flow Dictation Host

This package is a focused build host for `src/bin/flow-dictate.rs`.
It keeps Zed's Agent composer STT helper build small by depending on the audio, WAV, Sherpa transducer, and process-bound whisper.cpp paths used by the dictation binary.

Build from the Flow repo root:

```powershell
cargo build --manifest-path tools\flow-dictation-host\Cargo.toml --target-dir G:\Dx\flow\target -j1
```

File-mode smoke examples:

```powershell
.\target\debug\flow-dictate.exe --file <16k-mono-wav>
.\target\debug\flow-dictate.exe --file <16k-mono-wav> --model parakeet-tdt-0.6b-v3-int8
.\target\debug\flow-dictate.exe --file <16k-mono-wav> --model nemotron-speech-streaming-en-0.6b-int8
.\target\debug\flow-dictate.exe --file <16k-mono-wav> --model whisper-tiny-ggml --whisper-bin <whisper-cli.exe> --whisper-model <ggml-tiny.bin>
.\target\debug\flow-dictate.exe --file <16k-mono-wav> --model whisper-tiny-ggml --whisper-cpp <whisper-cli.exe> --whisper-model <ggml-tiny.bin> --whisper-language en
```

Live device inspection requires an authorized validation window because it touches host audio devices:

```powershell
.\target\debug\flow-dictate.exe --devices
.\target\debug\flow-dictate.exe --meter
```

The focused host supports Sherpa Parakeet, Sherpa Nemotron, and whisper.cpp Whisper model paths:

- Parakeet TDT 0.6B v3 INT8, the default, under `models\stt\parakeet-tdt-0.6b-v3-int8`.
- Nemotron Speech Streaming EN 0.6B INT8 under `models\stt\nemotron-speech-streaming-en-0.6b-int8`; it remains listed but not runtime-proved until the model files and governed smoke proof pass.
- Whisper Tiny GGML as an explicit opt-in through `--model whisper-tiny-ggml`.

Install or repair the default Parakeet bundle with `scripts\download_sherpa_parakeet_stt.ps1` before live dictation proof.
Parakeet and Nemotron require non-empty Sherpa transducer files (`encoder.int8.onnx`, `decoder.int8.onnx`, `joiner.int8.onnx`, and `tokens.txt`) in the paths above.
Install or repair the default Whisper Tiny runtime with `scripts\download_whisper.ps1`; downloaded model and whisper.cpp runtime artifacts are local ignored files.
Whisper requires a local non-empty `whisper-cli.exe` and GGML model file. Resolve the binary with `--whisper-bin`, `--whisper-cpp`, `FLOW_WHISPER_CPP_BINARY`, `DX_WHISPER_CPP_BINARY`, `FLOW_WHISPER_CPP_EXE`, `FLOW_WHISPER_CPP`, or the known Flow-local whisper.cpp build paths. Resolve the model with `--whisper-model`, `FLOW_WHISPER_MODEL`, `DX_FLOW_WHISPER_MODEL`, or the default `models\stt\ggml-tiny.bin`. Resolve the language with `--whisper-language`, `FLOW_WHISPER_LANGUAGE`, `DX_FLOW_WHISPER_LANGUAGE`, or the default `en`. Without the required binary and model files, the host fails closed before transcription instead of falling back to another STT model.
