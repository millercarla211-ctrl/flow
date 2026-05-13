import { FlowVoicePanel } from "@/shared/ui/FlowVoicePanel";
import TranscriptionList from "@/features/transcriptions/components/TranscriptionList";

export function VoiceWorkspace({
  modeLabel,
  hint,
  showCleanupButtons,
  isActive,
  historyDisabled,
  onOpenDataSettings,
}: {
  modeLabel: string;
  hint: string;
  showCleanupButtons: boolean;
  isActive: boolean;
  historyDisabled: boolean;
  onOpenDataSettings: () => void;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="shrink-0">
        <div className="ui-text-section-label ui-color-muted">WhisperFlow Beater</div>
        <h1 className="mt-2 text-2xl font-semibold tracking-tight text-[var(--foreground)]">
          Voice
        </h1>
        <p className="mt-2 max-w-2xl text-sm leading-6 text-[var(--muted-foreground)]">
          Fast local dictation, overlay controls, STT, TTS, OCR handoff, dictionary, snippets, and
          paste into focused apps.
        </p>
      </div>
      <div className="mt-5 min-h-0 flex-1">
        <FlowVoicePanel
          modeLabel={modeLabel}
          headline="Free, unlimited local speech"
          hint={hint}
          badges={["WhisperFlow Beater", "STT", "TTS"]}
        />
        <TranscriptionList
          showLlmButtons={showCleanupButtons}
          isActive={isActive}
          historyDisabled={historyDisabled}
          onOpenDataSettings={onOpenDataSettings}
        />
      </div>
    </div>
  );
}
