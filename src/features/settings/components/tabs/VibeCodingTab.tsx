import { useLingui } from "@lingui/react/macro";
import { motion, type Variants } from "framer-motion";
import { Braces, Code2, FileCode2, ScanText, Terminal } from "lucide-react";
import type { ReactNode } from "react";
import ToggleSwitch from "../../../../shared/ui/ToggleSwitch";

type VibeCodingTabProps = {
  variants: Variants;
  enabled: boolean;
  onEnabledChange: (value: boolean) => void;
  variableRecognition: boolean;
  onVariableRecognitionChange: (value: boolean) => void;
  fileTagging: boolean;
  onFileTaggingChange: (value: boolean) => void;
  includeWindowContext: boolean;
  onIncludeWindowContextChange: (value: boolean) => void;
};

const supportedApps = ["Cursor", "VS Code", "Windsurf", "Terminal", "PowerShell"] as const;

export default function VibeCodingTab({
  variants,
  enabled,
  onEnabledChange,
  variableRecognition,
  onVariableRecognitionChange,
  fileTagging,
  onFileTaggingChange,
  includeWindowContext,
  onIncludeWindowContextChange,
}: VibeCodingTabProps) {
  const { t } = useLingui();
  const childDisabled = !enabled;

  return (
    <motion.div
      key="vibe"
      variants={variants}
      initial="hidden"
      animate="visible"
      exit="exit"
      className="space-y-6"
    >
      <div className="space-y-1">
        <h2 className="ui-text-title font-normal ui-color-primary">
          {t({
            id: "settings.vibe_coding.title",
            message: "Vibe Coding",
          })}
        </h2>
        <p className="ui-text-body-sm ui-color-muted max-w-xl">
          {t({
            id: "settings.vibe_coding.subtitle",
            message:
              "Developer dictation keeps symbols, files, commands, and active editor hints intact.",
          })}
        </p>
      </div>

      <section className="rounded-xl border border-border-primary bg-surface-surface p-4">
        <SettingRow
          icon={<Code2 size={16} aria-hidden="true" />}
          title={t({
            id: "settings.vibe_coding.master.title",
            message: "Coding mode",
          })}
          body={t({
            id: "settings.vibe_coding.master.body",
            message: "Use technical cleanup rules whenever the active style is Coding.",
          })}
          enabled={enabled}
          onToggle={() => onEnabledChange(!enabled)}
          ariaLabel={t({
            id: "settings.vibe_coding.master.aria",
            message: "Toggle Vibe Coding",
          })}
        />

        <div className="mt-4 grid grid-cols-2 gap-2">
          {supportedApps.map((app) => (
            <div
              key={app}
              className="rounded-lg border border-border-primary bg-surface-elevated px-3 py-2 ui-text-label ui-color-secondary"
            >
              {app}
            </div>
          ))}
        </div>
      </section>

      <section className="grid grid-cols-1 gap-3">
        <SettingRow
          icon={<Braces size={16} aria-hidden="true" />}
          title={t({
            id: "settings.vibe_coding.variables.title",
            message: "Variable recognition",
          })}
          body={t({
            id: "settings.vibe_coding.variables.body",
            message:
              "Preserve technical casing for variables, functions, classes, packages, and commands.",
          })}
          enabled={variableRecognition}
          disabled={childDisabled}
          onToggle={() => onVariableRecognitionChange(!variableRecognition)}
          ariaLabel={t({
            id: "settings.vibe_coding.variables.aria",
            message: "Toggle variable recognition",
          })}
        />

        <SettingRow
          icon={<FileCode2 size={16} aria-hidden="true" />}
          title={t({
            id: "settings.vibe_coding.files.title",
            message: "File tagging",
          })}
          body={t({
            id: "settings.vibe_coding.files.body",
            message:
              "Keep @file references and code extensions when dictating into AI coding chats.",
          })}
          enabled={fileTagging}
          disabled={childDisabled}
          onToggle={() => onFileTaggingChange(!fileTagging)}
          ariaLabel={t({
            id: "settings.vibe_coding.files.aria",
            message: "Toggle file tagging",
          })}
        />

        <SettingRow
          icon={<ScanText size={16} aria-hidden="true" />}
          title={t({
            id: "settings.vibe_coding.context.title",
            message: "Editor context",
          })}
          body={t({
            id: "settings.vibe_coding.context.body",
            message: "Use the active app and window title as spelling context for technical names.",
          })}
          enabled={includeWindowContext}
          disabled={childDisabled}
          onToggle={() => onIncludeWindowContextChange(!includeWindowContext)}
          ariaLabel={t({
            id: "settings.vibe_coding.context.aria",
            message: "Toggle editor context",
          })}
        />
      </section>

      <section className="rounded-xl border border-border-primary bg-surface-surface p-4">
        <div className="flex items-center gap-2 ui-text-body-sm-strong ui-color-primary">
          <Terminal size={15} aria-hidden="true" />
          {t({
            id: "settings.vibe_coding.terminal.title",
            message: "Terminal transform",
          })}
        </div>
        <p className="mt-2 ui-text-body-sm ui-color-muted">
          {t({
            id: "settings.vibe_coding.terminal.body",
            message:
              "Terminal command transforms follow the same casing, path, package, and flag rules.",
          })}
        </p>
      </section>
    </motion.div>
  );
}

function SettingRow({
  icon,
  title,
  body,
  enabled,
  disabled = false,
  onToggle,
  ariaLabel,
}: {
  icon: ReactNode;
  title: string;
  body: string;
  enabled: boolean;
  disabled?: boolean;
  onToggle: () => void;
  ariaLabel: string;
}) {
  return (
    <div
      className={`flex items-center justify-between gap-4 rounded-xl border border-border-primary bg-surface-surface p-4 ${
        disabled ? "opacity-60" : ""
      }`}
    >
      <div className="min-w-0">
        <div className="flex items-center gap-2 ui-text-body-sm-strong ui-color-primary">
          <span className="text-content-muted">{icon}</span>
          {title}
        </div>
        <p className="mt-1 ui-text-body-sm ui-color-muted">{body}</p>
      </div>
      <ToggleSwitch
        enabled={enabled}
        disabled={disabled}
        onToggle={onToggle}
        ariaLabel={ariaLabel}
        size="md"
      />
    </div>
  );
}
