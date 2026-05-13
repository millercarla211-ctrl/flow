import type { ReactNode } from "react";

import type { FridayModelOption } from "@/features/ai";
import { FRIDAY_LOCAL_MODELS } from "@/features/ai";

export const INPUT_CLASS =
  "h-9 w-full rounded-md border border-[var(--border)] bg-[var(--background)] px-3 text-sm text-[var(--foreground)] outline-none transition-colors placeholder:text-[var(--muted-foreground)] focus:border-[var(--border-hover)]";

export const TEXTAREA_CLASS =
  "min-h-24 w-full resize-none rounded-md border border-[var(--border)] bg-[var(--background)] px-3 py-2 text-sm text-[var(--foreground)] outline-none transition-colors placeholder:text-[var(--muted-foreground)] focus:border-[var(--border-hover)]";

export function EmptyState({ title, body }: { title: string; body: string }) {
  return (
    <div className="rounded-lg border border-dashed border-[var(--border)] bg-[var(--background)] p-5 text-center">
      <div className="text-sm font-semibold text-[var(--foreground)]">{title}</div>
      <p className="mx-auto mt-2 max-w-xl text-sm leading-6 text-[var(--muted-foreground)]">
        {body}
      </p>
    </div>
  );
}

export function RecordShell({
  icon,
  title,
  subtitle,
  children,
}: {
  icon: ReactNode;
  title: string;
  subtitle: string;
  children: ReactNode;
}) {
  return (
    <div className="rounded-lg border border-[var(--border)] bg-[var(--background)] p-3">
      <div className="flex items-start gap-3">
        <div className="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-md border border-[var(--border)] bg-[var(--secondary)]">
          {icon}
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-semibold text-[var(--foreground)]">{title}</div>
          <p className="mt-1 text-xs leading-5 text-[var(--muted-foreground)]">{subtitle}</p>
          {children}
        </div>
      </div>
    </div>
  );
}

export function ModelSelect({
  value,
  onChange,
}: {
  value: string;
  onChange: (modelKey: string) => void;
}) {
  return (
    <select
      className={INPUT_CLASS}
      value={value}
      onChange={(event) => onChange(event.target.value)}
    >
      {FRIDAY_LOCAL_MODELS.map((model: FridayModelOption) => (
        <option key={model.key} value={model.key}>
          {model.label}
        </option>
      ))}
    </select>
  );
}
