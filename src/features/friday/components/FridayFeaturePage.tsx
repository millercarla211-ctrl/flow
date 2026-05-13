import { CheckCircle2, ShieldCheck } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { FridayLocalWorkspace } from "./FridayLocalWorkspaces";
import { FRIDAY_FEATURE_SPECS, type FridayAssistantView } from "../pageData";

export function FridayFeaturePage({ view }: { view: FridayAssistantView }) {
  const spec = FRIDAY_FEATURE_SPECS[view];

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
            <FridayLocalWorkspace view={view} />
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
