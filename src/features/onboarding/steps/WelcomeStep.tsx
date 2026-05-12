import { useLingui } from "@lingui/react/macro";
import { motion } from "framer-motion";
import type { ReactNode } from "react";
import {
  ArrowRight,
  BadgeCheck,
  Cpu,
  LockKeyhole,
  MessageSquareText,
  Mic,
  Sparkles,
  WifiOff,
  Zap,
} from "lucide-react";
import DotMatrix from "../../../shared/ui/DotMatrix";
import type { TranscriptionMode } from "../../../types";
import type { StepMotionProps } from "./shared";

interface WelcomeStepProps {
  stepMotionProps: StepMotionProps;
  hasStepTransitioned: boolean;
  selectedMode: TranscriptionMode;
  onSelectMode: (mode: TranscriptionMode) => void;
  onNext: () => void;
}

const rawSpeech =
  "um can you turn this into a clear update for the team and make it sound confident";
const polishedSpeech = "Please turn this into a clear, confident update for the team.";

const useCases = ["Messages", "Code", "Notes", "Support", "Docs"];

export function WelcomeStep({
  stepMotionProps,
  hasStepTransitioned,
  selectedMode,
  onSelectMode,
  onNext,
}: WelcomeStepProps) {
  const { t } = useLingui();

  return (
    <motion.div
      key="welcome"
      {...stepMotionProps}
      initial={hasStepTransitioned ? "enter" : false}
      className="grid w-full max-w-5xl items-center gap-8 lg:grid-cols-[0.94fr_1.06fr]"
    >
      <section className="text-left">
        <motion.div
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.28, ease: "easeOut" }}
          className="mb-5 inline-flex items-center gap-2 rounded-full border border-border-primary bg-surface-surface px-3 py-1.5 ui-text-meta-strong ui-color-muted"
        >
          <span className="flex h-5 w-5 items-center justify-center rounded-full border border-border-primary bg-surface-elevated">
            <Mic size={11} aria-hidden="true" />
          </span>
          {t({
            id: "onboarding.welcome.eyebrow",
            message: "Free, unlimited dictation for every app",
          })}
        </motion.div>

        <h1 className="max-w-xl text-[2.75rem] font-semibold leading-[0.98] tracking-normal text-content-primary">
          {t({
            id: "onboarding.welcome.title",
            message: "Flow writes as fast as you speak.",
          })}
        </h1>

        <p className="mt-4 max-w-lg ui-text-body-lg text-content-muted">
          {t({
            id: "onboarding.welcome.subtitle",
            message:
              "Speak naturally and Flow turns it into clean text wherever you work. Local by default, private by design, and built for unlimited daily use.",
          })}
        </p>

        <div className="mt-5 flex flex-wrap gap-2">
          {useCases.map((useCase) => (
            <span
              key={useCase}
              className="rounded-full border border-border-primary bg-surface-surface px-3 py-1 ui-text-meta-strong text-content-secondary"
            >
              {useCase}
            </span>
          ))}
        </div>

        <div className="mt-7 grid gap-2 sm:grid-cols-3">
          <IntroMetric icon={<BadgeCheck size={14} />} value="Free" label="no word limits" />
          <IntroMetric icon={<Zap size={14} />} value="Fast" label="local transcription" />
          <IntroMetric icon={<WifiOff size={14} />} value="Private" label="on-device first" />
        </div>

        <div className="mt-7 rounded-2xl border border-border-primary bg-surface-surface p-2">
          <button
            type="button"
            onClick={() => onSelectMode("local")}
            className={`flex w-full items-center justify-between gap-4 rounded-xl border px-4 py-3 text-left transition-all ${
              selectedMode === "local"
                ? "border-border-hover bg-surface-elevated shadow-[inset_0_1px_0_rgba(255,255,255,0.08)]"
                : "border-border-primary bg-surface-secondary hover:border-border-secondary"
            }`}
            aria-pressed={selectedMode === "local"}
            aria-label={t({
              id: "onboarding.welcome.local.aria",
              message: "Select Local mode, privacy-first on-device transcription",
            })}
          >
            <span className="flex min-w-0 items-center gap-3">
              <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-border-primary bg-bg-primary ui-color-primary">
                <Cpu size={16} aria-hidden="true" />
              </span>
              <span className="min-w-0">
                <span className="block ui-text-body-sm-strong ui-color-primary">
                  {t({
                    id: "onboarding.welcome.local.title",
                    message: "Free unlimited local mode",
                  })}
                </span>
                <span className="block truncate ui-text-meta ui-color-muted">
                  {t({
                    id: "onboarding.welcome.local.feature.models",
                    message:
                      "Fast on-device transcription, optional remote providers when you choose",
                  })}
                </span>
              </span>
            </span>
            <span className="rounded-full border border-border-primary bg-surface-surface px-2 py-1 ui-text-micro ui-color-muted">
              {t({
                id: "onboarding.welcome.local.feature.free",
                message: "No account",
              })}
            </span>
          </button>
        </div>

        <button
          onClick={onNext}
          className="mt-6 inline-flex items-center justify-center gap-2 rounded-full bg-content-primary px-5 py-2.5 ui-text-body-lg font-semibold text-surface-secondary transition-colors hover:bg-white"
        >
          {t({
            id: "onboarding.welcome.cta",
            message: "Start using Flow",
          })}
          <ArrowRight size={15} aria-hidden="true" />
        </button>
      </section>

      <WelcomeProofPanel />
    </motion.div>
  );
}

