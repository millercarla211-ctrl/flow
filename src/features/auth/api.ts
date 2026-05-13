import { emit } from "@tauri-apps/api/event";
import { createAuthClient } from "better-auth/react";

export type User = {
  $id: string;
  $createdAt: string;
  $updatedAt: string;
  name: string;
  email: string;
  labels: string[];
  prefs: Record<string, unknown>;
};

export type Session = {
  $id: string;
  current: boolean;
  osName: string;
  clientName: string;
  countryName: string;
};

export type SessionList = {
  total: number;
  sessions: Session[];
};

export type Jwt = {
  jwt: string;
};

type AuthError = {
  message?: string;
  status?: number;
  statusText?: string;
};

type AuthResult<T> = Promise<{
  data: T | null;
  error: AuthError | null;
}>;

type BetterAuthUser = {
  id: string;
  name: string;
  email: string;
  emailVerified: boolean;
  image?: string | null;
  createdAt: Date | string | number;
  updatedAt: Date | string | number;
};

type BetterAuthSession = {
  id: string;
  token: string;
  userId: string;
  expiresAt: Date | string | number;
  createdAt: Date | string | number;
  updatedAt: Date | string | number;
  ipAddress?: string | null;
  userAgent?: string | null;
};

type BetterAuthClient = {
  signUp: {
    email(input: {
      name: string;
      email: string;
      password: string;
      rememberMe?: boolean;
    }): AuthResult<{ token: string | null; user: BetterAuthUser }>;
  };
  signIn: {
    email(input: {
      email: string;
      password: string;
      rememberMe?: boolean;
    }): AuthResult<{ redirect: boolean; token: string; user: BetterAuthUser }>;
  };
  signOut(): AuthResult<unknown>;
  getSession(): AuthResult<{ session: BetterAuthSession; user: BetterAuthUser } | null>;
  updateUser(input: { name?: string; image?: string | null }): AuthResult<{ status: boolean }>;
  changePassword(input: {
    currentPassword: string;
    newPassword: string;
    revokeOtherSessions?: boolean;
  }): AuthResult<{ token: string | null; user: BetterAuthUser }>;
  listSessions(): AuthResult<BetterAuthSession[]>;
  revokeSession(input: { token: string }): AuthResult<{ status: boolean }>;
  revokeSessions(): AuthResult<{ status: boolean }>;
};

const authBaseUrl = process.env.NEXT_PUBLIC_FLOW_AUTH_BASE_URL?.trim();
const authConfigured = Boolean(authBaseUrl);
const authClient = createAuthClient({
  baseURL: authBaseUrl || "http://127.0.0.1:0",
}) as unknown as BetterAuthClient;

function requireAuthClient(): BetterAuthClient {
  if (!authConfigured) {
    throw new Error("Flow account sync is not configured for this local build.");
  }
  return authClient;
}

function emitAuthChanged() {
  emit("auth:changed").catch(() => {});
}

function errorMessage(error: AuthError | null, fallback: string): string {
  return error?.message || error?.statusText || fallback;
}

async function unwrapAuth<T>(request: AuthResult<T>, fallback: string): Promise<T> {
  const response = await request;
  if (response.error) {
    throw new Error(errorMessage(response.error, fallback));
  }
  if (response.data === null) {
    throw new Error(fallback);
  }
  return response.data;
}

function toIsoString(value: Date | string | number | null | undefined): string {
  if (!value) return new Date().toISOString();
  if (value instanceof Date) return value.toISOString();
  return new Date(value).toISOString();
}

function mapUser(user: BetterAuthUser): User {
  return {
    $id: user.id,
    $createdAt: toIsoString(user.createdAt),
    $updatedAt: toIsoString(user.updatedAt),
    name: user.name,
    email: user.email,
    labels: ["cloud"],
    prefs: user.image ? { avatar: user.image } : {},
  };
}

function parseClientName(userAgent: string | null | undefined): string {
  const ua = userAgent ?? "";
  if (ua.includes("Edg/")) return "Microsoft Edge";
  if (ua.includes("Chrome/")) return "Chrome";
  if (ua.includes("Firefox/")) return "Firefox";
  if (ua.includes("Safari/")) return "Safari";
  if (ua.includes("Tauri")) return "Flow Desktop";
  return "Flow";
}

function parseOsName(userAgent: string | null | undefined): string {
  const ua = userAgent ?? "";
  if (ua.includes("Windows")) return "Windows";
  if (ua.includes("Mac OS") || ua.includes("Macintosh")) return "macOS";
  if (ua.includes("Linux")) return "Linux";
  if (ua.includes("Android")) return "Android";
  if (ua.includes("iPhone") || ua.includes("iPad")) return "iOS";
  return "Current device";
}

