export type FridaySyncHealthStatus = "ready" | "partial" | "local-only" | "error";

export type FridaySyncHealthResult = {
  checkedAt: string;
  latencyMs: number;
  message: string;
  ready: boolean;
  requirements: {
    authUrlConfigured: boolean;
    databaseConfigured: boolean;
    tokenConfigured: boolean;
  };
  route: string;
  status: FridaySyncHealthStatus;
};

type RawFridaySyncHealth = Omit<FridaySyncHealthResult, "route" | "status"> & {
  status?: string;
};

function nowMs() {
  return typeof performance === "undefined" ? Date.now() : performance.now();
}

function normalizeStatus(status: string | undefined): FridaySyncHealthStatus {
  if (status === "ready" || status === "partial" || status === "local-only") return status;
  return "error";
}

export async function checkFridaySyncHealth({
  route = "/api/friday/sync/status",
  timeoutMs = 10_000,
}: {
  route?: string;
  timeoutMs?: number;
} = {}): Promise<FridaySyncHealthResult> {
  const startedAt = nowMs();
  const controller = new AbortController();
  const timeout = window.setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetch(route, {
      method: "GET",
      signal: controller.signal,
    });
    const body = (await response.json().catch(() => null)) as RawFridaySyncHealth | null;
    const latencyMs = Math.round(nowMs() - startedAt);

    if (!body) {
      return {
        checkedAt: new Date().toISOString(),
        latencyMs,
        message: `Sync status returned ${response.status}.`,
        ready: false,
        requirements: {
          authUrlConfigured: false,
          databaseConfigured: false,
          tokenConfigured: false,
        },
        route,
        status: "error",
      };
    }

    return {
      checkedAt: body.checkedAt,
      latencyMs: body.latencyMs || latencyMs,
      message: body.message,
      ready: Boolean(body.ready && response.ok),
      requirements: body.requirements,
      route,
      status: response.ok ? normalizeStatus(body.status) : "error",
    };
  } catch (error) {
    const latencyMs = Math.round(nowMs() - startedAt);
    const message =
      error instanceof DOMException && error.name === "AbortError"
        ? "Sync check timed out."
        : error instanceof Error
          ? error.message
          : "Sync check failed.";

    return {
      checkedAt: new Date().toISOString(),
      latencyMs,
      message,
      ready: false,
      requirements: {
        authUrlConfigured: false,
        databaseConfigured: false,
        tokenConfigured: false,
      },
      route,
      status: "error",
    };
  } finally {
    window.clearTimeout(timeout);
  }
}

