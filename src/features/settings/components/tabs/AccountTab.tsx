import { useLingui } from "@lingui/react/macro";
import { motion, type Variants } from "framer-motion";
import { Lock, Loader2 } from "lucide-react";
import { useState } from "react";
import AccountView from "../AccountView";
import { createAccount, loginWithEmail, type User as AuthUser } from "../../../auth/api";

type AccountTabProps = {
  variants: Variants;
  authLoading: boolean;
  currentUser: AuthUser | null;
  cloudSyncEnabled: boolean;
  setCloudSyncEnabled: (value: boolean) => void;
  onUpdateUser: () => Promise<void>;
  handleSignOut: () => Promise<void>;
  handleCancelAuth: () => void;
};

const AccountTab = ({
  variants,
  authLoading,
  currentUser,
  cloudSyncEnabled,
  setCloudSyncEnabled,
  onUpdateUser,
  handleSignOut,
  handleCancelAuth,
}: AccountTabProps) => {
  const { t } = useLingui();
  const [authMode, setAuthMode] = useState<"sign-in" | "sign-up">("sign-in");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const cloudStatus = t({
    id: "settings.account.cloud.status",
    message: "optional",
  });

  const handleAuthSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSubmitError(null);
    setSubmitting(true);

    try {
      if (authMode === "sign-up") {
        await createAccount(email.trim(), password, name.trim());
      } else {
        await loginWithEmail(email.trim(), password);
      }

      setPassword("");
      await onUpdateUser();
    } catch (error) {
      setSubmitError(
        error instanceof Error
          ? error.message
          : t({
              id: "settings.account.auth.failed",
              message: "Authentication failed.",
            }),
      );
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <motion.div
      key="account"
      variants={variants}
      initial="hidden"
      animate="visible"
      exit="exit"
      className="space-y-4"
    >
      <header>
        <h1 className="ui-text-title-lg font-medium ui-color-primary">
          {t({
            id: "settings.account.title",
            message: "Account",
          })}
        </h1>
        <p className="mt-1 ui-text-body-sm ui-color-muted">
          {t({
            id: "settings.account.subtitle",
            message: "Manage your profile, sessions, and subscription.",
          })}
        </p>
      </header>

      {authLoading ? (
        <div className="flex flex-col items-center justify-center py-16">
          <Loader2 size={24} className="animate-spin text-content-muted mb-3" />
          <p className="ui-text-body-sm ui-color-muted mb-3">
            {t({
              id: "settings.account.loading",
              message: "Loading...",
            })}
          </p>
          <button
            onClick={handleCancelAuth}
            className="ui-text-label ui-color-disabled hover:text-content-muted transition-colors"
          >
            {t({
              id: "settings.account.cancel",
              message: "Cancel",
            })}
          </button>
        </div>
      ) : currentUser ? (
        <AccountView
          currentUser={currentUser}
          cloudSyncEnabled={cloudSyncEnabled}
          onCloudSyncToggle={() => setCloudSyncEnabled(!cloudSyncEnabled)}
          onUserUpdate={async () => {
            await onUpdateUser();
          }}
          onSignOut={handleSignOut}
        />
      ) : (
        <div className="rounded-2xl border border-border-primary bg-surface-tertiary p-5">
          <div className="flex flex-col gap-5 lg:flex-row lg:items-start">
            <div className="mt-0.5 rounded-xl p-2">
              <Lock size={16} className="text-content-secondary" />
            </div>
            <div className="flex-1 min-w-0">
              <h2 className="ui-text-body-strong font-semibold ui-color-primary">
                {t({
                  id: "settings.account.cloud.title",
                  message: "Flow Sync",
                })}
              </h2>
              <p className="mt-1 ui-text-label ui-color-muted leading-relaxed">
                {t({
                  id: "settings.account.cloud.body",
                  message: `Flow accounts and sync are ${cloudStatus}. Flow stays fully usable offline and local-first.`,
                })}
              </p>
              <div className="mt-3 ui-text-meta ui-color-disabled">
                {t({
                  id: "settings.account.cloud.disabled",
                  message: "Sign in only if you want profile and session continuity.",
                })}
              </div>
            </div>

            <form
              onSubmit={handleAuthSubmit}
              className="w-full rounded-xl border border-border-primary bg-surface-surface p-4 lg:w-[320px]"
            >
              <div className="mb-4 flex rounded-lg border border-border-secondary bg-surface-tertiary p-1">
                <button
                  type="button"
                  onClick={() => setAuthMode("sign-in")}
                  className={`flex-1 rounded-md px-3 py-1.5 ui-text-label transition-colors ${
                    authMode === "sign-in"
                      ? "bg-content-primary text-surface-primary"
                      : "ui-color-muted hover:ui-color-primary"
                  }`}
                >
                  {t({
                    id: "settings.account.auth.sign_in_tab",
                    message: "Sign in",
                  })}
                </button>
                <button
                  type="button"
                  onClick={() => setAuthMode("sign-up")}
                  className={`flex-1 rounded-md px-3 py-1.5 ui-text-label transition-colors ${
                    authMode === "sign-up"
                      ? "bg-content-primary text-surface-primary"
                      : "ui-color-muted hover:ui-color-primary"
                  }`}
                >
                  {t({
                    id: "settings.account.auth.create_tab",
                    message: "Create",
                  })}
                </button>
              </div>

              <div className="space-y-3">
                {authMode === "sign-up" && (
                  <input
                    value={name}
                    onChange={(event) => setName(event.target.value)}
                    placeholder={t({
                      id: "settings.account.auth.name",
                      message: "Name",
                    })}
                    className="w-full rounded-lg border border-border-secondary bg-surface-tertiary px-3 py-2 ui-text-body-sm ui-color-primary outline-hidden transition-colors placeholder-content-disabled focus:border-border-hover"
                  />
                )}
                <input
                  value={email}
                  onChange={(event) => setEmail(event.target.value)}
                  type="email"
                  autoComplete="email"
                  required
                  placeholder={t({
                    id: "settings.account.auth.email",
                    message: "Email",
                  })}
                  className="w-full rounded-lg border border-border-secondary bg-surface-tertiary px-3 py-2 ui-text-body-sm ui-color-primary outline-hidden transition-colors placeholder-content-disabled focus:border-border-hover"
                />
                <input
                  value={password}
                  onChange={(event) => setPassword(event.target.value)}
                  type="password"
                  autoComplete={authMode === "sign-in" ? "current-password" : "new-password"}
                  required
                  minLength={8}
                  placeholder={t({
                    id: "settings.account.auth.password",
                    message: "Password",
                  })}
                  className="w-full rounded-lg border border-border-secondary bg-surface-tertiary px-3 py-2 ui-text-body-sm ui-color-primary outline-hidden transition-colors placeholder-content-disabled focus:border-border-hover"
                />
              </div>

              {submitError && (
                <p className="mt-3 rounded-lg border border-red-500/20 bg-red-500/10 px-3 py-2 ui-text-label ui-color-error-strong">
                  {submitError}
                </p>
              )}

              <button
                type="submit"
                disabled={submitting}
                className="mt-4 flex w-full items-center justify-center rounded-lg bg-content-primary px-3 py-2 ui-text-body-sm-strong text-surface-primary transition-colors hover:bg-content-secondary disabled:opacity-60"
              >
                {submitting ? (
                  <Loader2 size={14} className="animate-spin" />
                ) : authMode === "sign-up" ? (
                  t({
                    id: "settings.account.auth.create_submit",
                    message: "Create account",
                  })
                ) : (
                  t({
                    id: "settings.account.auth.sign_in_submit",
                    message: "Sign in",
                  })
                )}
              </button>
            </form>
          </div>
        </div>
      )}
    </motion.div>
  );
};

export default AccountTab;
