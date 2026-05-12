import { useLingui } from "@lingui/react/macro";
import { motion } from "framer-motion";
import { Mail } from "lucide-react";
import type { StepMotionProps } from "./shared";

interface SigninStepProps {
  stepMotionProps: StepMotionProps;
  onNext: () => void;
}

export function SigninStep({ stepMotionProps, onNext }: SigninStepProps) {
  const { t } = useLingui();
  const syncStatus = t({
    id: "onboarding.signin.status",
    message: "optional",
  });

  return (
    <motion.div
      key="local-signin"
      {...stepMotionProps}
      initial="enter"
      className="flex flex-col items-center text-center w-full max-w-sm"
    >
      <div className="mb-4 rounded-2xl bg-local/10 p-4">
        <Mail size={28} className="text-local" />
      </div>
      <h2 className="ui-text-title-lg font-semibold text-content-primary mb-2">
        {t({
          id: "onboarding.signin.title",
          message: "Transcription Sync",
        })}
      </h2>
      <p className="ui-text-body-lg text-content-muted mb-2 leading-relaxed">
        {t({
          id: "onboarding.signin.subtitle",
          message: `Cloud sync is ${syncStatus}.`,
        })}
      </p>
      <p className="ui-text-body-sm text-content-disabled mb-7 leading-relaxed">
        {t({
          id: "onboarding.signin.body",
          message:
            "Flow is ready without an account. Keep everything local, or connect sync later when you want it.",
        })}
      </p>

      <button
        type="button"
        onClick={onNext}
        className="w-full flex items-center justify-center gap-2 rounded-lg bg-content-primary px-5 py-3 ui-text-body-lg font-semibold text-surface-secondary hover:bg-white transition-colors"
      >
        {t({
          id: "onboarding.signin.continue",
          message: "Continue",
        })}
      </button>
    </motion.div>
  );
}
