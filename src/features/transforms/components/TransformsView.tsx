import { useCallback, useEffect, useMemo, useState } from "react";
import { useLingui } from "@lingui/react/macro";
import { invoke } from "@tauri-apps/api/core";
import {
  Check,
  ClipboardPaste,
  Copy,
  History,
  RefreshCw,
  Save,
  SendHorizontal,
  ShieldOff,
  SplitSquareHorizontal,
  Trash2,
  WandSparkles,
} from "lucide-react";
import DotMatrix from "../../../shared/ui/DotMatrix";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCreateScratchpadEntry } from "../../scratchpad/queries";
import {
  useDeleteTransformHistoryEntry,
  useTransformHistory,
  useTransformPresets,
  useTransformSource,
  useTransformText,
} from "../queries";
import type { TransformHistoryEntry, TransformResult } from "../../../types";

const sampleText =
  "Hey, I wanted to quickly follow up about the thing we discussed yesterday. I think we should make the plan clearer and easier for everyone to act on.";

type DiffToken = {
  type: "same" | "added" | "removed";
  value: string;
};

type TransformLoadPayload = {
  text?: string;
  source?: string;
};

type TransformPasteResult = {
  pasted: boolean;
  copied: boolean;
  message: string;
};

const variantKey = (variant: TransformResult) => variant.history_id ?? variant.label;

