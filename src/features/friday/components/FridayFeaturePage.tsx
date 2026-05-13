import { useState } from "react";
import { ArrowRight, CheckCircle2, ShieldCheck } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { FRIDAY_FEATURE_SPECS, type FridayAssistantView } from "../pageData";

export function FridayFeaturePage({ view }: { view: FridayAssistantView }) {
  const spec = FRIDAY_FEATURE_SPECS[view];
  const [isPrepared, setIsPrepared] = useState(false);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto pb-6">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="max-w-2xl">
          <div className="ui-text-section-label ui-color-muted">{spec.eyebrow}</div>
          <h1 className="mt-2 text-2xl font-semibold tracking-tight text-[var(--foreground)]">
            {spec.title}
          </h1>
          <p className="mt-2 text-sm leading-6 text-[var(--muted-foreground)]">{spec.summary}</p>
        </div>
        <Badge variant="outline" className="border-[var(--border)] bg-[var(--secondary)]">
          {spec.status}
        </Badge>
      </div>

      <div className="grid gap-3 lg:grid-cols-[1fr_340px]">
        <Card className="py-0">
          <CardContent className="p-4">
            <div className="flex items-center justify-between gap-3">
              <div>
                <div className="text-sm font-semibold text-[var(--foreground)]">
                  Workspace state
                </div>
                <p className="mt-1 text-sm leading-6 text-[var(--muted-foreground)]">
                  {spec.emptyState}
                </p>
              </div>
              <Button type="button" onClick={() => setIsPrepared(true)}>
                {isPrepared ? "Prepared" : spec.primaryAction}
                <ArrowRight size={16} />
              </Button>
            </div>
            <div className="mt-4 rounded-lg border border-dashed border-[var(--border)] bg-[var(--background)] p-6 text-center">
              <div className="text-sm font-semibold text-[var(--foreground)]">
                {isPrepared ? "Workspace prepared" : spec.emptyState}
              </div>
              <p className="mx-auto mt-2 max-w-xl text-sm leading-6 text-[var(--muted-foreground)]">
                {isPrepared
                  ? "Friday created a local draft area for this workflow. Add files, instructions, or a prompt to continue."
                  : "Choose an action when you are ready. Friday will keep the workflow local unless you enable a connector or provider."}
              </p>
            </div>
            <div className="mt-3 grid gap-2 md:grid-cols-3">
              {[
                ["Empty", isPrepared ? "Cleared" : "Waiting for input"],
                ["Loading", isPrepared ? "Ready to run" : "Idle"],
                ["Error", "No blocked providers"],
              ].map(([label, value]) => (
                <div
                  key={label}
                  className="rounded-md border border-[var(--border)] bg-[var(--secondary)] px-3 py-2"
                >
                  <div className="text-[10px] uppercase tracking-[0.16em] text-[var(--muted-foreground)]">
                    {label}
                  </div>
                  <div className="mt-1 truncate text-xs font-medium text-[var(--foreground)]">
                    {value}
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <div className="space-y-3">
          <Card className="py-0">
            <CardContent className="p-4">
              <div className="flex items-center gap-2 text-sm font-semibold text-[var(--foreground)]">
                <ShieldCheck size={16} />
                Data boundary
              </div>
              <p className="mt-2 text-xs leading-5 text-[var(--muted-foreground)]">
                {spec.dataBoundary}
              </p>
            </CardContent>
          </Card>
          <Card className="py-0">
            <CardContent className="p-4">
              <div className="text-sm font-semibold text-[var(--foreground)]">Capabilities</div>
              <div className="mt-3 space-y-2">
                {spec.capabilities.map((capability) => (
                  <div
                    key={capability}
                    className="flex items-center gap-2 text-xs text-[var(--muted-foreground)]"
                  >
                    <CheckCircle2 size={14} className="text-[var(--foreground)]" />
                    {capability}
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
