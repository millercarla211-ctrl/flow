import { useLingui } from "@lingui/react/macro";
import React, { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import ReactMarkdown, { Components } from "react-markdown";
import remarkBreaks from "remark-breaks";
import { invoke } from "@tauri-apps/api/core";
import {
  CheckSquare,
  ClipboardCheck,
  ClipboardX,
  Clock3,
  Cloud,
  Copy,
  BookPlus,
  FilePlus2,
  FileText,
  Gauge,
  HardDrive,
  Mic2,
  Pin,
  PinOff,
  Trash2,
  RotateCw,
  SendHorizontal,
  Check,
  ChevronDown,
  ChevronUp,
  MoreVertical,
  AlertTriangle,
  Undo2,
  WandSparkles,
  Square,
  Timer,
  X,
  type LucideIcon,
} from "lucide-react";
import type { TranscriptionRecord } from "../../../types";
import DotMatrix from "../../../shared/ui/DotMatrix";
import { useClickOutside } from "../../../shared/hooks/useClickOutside";

const markdownComponents: Components = {
  p: ({ children }) => <p className="mb-2 last:mb-0">{children}</p>,
  strong: ({ children }) => (
    <strong className="font-semibold text-content-primary">{children}</strong>
  ),
  em: ({ children }) => <em className="italic">{children}</em>,
  code: ({ children }) => (
    <code className="px-1 py-0.5 rounded-sm bg-surface-elevated ui-text-body-sm font-mono ui-color-primary">
      {children}
    </code>
  ),
  ul: ({ children }) => (
    <ul className="list-disc list-inside mb-2 last:mb-0 space-y-0.5">{children}</ul>
  ),
  ol: ({ children }) => (
    <ol className="list-decimal list-inside mb-2 last:mb-0 space-y-0.5">{children}</ol>
  ),
  li: ({ children }) => <li className="ui-text-body">{children}</li>,
};

interface TranscriptionItemProps {
  record: TranscriptionRecord;
  onDelete: (id: string) => Promise<void>;
  onTogglePinned?: (id: string, pinned: boolean) => Promise<void>;
  onRetry: (id: string) => Promise<void>;
  onCancelRetry?: (id: string) => Promise<void>;
  onRetryLlm?: (id: string) => Promise<void>;
  onUndoLlm?: (id: string) => Promise<void>;
  isRetrying?: boolean;
  showLlmButtons?: boolean;
  shiftHeld?: boolean;
  showDate?: boolean;
  selectionMode?: boolean;
  selected?: boolean;
  onSelectionChange?: (id: string, selected: boolean) => void;
}

type PasteTextResult = {
  pasted: boolean;
  copied: boolean;
  message: string;
};

type DiagnosticTone = "neutral" | "local" | "cloud" | "warning" | "error";

interface DiagnosticItem {
  key: string;
  icon: LucideIcon;
  label: string;
  title: string;
  tone?: DiagnosticTone;
}

const diagnosticToneClass: Record<DiagnosticTone, string> = {
  neutral: "border-border-primary bg-surface-surface ui-color-muted",
  local: "border-local/30 bg-local/10 text-local",
  cloud: "border-border-primary bg-surface-elevated ui-color-secondary",
  warning: "border-border-primary bg-surface-elevated ui-color-secondary",
  error: "border-red-500/20 bg-red-500/5 ui-color-error-strong",
};

const formatDiagnosticDuration = (seconds: number) => {
  const safeSeconds = Number.isFinite(seconds) ? Math.max(0, seconds) : 0;
  if (safeSeconds < 0.1) return "0s";
  if (safeSeconds < 10) return `${safeSeconds.toFixed(1)}s`;
  if (safeSeconds < 60) return `${Math.round(safeSeconds)}s`;

  const minutes = Math.floor(safeSeconds / 60);
  const remainingSeconds = Math.round(safeSeconds % 60);
  return remainingSeconds > 0 ? `${minutes}m ${remainingSeconds}s` : `${minutes}m`;
};

const formatWordCount = (wordCount: number) => (wordCount === 1 ? "1 word" : `${wordCount} words`);

const formatWordsPerMinute = (wordCount: number, seconds: number) => {
  if (!Number.isFinite(seconds) || seconds < 1 || wordCount <= 0) return null;
  const wordsPerMinute = Math.round(wordCount / (seconds / 60));
  return wordsPerMinute > 0 ? `${wordsPerMinute} wpm` : null;
};

const formatElapsed = (milliseconds?: number | null) => {
  if (milliseconds == null || !Number.isFinite(milliseconds) || milliseconds < 0) return null;
  if (milliseconds < 1000) return `${Math.round(milliseconds)}ms`;
  if (milliseconds < 10_000) return `${(milliseconds / 1000).toFixed(1)}s`;
  return `${Math.round(milliseconds / 1000)}s`;
};

const truncateDiagnosticLabel = (label: string) =>
  label.length > 28 ? `${label.slice(0, 25)}...` : label;

type DiagnosticsChipProps = Omit<DiagnosticItem, "key">;

const DiagnosticsChip: React.FC<DiagnosticsChipProps> = ({
  icon: Icon,
  label,
  title,
  tone = "neutral",
}) => (
  <span
    className={`inline-flex max-w-full items-center gap-1 rounded-full border px-2 py-0.5 ui-text-micro ${diagnosticToneClass[tone]}`}
    title={title}
  >
    <Icon size={10} aria-hidden="true" className="shrink-0 opacity-80" />
    <span className="truncate">{truncateDiagnosticLabel(label)}</span>
  </span>
);

const TranscriptionItem: React.FC<TranscriptionItemProps> = ({
  record,
  onDelete,
  onTogglePinned,
  onRetry,
  onCancelRetry,
  onRetryLlm,
  onUndoLlm,
  isRetrying = false,
  showLlmButtons = false,
  shiftHeld = false,
  showDate = false,
  selectionMode = false,
  selected = false,
  onSelectionChange,
}) => {
  const { t } = useLingui();
  const [copied, setCopied] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [isPinning, setIsPinning] = useState(false);
  const [isCancellingRetry, setIsCancellingRetry] = useState(false);
  const [isRetryingLlm, setIsRetryingLlm] = useState(false);
  const [isUndoingLlm, setIsUndoingLlm] = useState(false);
  const [isPasting, setIsPasting] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);
  const [isOverflowing, setIsOverflowing] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [selectionText, setSelectionText] = useState("");
  const [actionStatus, setActionStatus] = useState<
    "saved" | "opened" | "dictionary" | "snippet" | "pasted" | null
  >(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const textRef = useRef<HTMLDivElement>(null);

  useClickOutside(
    menuRef,
    () => {
      setMenuOpen(false);
      setSelectionText("");
    },
    menuOpen,
  );

  useEffect(() => {
    if (isExpanded) {
      setIsOverflowing(true);
      return;
    }

    const element = textRef.current;
    if (!element) return;

    let frameId = 0;
    const updateOverflow = () => {
      if (frameId) {
        window.cancelAnimationFrame(frameId);
      }
      frameId = window.requestAnimationFrame(() => {
        setIsOverflowing(element.scrollHeight > element.clientHeight);
      });
    };

    updateOverflow();

    const observer = new ResizeObserver(updateOverflow);
    observer.observe(element);

    return () => {
      observer.disconnect();
      if (frameId) {
        window.cancelAnimationFrame(frameId);
      }
    };
  }, [record.text, isExpanded]);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(record.text);
      setCopied(true);
      setActionStatus(null);
      setMenuOpen(false);
      setSelectionText("");
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  };

  const handleCopySelection = async () => {
    if (!selectionText.trim()) return;
    try {
      await navigator.clipboard.writeText(selectionText);
      setActionStatus(null);
      setMenuOpen(false);
      setSelectionText("");
    } catch (err) {
      console.error("Failed to copy selection:", err);
    }
  };

  const selectedActionText = () => selectionText.trim() || record.text.trim();

  const flashActionStatus = (status: "saved" | "opened" | "dictionary" | "snippet" | "pasted") => {
    setActionStatus(status);
    window.setTimeout(() => setActionStatus(null), 1600);
  };

  const handlePasteText = async () => {
    const text = selectedActionText();
    if (!text || isPasting) return;
    setIsPasting(true);
    try {
      const result = await invoke<PasteTextResult>("paste_text_to_focused_app", { text });
      setMenuOpen(false);
      setSelectionText("");
      if (result.copied) {
        setCopied(true);
        window.setTimeout(() => setCopied(false), 1400);
      } else {
        flashActionStatus("pasted");
      }
    } catch (err) {
      console.error("Failed to paste transcription:", err);
    } finally {
      setIsPasting(false);
    }
  };

  const handleAddSelectionToDictionary = async () => {
    const entry = selectionText.trim();
    if (!entry) return;
    try {
      await invoke("add_dictionary_entries", { entries: [entry] });
      setMenuOpen(false);
      setSelectionText("");
      flashActionStatus("dictionary");
    } catch (err) {
      console.error("Failed to add selection to Dictionary:", err);
    }
  };

  const handleCreateSnippetFromSelection = async () => {
    const expansion = selectionText.trim();
    if (!expansion) return;
    try {
      await invoke("open_snippets_view", { expansion });
      setMenuOpen(false);
      setSelectionText("");
      flashActionStatus("snippet");
    } catch (err) {
      console.error("Failed to open Snippets with selection:", err);
    }
  };

  const handleSaveToScratchpad = async () => {
    const body = selectedActionText();
    if (!body) return;
    try {
      await invoke("create_scratchpad_entry", {
        body,
        source: selectionText.trim() ? "transcription-selection" : "transcription",
      });
      setMenuOpen(false);
      setSelectionText("");
      flashActionStatus("saved");
    } catch (err) {
      console.error("Failed to save transcription to Scratchpad:", err);
    }
  };

  const handleOpenTransform = async () => {
    const text = selectedActionText();
    if (!text) return;
    try {
      await invoke("open_transforms_view", { text });
      setMenuOpen(false);
      setSelectionText("");
      flashActionStatus("opened");
    } catch (err) {
      console.error("Failed to open transcription in Transforms:", err);
    }
  };

  const handleDelete = async () => {
    if (isDeleting) return;
    setIsDeleting(true);
    setMenuOpen(false);
    setSelectionText("");
    try {
      await onDelete(record.id);
    } catch (err) {
      console.error("Failed to delete:", err);
    } finally {
      setIsDeleting(false);
    }
  };

  const handleTogglePinned = async () => {
    if (isPinning || !onTogglePinned) return;
    setIsPinning(true);
    setMenuOpen(false);
    setSelectionText("");
    try {
      await onTogglePinned(record.id, !record.pinned);
    } catch (err) {
      console.error("Failed to update pinned transcription:", err);
    } finally {
      setIsPinning(false);
    }
  };

  const handleRetry = async () => {
    if (isRetrying) return;
    setMenuOpen(false);
    setSelectionText("");
    try {
      await onRetry(record.id);
    } catch (err) {
      console.error("Failed to retry:", err);
    }
  };

  const handleCancelRetry = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.stopPropagation();
    if (isCancellingRetry || !onCancelRetry) return;
    setIsCancellingRetry(true);
    try {
      await onCancelRetry(record.id);
    } catch (err) {
      console.error("Failed to stop retry:", err);
    } finally {
      setIsCancellingRetry(false);
    }
  };

  const handleRetryLlm = async () => {
    if (isRetryingLlm || !onRetryLlm) return;
    setIsRetryingLlm(true);
    setMenuOpen(false);
    setSelectionText("");
    try {
      await onRetryLlm(record.id);
    } catch (err) {
      console.error("Failed to retry cleanup:", err);
    } finally {
      setIsRetryingLlm(false);
    }
  };

  const handleUndoLlm = async () => {
    if (isUndoingLlm || !onUndoLlm) return;
    setIsUndoingLlm(true);
    setMenuOpen(false);
    setSelectionText("");
    try {
      await onUndoLlm(record.id);
    } catch (err) {
      console.error("Failed to undo cleanup:", err);
    } finally {
      setIsUndoingLlm(false);
    }
  };

  const timestamp = new Date(record.timestamp);
  const timeStr = timestamp.toLocaleTimeString([], {
    hour: "numeric",
    minute: "2-digit",
  });
  const dateStr = timestamp.toLocaleDateString([], {
    month: "short",
    day: "numeric",
  });
  const isError = record.status === "error";
  const canRetryFromAudio = record.audio_available;
  const errorMessage =
    record.error_message ||
    t({
      id: "transcriptions.item.error.default",
      message: "Transcription failed",
    });
  const displayText = isError ? null : record.text;
  const normalizedModel = (record.speech_model ?? "").trim();
  const isCloudModel = normalizedModel.startsWith("cloud-");
  const speechModelLabel = normalizedModel
    ? isCloudModel
      ? normalizedModel.slice(6)
      : normalizedModel
    : null;
  const llmModelLabel = record.llm_model?.trim() || null;
  const modeLabel = record.mode_name?.trim() || null;
  const autoTransformLabel = record.auto_transform_label?.trim() || null;
  const allowContextMenu = !selectionMode && !isRetryingLlm && !isUndoingLlm;
  const hasSelection = selectionText.trim().length > 0;
  const wordCount = Math.max(0, record.word_count ?? 0);
  const audioDurationSeconds = Math.max(0, record.audio_duration_seconds ?? 0);
  const wpmLabel = formatWordsPerMinute(wordCount, audioDurationSeconds);
  const sttElapsedLabel = formatElapsed(record.stt_elapsed_ms);
  const cleanupElapsedLabel = formatElapsed(record.cleanup_elapsed_ms);
  const pasteElapsedLabel = formatElapsed(record.paste_elapsed_ms);
  const totalElapsedLabel = formatElapsed(record.total_elapsed_ms);
  const diagnostics: DiagnosticItem[] = [];

  if (record.pinned) {
    diagnostics.push({
      key: "pinned",
      icon: Pin,
      label: "Pinned",
      title: "Pinned transcription",
      tone: "local",
    });
  }

  if (audioDurationSeconds > 0) {
    diagnostics.push({
      key: "duration",
      icon: Clock3,
      label: `${formatDiagnosticDuration(audioDurationSeconds)} audio`,
      title: "Captured audio length",
    });
  }

  if (!isError && wordCount > 0) {
    diagnostics.push({
      key: "words",
      icon: FileText,
      label: formatWordCount(wordCount),
      title: "Transcript size",
    });
  }

  if (!isError && wpmLabel) {
    diagnostics.push({
      key: "speed",
      icon: Gauge,
      label: wpmLabel,
      title: "Estimated speaking pace",
    });
  }

  if (sttElapsedLabel) {
    diagnostics.push({
      key: "stt-time",
      icon: Timer,
      label: `STT ${sttElapsedLabel}`,
      title: "Local speech recognition time",
    });
  }

  if (cleanupElapsedLabel) {
    diagnostics.push({
      key: "cleanup-time",
      icon: WandSparkles,
      label: `Cleanup ${cleanupElapsedLabel}`,
      title: "AI cleanup time",
      tone: "local",
    });
  }

  if (pasteElapsedLabel) {
    diagnostics.push({
      key: "paste-time",
      icon: record.auto_paste_succeeded ? ClipboardCheck : ClipboardX,
      label: `Paste ${pasteElapsedLabel}`,
      title: record.auto_paste_succeeded
        ? "Auto-paste succeeded"
        : "Auto-paste failed and used fallback",
      tone: record.auto_paste_succeeded ? "local" : "warning",
    });
  } else if (record.auto_paste_requested) {
    diagnostics.push({
      key: "paste-result",
      icon: record.auto_paste_succeeded ? ClipboardCheck : ClipboardX,
      label: record.auto_paste_succeeded ? "Pasted" : "Paste fallback",
      title: record.auto_paste_succeeded
        ? "Auto-paste succeeded"
        : "Auto-paste failed and used fallback",
      tone: record.auto_paste_succeeded ? "local" : "warning",
    });
  }

  if (totalElapsedLabel) {
    diagnostics.push({
      key: "total-time",
      icon: Clock3,
      label: `Total ${totalElapsedLabel}`,
      title: "Total processing time after recording stopped",
    });
  }

  if (speechModelLabel) {
    diagnostics.push({
      key: "speech-model",
      icon: isCloudModel ? Cloud : Mic2,
      label: speechModelLabel,
      title: isCloudModel ? "Cloud speech model" : "Local speech model",
      tone: isCloudModel ? "cloud" : "local",
    });
  }

  if (!isError && record.llm_cleaned) {
    if (autoTransformLabel) {
      diagnostics.push({
        key: "auto-transform",
        icon: WandSparkles,
        label: `Auto ${autoTransformLabel}`,
        title: "Auto Transform applied before paste",
        tone: "local",
      });
    }

    diagnostics.push({
      key: "cleanup",
      icon: WandSparkles,
      label: llmModelLabel ? `Cleaned by ${llmModelLabel}` : "Cleaned",
      title: "AI cleanup applied",
      tone: "local",
    });
  }

  if (modeLabel) {
    diagnostics.push({
      key: "mode",
      icon: HardDrive,
      label: modeLabel,
      title: "Matched dictation style",
    });
  }

  if (isError && canRetryFromAudio) {
    diagnostics.push({
      key: "retry-audio",
      icon: RotateCw,
      label: "Audio saved",
      title: "This failed recording can be retried from saved audio",
      tone: "warning",
    });
  }

  const captureSelectionText = () => {
    const selection = window.getSelection();
    if (!selection) return "";
    const text = selection.toString();
    if (!text.trim()) return "";
    const anchor = selection.anchorNode;
    const focus = selection.focusNode;
    if (
      textRef.current &&
      ((anchor && textRef.current.contains(anchor)) || (focus && textRef.current.contains(focus)))
    ) {
      return text;
    }
    return "";
  };

  const openMenu = () => {
    setSelectionText(captureSelectionText());
    setMenuOpen(true);
  };

  return (
    <div
      className="group relative"
      onContextMenu={(event) => {
        if (!allowContextMenu) return;
        event.preventDefault();
        if (shiftHeld) {
          handleDelete();
        } else {
          openMenu();
        }
      }}
    >
      <div
        className={`flex items-start gap-2 py-2.5 px-3 rounded-lg transition-colors ${
          selected
            ? "bg-[var(--surface-interactive-strong)]"
            : isError
              ? "bg-red-500/[0.03]"
              : "hover:bg-[var(--surface-interactive)]"
        }`}
      >
        {selectionMode && (
          <button
            type="button"
            onClick={(event) => {
              event.stopPropagation();
              onSelectionChange?.(record.id, !selected);
            }}
            className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-content-muted transition-colors hover:bg-surface-elevated hover:text-content-primary"
            aria-label={
              selected
                ? t({
                    id: "transcriptions.item.deselect",
                    message: "Deselect transcription",
                  })
                : t({
                    id: "transcriptions.item.select",
                    message: "Select transcription",
                  })
            }
          >
            {selected ? (
              <CheckSquare size={15} aria-hidden="true" />
            ) : (
              <Square size={15} aria-hidden="true" />
            )}
          </button>
        )}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-x-2 mb-1 ui-text-meta ui-color-disabled whitespace-nowrap overflow-hidden">
            {showDate && (
              <>
                <span>{dateStr}</span>
                <span aria-hidden="true" className="opacity-60">
                  ·
                </span>
              </>
            )}
            <span>{timeStr}</span>
            {record.pinned && (
              <>
                <span aria-hidden="true" className="opacity-60">
                  Â·
                </span>
                <span className="flex items-center gap-1 ui-color-primary font-medium">
                  <Pin size={10} aria-hidden="true" className="opacity-80" />
                  {t({
                    id: "transcriptions.item.pinned",
                    message: "Pinned",
                  })}
                </span>
              </>
            )}
            {isError && (
              <>
                <span aria-hidden="true" className="opacity-60">
                  ·
                </span>
                <span className="flex items-center gap-1 ui-color-error-strong font-medium">
                  <AlertTriangle size={10} aria-hidden="true" className="opacity-80" />
                  {t({
                    id: "transcriptions.item.failed",
                    message: "Failed",
                  })}
                </span>
              </>
            )}
            {isRetrying && (
              <>
                <span aria-hidden="true" className="opacity-60">
                  ·
                </span>
                <button
                  onClick={(event) => {
                    void handleCancelRetry(event);
                  }}
                  disabled={isCancellingRetry || !onCancelRetry}
                  className="flex items-center gap-1 ui-color-primary font-medium group/stop hover:text-content-primary transition-colors"
                  aria-label={t({
                    id: "transcriptions.item.stop_retry",
                    message: "Stop transcription",
                  })}
                  title={t({
                    id: "transcriptions.item.stop_retry",
                    message: "Stop transcription",
                  })}
                >
                  <span className="relative inline-flex items-center justify-center w-[9px] h-[9px]">
                    <DotMatrix
                      rows={1}
                      cols={1}
                      activeDots={[0]}
                      dotSize={3}
                      gap={1}
                      color="var(--color-text-muted)"
                      className="opacity-80 transition-opacity group-hover/stop:opacity-0"
                    />
                    <X
                      size={9}
                      className="absolute opacity-0 transition-opacity group-hover/stop:opacity-100"
                      aria-hidden="true"
                    />
                  </span>
                  {t({
                    id: "transcriptions.item.retrying",
                    message: "Retrying...",
                  })}
                </button>
              </>
            )}
          </div>

          {isError ? (
            <p className="ui-text-body-sm ui-color-error-soft">{errorMessage}</p>
          ) : (
            <>
              <div
                ref={textRef}
                className={`ui-text-body ui-color-primary leading-relaxed select-text cursor-text overflow-hidden break-words ${!isExpanded ? "line-clamp-3" : ""}`}
                onMouseUp={() => setSelectionText(captureSelectionText())}
                onKeyUp={() => setSelectionText(captureSelectionText())}
              >
                <ReactMarkdown components={markdownComponents} remarkPlugins={[remarkBreaks]}>
                  {displayText || ""}
                </ReactMarkdown>
              </div>
              {(isOverflowing || isExpanded) && (
                <button
                  onClick={() => setIsExpanded(!isExpanded)}
                  className="flex items-center gap-1 mt-1 -ml-0.5 px-1 py-0.5 ui-text-meta ui-color-muted hover:text-content-secondary transition-colors rounded"
                  aria-label={
                    isExpanded
                      ? t({
                          id: "transcriptions.item.show_less",
                          message: "Show less",
                        })
                      : t({
                          id: "transcriptions.item.show_more",
                          message: "Show more",
                        })
                  }
                >
                  {isExpanded ? (
                    <>
                      <ChevronUp size={11} aria-hidden="true" />
                      <span>
                        {t({
                          id: "transcriptions.item.show_less",
                          message: "Show less",
                        })}
                      </span>
                    </>
                  ) : (
                    <>
                      <ChevronDown size={11} aria-hidden="true" />
                      <span>
                        {t({
                          id: "transcriptions.item.show_more",
                          message: "Show more",
                        })}
                      </span>
                    </>
                  )}
                </button>
              )}
            </>
          )}
          {diagnostics.length > 0 && (
            <div className="mt-2 flex flex-wrap items-center gap-1.5 pr-2">
              {diagnostics.map(({ key, ...item }) => (
                <DiagnosticsChip key={key} {...item} />
              ))}
            </div>
          )}
        </div>

        {!isRetrying && !isRetryingLlm && !isUndoingLlm && (
          <div className="relative shrink-0 flex items-center gap-1" ref={menuRef}>
            {onTogglePinned && (
              <motion.button
                onClick={handleTogglePinned}
                disabled={isPinning}
                whileTap={{ scale: 0.95 }}
                className={`p-1.5 rounded-md transition-colors hover:bg-surface-elevated disabled:opacity-50 ${
                  record.pinned
                    ? "opacity-100 ui-color-primary"
                    : "opacity-0 group-hover:opacity-100 ui-color-muted"
                }`}
                title={
                  record.pinned
                    ? t({
                        id: "transcriptions.item.unpin",
                        message: "Unpin transcription",
                      })
                    : t({
                        id: "transcriptions.item.pin",
                        message: "Pin transcription",
                      })
                }
                aria-label={
                  record.pinned
                    ? t({
                        id: "transcriptions.item.unpin",
                        message: "Unpin transcription",
                      })
                    : t({
                        id: "transcriptions.item.pin",
                        message: "Pin transcription",
                      })
                }
              >
                {record.pinned ? (
                  <PinOff size={14} aria-hidden="true" />
                ) : (
                  <Pin size={14} aria-hidden="true" />
                )}
              </motion.button>
            )}

            {!isError && (
              <>
                <motion.button
                  onClick={handlePasteText}
                  disabled={isPasting}
                  whileTap={{ scale: 0.95 }}
                  className="p-1.5 rounded-md transition-colors opacity-0 group-hover:opacity-100 hover:bg-surface-elevated disabled:opacity-40"
                  title={t({
                    id: "transcriptions.item.paste_transcription",
                    message: "Paste transcription",
                  })}
                  aria-label={t({
                    id: "transcriptions.item.paste_transcription",
                    message: "Paste transcription",
                  })}
                >
                  <SendHorizontal
                    size={14}
                    className={`text-content-secondary ${isPasting ? "animate-pulse" : ""}`}
                    aria-hidden="true"
                  />
                </motion.button>
                <motion.button
                  onClick={handleCopy}
                  whileTap={{ scale: 0.95 }}
                  className={`p-1.5 rounded-md transition-colors opacity-0 group-hover:opacity-100 hover:bg-surface-elevated ${
                    copied ? "bg-surface-elevated" : ""
                  }`}
                  title={
                    copied
                      ? t({
                          id: "transcriptions.item.copied",
                          message: "Copied",
                        })
                      : t({
                          id: "transcriptions.item.copy_transcription",
                          message: "Copy transcription",
                        })
                  }
                  aria-label={
                    copied
                      ? t({
                          id: "transcriptions.item.copied",
                          message: "Copied",
                        })
                      : t({
                          id: "transcriptions.item.copy_transcription",
                          message: "Copy transcription",
                        })
                  }
                >
                  {copied ? (
                    <Check size={14} className="text-success" aria-hidden="true" />
                  ) : (
                    <Copy size={14} className="text-content-secondary" aria-hidden="true" />
                  )}
                </motion.button>
              </>
            )}

            <motion.button
              onClick={() => {
                if (shiftHeld) {
                  handleDelete();
                } else {
                  if (menuOpen) {
                    setMenuOpen(false);
                    setSelectionText("");
                  } else {
                    openMenu();
                  }
                }
              }}
              whileTap={{ scale: 0.95 }}
              className={`p-1.5 rounded-md transition-colors ${
                shiftHeld ? "hover:bg-red-500/10" : "hover:bg-surface-elevated"
              }`}
              title={
                shiftHeld
                  ? t({
                      id: "transcriptions.item.delete",
                      message: "Delete",
                    })
                  : t({
                      id: "transcriptions.item.more_options",
                      message: "More options",
                    })
              }
              aria-label={
                shiftHeld
                  ? t({
                      id: "transcriptions.item.delete",
                      message: "Delete",
                    })
                  : t({
                      id: "transcriptions.item.more_options",
                      message: "More options",
                    })
              }
              aria-haspopup="true"
              aria-expanded={menuOpen}
            >
              {shiftHeld ? (
                <Trash2 size={14} className="text-red-400" aria-hidden="true" />
              ) : (
                <MoreVertical size={14} className="text-content-muted" aria-hidden="true" />
              )}
            </motion.button>

            <AnimatePresence>
              {menuOpen && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.95, y: -4 }}
                  animate={{ opacity: 1, scale: 1, y: 0 }}
                  exit={{ opacity: 0, scale: 0.95, y: -4 }}
                  transition={{ duration: 0.12 }}
                  className="ui-surface-menu absolute right-0 top-full mt-1 z-[100] min-w-[200px] origin-top-right"
                >
                  {(speechModelLabel || modeLabel || llmModelLabel) && (
                    <>
                      <div className="px-3 pt-2.5 pb-2 space-y-0.5">
                        <div className="ui-text-meta ui-color-disabled">
                          {dateStr} · {timeStr}
                        </div>
                        {speechModelLabel && (
                          <div
                            className={`ui-text-meta truncate ${isCloudModel ? "ui-color-primary" : "ui-color-secondary"}`}
                          >
                            {speechModelLabel}
                          </div>
                        )}
                        {llmModelLabel && record.llm_cleaned && (
                          <div className="ui-text-meta ui-color-local truncate">
                            {llmModelLabel}
                          </div>
                        )}
                        {modeLabel && (
                          <div className="ui-text-meta ui-color-secondary truncate">
                            {modeLabel}
                          </div>
                        )}
                      </div>
                      <div className="h-px bg-border-secondary mx-2" />
                    </>
                  )}
                  {onTogglePinned && (
                    <button
                      onClick={handleTogglePinned}
                      disabled={isPinning}
                      className="flex w-full items-center gap-2.5 px-3 py-2 ui-text-menu-item ui-color-secondary hover:bg-surface-elevated transition-colors disabled:opacity-50"
                    >
                      {record.pinned ? (
                        <PinOff size={12} className="text-content-muted" />
                      ) : (
                        <Pin size={12} className="text-content-muted" />
                      )}
                      <span>
                        {record.pinned
                          ? t({
                              id: "transcriptions.item.unpin",
                              message: "Unpin transcription",
                            })
                          : t({
                              id: "transcriptions.item.pin",
                              message: "Pin transcription",
                            })}
                      </span>
                    </button>
                  )}
                  {selectionText.trim().length > 0 && (
                    <>
                      {onTogglePinned && <div className="h-px bg-border-secondary mx-2" />}
                      <button
                        onClick={handleCopySelection}
                        className="flex w-full items-center gap-2.5 px-3 py-2 ui-text-menu-item ui-color-secondary hover:bg-surface-elevated transition-colors"
                      >
                        <Copy size={12} className="text-content-muted" />
                        <span>
                          {t({
                            id: "transcriptions.item.copy_selection",
                            message: "Copy selection",
                          })}
                        </span>
                      </button>
                      <button
                        onClick={handleAddSelectionToDictionary}
                        className="flex w-full items-center gap-2.5 px-3 py-2 ui-text-menu-item ui-color-secondary hover:bg-surface-elevated transition-colors"
                      >
                        <BookPlus size={12} className="text-content-muted" />
                        <span>
                          {t({
                            id: "transcriptions.item.add_selection_to_dictionary",
                            message: "Add selection to Dictionary",
                          })}
                        </span>
                      </button>
                      <button
                        onClick={handleCreateSnippetFromSelection}
                        className="flex w-full items-center gap-2.5 px-3 py-2 ui-text-menu-item ui-color-secondary hover:bg-surface-elevated transition-colors"
                      >
                        <FilePlus2 size={12} className="text-content-muted" />
                        <span>
                          {t({
                            id: "transcriptions.item.create_snippet_from_selection",
                            message: "Create snippet from selection",
                          })}
                        </span>
                      </button>
                      <div className="h-px bg-border-secondary mx-2" />
                    </>
                  )}
                  {!isError && (
                    <>
                      <button
                        onClick={handlePasteText}
                        disabled={isPasting}
                        className="flex w-full items-center gap-2.5 px-3 py-2 ui-text-menu-item ui-color-secondary hover:bg-surface-elevated transition-colors disabled:opacity-50"
                      >
                        <SendHorizontal size={12} className="text-content-muted" />
                        <span>
                          {hasSelection
                            ? t({
                                id: "transcriptions.item.paste_selection",
                                message: "Paste selection",
                              })
                            : t({
                                id: "transcriptions.item.paste_transcription",
                                message: "Paste transcription",
                              })}
                        </span>
                      </button>
                      <button
                        onClick={handleSaveToScratchpad}
                        className="flex w-full items-center gap-2.5 px-3 py-2 ui-text-menu-item ui-color-secondary hover:bg-surface-elevated transition-colors"
                      >
                        <FileText size={12} className="text-content-muted" />
                        <span>
                          {hasSelection
                            ? t({
                                id: "transcriptions.item.save_selection",
                                message: "Save selection",
                              })
                            : t({
                                id: "transcriptions.item.save_to_scratchpad",
                                message: "Save to Scratchpad",
                              })}
                        </span>
                      </button>
                      <button
                        onClick={handleOpenTransform}
                        className="flex w-full items-center gap-2.5 px-3 py-2 ui-text-menu-item ui-color-secondary hover:bg-surface-elevated transition-colors"
                      >
                        <WandSparkles size={12} className="text-content-muted" />
                        <span>
                          {hasSelection
                            ? t({
                                id: "transcriptions.item.transform_selection",
                                message: "Transform selection",
                              })
                            : t({
                                id: "transcriptions.item.transform",
                                message: "Transform",
                              })}
                        </span>
                      </button>
                      <div className="h-px bg-border-secondary mx-2" />
                    </>
                  )}
                  {canRetryFromAudio && (
                    <button
                      onClick={handleRetry}
                      disabled={isRetrying}
                      className="flex w-full items-center gap-2.5 px-3 py-2 ui-text-menu-item ui-color-secondary hover:bg-surface-elevated transition-colors disabled:opacity-50"
                    >
                      <RotateCw size={12} className="text-content-muted" />
                      <span>
                        {t({
                          id: "transcriptions.item.retry",
                          message: "Retry",
                        })}
                      </span>
                    </button>
                  )}

                  {!isError && onRetryLlm && showLlmButtons && !isCloudModel && (
                    <button
                      onClick={handleRetryLlm}
                      disabled={isRetryingLlm}
                      className="flex w-full items-center gap-2.5 px-3 py-2 ui-text-menu-item ui-color-secondary hover:bg-surface-elevated transition-colors disabled:opacity-50"
                    >
                      <RotateCw size={12} className="text-local" />
                      <span>
                        {record.llm_cleaned
                          ? t({
                              id: "transcriptions.item.retry_cleanup",
                              message: "Retry cleanup",
                            })
                          : t({
                              id: "transcriptions.item.run_cleanup",
                              message: "Run cleanup",
                            })}
                      </span>
                    </button>
                  )}

                  {!isError &&
                    record.llm_cleaned &&
                    record.raw_text &&
                    onUndoLlm &&
                    showLlmButtons &&
                    !isCloudModel && (
                      <button
                        onClick={handleUndoLlm}
                        disabled={isUndoingLlm}
                        className="flex w-full items-center gap-2.5 px-3 py-2 ui-text-menu-item ui-color-secondary hover:bg-surface-elevated transition-colors disabled:opacity-50"
                      >
                        <Undo2 size={12} className="text-content-muted" />
                        <span>
                          {t({
                            id: "transcriptions.item.restore_original",
                            message: "Restore original transcript",
                          })}
                        </span>
                      </button>
                    )}

                  {(canRetryFromAudio ||
                    (!isError && onRetryLlm && showLlmButtons && !isCloudModel) ||
                    (!isError &&
                      record.llm_cleaned &&
                      record.raw_text &&
                      onUndoLlm &&
                      showLlmButtons &&
                      !isCloudModel)) && <div className="h-px bg-border-secondary mx-2" />}
                  <button
                    onClick={handleDelete}
                    disabled={isDeleting}
                    className="flex w-full items-center gap-2.5 px-3 py-2 ui-text-menu-item ui-color-error-strong hover:bg-red-500/10 transition-colors disabled:opacity-50"
                  >
                    <Trash2 size={12} />
                    <span>
                      {t({
                        id: "transcriptions.item.delete",
                        message: "Delete",
                      })}
                    </span>
                  </button>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        )}

        {actionStatus && (
          <div className="flex items-center gap-1.5 ui-text-meta ui-color-secondary">
            <Check size={12} />
            <span>
              {actionStatus === "saved"
                ? t({
                    id: "transcriptions.item.saved_to_scratchpad",
                    message: "Saved to Scratchpad",
                  })
                : actionStatus === "dictionary"
                  ? t({
                      id: "transcriptions.item.added_to_dictionary",
                      message: "Added to Dictionary",
                    })
                  : actionStatus === "snippet"
                    ? t({
                        id: "transcriptions.item.opened_snippets",
                        message: "Opened in Snippets",
                      })
                    : actionStatus === "pasted"
                      ? t({
                          id: "transcriptions.item.pasted",
                          message: "Pasted",
                        })
                      : t({
                          id: "transcriptions.item.opened_transforms",
                          message: "Opened in Transforms",
                        })}
            </span>
          </div>
        )}
        {isRetryingLlm && (
          <div className="flex items-center gap-1.5 ui-text-meta ui-color-local">
            <RotateCw size={12} className="animate-spin" />
            <span>
              {t({
                id: "transcriptions.item.cleaning",
                message: "Cleaning...",
              })}
            </span>
          </div>
        )}
        {isUndoingLlm && (
          <div className="flex items-center gap-1.5 ui-text-meta ui-color-muted">
            <Undo2 size={12} className="animate-pulse" />
            <span>
              {t({
                id: "transcriptions.item.reverting",
                message: "Reverting...",
              })}
            </span>
          </div>
        )}
      </div>
    </div>
  );
};

export default React.memo(TranscriptionItem);
