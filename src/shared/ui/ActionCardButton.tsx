import type {
  ButtonHTMLAttributes,
  CSSProperties,
  ReactNode,
} from "react";

export type ActionCardAccent = {
  borderColor: string;
  backgroundColor: string;
  shadowColor: string;
  insetColor: string;
};

export const ACTION_CARD_BUTTON_ACCENTS = {
  interactive: {
    borderColor: "var(--color-interactive-30)",
    backgroundColor: "var(--color-interactive-10)",
    shadowColor: "var(--color-interactive-20)",
    insetColor: "var(--color-interactive-10)",
  },
  amber: {
    borderColor: "var(--color-cloud-30)",
    backgroundColor: "var(--color-cloud-10)",
    shadowColor: "var(--color-cloud-20)",
    insetColor: "var(--color-cloud-10)",
  },
  cloud: {
    borderColor: "var(--color-cloud-30)",
    backgroundColor: "var(--color-cloud-10)",
    shadowColor: "var(--color-cloud-20)",
    insetColor: "var(--color-cloud-10)",
  },
  local: {
    borderColor: "var(--color-local-30)",
    backgroundColor: "var(--color-local-10)",
    shadowColor: "var(--color-local-20)",
    insetColor: "var(--color-local-10)",
  },
  accent: {
    borderColor: "var(--color-accent-30)",
    backgroundColor: "var(--color-accent-10)",
    shadowColor: "var(--color-accent-20)",
    insetColor: "var(--color-accent-10)",
  },
  error: {
    borderColor: "rgba(239, 68, 68, 0.3)",
    backgroundColor: "rgba(239, 68, 68, 0.08)",
    shadowColor: "rgba(239, 68, 68, 0.2)",
    insetColor: "rgba(239, 68, 68, 0.1)",
  },
} satisfies Record<string, ActionCardAccent>;

type ActionCardButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  title: string;
  description?: string;
  icon?: ReactNode;
  accent?: Partial<ActionCardAccent>;
  iconClassName?: string;
  titleClassName?: string;
  descriptionClassName?: string;
  contentClassName?: string;
  fullWidth?: boolean;
};

const joinClasses = (...classes: Array<string | false | null | undefined>) =>
  classes.filter(Boolean).join(" ");

const BASE_SHADOW =
  "0 3px 0 -1px rgba(0, 0, 0, 0.5), inset 0 1px 0 0 rgba(255, 255, 255, 0.06)";

const HOVER_SHADOW =
  "0 3px 0 -1px rgba(0, 0, 0, 0.5), inset 0 1px 0 0 rgba(255, 255, 255, 0.12)";

const ActionCardButton = ({
  title,
  description,
  icon,
  accent,
  iconClassName,
  titleClassName,
  descriptionClassName,
  contentClassName,
  fullWidth = true,
  className,
  style,
  type = "button",
  ...props
}: ActionCardButtonProps) => {
  const resolvedAccent = {
    ...ACTION_CARD_BUTTON_ACCENTS.interactive,
    ...accent,
  };
  const actionStyle = {
    "--action-card-border": resolvedAccent.borderColor,
    "--action-card-background": resolvedAccent.backgroundColor,
    "--action-card-hover-shadow": fullWidth
      ? HOVER_SHADOW
      : "var(--shadow-sm)",
    "--action-card-rest-shadow": fullWidth ? BASE_SHADOW : "none",
    ...style,
  } as CSSProperties;

  return (
    <button
      type={type}
      className={joinClasses(
        "group rounded-lg border border-border-primary bg-surface-surface text-left [box-shadow:var(--action-card-rest-shadow)] outline-hidden transition-[transform,box-shadow,border-color,background-color] duration-100 ease-out hover:border-[var(--action-card-border)] hover:bg-[var(--action-card-background)] hover:[box-shadow:var(--action-card-hover-shadow)] active:[box-shadow:none] focus-visible:ring-2 focus-visible:ring-border-hover disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:translate-y-0 disabled:hover:border-border-primary disabled:hover:bg-surface-surface disabled:hover:[box-shadow:var(--action-card-rest-shadow)]",
        fullWidth
          ? "w-full px-3 py-2.5 active:translate-y-[2px]"
          : "inline-flex w-auto px-2.5 py-1",
        className,
      )}
      style={actionStyle}
      {...props}
    >
      <div
        className={joinClasses(
          fullWidth ? "flex items-start gap-3" : "flex w-full items-center gap-1.5",
          contentClassName,
        )}
      >
        {icon ? (
          <span
            aria-hidden="true"
            className={joinClasses(
              fullWidth
                ? "mt-0.5 flex size-5 shrink-0 items-center justify-center ui-color-primary"
                : "flex shrink-0 items-center justify-center text-[var(--color-text-muted)] transition-colors duration-150 group-hover:text-[var(--color-text-primary)]",
              iconClassName,
            )}
          >
            {icon}
          </span>
        ) : null}

        <div className="min-w-0">
          <span
            className={joinClasses(
              fullWidth
                ? "ui-text-label-strong ui-color-primary block"
                : "ui-text-button ui-color-secondary block",
              titleClassName,
            )}
          >
            {title}
          </span>
          {description ? (
            <span
              className={joinClasses(
                "ui-text-micro ui-color-disabled block",
                descriptionClassName,
              )}
            >
              {description}
            </span>
          ) : null}
        </div>
      </div>
    </button>
  );
};

export default ActionCardButton;
