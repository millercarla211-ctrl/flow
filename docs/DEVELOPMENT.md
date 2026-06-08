# Development Guide

## Setup

1. Install Rust (edition 2024):
```bash
rustup update
rustup default stable
```

2. Download models:
```bash
pwsh scripts/download_moonshine_onnx.ps1
```

3. Build project:
```bash
cargo build --release
```

## Project Structure

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed structure.

## Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_test

# Benchmarks
cargo bench
```

## Examples

```bash
# Basic transcription
cargo run --example transcribe

# Full Wispr Flow pipeline
cargo run --example wispr_flow
```

## CLI Usage

```bash
# Speech-to-text file input
cargo run --bin flow-dictate -- --file tmp/input.wav --model parakeet-tdt-0.6b-v3-int8

# Full pipeline (STT + LLM)
cargo run -- --wispr audio.mp3

# Text-to-speech file output
cargo run --bin flow-tts -- --text "Hello world" --output tmp/speech.wav
```

## Code Style

- Use `rustfmt` for formatting
- Use `clippy` for linting
- Follow Rust 2024 edition idioms
- Document public APIs

## Performance Profiling

```bash
# Profile with flamegraph
cargo flamegraph --bin flow -- --transcribe audio.mp3

# Memory profiling
cargo run --release -- --transcribe audio.mp3
```

## Contributing

1. Create feature branch
2. Write tests
3. Run `cargo fmt` and `cargo clippy`
4. Submit pull request