const formatHistoryTime = (value: string) => {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleString([], {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
};

const words = (value: string) => value.trim().split(/\s+/).filter(Boolean);

const buildWordDiff = (before: string, after: string): DiffToken[] => {
  const left = words(before);
  const right = words(after);
  if (left.length + right.length > 700) {
    return [
      { type: "removed", value: before },
      { type: "added", value: after },
    ];
  }

  const table = Array.from({ length: left.length + 1 }, () =>
    Array.from({ length: right.length + 1 }, () => 0),
  );

  for (let i = left.length - 1; i >= 0; i -= 1) {
    for (let j = right.length - 1; j >= 0; j -= 1) {
      table[i][j] =
        left[i] === right[j] ? table[i + 1][j + 1] + 1 : Math.max(table[i + 1][j], table[i][j + 1]);
    }
  }

  const diff: DiffToken[] = [];
  let i = 0;
  let j = 0;
  while (i < left.length && j < right.length) {
    if (left[i] === right[j]) {
      diff.push({ type: "same", value: left[i] });
      i += 1;
      j += 1;
    } else if (table[i + 1][j] >= table[i][j + 1]) {
      diff.push({ type: "removed", value: left[i] });
      i += 1;
    } else {
      diff.push({ type: "added", value: right[j] });
      j += 1;
    }
  }
  while (i < left.length) {
    diff.push({ type: "removed", value: left[i] });
    i += 1;
  }
  while (j < right.length) {
    diff.push({ type: "added", value: right[j] });
    j += 1;
  }
  return diff;
};

const historyToResult = (entry: TransformHistoryEntry): TransformResult => ({
  history_id: entry.id,
  preset_id: entry.preset_id,
  label: entry.label,
  original: entry.original,
  transformed: entry.transformed,
  instruction: entry.instruction,
  created_at: entry.created_at,
});

export default function TransformsView({
  isActive = true,
  historyDisabled = false,
  onOpenDataSettings,
}: {
  isActive?: boolean;
  historyDisabled?: boolean;
  onOpenDataSettings?: () => void;
}) {
  const { t } = useLingui();
  const presetsQuery = useTransformPresets(isActive);
  const historyQuery = useTransformHistory(isActive && !historyDisabled, 12);
  const transformMutation = useTransformText();
  const sourceMutation = useTransformSource();
  const deleteHistoryMutation = useDeleteTransformHistoryEntry();
  const scratchpadMutation = useCreateScratchpadEntry();
  const [sourceText, setSourceText] = useState("");
  const [customInstruction, setCustomInstruction] = useState("");
  const [activeVariant, setActiveVariant] = useState<"original" | string>("original");
  const [variants, setVariants] = useState<TransformResult[]>([]);
  const [copied, setCopied] = useState(false);
  const [isPasting, setIsPasting] = useState(false);
  const [reviewMode, setReviewMode] = useState<"result" | "diff">("result");
  const [sourceHint, setSourceHint] = useState<string | null>(null);
  const presets = presetsQuery.data ?? [];
  const history = historyDisabled ? [] : (historyQuery.data ?? []);

  const activeResult = useMemo(() => {
    if (activeVariant === "original") return null;
    return variants.find((variant) => variantKey(variant) === activeVariant) ?? null;
  }, [activeVariant, variants]);

  const selectedText = useMemo(() => {
    if (activeVariant === "original") return sourceText;
    return activeResult?.transformed ?? sourceText;
  }, [activeResult, activeVariant, sourceText]);

  const diffTokens = useMemo(
    () => (activeResult ? buildWordDiff(activeResult.original, activeResult.transformed) : []),
    [activeResult],
  );

  const canTransform = sourceText.trim().length > 0 && !transformMutation.isPending;

  const setSource = useCallback((text: string, hint?: string | null) => {
    setSourceText(text);
    setActiveVariant("original");
    setReviewMode("result");
    setSourceHint(hint ?? null);
  }, []);

  useEffect(() => {
    let cancelled = false;
    let unlisten: UnlistenFn | null = null;

    listen<TransformLoadPayload>("transforms:load_text", (event) => {
      const text = event.payload?.text?.trim();
      if (!text || cancelled) return;
      const source = event.payload?.source === "overlay" ? "Loaded from overlay" : "Loaded text";
      setSource(text, source);
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [setSource]);

  const importSource = async () => {
    const source = await sourceMutation.mutateAsync();
    setSource(
      source.text,
      source.source === "selection" ? "Imported selected text" : "Imported clipboard text",
    );
  };

  const applyResult = (result: TransformResult) => {
    setVariants((current) => [
      result,
      ...current.filter((variant) => variantKey(variant) !== variantKey(result)),
    ]);
    setActiveVariant(variantKey(result));
    setReviewMode("diff");
  };

  const runPreset = async (presetId: string) => {
    if (!canTransform) return;
    const result = await transformMutation.mutateAsync({
      text: sourceText,
      presetId,
    });
    applyResult(result);
  };

  const runCustom = async () => {
    if (!canTransform || !customInstruction.trim()) return;
    const result = await transformMutation.mutateAsync({
      text: sourceText,
      instruction: customInstruction,
    });
    applyResult(result);
  };

  const copySelected = async () => {
    if (!selectedText.trim()) return;
    await navigator.clipboard.writeText(selectedText);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1400);
  };

  const saveSelected = async () => {
    if (!selectedText.trim()) return;
    await scratchpadMutation.mutateAsync({
      body: selectedText,
      source: "transform",
    });
  };

  const pasteSelected = async () => {
    if (!selectedText.trim() || isPasting) return;
    setIsPasting(true);
    try {
      const result = await invoke<TransformPasteResult>("paste_transform_result", {
        text: selectedText,
      });
      if (result.copied) {
        setCopied(true);
        window.setTimeout(() => setCopied(false), 1400);
      }
    } finally {
      setIsPasting(false);
    }
  };

  const loadHistory = (entry: TransformHistoryEntry) => {
    const result = historyToResult(entry);
    setSourceText(entry.original);
    setVariants((current) => [
      result,
      ...current.filter((variant) => variantKey(variant) !== result.history_id),
    ]);
    setActiveVariant(entry.id);
    setReviewMode("diff");
    setSourceHint("Loaded from transform history");
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="mb-5 flex flex-wrap items-end justify-between gap-3">
        <div>
          <div className="ui-text-section-label ui-color-muted">
            {t({ id: "transforms.eyebrow", message: "Transforms" })}
          </div>
          <h1 className="mt-1 ui-text-title font-medium ui-color-primary">
            {t({ id: "transforms.title", message: "Repolish text" })}
          </h1>
          <p className="mt-1 max-w-xl ui-text-body-sm ui-color-muted">
            {t({
              id: "transforms.subtitle",
              message:
                "Use selected text, clipboard text, presets, or a custom instruction to rewrite and compare.",
            })}
          </p>
        </div>
        <button
          type="button"
          onClick={importSource}
          disabled={sourceMutation.isPending}
          className="ui-button-ghost h-9 gap-2 rounded-full border border-border-primary px-4 ui-text-button-sm disabled:opacity-40"
        >
          <ClipboardPaste size={16} />
          {sourceMutation.isPending ? "Importing..." : "Use selection"}
        </button>
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-[minmax(300px,440px)_1fr] gap-4">
        <section className="flex min-h-0 flex-col rounded-lg border border-border-primary bg-surface-surface">
          <div className="border-b border-border-primary px-4 py-3">
            <div className="ui-text-body-sm-strong ui-color-primary">
              {t({ id: "transforms.source.title", message: "Source text" })}
            </div>
            <div className="mt-1 ui-text-meta ui-color-muted">
              {t({
                id: "transforms.source.hint",
                message: "Import selected text first; Flow falls back to clipboard when needed.",
              })}
            </div>
            {sourceHint && <div className="mt-2 ui-text-micro ui-color-disabled">{sourceHint}</div>}
          </div>
          <div className="flex min-h-0 flex-1 flex-col p-4">
            <textarea
              value={sourceText}
              onChange={(event) => {
                setSourceText(event.target.value);
                setActiveVariant("original");
                setSourceHint(null);
              }}
              placeholder={sampleText}
              className="min-h-0 flex-1 resize-none rounded-md border border-border-primary bg-surface-elevated px-3 py-2 ui-text-input ui-color-primary placeholder:text-content-disabled focus:outline-none focus:ring-2 focus:ring-[var(--border-strong)]"
            />
            <div className="mt-3 grid grid-cols-2 gap-2">
              {presetsQuery.isLoading ? (
                <div className="col-span-2 flex h-20 items-center justify-center">
                  <DotMatrix cols={10} rows={4} dotSize={2} gap={5} />
                </div>
              ) : (
                presets.map((preset) => (
                  <button
                    key={preset.id}
                    type="button"
                    onClick={() => runPreset(preset.id)}
                    disabled={!canTransform}
                    className="ui-button-ghost h-9 justify-start gap-2 rounded-md border border-border-primary px-3 ui-text-button-sm disabled:opacity-40"
                  >
                    <WandSparkles size={14} />
                    {preset.label}
                  </button>
                ))
              )}
            </div>

            <div className="mt-4 rounded-md border border-border-primary bg-[var(--surface-interactive)] p-3">
              <div className="ui-text-meta-strong ui-color-muted">
                {t({ id: "transforms.custom.label", message: "Custom instruction" })}
              </div>
              <textarea
                value={customInstruction}
                onChange={(event) => setCustomInstruction(event.target.value)}
                placeholder={t({
                  id: "transforms.custom.placeholder",
                  message: "Make it shorter, warmer, and direct.",
                })}
                className="mt-2 h-20 w-full resize-none rounded-md border border-border-primary bg-surface-elevated px-3 py-2 ui-text-input ui-color-primary placeholder:text-content-disabled focus:outline-none focus:ring-2 focus:ring-[var(--border-strong)]"
              />
              <button
                type="button"
                onClick={runCustom}
                disabled={!canTransform || !customInstruction.trim()}
                className="mt-2 inline-flex h-8 items-center gap-2 rounded-full border border-border-secondary bg-[var(--surface-interactive-strong)] px-3 ui-text-button-sm ui-color-primary transition-colors hover:bg-[var(--surface-interactive-pressed)] disabled:opacity-40"
              >
                <RefreshCw size={14} />
                {t({ id: "transforms.custom.run", message: "Transform" })}
              </button>
            </div>

            <div className="mt-4 min-h-0 rounded-md border border-border-primary bg-[var(--surface-interactive)] p-3">
              <div className="mb-2 flex items-center justify-between gap-3">
                <div className="flex items-center gap-2 ui-text-meta-strong ui-color-muted">
                  <History size={14} />
                  Recent edits
                </div>
                <span className="ui-text-micro ui-color-disabled">{history.length}</span>
              </div>

              {historyDisabled ? (
                <div className="rounded-md border border-border-primary bg-surface-elevated px-3 py-6 text-center">
                  <ShieldOff
                    size={16}
                    className="mx-auto mb-2 text-content-disabled"
                    aria-hidden="true"
                  />
                  <div className="ui-text-meta-strong ui-color-secondary">
                    {t({
                      id: "transforms.history.disabled.title",
                      message: "History paused",
                    })}
                  </div>
                  <div className="mx-auto mt-1 max-w-[220px] ui-text-micro ui-color-muted">
                    {t({
                      id: "transforms.history.disabled.body",
                      message:
                        "Transform history is not stored while Local Data Storage is set to Never store.",
                    })}
                  </div>
                  {onOpenDataSettings && (
                    <button
                      type="button"
                      onClick={onOpenDataSettings}
                      className="mt-3 rounded-full border border-border-primary px-3 py-1 ui-text-meta font-medium ui-color-primary transition-colors hover:border-border-secondary hover:bg-surface-surface"
                    >
                      {t({
                        id: "transforms.history.disabled.open_settings",
                        message: "Change data storage",
                      })}
                    </button>
                  )}
                </div>
              ) : historyQuery.isLoading ? (
                <div className="flex h-24 items-center justify-center">
                  <DotMatrix cols={10} rows={4} dotSize={2} gap={5} />
                </div>
              ) : history.length === 0 ? (
                <div className="rounded-md border border-border-primary bg-surface-elevated px-3 py-6 text-center ui-text-meta ui-color-muted">
                  Transforms you run will appear here.
                </div>
              ) : (
                <div className="max-h-44 space-y-2 overflow-y-auto pr-1">
                  {history.map((entry) => (
                    <article
                      key={entry.id}
                      className="group grid grid-cols-[1fr_auto] gap-2 rounded-md border border-border-primary bg-surface-surface p-2"
                    >
                      <button
                        type="button"
                        onClick={() => loadHistory(entry)}
                        className="min-w-0 text-left"
                      >
                        <div className="flex items-center gap-2">
                          <span className="truncate ui-text-meta-strong ui-color-primary">
                            {entry.label}
                          </span>
                          <span className="shrink-0 ui-text-micro ui-color-disabled">
                            {formatHistoryTime(entry.created_at)}
                          </span>
                        </div>
                        <div className="mt-1 line-clamp-2 ui-text-micro ui-color-muted">
                          {entry.transformed}
                        </div>
                      </button>
                      <button
                        type="button"
                        onClick={() => deleteHistoryMutation.mutate(entry.id)}
                        className="ui-button-ghost h-7 w-7 opacity-60 transition-opacity group-hover:opacity-100"
                        aria-label="Delete transform history item"
                      >
                        <Trash2 size={13} />
                      </button>
                    </article>
                  ))}
                </div>
              )}
            </div>
          </div>
        </section>

        <section className="flex min-h-0 flex-col rounded-lg border border-border-primary bg-surface-surface">
          <div className="flex min-h-14 items-center justify-between gap-3 border-b border-border-primary px-4">
            <div className="flex min-w-0 items-center gap-2">
              <button
                type="button"
                data-active={activeVariant === "original"}
                onClick={() => setActiveVariant("original")}
                className="ui-button-ghost h-8 rounded-full border border-border-primary px-3 ui-text-button-sm data-[active=true]:border-border-secondary"
              >
                {t({ id: "transforms.variant.original", message: "Original" })}
              </button>
              {variants.map((variant) => (
                <button
                  key={variantKey(variant)}
                  type="button"
                  data-active={activeVariant === variantKey(variant)}
                  onClick={() => setActiveVariant(variantKey(variant))}
                  className="ui-button-ghost h-8 rounded-full border border-border-primary px-3 ui-text-button-sm data-[active=true]:border-border-secondary"
                >
                  {variant.label}
                </button>
              ))}
            </div>
            <div className="flex shrink-0 items-center gap-1">
              <button
                type="button"
                data-active={reviewMode === "diff"}
                onClick={() => setReviewMode((mode) => (mode === "diff" ? "result" : "diff"))}
                disabled={!activeResult}
                className="ui-button-ghost h-8 w-8 disabled:opacity-40 data-[active=true]:border data-[active=true]:border-border-secondary"
                aria-label="Toggle diff review"
              >
                <SplitSquareHorizontal size={15} />
              </button>
              <button
                type="button"
                onClick={copySelected}
                disabled={!selectedText.trim()}
                className="ui-button-ghost h-8 w-8 disabled:opacity-40"
                aria-label={t({ id: "transforms.copy", message: "Copy selected variant" })}
              >
                {copied ? <Check size={15} /> : <Copy size={15} />}
              </button>
              <button
                type="button"
                onClick={saveSelected}
                disabled={!selectedText.trim() || scratchpadMutation.isPending}
                className="ui-button-ghost h-8 w-8 disabled:opacity-40"
                aria-label={t({ id: "transforms.save", message: "Save selected variant" })}
              >
                <Save size={15} />
              </button>
              <button
                type="button"
                onClick={pasteSelected}
                disabled={!selectedText.trim() || isPasting}
                className="ui-button-ghost h-8 w-8 disabled:opacity-40"
                aria-label={t({ id: "transforms.paste", message: "Paste selected variant" })}
                title={t({ id: "transforms.paste", message: "Paste selected variant" })}
              >
                <SendHorizontal size={15} className={isPasting ? "animate-pulse" : undefined} />
              </button>
            </div>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto p-4">
            {transformMutation.isPending ? (
              <div className="flex h-full min-h-72 flex-col items-center justify-center">
                <DotMatrix cols={12} rows={5} dotSize={2} gap={5} animated />
                <div className="mt-4 ui-text-body-sm ui-color-muted">
                  {t({ id: "transforms.loading", message: "Transforming..." })}
                </div>
              </div>
            ) : reviewMode === "diff" && activeResult ? (
              <div className="space-y-4">
                <div className="grid gap-3 md:grid-cols-2">
                  <div className="rounded-md border border-border-primary bg-surface-elevated p-3">
                    <div className="mb-2 ui-text-micro ui-color-disabled">Original</div>
                    <div className="max-h-52 overflow-y-auto whitespace-pre-wrap ui-text-body-sm ui-color-muted">
                      {activeResult.original}
                    </div>
                  </div>
                  <div className="rounded-md border border-border-primary bg-surface-elevated p-3">
                    <div className="mb-2 ui-text-micro ui-color-disabled">Result</div>
                    <div className="max-h-52 overflow-y-auto whitespace-pre-wrap ui-text-body-sm ui-color-primary">
                      {activeResult.transformed}
                    </div>
                  </div>
                </div>
                <div className="rounded-md border border-border-primary bg-[var(--surface-interactive)] p-3">
                  <div className="mb-2 ui-text-micro ui-color-disabled">Word changes</div>
                  <div className="max-h-72 overflow-y-auto ui-text-body-sm leading-6">
                    {diffTokens.map((token, index) => (
                      <span
                        key={`${token.type}-${index}-${token.value}`}
                        className={
                          token.type === "added"
                            ? "mx-0.5 rounded bg-[var(--color-success)]/15 px-1 ui-color-success-strong"
                            : token.type === "removed"
                              ? "mx-0.5 rounded bg-[var(--color-error)]/15 px-1 ui-color-error-soft line-through"
                              : "mx-0.5 ui-color-muted"
                        }
                      >
                        {token.value}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            ) : selectedText.trim() ? (
              <div className="whitespace-pre-wrap ui-text-body ui-color-primary">
                {selectedText}
              </div>
            ) : (
              <div className="flex h-full min-h-72 flex-col items-center justify-center px-6 text-center">
                <div className="flex h-11 w-11 items-center justify-center rounded-full border border-border-primary bg-surface-elevated">
                  <WandSparkles size={20} className="ui-color-muted" />
                </div>
                <div className="mt-4 ui-text-body-sm-strong ui-color-secondary">
                  {t({ id: "transforms.empty.title", message: "No text yet" })}
                </div>
                <div className="mt-1 max-w-sm ui-text-meta ui-color-muted">
                  {t({
                    id: "transforms.empty.body",
                    message: "Paste text on the left and choose a preset.",
                  })}
                </div>
              </div>
            )}
          </div>
          {transformMutation.error && (
            <div className="border-t border-border-primary px-4 py-3 ui-text-meta ui-color-error-soft">
              {transformMutation.error instanceof Error
                ? transformMutation.error.message
                : String(transformMutation.error)}
            </div>
          )}
          {sourceMutation.error && (
            <div className="border-t border-border-primary px-4 py-3 ui-text-meta ui-color-error-soft">
              {sourceMutation.error instanceof Error
                ? sourceMutation.error.message
                : String(sourceMutation.error)}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
