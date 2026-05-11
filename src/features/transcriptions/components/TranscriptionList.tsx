import { useLingui } from "@lingui/react/macro";
import React, { useState, useCallback, useMemo, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import {
  Search,
  X,
  ArrowDownUp,
  Check,
  CheckSquare,
  Copy,
  Filter,
  FileText,
  Trash2,
  WandSparkles,
} from "lucide-react";
import { Virtuoso } from "react-virtuoso";
import {
  useTranscriptionList,
  useDeleteTranscription,
  useRetryTranscription,
  useRetryLlmCleanup,
  useUndoLlmCleanup,
} from "../queries";
import TranscriptionItem from "./TranscriptionItem";
import DotMatrix from "../../../shared/ui/DotMatrix";
import { useDebouncedValue } from "../../../shared/hooks/useDebouncedValue";
import { useShiftHeld } from "../../../shared/hooks/useShiftHeld";
import { useClickOutside } from "../../../shared/hooks/useClickOutside";
import type { TranscriptionRecord } from "../../../types";

interface TranscriptionListProps {
  showLlmButtons?: boolean;
  isActive?: boolean;
}

type SortKey = "recent" | "oldest" | "longest" | "shortest";
type FilterKey = "all" | "success" | "failed" | "paste_fallback" | "cleaned";

type ListEntry =
  | { type: "header"; id: string; label: string }
  | { type: "item"; record: TranscriptionRecord };

const startOfDay = (date: Date) => new Date(date.getFullYear(), date.getMonth(), date.getDate());

const TranscriptionList: React.FC<TranscriptionListProps> = ({
  showLlmButtons = false,
  isActive = true,
}) => {
  const { t } = useLingui();
  const [searchQuery, setSearchQuery] = useState("");
  const [searchOpen, setSearchOpen] = useState(false);
  const [sortKey, setSortKey] = useState<SortKey>("recent");
  const [sortOpen, setSortOpen] = useState(false);
  const [filterKey, setFilterKey] = useState<FilterKey>("all");
  const [filterOpen, setFilterOpen] = useState(false);
  const [selectionMode, setSelectionMode] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() => new Set());
  const [batchStatus, setBatchStatus] = useState<string | null>(null);
  const [batchError, setBatchError] = useState<string | null>(null);
  const [batchBusy, setBatchBusy] = useState<"copy" | "scratchpad" | "transform" | "delete" | null>(
    null,
  );
  const sortRef = useRef<HTMLDivElement>(null);
  const filterRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const debouncedSearchQuery = useDebouncedValue(searchQuery, 300);
  const shiftHeld = useShiftHeld(isActive);

  useClickOutside(sortRef, () => setSortOpen(false), sortOpen);
  useClickOutside(filterRef, () => setFilterOpen(false), filterOpen);

  useEffect(() => {
    if (!searchOpen) return;
    const id = requestAnimationFrame(() => {
      searchInputRef.current?.focus();
    });
    return () => cancelAnimationFrame(id);
  }, [searchOpen]);

  const {
    data: transcriptions = [],
    isLoading,
    isFetched,
  } = useTranscriptionList(debouncedSearchQuery, isActive);
  const totalCount = transcriptions.length;
  const deleteMutation = useDeleteTranscription();
  const {
    retry: retryMutation,
    cancelRetry: cancelRetryMutation,
    retryingIds,
  } = useRetryTranscription(isActive);
  const retryLlmMutation = useRetryLlmCleanup();
  const undoLlmMutation = useUndoLlmCleanup();
  const retryingIdSet = useMemo(() => new Set(retryingIds), [retryingIds]);
  const filterCounts = useMemo(
    () =>
      transcriptions.reduce(
        (counts, record) => {
          counts.all += 1;
          if (record.status === "success") counts.success += 1;
          if (record.status === "error") counts.failed += 1;
          if (record.auto_paste_requested && record.auto_paste_succeeded === false) {
            counts.paste_fallback += 1;
          }
          if (record.llm_cleaned) counts.cleaned += 1;
          return counts;
        },
        {
          all: 0,
          success: 0,
          failed: 0,
          paste_fallback: 0,
          cleaned: 0,
        } satisfies Record<FilterKey, number>,
      ),
    [transcriptions],
  );

  const filteredTranscriptions = useMemo(() => {
    switch (filterKey) {
      case "success":
        return transcriptions.filter((record) => record.status === "success");
      case "failed":
        return transcriptions.filter((record) => record.status === "error");
      case "paste_fallback":
        return transcriptions.filter(
          (record) => record.auto_paste_requested && record.auto_paste_succeeded === false,
        );
      case "cleaned":
        return transcriptions.filter((record) => record.llm_cleaned);
      case "all":
      default:
        return transcriptions;
    }
  }, [transcriptions, filterKey]);

  const sortedTranscriptions = useMemo(() => {
    if (sortKey === "recent") return filteredTranscriptions;
    const copy = [...filteredTranscriptions];
    switch (sortKey) {
      case "oldest":
        copy.sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());
        break;
      case "longest":
        copy.sort((a, b) => (b.word_count ?? 0) - (a.word_count ?? 0));
        break;
      case "shortest":
        copy.sort((a, b) => (a.word_count ?? 0) - (b.word_count ?? 0));
        break;
    }
    return copy;
  }, [filteredTranscriptions, sortKey]);

  const isTimeSorted = sortKey === "recent" || sortKey === "oldest";
  const selectedRecords = useMemo(
    () => sortedTranscriptions.filter((record) => selectedIds.has(record.id)),
    [selectedIds, sortedTranscriptions],
  );
  const selectedTextRecords = useMemo(
    () =>
      selectedRecords.filter(
        (record) => record.status === "success" && record.text.trim().length > 0,
      ),
    [selectedRecords],
  );
  const allVisibleSelected =
    sortedTranscriptions.length > 0 &&
    sortedTranscriptions.every((record) => selectedIds.has(record.id));
  const selectedBatchText = useMemo(
    () => selectedTextRecords.map((record) => record.text.trim()).join("\n\n"),
    [selectedTextRecords],
  );

  useEffect(() => {
    if (selectedIds.size === 0) return;
    const visibleIds = new Set(transcriptions.map((record) => record.id));
    setSelectedIds((current) => {
      const next = new Set(Array.from(current).filter((id) => visibleIds.has(id)));
      return next.size === current.size ? current : next;
    });
  }, [selectedIds.size, transcriptions]);

  const formatGroupLabel = useCallback(
    (date: Date) => {
      const now = new Date();
      const today = startOfDay(now);
      const target = startOfDay(date);
      const diffDays = Math.round((today.getTime() - target.getTime()) / 86400000);
      if (diffDays === 0) return t({ id: "transcriptions.group.today", message: "Today" });
      if (diffDays === 1)
        return t({
          id: "transcriptions.group.yesterday",
          message: "Yesterday",
        });
      if (diffDays > 1 && diffDays < 7) {
        return target.toLocaleDateString([], { weekday: "long" });
      }
      if (target.getFullYear() === now.getFullYear()) {
        return target.toLocaleDateString([], {
          month: "long",
          day: "numeric",
        });
      }
      return target.toLocaleDateString([], {
        month: "short",
        day: "numeric",
        year: "numeric",
      });
    },
    [t],
  );

  const entries: ListEntry[] = useMemo(() => {
    if (!isTimeSorted) {
      return sortedTranscriptions.map((record) => ({
        type: "item" as const,
        record,
      }));
    }
    const result: ListEntry[] = [];
    let currentLabel: string | null = null;
    for (const record of sortedTranscriptions) {
      const label = formatGroupLabel(new Date(record.timestamp));
      if (label !== currentLabel) {
        result.push({
          type: "header",
          id: `h-${label}-${record.id}`,
          label,
        });
        currentLabel = label;
      }
      result.push({ type: "item", record });
    }
    return result;
  }, [sortedTranscriptions, isTimeSorted, formatGroupLabel]);

  const deleteTranscription = useCallback(
    async (id: string) => {
      await deleteMutation.mutateAsync(id);
    },
    [deleteMutation],
  );

  const retryTranscription = useCallback(
    async (id: string) => {
      await retryMutation.mutateAsync(id);
    },
    [retryMutation],
  );

  const cancelRetryTranscription = useCallback(
    async (id: string) => {
      await cancelRetryMutation.mutateAsync(id);
    },
    [cancelRetryMutation],
  );

  const retryLlmCleanup = useCallback(
    async (id: string) => {
      await retryLlmMutation.mutateAsync(id);
    },
    [retryLlmMutation],
  );

  const undoLlmCleanup = useCallback(
    async (id: string) => {
      await undoLlmMutation.mutateAsync(id);
    },
    [undoLlmMutation],
  );

  const flashBatchStatus = (message: string) => {
    setBatchStatus(message);
    setBatchError(null);
    window.setTimeout(() => setBatchStatus(null), 2400);
  };

  const clearSelection = () => {
    setSelectedIds(new Set());
    setSelectionMode(false);
  };

  const toggleRecordSelection = useCallback((id: string, selected: boolean) => {
    setSelectedIds((current) => {
      const next = new Set(current);
      if (selected) next.add(id);
      else next.delete(id);
      return next;
    });
  }, []);

  const toggleAllVisible = () => {
    setSelectedIds((current) => {
      if (allVisibleSelected) return new Set();
      const next = new Set(current);
      sortedTranscriptions.forEach((record) => next.add(record.id));
      return next;
    });
  };

  const runBatchAction = async (
    action: "copy" | "scratchpad" | "transform" | "delete",
    work: () => Promise<string>,
  ) => {
    if (batchBusy || selectedRecords.length === 0) return;
    setBatchBusy(action);
    setBatchError(null);
    setBatchStatus(null);
    try {
      const message = await work();
      flashBatchStatus(message);
    } catch (error) {
      setBatchError(error instanceof Error ? error.message : String(error));
    } finally {
      setBatchBusy(null);
    }
  };

  const copySelected = () =>
    runBatchAction("copy", async () => {
      if (!selectedBatchText) throw new Error("Selected items have no transcript text.");
      await navigator.clipboard.writeText(selectedBatchText);
      return t({
        id: "transcriptions.batch.copied",
        message: `Copied ${selectedTextRecords.length} transcripts`,
      });
    });

  const saveSelectedToScratchpad = () =>
    runBatchAction("scratchpad", async () => {
      if (!selectedBatchText) throw new Error("Selected items have no transcript text.");
      await invoke("create_scratchpad_entry", {
        body: selectedBatchText,
        source: "transcription-batch",
      });
      return t({
        id: "transcriptions.batch.saved",
        message: `Saved ${selectedTextRecords.length} transcripts to Scratchpad`,
      });
    });

  const transformSelected = () =>
    runBatchAction("transform", async () => {
      if (!selectedBatchText) throw new Error("Selected items have no transcript text.");
      await invoke("open_transforms_view", { text: selectedBatchText });
      return t({
        id: "transcriptions.batch.opened_transform",
        message: "Opened selected transcripts in Transforms",
      });
    });

  const deleteSelected = () =>
    runBatchAction("delete", async () => {
      for (const record of selectedRecords) {
        await deleteMutation.mutateAsync(record.id);
      }
      const deletedCount = selectedRecords.length;
      clearSelection();
      return t({
        id: "transcriptions.batch.deleted",
        message: `Deleted ${deletedCount} transcripts`,
      });
    });

  const sortOptions: { value: SortKey; label: string }[] = [
    {
      value: "recent",
      label: t({
        id: "transcriptions.sort.recent",
        message: "Newest first",
      }),
    },
    {
      value: "oldest",
      label: t({
        id: "transcriptions.sort.oldest",
        message: "Oldest first",
      }),
    },
    {
      value: "longest",
      label: t({
        id: "transcriptions.sort.longest",
        message: "Longest",
      }),
    },
    {
      value: "shortest",
      label: t({
        id: "transcriptions.sort.shortest",
        message: "Shortest",
      }),
    },
  ];
  const filterOptions: { value: FilterKey; label: string; count: number }[] = [
    {
      value: "all",
      label: t({
        id: "transcriptions.filter.all",
        message: "All",
      }),
      count: filterCounts.all,
    },
    {
      value: "success",
      label: t({
        id: "transcriptions.filter.success",
        message: "Successful",
      }),
      count: filterCounts.success,
    },
    {
      value: "failed",
      label: t({
        id: "transcriptions.filter.failed",
        message: "Failed",
      }),
      count: filterCounts.failed,
    },
    {
      value: "paste_fallback",
      label: t({
        id: "transcriptions.filter.paste_fallback",
        message: "Paste fallback",
      }),
      count: filterCounts.paste_fallback,
    },
    {
      value: "cleaned",
      label: t({
        id: "transcriptions.filter.cleaned",
        message: "Cleaned",
      }),
      count: filterCounts.cleaned,
    },
  ];
  const activeFilterLabel =
    filterOptions.find((option) => option.value === filterKey)?.label ??
    t({
      id: "transcriptions.filter.all",
      message: "All",
    });

  const renderEntry = useCallback(
    (_index: number, entry: ListEntry) => {
      if (entry.type === "header") {
        return (
          <div className="transcription-entry-fade flex items-center gap-3 pt-6 pb-2 px-1 first:pt-1">
            <span className="ui-text-body-sm-strong ui-color-secondary shrink-0">
              {entry.label}
            </span>
            <div className="ui-divider-trailing flex-1" aria-hidden="true" />
          </div>
        );
      }

      const record = entry.record;
      return (
        <div className="transcription-entry-fade">
          <TranscriptionItem
            record={record}
            isRetrying={retryingIdSet.has(record.id)}
            onDelete={deleteTranscription}
            onRetry={retryTranscription}
            onCancelRetry={cancelRetryTranscription}
            onRetryLlm={retryLlmCleanup}
            onUndoLlm={undoLlmCleanup}
            showLlmButtons={showLlmButtons}
            shiftHeld={shiftHeld}
            showDate={!isTimeSorted}
            selectionMode={selectionMode}
            selected={selectedIds.has(record.id)}
            onSelectionChange={toggleRecordSelection}
          />
        </div>
      );
    },
    [
      retryingIdSet,
      deleteTranscription,
      retryTranscription,
      cancelRetryTranscription,
      retryLlmCleanup,
      undoLlmCleanup,
      showLlmButtons,
      shiftHeld,
      isTimeSorted,
      selectionMode,
      selectedIds,
      toggleRecordSelection,
    ],
  );

  const virtuosoComponents = useMemo(
    () => ({
      Header: () => <div className="h-3" />,
      Footer: () => <div className="h-3" />,
    }),
    [],
  );

  const closeSearch = () => {
    setSearchOpen(false);
    setSearchQuery("");
  };

  if (isLoading && transcriptions.length === 0 && !debouncedSearchQuery && !isFetched) {
    return (
      <div className="flex items-center justify-center py-12">
        <DotMatrix
          rows={2}
          cols={8}
          activeDots={[0, 1, 2, 3, 4, 5, 6, 7]}
          dotSize={3}
          gap={3}
          color="var(--color-text-muted)"
          animated
          className="opacity-50"
        />
      </div>
    );
  }

  const hasAnyResults = sortedTranscriptions.length > 0;
  const showEmptyState = totalCount === 0 && !debouncedSearchQuery && !isLoading;
  const hasSelectedRecords = selectedRecords.length > 0;
  const hasSelectedText = selectedTextRecords.length > 0;
  const noResultsMessage =
    filterKey !== "all"
      ? t({
          id: "transcriptions.list.no_filter_results",
          message: `No ${activeFilterLabel.toLowerCase()} transcriptions`,
        })
      : t({
          id: "transcriptions.list.no_results",
          message: `No results for "${debouncedSearchQuery}"`,
        });

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.25, ease: "easeOut" }}
      className="w-full flex-1 min-h-0 flex flex-col"
    >
      <div className="mb-3 flex min-h-8 shrink-0 items-center justify-between gap-2">
        <div className="flex min-w-0 flex-1 items-center gap-2">
          <AnimatePresence>
            {selectionMode && (
              <motion.div
                key="batch-toolbar"
                initial={{ opacity: 0, x: -8 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -8 }}
                transition={{ duration: 0.14 }}
                className="flex min-w-0 items-center gap-1 rounded-full border border-border-primary bg-surface-secondary/70 px-1.5 py-1"
              >
                <button
                  type="button"
                  onClick={toggleAllVisible}
                  className="ui-button-ghost h-7 gap-1 rounded-full px-2 ui-text-meta"
                >
                  <CheckSquare size={13} aria-hidden="true" />
                  {allVisibleSelected
                    ? t({ id: "transcriptions.batch.none", message: "None" })
                    : t({ id: "transcriptions.batch.all", message: "All" })}
                </button>
                <span className="px-2 ui-text-meta ui-color-muted tabular-nums">
                  {selectedRecords.length}
                </span>
                <button
                  type="button"
                  onClick={copySelected}
                  disabled={!hasSelectedText || batchBusy !== null}
                  className="ui-button-ghost h-7 w-7 disabled:opacity-40"
                  aria-label={t({ id: "transcriptions.batch.copy", message: "Copy selected" })}
                  title={t({ id: "transcriptions.batch.copy", message: "Copy selected" })}
                >
                  <Copy size={13} aria-hidden="true" />
                </button>
                <button
                  type="button"
                  onClick={saveSelectedToScratchpad}
                  disabled={!hasSelectedText || batchBusy !== null}
                  className="ui-button-ghost h-7 w-7 disabled:opacity-40"
                  aria-label={t({
                    id: "transcriptions.batch.save",
                    message: "Save selected to Scratchpad",
                  })}
                  title={t({
                    id: "transcriptions.batch.save",
                    message: "Save selected to Scratchpad",
                  })}
                >
                  <FileText size={13} aria-hidden="true" />
                </button>
                <button
                  type="button"
                  onClick={transformSelected}
                  disabled={!hasSelectedText || batchBusy !== null}
                  className="ui-button-ghost h-7 w-7 disabled:opacity-40"
                  aria-label={t({
                    id: "transcriptions.batch.transform",
                    message: "Transform selected",
                  })}
                  title={t({
                    id: "transcriptions.batch.transform",
                    message: "Transform selected",
                  })}
                >
                  <WandSparkles size={13} aria-hidden="true" />
                </button>
                <button
                  type="button"
                  onClick={deleteSelected}
                  disabled={!hasSelectedRecords || batchBusy !== null}
                  className="ui-button-ghost h-7 w-7 text-content-muted hover:text-error disabled:opacity-40"
                  aria-label={t({ id: "transcriptions.batch.delete", message: "Delete selected" })}
                  title={t({ id: "transcriptions.batch.delete", message: "Delete selected" })}
                >
                  <Trash2 size={13} aria-hidden="true" />
                </button>
                <button
                  type="button"
                  onClick={clearSelection}
                  className="ui-button-ghost h-7 w-7"
                  aria-label={t({ id: "transcriptions.batch.close", message: "Close selection" })}
                  title={t({ id: "transcriptions.batch.close", message: "Close selection" })}
                >
                  <X size={13} aria-hidden="true" />
                </button>
              </motion.div>
            )}
          </AnimatePresence>
          {(batchStatus || batchError) && (
            <span
              className={`truncate ui-text-meta ${
                batchError ? "ui-color-error-soft" : "ui-color-secondary"
              }`}
              role="status"
            >
              {batchError ?? batchStatus}
            </span>
          )}
        </div>

        <div className="flex items-center justify-end gap-1">
          <div className="relative" ref={filterRef}>
            <button
              type="button"
              onClick={() => setFilterOpen((open) => !open)}
              aria-haspopup="menu"
              aria-expanded={filterOpen}
              aria-label={t({
                id: "transcriptions.list.filter.aria",
                message: "Filter transcriptions",
              })}
              title={activeFilterLabel}
              className={`ui-button-ghost h-8 gap-1.5 rounded-full px-2 ${
                filterKey !== "all" ? "bg-surface-elevated ui-color-primary" : ""
              }`}
            >
              <Filter size={13} aria-hidden="true" />
              {filterKey !== "all" && (
                <span className="max-w-20 truncate ui-text-meta">{activeFilterLabel}</span>
              )}
            </button>
            <AnimatePresence>
              {filterOpen && (
                <motion.div
                  role="menu"
                  initial={{ opacity: 0, scale: 0.98, y: -2 }}
                  animate={{ opacity: 1, scale: 1, y: 0 }}
                  exit={{ opacity: 0, scale: 0.98, y: -2 }}
                  transition={{ duration: 0.12 }}
                  className="ui-surface-menu absolute right-0 top-full mt-1 z-30 min-w-[190px]"
                >
                  {filterOptions.map((opt) => {
                    const selected = opt.value === filterKey;
                    return (
                      <button
                        key={opt.value}
                        type="button"
                        role="menuitemradio"
                        aria-checked={selected}
                        onClick={() => {
                          setFilterKey(opt.value);
                          setFilterOpen(false);
                          setSelectedIds(new Set());
                        }}
                        className={`flex w-full items-center justify-between gap-3 px-3 py-1.5 ui-text-body-sm transition-colors ${
                          selected
                            ? "ui-color-primary bg-[var(--surface-interactive-strong)]"
                            : "ui-color-secondary hover:bg-[var(--surface-interactive)] hover:text-content-primary"
                        }`}
                      >
                        <span className="truncate">{opt.label}</span>
                        <span className="flex shrink-0 items-center gap-2">
                          <span className="ui-text-meta tabular-nums ui-color-muted">
                            {opt.count}
                          </span>
                          <span className="w-3 flex items-center justify-center">
                            {selected && <Check size={12} aria-hidden="true" />}
                          </span>
                        </span>
                      </button>
                    );
                  })}
                </motion.div>
              )}
            </AnimatePresence>
          </div>

          <AnimatePresence initial={false} mode="wait">
            {searchOpen ? (
              <motion.div
                key="search-input"
                initial={{ opacity: 0, width: 32 }}
                animate={{ opacity: 1, width: 240 }}
                exit={{ opacity: 0, width: 32 }}
                transition={{ duration: 0.16, ease: "easeOut" }}
                className="flex items-center gap-1.5 h-8 px-2.5 rounded-md border border-border-primary bg-surface-secondary/60 overflow-hidden"
              >
                <Search size={12} className="text-content-disabled shrink-0" aria-hidden="true" />
                <input
                  ref={searchInputRef}
                  type="text"
                  autoFocus
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Escape") closeSearch();
                  }}
                  placeholder={t({
                    id: "transcriptions.list.search.placeholder_short",
                    message: "Search",
                  })}
                  aria-label={t({
                    id: "transcriptions.list.search.aria",
                    message: "Search transcriptions",
                  })}
                  className="bg-transparent ui-text-body-sm ui-color-secondary placeholder-content-disabled outline-hidden flex-1 min-w-0"
                />
                <button
                  onClick={closeSearch}
                  aria-label={t({
                    id: "transcriptions.list.search.close",
                    message: "Close search",
                  })}
                  className="p-0.5 rounded text-content-disabled hover:text-content-muted transition-colors shrink-0"
                >
                  <X size={12} aria-hidden="true" />
                </button>
              </motion.div>
            ) : (
              <motion.button
                key="search-button"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.12 }}
                onClick={() => setSearchOpen(true)}
                aria-label={t({
                  id: "transcriptions.list.search.open",
                  message: "Search transcriptions",
                })}
                className="ui-button-ghost h-8 w-8"
              >
                <Search size={13} aria-hidden="true" />
              </motion.button>
            )}
          </AnimatePresence>

          <div className="relative" ref={sortRef}>
            <button
              type="button"
              onClick={() => setSortOpen((open) => !open)}
              aria-haspopup="menu"
              aria-expanded={sortOpen}
              aria-label={t({
                id: "transcriptions.list.sort.aria",
                message: "Sort transcriptions",
              })}
              className="ui-button-ghost h-8 w-8"
            >
              <ArrowDownUp size={13} aria-hidden="true" />
            </button>
            <AnimatePresence>
              {sortOpen && (
                <motion.div
                  role="menu"
                  initial={{ opacity: 0, scale: 0.98, y: -2 }}
                  animate={{ opacity: 1, scale: 1, y: 0 }}
                  exit={{ opacity: 0, scale: 0.98, y: -2 }}
                  transition={{ duration: 0.12 }}
                  className="ui-surface-menu absolute right-0 top-full mt-1 z-30 min-w-[160px]"
                >
                  {sortOptions.map((opt) => {
                    const selected = opt.value === sortKey;
                    return (
                      <button
                        key={opt.value}
                        type="button"
                        role="menuitemradio"
                        aria-checked={selected}
                        onClick={() => {
                          setSortKey(opt.value);
                          setSortOpen(false);
                        }}
                        className={`flex w-full items-center justify-between gap-3 px-3 py-1.5 ui-text-body-sm transition-colors ${
                          selected
                            ? "ui-color-primary bg-[var(--surface-interactive-strong)]"
                            : "ui-color-secondary hover:bg-[var(--surface-interactive)] hover:text-content-primary"
                        }`}
                      >
                        <span>{opt.label}</span>
                        <span className="w-3 flex items-center justify-center shrink-0">
                          {selected && <Check size={12} aria-hidden="true" />}
                        </span>
                      </button>
                    );
                  })}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
          <button
            type="button"
            onClick={() => {
              setSelectionMode((current) => !current);
              setBatchError(null);
              setBatchStatus(null);
            }}
            aria-pressed={selectionMode}
            aria-label={t({
              id: "transcriptions.batch.toggle",
              message: "Select transcriptions",
            })}
            className={`ui-button-ghost h-8 w-8 ${selectionMode ? "bg-surface-elevated ui-color-primary" : ""}`}
          >
            <CheckSquare size={13} aria-hidden="true" />
          </button>
        </div>
      </div>

      <div className="relative flex-1 min-h-0 overflow-hidden">
        <div
          className="pointer-events-none absolute left-0 right-3 top-0 h-6 z-10"
          style={{
            background: "linear-gradient(to bottom, var(--color-bg-tertiary), transparent)",
          }}
          aria-hidden="true"
        />
        <div
          className="pointer-events-none absolute left-0 right-3 bottom-0 h-8 z-10"
          style={{
            background: "linear-gradient(to top, var(--color-bg-tertiary), transparent)",
          }}
          aria-hidden="true"
        />
        {showEmptyState ? (
          <div className="h-full flex flex-col items-center justify-center text-center">
            <DotMatrix
              rows={4}
              cols={4}
              activeDots={[0, 3, 5, 6, 9, 10, 12, 15]}
              dotSize={4}
              gap={4}
              color="var(--color-text-disabled)"
              className="opacity-40 mb-4"
              aria-hidden="true"
            />
            <p className="ui-text-body ui-color-muted max-w-xs">
              {t({
                id: "transcriptions.list.empty",
                message: "Your recent transcriptions will appear here",
              })}
            </p>
          </div>
        ) : hasAnyResults || isLoading ? (
          <Virtuoso
            style={{ height: "100%" }}
            data={entries}
            defaultItemHeight={90}
            overscan={400}
            increaseViewportBy={200}
            computeItemKey={(_index, entry) =>
              entry.type === "header" ? entry.id : entry.record.id
            }
            components={virtuosoComponents}
            itemContent={renderEntry}
            className="custom-scrollbar scrollbar-gutter"
          />
        ) : (
          <div className="h-full flex flex-col items-center justify-center">
            <Search size={18} className="text-content-disabled mb-2" aria-hidden="true" />
            <p className="ui-text-body-sm ui-color-muted">{noResultsMessage}</p>
          </div>
        )}
      </div>
    </motion.div>
  );
};

export default React.memo(TranscriptionList);
