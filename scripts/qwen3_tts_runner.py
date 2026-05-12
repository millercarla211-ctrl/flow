#!/usr/bin/env python3
"""Small local TTS runner used by Flow.

The Rust app writes this script into Flow's data directory before invoking it.
Keep imports inside main paths so missing Python dependencies produce a useful
runtime error instead of breaking app startup.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run local TTS from Flow")
    parser.add_argument("--server", action="store_true")
    parser.add_argument("--model-dir")
    parser.add_argument(
        "--model-kind", choices=["voice_clone", "custom_voice", "kokoro"], required=True
    )
    parser.add_argument("--text")
    parser.add_argument("--output")
    parser.add_argument("--language", default="English")
    parser.add_argument("--reference-audio")
    parser.add_argument("--reference-text")
    parser.add_argument("--speaker")
    parser.add_argument("--instruct")
    parser.add_argument("--device", default="auto", choices=["auto", "cpu", "cuda"])
    return parser.parse_args()


def resolve_device(requested: str):
    import torch

    if requested == "cuda":
        if not torch.cuda.is_available():
            raise RuntimeError("CUDA was requested, but torch.cuda is not available.")
        return "cuda:0", torch.bfloat16, "flash_attention_2"

    if requested == "auto" and torch.cuda.is_available():
        return "cuda:0", torch.bfloat16, "flash_attention_2"

    return "cpu", torch.float32, "eager"


def first_supported_speaker(model) -> str:
    if hasattr(model, "get_supported_speakers"):
        speakers = model.get_supported_speakers()
        if speakers:
            return speakers[0]
    return "Cherry"


def load_model(model_dir: str, device: str):
    from qwen_tts import Qwen3TTSModel

    resolved_device, dtype, attn = resolve_device(device)
    kwargs = {
        "device_map": resolved_device,
        "dtype": dtype,
    }
    if attn == "flash_attention_2":
        kwargs["attn_implementation"] = "flash_attention_2"
    return Qwen3TTSModel.from_pretrained(model_dir, **kwargs)


def kokoro_lang_code(language: str) -> str:
    normalized = (language or "").strip().lower()
    if normalized in {"en", "english", "american english", "us english"}:
        return "a"
    if normalized in {"british english", "uk english"}:
        return "b"
    return "a"


def load_kokoro_pipeline(language: str, device: str, model_dir: str | None = None):
    import torch
    from kokoro import KModel, KPipeline

    resolved_device = "cpu" if device == "auto" else device
    local_dir = Path(model_dir) if model_dir else None
    if local_dir and (local_dir / "config.json").exists() and (local_dir / "kokoro-v1_0.pth").exists():
        model = KModel(
            repo_id="hexgrad/Kokoro-82M",
            config=str(local_dir / "config.json"),
            model=str(local_dir / "kokoro-v1_0.pth"),
        ).to(resolved_device).eval()
        pipeline = KPipeline(
            lang_code=kokoro_lang_code(language),
            repo_id="hexgrad/Kokoro-82M",
            model=model,
            device=resolved_device,
        )
        voices_dir = local_dir / "voices"
        if voices_dir.exists():
            for voice_file in voices_dir.glob("*.pt"):
                pipeline.voices[voice_file.stem] = torch.load(
                    str(voice_file), map_location="cpu", weights_only=True
                )
        return pipeline

    return KPipeline(
        lang_code=kokoro_lang_code(language),
        repo_id="hexgrad/Kokoro-82M",
        device=resolved_device,
    )


def generate_kokoro_audio(pipeline, text: str, voice: str):
    import numpy as np

    audio_parts = [audio for _, _, audio in pipeline(text, voice=voice, speed=1.0)]
    if not audio_parts:
        raise RuntimeError("Kokoro did not produce audio.")
    audio = np.concatenate(audio_parts) if len(audio_parts) > 1 else audio_parts[0]
    return audio, 24000


DEFAULT_KOKORO_VOICE = "af_bella"


def generate_kokoro(args: argparse.Namespace):
    voice = (args.speaker or "").strip() or DEFAULT_KOKORO_VOICE
    pipeline = load_kokoro_pipeline(args.language, args.device, args.model_dir)
    return generate_kokoro_audio(pipeline, args.text or "", voice)


def run_kokoro_server(args: argparse.Namespace) -> int:
    import soundfile as sf

    started = time.perf_counter()
    pipeline = load_kokoro_pipeline(args.language, args.device, args.model_dir)
    generate_kokoro_audio(pipeline, "Ready.", (args.speaker or "").strip() or DEFAULT_KOKORO_VOICE)
    print(
        json.dumps(
            {
                "ok": True,
                "ready": True,
                "elapsed_seconds": round(time.perf_counter() - started, 3),
            },
            ensure_ascii=False,
        ),
        flush=True,
    )

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
            text = str(request.get("text", "")).strip()
            output = str(request.get("output", "")).strip()
            voice = str(request.get("speaker", "")).strip() or DEFAULT_KOKORO_VOICE
            if not text:
                raise RuntimeError("No text provided.")
            if not output:
                raise RuntimeError("No output path provided.")

            request_started = time.perf_counter()
            output_path = Path(output)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            wavs, sample_rate = generate_kokoro_audio(pipeline, text, voice)
            sf.write(str(output_path), wavs, sample_rate)
            print(
                json.dumps(
                    {
                        "ok": True,
                        "output": str(output_path),
                        "elapsed_seconds": round(time.perf_counter() - request_started, 3),
                    },
                    ensure_ascii=False,
                ),
                flush=True,
            )
        except Exception as exc:
            print(json.dumps({"ok": False, "error": str(exc)}, ensure_ascii=False), flush=True)

    return 0


def main() -> int:
    args = parse_args()
    if args.server:
        try:
            return run_kokoro_server(args)
        except Exception as exc:
            print(
                json.dumps(
                    {
                        "ok": False,
                        "error": str(exc),
                        "hint": "Run scripts/setup-kokoro-tts-runtime.ps1 for Kokoro.",
                    },
                    ensure_ascii=False,
                ),
                file=sys.stderr,
            )
            return 1

    if not args.text or not args.output:
        print(
            json.dumps(
                {"ok": False, "error": "--text and --output are required outside server mode"},
                ensure_ascii=False,
            ),
            file=sys.stderr,
        )
        return 1

    started = time.perf_counter()
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    try:
        import soundfile as sf

        if args.model_kind == "kokoro":
            wavs, sample_rate = generate_kokoro(args)
            sf.write(str(output_path), wavs, sample_rate)
        else:
            if not args.model_dir:
                raise RuntimeError("Qwen3-TTS needs --model-dir.")
            model = load_model(args.model_dir, args.device)
            if args.model_kind == "voice_clone":
                if not args.reference_audio:
                    raise RuntimeError("Qwen3-TTS Base voice cloning needs a reference audio path.")

                clone_kwargs = {
                    "text": args.text,
                    "language": args.language,
                    "ref_audio": args.reference_audio,
                }
                if args.reference_text:
                    clone_kwargs["ref_text"] = args.reference_text
                else:
                    clone_kwargs["x_vector_only_mode"] = True

                wavs, sample_rate = model.generate_voice_clone(**clone_kwargs)
            else:
                speaker = (args.speaker or "").strip() or first_supported_speaker(model)
                custom_kwargs = {
                    "text": args.text,
                    "language": args.language,
                    "speaker": speaker,
                }
                if args.instruct:
                    custom_kwargs["instruct"] = args.instruct
                wavs, sample_rate = model.generate_custom_voice(**custom_kwargs)

            sf.write(str(output_path), wavs[0], sample_rate)
        print(
            json.dumps(
                {
                    "ok": True,
                    "output": str(output_path),
                    "elapsed_seconds": round(time.perf_counter() - started, 3),
                },
                ensure_ascii=False,
            )
        )
        return 0
    except Exception as exc:
        print(
            json.dumps(
                {
                    "ok": False,
                    "error": str(exc),
                    "hint": "Run scripts/setup-kokoro-tts-runtime.ps1 for Kokoro or scripts/setup-qwen3-tts-runtime.ps1 for Qwen.",
                },
                ensure_ascii=False,
            ),
            file=sys.stderr,
        )
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