function mapSession(session: BetterAuthSession, currentToken: string | null): Session {
  return {
    $id: session.token,
    current: session.token === currentToken,
    osName: parseOsName(session.userAgent),
    clientName: parseClientName(session.userAgent),
    countryName: session.ipAddress ? "Network session" : "Local session",
  };
}

export async function createAccount(email: string, password: string, name?: string): Promise<User> {
  const client = requireAuthClient();
  const data = await unwrapAuth(
    client.signUp.email({
      email,
      password,
      name: name?.trim() || email.split("@")[0] || "Flow User",
      rememberMe: true,
    }),
    "Failed to create your Flow account.",
  );
  emitAuthChanged();
  return mapUser(data.user);
}

export async function loginWithEmail(email: string, password: string): Promise<User> {
  const client = requireAuthClient();
  const data = await unwrapAuth(
    client.signIn.email({
      email,
      password,
      rememberMe: true,
    }),
    "Failed to sign in.",
  );
  emitAuthChanged();
  return mapUser(data.user);
}

export async function logout(): Promise<void> {
  const client = requireAuthClient();
  await unwrapAuth(client.signOut(), "Failed to sign out.");
  emitAuthChanged();
}

export async function logoutAll(): Promise<void> {
  const client = requireAuthClient();
  await unwrapAuth(client.revokeSessions(), "Failed to sign out all sessions.");
  emitAuthChanged();
}

export async function getCurrentUser(): Promise<User | null> {
  if (!authConfigured) return null;
  try {
    const response = await authClient.getSession();
    if (response.error || !response.data?.user) return null;
    return mapUser(response.data.user);
  } catch {
    return null;
  }
}

export async function createJwt(): Promise<Jwt> {
  const client = requireAuthClient();
  const data = await unwrapAuth(client.getSession(), "No active Flow auth session.");
  if (!data?.session.token) throw new Error("No active Flow auth session.");
  return { jwt: data.session.token };
}

export async function updateName(name: string): Promise<User> {
  const client = requireAuthClient();
  await unwrapAuth(client.updateUser({ name }), "Failed to update your name.");
  const user = await getCurrentUser();
  if (!user) throw new Error("Your Flow session expired.");
  emitAuthChanged();
  return user;
}

export async function updatePassword(newPassword: string, oldPassword: string): Promise<User> {
  const client = requireAuthClient();
  const data = await unwrapAuth(
    client.changePassword({
      currentPassword: oldPassword,
      newPassword,
      revokeOtherSessions: false,
    }),
    "Failed to update your password.",
  );
  emitAuthChanged();
  return mapUser(data.user);
}

export async function listSessions(): Promise<SessionList> {
  const client = requireAuthClient();
  const [sessions, current] = await Promise.all([
    unwrapAuth(client.listSessions(), "Failed to load sessions."),
    client.getSession().catch(() => ({ data: null, error: null })),
  ]);

  const currentToken = current.data?.session.token ?? null;
  return {
    total: sessions.length,
    sessions: sessions.map((session) => mapSession(session, currentToken)),
  };
}

export async function deleteSessionById(sessionId: string): Promise<void> {
  const client = requireAuthClient();
  await unwrapAuth(client.revokeSession({ token: sessionId }), "Failed to revoke session.");
}

// Cloud usage stats (localStorage cache)

export type CloudUsageStats = {
  cloud_minutes_this_month: number;
  cloud_hours_lifetime: number;
  cloud_transcriptions_count: number;
  cloud_transcriptions_this_month: number;
};

const USAGE_CACHE_KEY = "flow_cloud_usage_cache";

const EMPTY_USAGE: CloudUsageStats = {
  cloud_minutes_this_month: 0,
  cloud_hours_lifetime: 0,
  cloud_transcriptions_count: 0,
  cloud_transcriptions_this_month: 0,
};

type UsageCache = {
  stats: CloudUsageStats;
  userId: string;
  monthKey: string;
};

function getMonthKey(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
}

function setCachedUsageStats(userId: string, stats: CloudUsageStats): void {
  const cache: UsageCache = { stats, userId, monthKey: getMonthKey() };
  localStorage.setItem(USAGE_CACHE_KEY, JSON.stringify(cache));
}

export function getCachedUsageStats(userId: string): CloudUsageStats | null {
  try {
    const cached = localStorage.getItem(USAGE_CACHE_KEY);
    if (!cached) return null;
    const data: UsageCache = JSON.parse(cached);
    if (data.userId !== userId || data.monthKey !== getMonthKey()) {
      localStorage.removeItem(USAGE_CACHE_KEY);
      return null;
    }
    return data.stats;
  } catch {
    return null;
  }
}

export async function getCloudUsageStats(userId: string): Promise<CloudUsageStats> {
  const stats = getCachedUsageStats(userId) ?? EMPTY_USAGE;
  setCachedUsageStats(userId, stats);
  return stats;
}
