import { useEffect, useMemo, useRef, useState, type MouseEvent } from "react";
import { useLingui } from "@lingui/react/macro";
import { motion, AnimatePresence } from "framer-motion";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useQueryClient } from "@tanstack/react-query";
import {
  BookPlus,
  Check,
  Clipboard,
  Copy,
  FileText,
  History,
  Plus,
  RotateCcw,
  Search,
  Trash2,
} from "lucide-react";
import DotMatrix from "../../../shared/ui/DotMatrix";
import { useDebouncedValue } from "../../../shared/hooks/useDebouncedValue";
import type { ScratchpadEntry, ScratchpadVersion } from "../../../types";
import {
  useCreateScratchpadEntry,
  useDeleteScratchpadEntry,
  useScratchpadEntries,
  useScratchpadVersions,
  useUpdateScratchpadEntry,
} from "../queries";
import { setDictionaryEntriesCache } from "../../dictionary/queries";

const formatRelativeTime = (value: string) => {
  const date = new Date(value);
  const deltaMs = Date.now() - date.getTime();
  if (Number.isNaN(deltaMs)) return "";
  const minutes = Math.max(0, Math.round(deltaMs / 60_000));
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return date.toLocaleDateString([], { month: "short", day: "numeric" });
};

const formatDetailedTime = (value: string) => {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleString([], {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
};

const sourceLabel = (source: string) => {
  switch (source) {
    case "local":
      return "Paste fallback";
    case "streaming":
      return "Streaming fallback";
    case "manual":
      return "Manual";
    default:
      return source;
  }
};

const dictionaryWordPattern = /[A-Za-z0-9'_-]/;

const cleanDictionaryCandidate = (value: string) =>
  value
    .replace(/\s+/g, " ")
    .replace(/^[^\w'-]+|[^\w'-]+$/g, "")
    .trim();

const truncateMenuText = (value: string) =>
  value.length > 34 ? `${value.slice(0, 31).trimEnd()}...` : value;

export default function ScratchpadView({ isActive = true }: { isActive?: boolean }) {
  const { t } = useLingui();
  const queryClient = useQueryClient();
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [draftTitle, setDraftTitle] = useState("");
  const [draftBody, setDraftBody] = useState("");
  const [isCreating, setIsCreating] = useState(false);
  const [copied, setCopied] = useState(false);
  const [showVersions, setShowVersions] = useState(false);
  const [dictionaryMenu, setDictionaryMenu] = useState<{
    x: number;
    y: number;
    text: string;
  } | null>(null);
  const [dictionaryNotice, setDictionaryNotice] = useState("");
  const editorRef = useRef<HTMLTextAreaElement>(null);
  const debouncedSearch = useDebouncedValue(searchQuery, 200);

  const entriesQuery = useScratchpadEntries(debouncedSearch, isActive);
  const entries = entriesQuery.data ?? [];
  const selectedEntry = useMemo(
    () => entries.find((entry) => entry.id === selectedId) ?? null,
    [entries, selectedId],
  );
  const versionsQuery = useScratchpadVersions(
    selectedEntry?.id ?? null,
    isActive && showVersions && Boolean(selectedEntry),
  );
  const createMutation = useCreateScratchpadEntry();
  const updateMutation = useUpdateScratchpadEntry();
  const deleteMutation = useDeleteScratchpadEntry();

  const versions = versionsQuery.data ?? [];

  const getEditorDictionaryCandidate = () => {
    const editor = editorRef.current;
    if (!editor) return "";

    const { selectionStart, selectionEnd, value } = editor;
    if (selectionStart !== selectionEnd) {
      return cleanDictionaryCandidate(value.slice(selectionStart, selectionEnd));
    }

    let start = selectionStart;
    let end = selectionEnd;

    while (start > 0 && dictionaryWordPattern.test(value[start - 1])) {
      start -= 1;
    }
    while (end < value.length && dictionaryWordPattern.test(value[end])) {
      end += 1;
    }

    return cleanDictionaryCandidate(value.slice(start, end));
  };

  const handleEditorContextMenu = (event: MouseEvent<HTMLTextAreaElement>) => {
    const candidate = getEditorDictionaryCandidate();
    if (!candidate) {
      setDictionaryMenu(null);
      return;
    }

    event.preventDefault();
    setDictionaryNotice("");
    setDictionaryMenu({
      x: event.clientX,
      y: event.clientY,
      text: candidate,
    });
  };

  const handleAddDictionaryCandidate = async () => {
    if (!dictionaryMenu?.text) return;

    const entries = await invoke<string[]>("add_dictionary_entries", {
      entries: [dictionaryMenu.text],
    });
    setDictionaryEntriesCache(queryClient, entries);
    setDictionaryNotice(
      t({
        id: "scratchpad.dictionary.added",
        message: "Added to Dictionary",
      }),
    );
    setDictionaryMenu(null);
    window.setTimeout(() => setDictionaryNotice(""), 1800);
  };

  useEffect(() => {
    if (!dictionaryMenu) return;

    const closeMenu = () => setDictionaryMenu(null);
    window.addEventListener("click", closeMenu);
    window.addEventListener("scroll", closeMenu, true);

    return () => {
      window.removeEventListener("click", closeMenu);
      window.removeEventListener("scroll", closeMenu, true);
    };
  }, [dictionaryMenu]);

  useEffect(() => {
    let cancelled = false;
    let unlistenEntryCreated: UnlistenFn | null = null;

    listen<ScratchpadEntry>("scratchpad:entry-created", (event) => {
      const entry = event.payload;
      if (entry.source !== "local" && entry.source !== "streaming") return;

      setIsCreating(false);
      setCopied(false);
      setShowVersions(false);
      setSelectedId(entry.id);
      setDraftTitle(entry.title);
      setDraftBody(entry.body);
    }).then((fn) => {
      if (cancelled) fn();
      else unlistenEntryCreated = fn;
    });

    return () => {
      cancelled = true;
      unlistenEntryCreated?.();
    };
  }, []);

  useEffect(() => {
    if (!isActive || selectedId || entries.length === 0 || isCreating) return;
    setSelectedId(entries[0].id);
  }, [entries, isActive, isCreating, selectedId]);

  useEffect(() => {
    if (!selectedEntry || isCreating) return;
    setDraftTitle(selectedEntry.title);
    setDraftBody(selectedEntry.body);
  }, [selectedEntry, isCreating]);

  const hasDraft = draftBody.trim().length > 0;
  const hasChanges = selectedEntry
    ? draftTitle.trim() !== selectedEntry.title || draftBody.trim() !== selectedEntry.body
    : hasDraft;

  const handleNew = () => {
    setSelectedId(null);
    setIsCreating(true);
    setDraftTitle("");
    setDraftBody("");
    setCopied(false);
    setShowVersions(false);
  };

  const handleSelect = (entry: ScratchpadEntry) => {
    setIsCreating(false);
    setCopied(false);
    setShowVersions(false);
    setSelectedId(entry.id);
  };

  const handleSave = async () => {
    if (!hasDraft) return;
    if (isCreating || !selectedEntry) {
      const created = await createMutation.mutateAsync({ body: draftBody, source: "manual" });
      setIsCreating(false);
      setSelectedId(created.id);
      return;
    }
    await updateMutation.mutateAsync({
      id: selectedEntry.id,
      title: draftTitle,
      body: draftBody,
    });
  };

  const handleDelete = async () => {
    if (!selectedEntry) return;
    await deleteMutation.mutateAsync(selectedEntry.id);
    setSelectedId(null);
    setDraftTitle("");
    setDraftBody("");
    setIsCreating(false);
  };

  const handleCopy = async () => {
    if (!draftBody.trim()) return;
    await navigator.clipboard.writeText(draftBody);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  };

  const handleRestoreVersion = (version: ScratchpadVersion) => {
    setDraftBody(version.body);
  };

  const empty = !entriesQuery.isLoading && entries.length === 0;

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="mb-5 flex flex-wrap items-end justify-between gap-3">
        <div>
          <div className="ui-text-section-label ui-color-muted">
            {t({ id: "scratchpad.eyebrow", message: "Scratchpad" })}
          </div>
          <h1 className="mt-1 ui-text-title font-medium ui-color-primary">
            {t({ id: "scratchpad.title", message: "Saved dictation" })}
          </h1>
          <p className="mt-1 max-w-xl ui-text-body-sm ui-color-muted">
            {t({
              id: "scratchpad.subtitle",
              message:
                "Paste failures and quick notes live here locally, so dictated text stays recoverable.",
            })}
          </p>
        </div>
        <button
          type="button"
          onClick={handleNew}
          className="ui-button-ghost h-9 gap-2 rounded-full border border-border-primary px-4 ui-text-button-sm"
        >
          <Plus size={16} />
          {t({ id: "scratchpad.new", message: "New note" })}
        </button>
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-[minmax(220px,320px)_1fr] gap-4">
        <section className="flex min-h-0 flex-col rounded-lg border border-border-primary bg-surface-surface">
          <div className="flex h-12 items-center gap-2 border-b border-border-primary px-3">
            <Search size={15} className="ui-color-disabled" />
            <input
              value={searchQuery}
              onChange={(event) => setSearchQuery(event.target.value)}
              placeholder={t({ id: "scratchpad.search", message: "Search notes" })}
              className="h-full min-w-0 flex-1 bg-transparent ui-text-input ui-color-primary placeholder:text-content-disabled focus:outline-none"
            />
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto p-2">
            {entriesQuery.isLoading ? (
              <div className="flex h-36 items-center justify-center">
                <DotMatrix cols={10} rows={4} dotSize={2} gap={5} />
              </div>
            ) : empty ? (
              <div className="flex h-52 flex-col items-center justify-center px-6 text-center">
                <FileText size={22} className="ui-color-disabled" />
                <div className="mt-3 ui-text-body-sm-strong ui-color-secondary">
                  {t({ id: "scratchpad.empty.title", message: "No notes yet" })}
                </div>
                <div className="mt-1 ui-text-meta ui-color-muted">
                  {t({
                    id: "scratchpad.empty.body",
                    message: "Flow will save here when paste cannot reach the focused app.",
                  })}
                </div>
              </div>
            ) : (
              <div className="space-y-1">
                {entries.map((entry) => (
                  <button
                    key={entry.id}
                    type="button"
                    onClick={() => handleSelect(entry)}
                    data-active={entry.id === selectedId}
                    className="ui-row-hover w-full rounded-md border border-transparent p-3 text-left data-[active=true]:border-border-secondary data-[active=true]:bg-[var(--surface-interactive-strong)]"
                  >
                    <div className="flex items-center justify-between gap-3">
                      <div className="truncate ui-text-body-sm-strong ui-color-primary">
                        {entry.title}
                      </div>
                      <div className="shrink-0 ui-text-micro ui-color-disabled">
                        {formatRelativeTime(entry.updated_at)}
                      </div>
                    </div>
                    <div className="mt-1 line-clamp-2 ui-text-meta ui-color-muted">
                      {entry.body}
                    </div>
                    <div className="mt-2 flex items-center gap-2 ui-text-micro ui-color-disabled">
                      <span>{sourceLabel(entry.source)}</span>
                      <span>v{entry.version}</span>
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>
        </section>

        <section className="flex min-h-0 flex-col rounded-lg border border-border-primary bg-surface-surface">
          <div className="flex min-h-14 items-center justify-between gap-3 border-b border-border-primary px-4">
            <input
              value={draftTitle}
              onChange={(event) => setDraftTitle(event.target.value)}
              placeholder={t({ id: "scratchpad.editor.title_placeholder", message: "Untitled" })}
              className="min-w-0 flex-1 bg-transparent ui-text-body-lg ui-color-primary placeholder:text-content-disabled focus:outline-none"
            />
            <div className="flex items-center gap-2">
              {selectedEntry && (
                <button
                  type="button"
                  onClick={() => setShowVersions((value) => !value)}
                  data-active={showVersions}
                  className="ui-button-ghost h-8 w-8 disabled:opacity-40 data-[active=true]:border data-[active=true]:border-border-secondary"
                  aria-label={t({ id: "scratchpad.versions", message: "Show version history" })}
                >
                  <History size={16} />
                </button>
              )}
              <button
                type="button"
                onClick={handleCopy}
                disabled={!hasDraft}
                className="ui-button-ghost h-8 w-8 disabled:opacity-40"
                aria-label={t({ id: "scratchpad.copy", message: "Copy note" })}
              >
                {copied ? <Check size={16} /> : <Copy size={16} />}
              </button>
              {selectedEntry && (
                <button
                  type="button"
                  onClick={handleDelete}
                  disabled={deleteMutation.isPending}
                  className="ui-button-ghost h-8 w-8 disabled:opacity-40"
                  aria-label={t({ id: "scratchpad.delete", message: "Delete note" })}
                >
                  <Trash2 size={16} />
                </button>
              )}
              <button
                type="button"
                onClick={handleSave}
                disabled={
                  !hasChanges || !hasDraft || updateMutation.isPending || createMutation.isPending
                }
                className="inline-flex h-8 items-center gap-2 rounded-full border border-border-secondary bg-[var(--surface-interactive-strong)] px-3 ui-text-button-sm ui-color-primary transition-colors hover:bg-[var(--surface-interactive-pressed)] disabled:opacity-40"
              >
                <Clipboard size={14} />
                {isCreating || !selectedEntry
                  ? t({ id: "scratchpad.create", message: "Create" })
                  : t({ id: "scratchpad.save", message: "Save" })}
              </button>
            </div>
          </div>

          <div className="min-h-0 flex-1 p-4">
            <textarea
              ref={editorRef}
              value={draftBody}
              onChange={(event) => setDraftBody(event.target.value)}
              onContextMenu={handleEditorContextMenu}
              spellCheck
              placeholder={t({
                id: "scratchpad.editor.body_placeholder",
                message: "Dictate or type here...",
              })}
              className="h-full w-full resize-none rounded-md border border-transparent bg-transparent p-0 ui-text-body ui-color-primary placeholder:text-content-disabled focus:outline-none"
            />
          </div>

          <AnimatePresence>
            {dictionaryMenu && (
              <motion.div
                initial={{ opacity: 0, scale: 0.96, y: -3 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.96, y: -3 }}
                transition={{ duration: 0.12 }}
                className="ui-surface-menu fixed z-[200] min-w-[210px] overflow-hidden"
                style={{ left: dictionaryMenu.x, top: dictionaryMenu.y }}
                onClick={(event) => event.stopPropagation()}
              >
                <button
                  type="button"
                  onClick={handleAddDictionaryCandidate}
                  className="flex w-full items-center gap-2.5 px-3 py-2 text-left ui-text-menu-item ui-color-secondary transition-colors hover:bg-surface-elevated"
                >
                  <BookPlus size={13} className="text-content-muted" />
                  <span className="min-w-0 truncate">
                    {t({
                      id: "scratchpad.dictionary.add",
                      message: "Add to Dictionary",
                    })}
                    : {truncateMenuText(dictionaryMenu.text)}
                  </span>
                </button>
              </motion.div>
            )}

            {showVersions && selectedEntry && (
              <motion.div
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 8 }}
                className="max-h-56 overflow-y-auto border-t border-border-primary bg-[var(--surface-interactive)] px-4 py-3"
              >
                <div className="mb-3 flex items-center justify-between gap-3">
                  <div>
                    <div className="ui-text-meta-strong ui-color-primary">Version history</div>
                    <div className="ui-text-micro ui-color-muted">
                      Restore a previous body into the editor, then save when ready.
                    </div>
                  </div>
                  <div className="ui-text-micro ui-color-disabled">v{selectedEntry.version}</div>
                </div>

                {versionsQuery.isLoading ? (
                  <div className="flex h-20 items-center justify-center">
                    <DotMatrix cols={10} rows={4} dotSize={2} gap={5} />
                  </div>
                ) : versions.length === 0 ? (
                  <div className="rounded-md border border-border-primary bg-surface-elevated px-3 py-6 text-center ui-text-meta ui-color-muted">
                    No saved versions yet.
                  </div>
                ) : (
                  <div className="space-y-2">
                    {versions.map((version) => (
                      <article
                        key={version.id}
                        className="grid grid-cols-[1fr_auto] gap-3 rounded-md border border-border-primary bg-surface-surface p-3"
                      >
                        <div className="min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="ui-text-meta-strong ui-color-primary">
                              v{version.version}
                            </span>
                            <span className="ui-text-micro ui-color-disabled">
                              {formatDetailedTime(version.created_at)}
                            </span>
                          </div>
                          <div className="mt-1 line-clamp-2 ui-text-meta ui-color-muted">
                            {version.body}
                          </div>
                        </div>
                        <button
                          type="button"
                          onClick={() => handleRestoreVersion(version)}
                          disabled={draftBody.trim() === version.body.trim()}
                          className="ui-button-ghost h-8 gap-2 rounded-full border border-border-primary px-3 ui-text-button-sm disabled:opacity-40"
                        >
                          <RotateCcw size={14} />
                          Restore
                        </button>
                      </article>
                    ))}
                  </div>
                )}
              </motion.div>
            )}

            {dictionaryNotice && (
              <motion.div
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 8 }}
                className="border-t border-border-primary px-4 py-3 ui-text-meta ui-color-secondary"
              >
                {dictionaryNotice}
              </motion.div>
            )}

            {hasChanges && hasDraft && (
              <motion.div
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 8 }}
                className="border-t border-border-primary px-4 py-3 ui-text-meta ui-color-muted"
              >
                {t({
                  id: "scratchpad.unsaved",
                  message: "Unsaved changes are local until you save.",
                })}
              </motion.div>
            )}
          </AnimatePresence>
        </section>
      </div>
    </div>
  );
}
