import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { motion } from "framer-motion";
import { Check, Clipboard, Copy, FileImage, FolderOpen, Loader2, ScanText } from "lucide-react";

type OcrStatus = {
  installed: boolean;
  label: string;
  engine: string;
  model_path: string;
  mmproj_path: string;
  runner_path: string;
  bytes_on_disk: number;
  missing_files: string[];
  benchmark_word_accuracy_percent: number;
  benchmark_char_accuracy_percent: number;
  benchmark_mean_elapsed_seconds: number;
};

type OcrResult = {
  text: string;
  model: string;
  image_path: string;
  image_data_url: string | null;
  elapsed_ms: number;
  runner_path: string;
};

type PasteTextResult = {
  pasted: boolean;
  copied: boolean;
  message: string;
};

const formatBytes = (bytes: number) => {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 MB";
  const mib = bytes / 1024 / 1024;
  if (mib < 1024) return `${mib.toFixed(0)} MB`;
  return `${(mib / 1024).toFixed(2)} GB`;
};

const formatElapsed = (ms: number) => {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
};

const fileNameFromPath = (path: string) => {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.length > 0 ? parts[parts.length - 1] : path;
};

export default function OcrView({ isActive = true }: { isActive?: boolean }) {
  const [status, setStatus] = useState<OcrStatus | null>(null);
  const [statusError, setStatusError] = useState("");
  const [imagePath, setImagePath] = useState("");
  const [prompt, setPrompt] = useState("Text Recognition:");
  const [result, setResult] = useState<OcrResult | null>(null);
  const [error, setError] = useState("");
  const [isRunning, setIsRunning] = useState(false);
  const [copied, setCopied] = useState(false);
  const [pasteMessage, setPasteMessage] = useState("");

  const refreshStatus = async () => {
    try {
      setStatusError("");
      setStatus(await invoke<OcrStatus>("get_ocr_status"));
    } catch (err) {
      setStatusError(err instanceof Error ? err.message : String(err));
    }
  };

  useEffect(() => {
    if (isActive) void refreshStatus();
  }, [isActive]);

  const readinessLabel = useMemo(() => {
    if (!status) return "Checking";
    return status.installed ? "Ready" : "Missing files";
  }, [status]);

  const chooseImage = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Images",
          extensions: ["png", "jpg", "jpeg", "webp", "bmp"],
        },
      ],
    });
    if (typeof selected === "string") {
      setImagePath(selected);
      setError("");
      setPasteMessage("");
    }
  };

  const runOcr = async () => {
    if (!imagePath.trim() || isRunning) return;
    setIsRunning(true);
    setError("");
    setPasteMessage("");
    setCopied(false);
    try {
      const nextResult = await invoke<OcrResult>("run_ocr_image", {
        imagePath: imagePath.trim(),
        prompt: prompt.trim() || null,
      });
      setResult(nextResult);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsRunning(false);
    }
  };

  const copyText = async () => {
    if (!result?.text) return;
    await navigator.clipboard.writeText(result.text);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1400);
  };

  const pasteText = async () => {
    if (!result?.text) return;
    const pasteResult = await invoke<PasteTextResult>("paste_text_to_focused_app", {
      text: result.text,
    });
    setPasteMessage(pasteResult.message);
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="mb-5 flex flex-wrap items-end justify-between gap-3">
        <div>
          <div className="ui-text-section-label ui-color-muted">Local OCR</div>
          <h1 className="mt-1 ui-text-title font-medium ui-color-primary">
            Read text from screenshots
          </h1>
          <p className="mt-1 max-w-2xl ui-text-body-sm ui-color-muted">
            GLM OCR runs from the local GGUF files on G drive. It is accurate on screenshots and
            tables, but CPU OCR still takes roughly a minute for large images.
          </p>
        </div>
        <button
          type="button"
          onClick={refreshStatus}
          className="ui-button-ghost h-9 gap-2 rounded-full border border-border-primary px-4 ui-text-button-sm"
        >
          <ScanText size={15} />
          Check runtime
        </button>
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-[minmax(280px,360px)_1fr] gap-4">
        <section className="flex min-h-0 flex-col rounded-lg border border-border-primary bg-surface-surface p-4">
          <div className="flex items-center justify-between gap-3">
            <div>
              <div className="ui-text-body-sm-strong ui-color-primary">
                {status?.label ?? "GLM OCR"}
              </div>
              <div className="ui-text-meta ui-color-muted">{status?.engine ?? "llama.cpp"}</div>
            </div>
            <span
              className={`rounded-full border px-2.5 py-1 ui-text-micro ${
                status?.installed
                  ? "border-border-secondary bg-[var(--surface-interactive-strong)] ui-color-primary"
                  : "border-border-primary bg-surface-elevated ui-color-muted"
              }`}
            >
              {readinessLabel}
            </span>
          </div>

          <div className="mt-4 space-y-2 rounded-md border border-border-primary bg-[var(--surface-interactive)] p-3">
            <div className="flex items-center justify-between gap-3 ui-text-meta">
              <span className="ui-color-muted">Disk</span>
              <span className="ui-color-primary">{formatBytes(status?.bytes_on_disk ?? 0)}</span>
            </div>
            <div className="flex items-center justify-between gap-3 ui-text-meta">
              <span className="ui-color-muted">Smoke score</span>
              <span className="ui-color-primary">
                {status?.benchmark_word_accuracy_percent ?? 94.3}% word /{" "}
                {status?.benchmark_char_accuracy_percent ?? 95.6}% char
              </span>
            </div>
            <div className="flex items-center justify-between gap-3 ui-text-meta">
              <span className="ui-color-muted">CPU average</span>
              <span className="ui-color-primary">
                {status?.benchmark_mean_elapsed_seconds ?? 66}s
              </span>
            </div>
          </div>

          <label className="mt-5 ui-text-meta-strong ui-color-secondary">Image path</label>
          <div className="mt-2 flex gap-2">
            <input
              value={imagePath}
              onChange={(event) => setImagePath(event.target.value)}
              placeholder="G:\\Friday\\tmp\\ocr-tests\\example-com.png"
              className="h-9 min-w-0 flex-1 rounded-md border border-border-primary bg-surface-elevated px-3 ui-text-input ui-color-primary placeholder:text-content-disabled focus:outline-none"
            />
            <button
              type="button"
              onClick={chooseImage}
              className="ui-button-ghost h-9 w-9 shrink-0 border border-border-primary"
              aria-label="Choose image"
            >
              <FolderOpen size={16} />
            </button>
          </div>

          <label className="mt-4 ui-text-meta-strong ui-color-secondary">Prompt</label>
          <input
            value={prompt}
            onChange={(event) => setPrompt(event.target.value)}
            className="mt-2 h-9 rounded-md border border-border-primary bg-surface-elevated px-3 ui-text-input ui-color-primary placeholder:text-content-disabled focus:outline-none"
          />

          <button
            type="button"
            onClick={runOcr}
            disabled={!status?.installed || !imagePath.trim() || isRunning}
            className="mt-4 inline-flex h-10 items-center justify-center gap-2 rounded-full border border-border-secondary bg-[var(--primary)] px-4 ui-text-button-sm text-[var(--primary-foreground)] transition-opacity disabled:opacity-40"
          >
            {isRunning ? <Loader2 size={16} className="animate-spin" /> : <ScanText size={16} />}
            {isRunning ? "Reading image..." : "Run OCR"}
          </button>

          {(statusError || error) && (
            <div className="mt-4 rounded-md border border-red-500/30 bg-red-500/10 p-3 ui-text-meta ui-color-error-soft">
              {statusError || error}
            </div>
          )}

          {status && !status.installed && (
            <div className="mt-4 min-h-0 rounded-md border border-border-primary bg-surface-elevated p-3">
              <div className="ui-text-meta-strong ui-color-primary">Missing runtime pieces</div>
              <div className="mt-2 max-h-28 space-y-1 overflow-y-auto">
                {status.missing_files.map((file) => (
                  <div key={file} className="truncate ui-text-micro ui-color-muted" title={file}>
                    {file}
                  </div>
                ))}
              </div>
            </div>
          )}
        </section>

        <section className="flex min-h-0 flex-col rounded-lg border border-border-primary bg-surface-surface">
          <div className="flex min-h-14 items-center justify-between gap-3 border-b border-border-primary px-4">
            <div className="min-w-0">
              <div className="truncate ui-text-body-sm-strong ui-color-primary">
                {result?.image_path
                  ? fileNameFromPath(result.image_path)
                  : imagePath
                    ? fileNameFromPath(imagePath)
                    : "No image selected"}
              </div>
              <div className="ui-text-micro ui-color-muted">
                {result
                  ? `${result.model} in ${formatElapsed(result.elapsed_ms)}`
                  : "Choose a screenshot, invoice, UI capture, or scanned image."}
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={copyText}
                disabled={!result?.text}
                className="ui-button-ghost h-8 gap-2 rounded-full border border-border-primary px-3 ui-text-button-sm disabled:opacity-40"
              >
                {copied ? <Check size={14} /> : <Copy size={14} />}
                Copy
              </button>
              <button
                type="button"
                onClick={pasteText}
                disabled={!result?.text}
                className="ui-button-ghost h-8 gap-2 rounded-full border border-border-primary px-3 ui-text-button-sm disabled:opacity-40"
              >
                <Clipboard size={14} />
                Paste
              </button>
            </div>
          </div>

          <div className="grid min-h-0 flex-1 grid-rows-[minmax(120px,220px)_1fr]">
            <div className="flex min-h-0 items-center justify-center border-b border-border-primary bg-[var(--surface-interactive)] p-3">
              {result?.image_data_url ? (
                <motion.img
                  initial={{ opacity: 0, scale: 0.98 }}
                  animate={{ opacity: 1, scale: 1 }}
                  src={result.image_data_url}
                  alt=""
                  className="max-h-full max-w-full rounded-md border border-border-primary object-contain"
                />
              ) : (
                <div className="flex flex-col items-center justify-center text-center">
                  <FileImage size={28} className="ui-color-disabled" />
                  <div className="mt-3 ui-text-body-sm-strong ui-color-secondary">
                    Image preview appears after OCR
                  </div>
                  <div className="mt-1 max-w-sm ui-text-meta ui-color-muted">
                    Friday copies the chosen image into `G:\Friday\data\ocr\inputs` before running
                    the local model.
                  </div>
                </div>
              )}
            </div>

            <div className="min-h-0 p-4">
              <textarea
                value={result?.text ?? ""}
                readOnly
                placeholder="OCR text will appear here..."
                className="h-full w-full resize-none rounded-md border border-transparent bg-transparent ui-text-body ui-color-primary placeholder:text-content-disabled focus:outline-none"
              />
            </div>
          </div>

          {pasteMessage && (
            <div className="border-t border-border-primary px-4 py-3 ui-text-meta ui-color-muted">
              {pasteMessage}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