function IntroMetric({ icon, value, label }: { icon: ReactNode; value: string; label: string }) {
  return (
    <div className="rounded-xl border border-border-primary bg-surface-surface px-3 py-3">
      <div className="flex items-center justify-between gap-3 ui-color-muted">
        <span>{icon}</span>
        <DotMatrix rows={1} cols={4} activeDots={[0, 2]} dotSize={2} gap={2} />
      </div>
      <div className="mt-3 ui-text-title font-semibold ui-color-primary">{value}</div>
      <div className="mt-0.5 ui-text-micro ui-color-disabled">{label}</div>
    </div>
  );
}

function WelcomeProofPanel() {
  return (
    <motion.section
      initial={{ opacity: 0, scale: 0.98, y: 10 }}
      animate={{ opacity: 1, scale: 1, y: 0 }}
      transition={{ type: "spring", stiffness: 220, damping: 24, delay: 0.05 }}
      className="relative min-h-[430px] overflow-hidden rounded-[28px] border border-border-primary bg-bg-primary p-4 shadow-2xl shadow-black/40"
    >
      <div className="absolute inset-0 opacity-[0.06]">
        <DotMatrix
          rows={18}
          cols={38}
          activeDots={Array.from({ length: 110 }, (_, index) => index * 3)}
          dotSize={2}
          gap={8}
          color="var(--color-text-primary)"
        />
      </div>

      <div className="relative flex h-full flex-col rounded-[22px] border border-border-primary bg-surface-secondary/90 p-4">
        <div className="mb-4 flex items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <span className="flex h-8 w-8 items-center justify-center rounded-full border border-border-primary bg-surface-elevated ui-color-primary">
              <Zap size={15} aria-hidden="true" />
            </span>
            <div>
              <div className="ui-text-body-sm-strong ui-color-primary">Flow in action</div>
              <div className="ui-text-micro ui-color-disabled">speech to polished text</div>
            </div>
          </div>
          <span className="rounded-full border border-border-primary bg-bg-primary px-2.5 py-1 ui-text-micro ui-color-muted">
            local first
          </span>
        </div>

        <div className="grid flex-1 gap-3">
          <SpeechCard title="What you say" icon={<Mic size={13} />} text={rawSpeech} muted />
          <motion.div
            aria-hidden="true"
            className="mx-auto flex h-11 w-[210px] items-center justify-center rounded-full border border-border-primary bg-black/80 px-4 shadow-[0_0_22px_rgba(255,255,255,0.08)]"
            animate={{ y: [0, -2, 0] }}
            transition={{ duration: 2.4, repeat: Infinity, ease: "easeInOut" }}
          >
            <div className="flex items-center gap-1.5">
              {Array.from({ length: 22 }, (_, index) => (
                <motion.span
                  key={index}
                  className="w-1 rounded-full bg-content-primary"
                  animate={{ height: [4, 8 + ((index * 7) % 18), 5] }}
                  transition={{
                    duration: 0.9,
                    repeat: Infinity,
                    delay: index * 0.025,
                    ease: "easeInOut",
                  }}
                />
              ))}
            </div>
          </motion.div>
          <SpeechCard
            title="What Flow writes"
            icon={<MessageSquareText size={13} />}
            text={polishedSpeech}
          />
        </div>

        <div className="mt-4 grid grid-cols-3 gap-2">
          <ProofChip icon={<LockKeyhole size={12} />} label="Local STT" />
          <ProofChip icon={<Sparkles size={12} />} label="Auto polish" />
          <ProofChip icon={<Zap size={12} />} label="Fast paste" />
        </div>
      </div>
    </motion.section>
  );
}

function SpeechCard({
  title,
  icon,
  text,
  muted = false,
}: {
  title: string;
  icon: ReactNode;
  text: string;
  muted?: boolean;
}) {
  return (
    <div
      className={`rounded-2xl border p-4 ${
        muted
          ? "border-border-primary bg-surface-surface/75"
          : "border-border-secondary bg-surface-elevated"
      }`}
    >
      <div className="mb-3 flex items-center gap-2 ui-text-meta-strong ui-color-muted">
        {icon}
        {title}
      </div>
      <p
        className={`ui-text-body-sm leading-relaxed ${muted ? "ui-color-muted" : "ui-color-primary"}`}
      >
        {text}
      </p>
    </div>
  );
}

function ProofChip({ icon, label }: { icon: ReactNode; label: string }) {
  return (
    <div className="flex items-center justify-center gap-1.5 rounded-full border border-border-primary bg-bg-primary px-2.5 py-1.5 ui-text-micro ui-color-muted">
      {icon}
      {label}
    </div>
  );
}
