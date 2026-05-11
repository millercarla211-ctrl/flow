import { useMemo, useState } from "react";
import { useLingui } from "@lingui/react/macro";
import { Check, Copy, RefreshCw, Save, WandSparkles } from "lucide-react";
import DotMatrix from "../../../shared/ui/DotMatrix";
import { useCreateScratchpadEntry } from "../../scratchpad/queries";
import { useTransformPresets, useTransformText } from "../queries";
import type { TransformResult } from "../../../types";

const sampleText =
  "Hey, I wanted to quickly follow up about the thing we discussed yesterday. I think we should make the plan clearer and easier for everyone to act on.";

export default function TransformsView({ isActive = true }: { isActive?: boolean }) {
  const { t } = useLingui();
  const presetsQuery = useTransformPresets(isActive);
  const transformMutation = useTransformText();
  const scratchpadMutation = useCreateScratchpadEntry();
  const [sourceText, setSourceText] = useState("");
  const [customInstruction, setCustomInstruction] = useState("");
  const [activeVariant, setActiveVariant] = useState<"original" | string>("original");
  const [variants, setVariants] = useState<TransformResult[]>([]);
  const [copied, setCopied] = useState(false);
  const presets = presetsQuery.data ?? [];

  const selectedText = useMemo(() => {
    if (activeVariant === "original") return sourceText;
    return variants.find((variant) => variant.label === activeVariant)?.transformed ?? sourceText;
  }, [activeVariant, sourceText, variants]);

  const canTransform = sourceText.trim().length > 0 && !transformMutation.isPending;

  const runPreset = async (presetId: string) => {
    if (!canTransform) return;
    const result = await transformMutation.mutateAsync({
      text: sourceText,
      presetId,
    });
    setVariants((current) => [
      result,
      ...current.filter((variant) => variant.label !== result.label),
    ]);
    setActiveVariant(result.label);
  };

  const runCustom = async () => {
    if (!canTransform || !customInstruction.trim()) return;
    const result = await transformMutation.mutateAsync({
      text: sourceText,
      instruction: customInstruction,
    });
    setVariants((current) => [
      result,
      ...current.filter((variant) => variant.label !== result.label),
    ]);
    setActiveVariant(result.label);
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

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="mb-5">
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
              "Use tone presets or a custom instruction to rewrite text, then copy it or keep it in Scratchpad.",
          })}
        </p>
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
                message: "Paste text here for now; selected-text hotkeys come next.",
              })}
            </div>
          </div>
          <div className="flex min-h-0 flex-1 flex-col p-4">
            <textarea
              value={sourceText}
              onChange={(event) => {
                setSourceText(event.target.value);
                setActiveVariant("original");
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
                  key={variant.label}
                  type="button"
                  data-active={activeVariant === variant.label}
                  onClick={() => setActiveVariant(variant.label)}
                  className="ui-button-ghost h-8 rounded-full border border-border-primary px-3 ui-text-button-sm data-[active=true]:border-border-secondary"
                >
                  {variant.label}
                </button>
              ))}
            </div>
            <div className="flex shrink-0 items-center gap-1">
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
        </section>
      </div>
    </div>
  );
}
