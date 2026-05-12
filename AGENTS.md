# Flow Agent Notes

This repo is a Rust-first local AI runtime. Preserve existing behavior and work additively.

## Local Model Roles

Flow uses a local role policy on this 24GB RAM Windows machine:

- `qwen3-0.6b` is the fastest helper model for Flow prompt enhancement, text cleanup, tiny rewrites, text conversion, labels, and low-latency support tasks.
- `xlam2-3b-fc-r-q4km` is the dedicated local tool-agent research model for strict JSON tool-routing/function-call decisions. It is CC-BY-NC-4.0, so use it for local evaluation/research, not commercial defaults.
- `qwen35-4b-revised-q4km` is the daily smart model for DX UI registry work, shadcn-style component edits, React/TypeScript/Tailwind prompts, Rust edits, general coding help, and normal useful answers.
- `qwen35-9b-q4km` is the slow backup when the 4B model fails and latency is acceptable.
- `gemma4-e4b-frontend-text-q4km` is no longer a daily quality-chat role on this OS. Keep Gemma for UI-generation, vision, and benchmark experiments only.

```powershell
cargo run --release --bin flow -- --model-roles
cargo run --release --bin flow -- --tool-model-candidates
cargo run --release --bin flow -- --install-model qwen35-4b-revised-q4km
cargo run --release --bin flow -- --install-model xlam2-3b-fc-r-q4km
cargo run --release --bin flow -- --install-model qwen3-0.6b
cargo run --release --bin flow -- --install-model qwen35-9b-q4km
cargo run --release --bin flow -- --models chat
cargo run --release --bin flow -- --plan chat qwen35-4b-revised-q4km
cargo run --release --bin flow -- --chat qwen35-4b-revised-q4km
cargo run --release --bin flow -- --tool-agent "choose a tool for this request"
cargo run --release --bin flow -- --tool-agent-tools examples/tool-agent/weather-tools.json "weather in Dhaka tomorrow"
cargo run --release --bin flow -- --chat qwen3-0.6b
```

## Local UI Model Eval

Use these exact commands for the local UI model path:

```powershell
cargo run --release --bin flow -- --ui-model-candidates
cargo run --release --bin flow -- --install-model gemma4-e4b-frontend-q4km
cargo run --release --bin flow -- --install-model webgen-4b-preview-i1-q4km
cargo run --release --bin flow -- --install-model qwendean-4b-q4km
cargo run --release --bin flow -- --models ui
cargo run --release --bin flow -- --models vlm
cargo run --release --bin flow -- --uigen-vision-google
cargo run --release --bin flow -- --uigen tmp/uigen-output/index.html "make a shadcn-style Google homepage clone"
cargo run --release --bin flow -- --uigen-google webgen-4b-preview-i1-q4km
cargo run --release --bin flow -- --uigen-google qwendean-4b-q4km
powershell -ExecutionPolicy Bypass -File scripts\uigen_vision_google_eval.ps1 -ForceGenerate
powershell -ExecutionPolicy Bypass -File scripts\uigen_google_eval.ps1 -ModelKey webgen-4b-preview-i1-q4km -ForceGenerate
powershell -ExecutionPolicy Bypass -File scripts\uigen_google_eval.ps1 -ModelKey qwendean-4b-q4km -ForceGenerate
```

The tested model files are ignored by git and live under `models/llm`. For screenshot cloning, prefer the `gemma4-e4b-frontend-q4km` vision path over text-only UI models, but keep the latest eval result honest: its first Google clone run failed the complete-HTML gate.
Default inference is CPU-safe; set `FLOW_LLAMA_GPU_LAYERS` only when the machine has enough GPU memory.

Do not claim a UI clone passes until the generated screenshots in `tmp/uigen-google` are inspected against the live Google reference screenshots.
