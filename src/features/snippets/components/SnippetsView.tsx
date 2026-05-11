import { useEffect, useMemo, useRef, useState, type ChangeEvent } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useLingui } from "@lingui/react/macro";
import { Check, Copy, Download, Pencil, Plus, Trash2, Upload, X } from "lucide-react";
import DotMatrix from "../../../shared/ui/DotMatrix";
import { useShiftHeld } from "../../../shared/hooks/useShiftHeld";
import {
  assertBulkImportFile,
  parseSnippetImport,
  SNIPPET_TRIGGER_MAX_LENGTH,
} from "../../../shared/lib/bulkImport";
import type { Snippet } from "../../../types";
import { useSettings } from "../../settings/queries";
import { useCreateSnippet, useDeleteSnippet, useSnippets, useUpdateSnippet } from "../queries";

type Draft = {
  id?: string;
  trigger: string;
  expansion: string;
};

const emptyDraft: Draft = { trigger: "", expansion: "" };
type SnippetLoadPayload = {
  expansion?: string;
};

export default function SnippetsView({ isActive = true }: { isActive?: boolean }) {
  const { t } = useLingui();
  const snippetsQuery = useSnippets(isActive);
  const settingsQuery = useSettings(undefined, isActive);
  const createMutation = useCreateSnippet();
  const updateMutation = useUpdateSnippet();
  const deleteMutation = useDeleteSnippet();
  const shiftHeld = useShiftHeld(isActive);
  const [draft, setDraft] = useState<Draft>(emptyDraft);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [backupStatus, setBackupStatus] = useState<string | null>(null);
  const [backupError, setBackupError] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);
  const importFileInputRef = useRef<HTMLInputElement>(null);

  const snippets = snippetsQuery.data ?? [];
  const settings = settingsQuery.data ?? null;
  const editing = Boolean(draft.id);
  const canSave = draft.trigger.trim().length > 0 && draft.expansion.trim().length > 0;
  const dictionaryConflictKeys = new Set(
    [
      ...(settings?.dictionary ?? []),
      ...(settings?.replacements ?? []).map((replacement) => replacement.from),
    ]
      .map((value) => value.trim().toLowerCase())
      .filter(Boolean),
  );

  const sortedSnippets = useMemo(
    () => [...snippets].sort((a, b) => a.trigger.localeCompare(b.trigger)),
    [snippets],
  );

  useEffect(() => {
    let cancelled = false;
    let unlisten: UnlistenFn | null = null;

    listen<SnippetLoadPayload>("snippets:load_expansion", (event) => {
      const expansion = event.payload?.expansion?.trim();
      if (!expansion || cancelled) return;
      setDraft({ trigger: "", expansion });
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const startEdit = (snippet: Snippet) => {
    setDraft({
      id: snippet.id,
      trigger: snippet.trigger,
      expansion: snippet.expansion,
    });
  };

  const resetDraft = () => setDraft(emptyDraft);

  const saveDraft = async () => {
    if (!canSave) return;
    if (dictionaryConflictKeys.has(draft.trigger.trim().toLowerCase())) {
      setBackupError("That trigger is already used in Dictionary.");
      return;
    }
    if (draft.id) {
      await updateMutation.mutateAsync({
        id: draft.id,
        trigger: draft.trigger,
        expansion: draft.expansion,
      });
    } else {
      await createMutation.mutateAsync({
        trigger: draft.trigger,
        expansion: draft.expansion,
      });
    }
    resetDraft();
  };

  const copyExpansion = async (snippet: Snippet) => {
    await navigator.clipboard.writeText(snippet.expansion);
    setCopiedId(snippet.id);
    window.setTimeout(() => setCopiedId(null), 1400);
  };

  const deleteSnippet = async (id: string) => {
    await deleteMutation.mutateAsync(id);
    if (draft.id === id) resetDraft();
  };

  const flashBackupStatus = (message: string) => {
    setBackupStatus(message);
    setBackupError(null);
    window.setTimeout(() => setBackupStatus(null), 2400);
  };

  const exportSnippets = async () => {
    const payload = {
      app: "Flow",
      type: "snippets",
      version: 1,
      exported_at: new Date().toISOString(),
      snippets: sortedSnippets.map(({ trigger, expansion }) => ({
        trigger,
        expansion,
      })),
    };
    await navigator.clipboard.writeText(JSON.stringify(payload, null, 2));
    flashBackupStatus(
      t({
        id: "snippets.backup.exported",
        message: "Snippets copied as JSON",
      }),
    );
  };

  const applySnippetImport = async (
    imported: ReturnType<typeof parseSnippetImport>,
    sourceLabel: "clipboard" | "file",
  ) => {
    const existingByTrigger = new Map(
      snippets.map((snippet) => [snippet.trigger.trim().toLowerCase(), snippet]),
    );
    let created = 0;
    let updated = 0;
    let conflictSkipped = 0;

    for (const item of imported.snippets) {
      if (dictionaryConflictKeys.has(item.trigger.toLowerCase())) {
        conflictSkipped += 1;
        continue;
      }
      const existing = existingByTrigger.get(item.trigger.toLowerCase());
      if (existing) {
        if (existing.expansion !== item.expansion || existing.trigger !== item.trigger) {
          await updateMutation.mutateAsync({
            id: existing.id,
            trigger: item.trigger,
            expansion: item.expansion,
          });
          updated += 1;
        }
      } else {
        await createMutation.mutateAsync(item);
        created += 1;
      }
    }

    flashBackupStatus(
      t({
        id: "snippets.backup.imported",
        message: `${sourceLabel === "file" ? "File import" : "Clipboard import"}: added ${created}, updated ${updated}, skipped ${imported.skipped + conflictSkipped}`,
      }),
    );
  };

  const importSnippets = async () => {
    if (importing) return;
    setImporting(true);
    setBackupError(null);
    setBackupStatus(null);
    try {
      const raw = await navigator.clipboard.readText();
      await applySnippetImport(parseSnippetImport(raw), "clipboard");
    } catch (error) {
      setBackupError(error instanceof Error ? error.message : String(error));
    } finally {
      setImporting(false);
    }
  };

  const importSnippetFile = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file || importing) return;

    setImporting(true);
    setBackupError(null);
    setBackupStatus(null);
    try {
      assertBulkImportFile(file, ["json"]);
      await applySnippetImport(parseSnippetImport(await file.text()), "file");
    } catch (error) {
      setBackupError(error instanceof Error ? error.message : String(error));
    } finally {
      setImporting(false);
    }
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="mb-5 flex flex-wrap items-end justify-between gap-3">
        <div>
          <div className="ui-text-section-label ui-color-muted">
            {t({ id: "snippets.eyebrow", message: "Snippets" })}
          </div>
          <h1 className="mt-1 ui-text-title font-medium ui-color-primary">
            {t({ id: "snippets.title", message: "Voice shortcuts" })}
          </h1>
          <p className="mt-1 max-w-xl ui-text-body-sm ui-color-muted">
            {t({
              id: "snippets.subtitle",
              message:
                "Say a trigger phrase during dictation and Flow expands it into reusable text.",
            })}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <input
            ref={importFileInputRef}
            type="file"
            accept=".json,application/json"
            className="hidden"
            onChange={importSnippetFile}
          />
          <button
            type="button"
            onClick={exportSnippets}
            disabled={snippets.length === 0}
            className="ui-button-ghost h-9 gap-2 rounded-full border border-border-primary px-4 ui-text-button-sm disabled:opacity-40"
          >
            <Download size={15} />
            {t({ id: "snippets.export", message: "Export" })}
          </button>
          <button
            type="button"
            onClick={shiftHeld ? importSnippets : () => importFileInputRef.current?.click()}
            title={
              shiftHeld
                ? "Import snippets from clipboard JSON"
                : "Import snippets from a Wispr-style JSON file"
            }
            disabled={importing}
            className="ui-button-ghost h-9 gap-2 rounded-full border border-border-primary px-4 ui-text-button-sm disabled:opacity-40"
          >
            <Upload size={15} />
            {importing
              ? t({ id: "snippets.importing", message: "Importing" })
              : t({ id: "snippets.import", message: "Import" })}
          </button>
          <button
            type="button"
            onClick={resetDraft}
            className="ui-button-ghost h-9 gap-2 rounded-full border border-border-primary px-4 ui-text-button-sm"
          >
            <Plus size={16} />
            {t({ id: "snippets.new", message: "New snippet" })}
          </button>
        </div>
      </div>

      {(backupStatus || backupError) && (
        <div
          className={`mb-3 rounded-lg border px-3 py-2 ui-text-body-sm ${
            backupError
              ? "border-red-500/30 bg-red-500/10 ui-color-error-soft"
              : "border-border-primary bg-surface-surface ui-color-secondary"
          }`}
          role="status"
        >
          {backupError ?? backupStatus}
        </div>
      )}

      <div className="grid min-h-0 flex-1 grid-cols-[minmax(280px,380px)_1fr] gap-4">
        <section className="flex min-h-0 flex-col rounded-lg border border-border-primary bg-surface-surface">
          <div className="border-b border-border-primary px-4 py-3">
            <div className="ui-text-body-sm-strong ui-color-primary">
              {editing
                ? t({ id: "snippets.form.edit_title", message: "Edit snippet" })
                : t({ id: "snippets.form.new_title", message: "Create snippet" })}
            </div>
            <div className="mt-1 ui-text-meta ui-color-muted">
              {t({
                id: "snippets.form.hint",
                message: "Triggers can be up to 59 characters.",
              })}
            </div>
          </div>
          <div className="space-y-4 p-4">
            <label className="block">
              <span className="ui-text-meta-strong ui-color-muted">
                {t({ id: "snippets.trigger", message: "Trigger" })}
              </span>
              <input
                value={draft.trigger}
                maxLength={SNIPPET_TRIGGER_MAX_LENGTH}
                onChange={(event) =>
                  setDraft((current) => ({ ...current, trigger: event.target.value }))
                }
                placeholder={t({
                  id: "snippets.trigger_placeholder",
                  message: "my email address",
                })}
                className="mt-2 h-10 w-full rounded-md border border-border-primary bg-surface-elevated px-3 ui-text-input ui-color-primary placeholder:text-content-disabled focus:outline-none focus:ring-2 focus:ring-[var(--border-strong)]"
              />
            </label>
            <label className="block">
              <span className="ui-text-meta-strong ui-color-muted">
                {t({ id: "snippets.expansion", message: "Expansion" })}
              </span>
              <textarea
                value={draft.expansion}
                onChange={(event) =>
                  setDraft((current) => ({ ...current, expansion: event.target.value }))
                }
                placeholder={t({
                  id: "snippets.expansion_placeholder",
                  message: "me@example.com",
                })}
                className="mt-2 h-36 w-full resize-none rounded-md border border-border-primary bg-surface-elevated px-3 py-2 ui-text-input ui-color-primary placeholder:text-content-disabled focus:outline-none focus:ring-2 focus:ring-[var(--border-strong)]"
              />
            </label>
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={saveDraft}
                disabled={!canSave || createMutation.isPending || updateMutation.isPending}
                className="inline-flex h-9 items-center gap-2 rounded-full border border-border-secondary bg-[var(--surface-interactive-strong)] px-4 ui-text-button-sm ui-color-primary transition-colors hover:bg-[var(--surface-interactive-pressed)] disabled:opacity-40"
              >
                <Check size={15} />
                {editing
                  ? t({ id: "snippets.update", message: "Update" })
                  : t({ id: "snippets.create", message: "Create" })}
              </button>
              {editing && (
                <button
                  type="button"
                  onClick={resetDraft}
                  className="ui-button-ghost h-9 gap-2 rounded-full border border-border-primary px-4 ui-text-button-sm"
                >
                  <X size={15} />
                  {t({ id: "snippets.cancel", message: "Cancel" })}
                </button>
              )}
            </div>
          </div>
        </section>

        <section className="flex min-h-0 flex-col rounded-lg border border-border-primary bg-surface-surface">
          <div className="flex h-12 items-center justify-between border-b border-border-primary px-4">
            <div className="ui-text-body-sm-strong ui-color-primary">
              {t({ id: "snippets.saved", message: "Saved snippets" })}
            </div>
            <div className="ui-text-meta ui-color-muted">
              {snippets.length} {t({ id: "snippets.count_suffix", message: "total" })}
            </div>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto p-2">
            {snippetsQuery.isLoading ? (
              <div className="flex h-44 items-center justify-center">
                <DotMatrix cols={10} rows={4} dotSize={2} gap={5} />
              </div>
            ) : sortedSnippets.length === 0 ? (
              <div className="flex h-56 flex-col items-center justify-center px-6 text-center">
                <Copy size={22} className="ui-color-disabled" />
                <div className="mt-3 ui-text-body-sm-strong ui-color-secondary">
                  {t({ id: "snippets.empty.title", message: "No snippets yet" })}
                </div>
                <div className="mt-1 ui-text-meta ui-color-muted">
                  {t({
                    id: "snippets.empty.body",
                    message: "Create one, then say its trigger phrase inside dictation.",
                  })}
                </div>
              </div>
            ) : (
              <div className="space-y-2">
                {sortedSnippets.map((snippet) => (
                  <article
                    key={snippet.id}
                    className="rounded-md border border-border-primary bg-[var(--surface-interactive)] p-3"
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0">
                        <div className="truncate ui-text-body-sm-strong ui-color-primary">
                          {snippet.trigger}
                        </div>
                        <div className="mt-1 whitespace-pre-wrap ui-text-body-sm ui-color-muted">
                          {snippet.expansion}
                        </div>
                      </div>
                      <div className="flex shrink-0 items-center gap-1">
                        <button
                          type="button"
                          onClick={() => copyExpansion(snippet)}
                          className="ui-button-ghost h-8 w-8"
                          aria-label={t({
                            id: "snippets.copy",
                            message: "Copy snippet expansion",
                          })}
                        >
                          {copiedId === snippet.id ? <Check size={15} /> : <Copy size={15} />}
                        </button>
                        <button
                          type="button"
                          onClick={() => startEdit(snippet)}
                          className="ui-button-ghost h-8 w-8"
                          aria-label={t({ id: "snippets.edit", message: "Edit snippet" })}
                        >
                          <Pencil size={15} />
                        </button>
                        <button
                          type="button"
                          onClick={() => deleteSnippet(snippet.id)}
                          className="ui-button-ghost h-8 w-8"
                          aria-label={t({ id: "snippets.delete", message: "Delete snippet" })}
                        >
                          <Trash2 size={15} />
                        </button>
                      </div>
                    </div>
                  </article>
                ))}
              </div>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
