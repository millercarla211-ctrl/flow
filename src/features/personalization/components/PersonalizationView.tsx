import { useLingui } from "@lingui/react/macro";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import {
  AlertTriangle,
  AppWindow,
  Crosshair,
  Download,
  Globe2,
  Plus,
  RefreshCw,
  Route,
  Sparkles,
  Upload,
} from "lucide-react";
import { useShiftHeld } from "../../../shared/hooks/useShiftHeld";
import ToggleSwitch from "../../../shared/ui/ToggleSwitch";
import DotMatrix from "../../../shared/ui/DotMatrix";
import type { Personality } from "../../../types";
import {
  buildWebsiteIconMap,
  createId,
  formatWebsitePreview,
  normalizeWebsite,
  type InstalledApp,
  type WebsiteIcon,
} from "./personalization-utils";
import PersonalityModal, {
  AppIconBadge,
  WebsiteFavicon,
  type PendingDeletePersonality,
} from "./PersonalityModal";

type StyleTemplate = {
  name: string;
  description: string;
  instructions: string[];
};

type StyleImportItem = {
  name: string;
  enabled: boolean;
  apps: string[];
  websites: string[];
  instructions: string[];
};

type StyleAssignmentConflict = {
  label: string;
  styles: string[];
  kind: "app" | "site";
};

type StyleCoverage = {
  enabledStyles: number;
  assignedApps: number;
  assignedSites: number;
  conflicts: StyleAssignmentConflict[];
};

type ActiveStyleMatch = {
  id: string;
  name: string;
  instruction_count: number;
};

type ActiveStylePreview = {
  permission_granted: boolean;
  context_available: boolean;
  app_name: string | null;
  window_title: string | null;
  url: string | null;
  matches: ActiveStyleMatch[];
};

const isObject = (value: unknown): value is Record<string, unknown> =>
  Boolean(value && typeof value === "object" && !Array.isArray(value));

const normalizeStringList = (value: unknown): string[] => {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const list: string[] = [];
  for (const item of value) {
    if (typeof item !== "string") continue;
    const normalized = item.trim();
    if (!normalized) continue;
    const key = normalized.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    list.push(normalized);
  }
  return list;
};

const normalizeStyleBackup = (value: unknown): StyleImportItem[] => {
  const source = Array.isArray(value)
    ? value
    : isObject(value) && Array.isArray(value.styles)
      ? value.styles
      : isObject(value) && Array.isArray(value.personalities)
        ? value.personalities
        : null;

  if (!source) {
    throw new Error("Clipboard does not contain a Friday style backup.");
  }

  const seen = new Set<string>();
  const styles: StyleImportItem[] = [];
  for (const item of source) {
    if (!isObject(item)) continue;
    const name = typeof item.name === "string" ? item.name.trim() : "";
    if (!name) continue;
    const key = name.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    styles.push({
      name,
      enabled: typeof item.enabled === "boolean" ? item.enabled : true,
      apps: normalizeStringList(item.apps),
      websites: normalizeStringList(item.websites),
      instructions: normalizeStringList(item.instructions),
    });
  }

  if (styles.length === 0) {
    throw new Error("No usable styles were found in the clipboard backup.");
  }

  return styles;
};

const addAssignment = (
  map: Map<string, { label: string; styles: Set<string> }>,
  rawLabel: string,
  styleName: string,
  lowerCase = false,
) => {
  const label = rawLabel.trim();
  if (!label) return;

  const normalizedLabel = lowerCase ? normalizeWebsite(label) : label;
  if (!normalizedLabel) return;

  const key = normalizedLabel.toLowerCase();
  const current = map.get(key) ?? {
    label: normalizedLabel,
    styles: new Set<string>(),
  };
  current.styles.add(styleName);
  map.set(key, current);
};

const createCoverageConflicts = (
  entries: Map<string, { label: string; styles: Set<string> }>,
  kind: "app" | "site",
) =>
  Array.from(entries.values())
    .filter((entry) => entry.styles.size > 1)
    .map((entry) => ({
      label: entry.label,
      styles: Array.from(entry.styles).sort((left, right) => left.localeCompare(right)),
      kind,
    }));

const PersonalizationView = ({ isActive = true }: { isActive?: boolean }) => {
  const { t } = useLingui();
  const [personalities, setPersonalities] = useState<Personality[]>([]);
  const [installedApps, setInstalledApps] = useState<InstalledApp[]>([]);
  const [websiteIconBySite, setWebsiteIconBySite] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activePersonalityId, setActivePersonalityId] = useState<string | null>(null);
  const [pendingDeletePersonality, setPendingDeletePersonality] =
    useState<PendingDeletePersonality | null>(null);
  const [backupStatus, setBackupStatus] = useState<string | null>(null);
  const [backupError, setBackupError] = useState<string | null>(null);
  const [importingBackup, setImportingBackup] = useState(false);
  const [activeStylePreview, setActiveStylePreview] = useState<ActiveStylePreview | null>(null);
  const [activeStylePreviewLoading, setActiveStylePreviewLoading] = useState(false);
  const [activeStylePreviewError, setActiveStylePreviewError] = useState<string | null>(null);
  const hasRequestedIconRefreshRef = useRef(false);
  const websiteIconRefreshKeyRef = useRef<string | null>(null);
  const persistVersionRef = useRef(0);
  const saveTimeoutRef = useRef<number | null>(null);
  const shiftHeld = useShiftHeld(isActive);

  const styleCoverage = useMemo<StyleCoverage>(() => {
    const appAssignments = new Map<string, { label: string; styles: Set<string> }>();
    const siteAssignments = new Map<string, { label: string; styles: Set<string> }>();
    let enabledStyles = 0;

    for (const personality of personalities) {
      if (!personality.enabled) continue;

      enabledStyles += 1;
      for (const app of personality.apps) {
        addAssignment(appAssignments, app, personality.name);
      }
      for (const site of personality.websites) {
        addAssignment(siteAssignments, site, personality.name, true);
      }
    }

    return {
      enabledStyles,
      assignedApps: appAssignments.size,
      assignedSites: siteAssignments.size,
      conflicts: [
        ...createCoverageConflicts(appAssignments, "app"),
        ...createCoverageConflicts(siteAssignments, "site"),
      ].sort((left, right) => left.label.localeCompare(right.label)),
    };
  }, [personalities]);

  const styleTemplates: StyleTemplate[] = useMemo(
    () => [
      {
        name: t({
          id: "personalization.template.professional.name",
          message: "Professional",
        }),
        description: t({
          id: "personalization.template.professional.description",
          message: "Crisp, clear, and work-ready.",
        }),
        instructions: [
          "Use a crisp professional tone.",
          "Preserve facts, names, numbers, and intent.",
          "Prefer concise sentences and clear next actions.",
        ],
      },
      {
        name: t({
          id: "personalization.template.casual.name",
          message: "Casual",
        }),
        description: t({
          id: "personalization.template.casual.description",
          message: "Natural, friendly, and low-friction.",
        }),
        instructions: [
          "Use a natural casual tone.",
          "Keep the wording warm and conversational.",
          "Do not add extra claims or change the user's intent.",
        ],
      },
      {
        name: t({
          id: "personalization.template.concise.name",
          message: "Concise",
        }),
        description: t({
          id: "personalization.template.concise.description",
          message: "Shorter, tighter, easier to scan.",
        }),
        instructions: [
          "Make the writing shorter and more scannable.",
          "Remove filler while preserving important details.",
          "Prefer plain language and direct structure.",
        ],
      },
      {
        name: t({
          id: "personalization.template.engineering.name",
          message: "Engineering",
        }),
        description: t({
          id: "personalization.template.engineering.description",
          message: "Precise notes for code and product work.",
        }),
        instructions: [
          "Write with precise engineering language.",
          "State behavior, constraints, risks, and verification clearly.",
          "Avoid vague praise and keep action items concrete.",
        ],
      },
    ],
    [t],
  );

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [personalityResp, appsResp] = await Promise.all([
        invoke<Personality[]>("get_personalities"),
        invoke<InstalledApp[]>("list_installed_apps"),
      ]);
      setPersonalities(personalityResp ?? []);
      setInstalledApps(appsResp ?? []);
    } catch (err) {
      console.error(err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  const loadActiveStylePreview = useCallback(async () => {
    setActiveStylePreviewLoading(true);
    setActiveStylePreviewError(null);
    try {
      const preview = await invoke<ActiveStylePreview>("get_active_style_preview");
      setActiveStylePreview(preview);
    } catch (err) {
      console.error(err);
      setActiveStylePreviewError(err instanceof Error ? err.message : String(err));
    } finally {
      setActiveStylePreviewLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!isActive) return;
    load();
    void loadActiveStylePreview();
  }, [isActive, load, loadActiveStylePreview]);

  const websiteDomains = useMemo(() => {
    const seen = new Set<string>();
    for (const personality of personalities) {
      for (const site of personality.websites) {
        const normalized = normalizeWebsite(site);
        if (normalized) {
          seen.add(normalized);
        }
      }
    }
    return Array.from(seen).sort();
  }, [personalities]);

  const loadWebsiteIcons = useCallback(async (sites: string[]) => {
    if (sites.length === 0) {
      setWebsiteIconBySite({});
      return;
    }
    try {
      const iconsResp = await invoke<WebsiteIcon[]>("list_website_icons", {
        sites,
      });
      setWebsiteIconBySite(buildWebsiteIconMap(iconsResp ?? []));
    } catch (err) {
      console.error(err);
    }
  }, []);

  useEffect(() => {
    if (!isActive) return;
    void loadWebsiteIcons(websiteDomains);
  }, [isActive, websiteDomains, loadWebsiteIcons]);

  useEffect(() => {
    if (!isActive) return;

    if (websiteDomains.length === 0) {
      websiteIconRefreshKeyRef.current = null;
      return;
    }

    const hasMissingIcons = websiteDomains.some((site) => !websiteIconBySite[site]);
    if (!hasMissingIcons) {
      websiteIconRefreshKeyRef.current = websiteDomains.join("|");
      return;
    }

    const currentKey = websiteDomains.join("|");
    if (websiteIconRefreshKeyRef.current === currentKey) {
      return;
    }

    const timer = window.setTimeout(async () => {
      websiteIconRefreshKeyRef.current = currentKey;
      try {
        const iconsResp = await invoke<WebsiteIcon[]>("list_website_icons", {
          sites: websiteDomains,
        });
        setWebsiteIconBySite(buildWebsiteIconMap(iconsResp ?? []));
      } catch {
        // Keep current icon map; website icon refresh is best-effort only.
      }
    }, 2500);

    return () => {
      window.clearTimeout(timer);
    };
  }, [isActive, websiteDomains, websiteIconBySite]);

  useEffect(() => {
    if (!isActive) return;

    if (hasRequestedIconRefreshRef.current || loading || installedApps.length === 0) {
      return;
    }

    const hasMissingIcons = installedApps.some((app) => !app.icon_path);
    if (!hasMissingIcons) {
      hasRequestedIconRefreshRef.current = true;
      return;
    }

    const timer = window.setTimeout(async () => {
      hasRequestedIconRefreshRef.current = true;
      try {
        const appsResp = await invoke<InstalledApp[]>("list_installed_apps");
        setInstalledApps(appsResp ?? []);
      } catch {
        // Keep current app list; icon refresh is best-effort only.
      }
    }, 2500);

    return () => {
      window.clearTimeout(timer);
    };
  }, [isActive, installedApps, loading]);

  const persistPersonalities = useCallback((next: Personality[]) => {
    const persistVersion = persistVersionRef.current + 1;
    persistVersionRef.current = persistVersion;

    if (saveTimeoutRef.current !== null) {
      window.clearTimeout(saveTimeoutRef.current);
    }

    saveTimeoutRef.current = window.setTimeout(async () => {
      setError(null);
      try {
        const cleaned = await invoke<Personality[]>("set_personalities", {
          personalities: next,
        });
        if (persistVersion !== persistVersionRef.current) {
          return;
        }
        setPersonalities(cleaned ?? next);
      } catch (err) {
        if (persistVersion !== persistVersionRef.current) {
          return;
        }
        console.error(err);
        setError(err instanceof Error ? err.message : String(err));
      }
    }, 500);
  }, []);

  useEffect(() => {
    return () => {
      if (saveTimeoutRef.current !== null) {
        window.clearTimeout(saveTimeoutRef.current);
      }
    };
  }, []);

  const updatePersonalities = useCallback(
    (updater: (prev: Personality[]) => Personality[]) => {
      setPersonalities((prev) => {
        const next = updater(prev);
        void persistPersonalities(next);
        return next;
      });
    },
    [persistPersonalities],
  );

  const flashBackupStatus = (message: string) => {
    setBackupStatus(message);
    setBackupError(null);
    window.setTimeout(() => setBackupStatus(null), 2400);
  };

  const handleExportStyles = async () => {
    const payload = {
      app: "Friday",
      type: "styles",
      version: 1,
      exported_at: new Date().toISOString(),
      styles: personalities.map(({ name, enabled, apps, websites, instructions }) => ({
        name,
        enabled,
        apps,
        websites,
        instructions,
      })),
    };
    await navigator.clipboard.writeText(JSON.stringify(payload, null, 2));
    flashBackupStatus(
      t({
        id: "personalization.backup.exported",
        message: "Styles copied as JSON",
      }),
    );
  };

  const handleImportStyles = async () => {
    if (importingBackup) return;
    setImportingBackup(true);
    setBackupError(null);
    setBackupStatus(null);
    try {
      const raw = await navigator.clipboard.readText();
      const imported = normalizeStyleBackup(JSON.parse(raw));
      updatePersonalities((prev) => {
        const mergedByName = new Map(prev.map((style) => [style.name.toLowerCase(), style]));
        for (const style of imported) {
          const existing = mergedByName.get(style.name.toLowerCase());
          mergedByName.set(style.name.toLowerCase(), {
            id: existing?.id ?? createId(),
            name: style.name,
            enabled: style.enabled,
            apps: style.apps,
            websites: style.websites,
            instructions: style.instructions,
          });
        }
        return Array.from(mergedByName.values());
      });
      flashBackupStatus(
        t({
          id: "personalization.backup.imported",
          message: `Imported ${imported.length} styles`,
        }),
      );
    } catch (error) {
      setBackupError(error instanceof Error ? error.message : String(error));
    } finally {
      setImportingBackup(false);
    }
  };

  const updatePersonality = useCallback(
    (id: string, patch: Partial<Personality>) => {
      updatePersonalities((prev) =>
        prev.map((personality) =>
          personality.id === id ? { ...personality, ...patch } : personality,
        ),
      );
    },
    [updatePersonalities],
  );

  const updatePersonalityList = useCallback(
    (id: string, updater: (current: Personality) => Personality) => {
      updatePersonalities((prev) =>
        prev.map((personality) => (personality.id === id ? updater(personality) : personality)),
      );
    },
    [updatePersonalities],
  );

  const handleAddMode = () => {
    const id = createId();
    const nextMode: Personality = {
      id,
      name: t({
        id: "personalization.new_mode.default_name",
        message: "New Style",
      }),
      enabled: true,
      apps: [],
      websites: [],
      instructions: [],
    };
    updatePersonalities((prev) => [...prev, nextMode]);
    setActivePersonalityId(id);
  };

  const handleAddTemplate = (template: StyleTemplate) => {
    const id = createId();
    const nextMode: Personality = {
      id,
      name: template.name,
      enabled: true,
      apps: [],
      websites: [],
      instructions: template.instructions,
    };
    updatePersonalities((prev) => [...prev, nextMode]);
    setActivePersonalityId(id);
  };

  const handleDeleteMode = useCallback(
    (id: string) => {
      updatePersonalities((prev) => prev.filter((mode) => mode.id !== id));
      setActivePersonalityId(null);
    },
    [updatePersonalities],
  );

  const requestDeleteModeConfirm = useCallback((personality: Personality) => {
    setPendingDeletePersonality({ id: personality.id, name: personality.name });
  }, []);

  const confirmDeleteMode = useCallback(() => {
    if (!pendingDeletePersonality) {
      return;
    }
    const targetId = pendingDeletePersonality.id;
    setPendingDeletePersonality(null);
    handleDeleteMode(targetId);
  }, [pendingDeletePersonality, handleDeleteMode]);

  const activePersonality = useMemo(() => {
    return personalities.find((personality) => personality.id === activePersonalityId) || null;
  }, [personalities, activePersonalityId]);

  const installedAppByName = useMemo(() => {
    const entries = installedApps.map((app) => [app.name.toLowerCase(), app] as const);
    return new Map(entries);
  }, [installedApps]);

  useEffect(() => {
    if (activePersonalityId && !activePersonality) {
      setActivePersonalityId(null);
    }
  }, [activePersonalityId, activePersonality]);

  useEffect(() => {
    if (!isActive) {
      return;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.defaultPrevented || event.key !== "Escape") {
        return;
      }

      if (pendingDeletePersonality) {
        event.preventDefault();
        setPendingDeletePersonality(null);
        return;
      }

      if (activePersonalityId) {
        event.preventDefault();
        setActivePersonalityId(null);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [activePersonalityId, isActive, pendingDeletePersonality]);

  return (
    <div className="w-full text-left max-w-7xl mx-auto px-0">
      <div className="flex items-start gap-3 mb-6 mt-2 md:-mt-6">
        <DotMatrix
          rows={2}
          cols={3}
          activeDots={[0, 1, 4, 5]}
          dotSize={3}
          gap={3}
          color="var(--color-section-marker-alt)"
        />
        <div className="flex-1 flex items-start justify-between gap-4">
          <div>
            <p className="ui-text-screen-title ui-color-primary tracking-tight">
              {t({
                id: "personalization.title",
                message: "Style",
              })}
            </p>
            <p className="mt-1 ui-text-body-sm ui-color-secondary">
              {t({
                id: "personalization.description",
                message:
                  "Tailor writing behavior for apps, sites, cleanup, Edit Mode, and transforms.",
              })}
            </p>
          </div>
          <div className="flex flex-wrap items-center justify-end gap-2">
            <button
              type="button"
              onClick={handleExportStyles}
              disabled={personalities.length === 0}
              className="ui-button-ghost h-9 gap-2 rounded-full border border-border-primary px-4 ui-text-button-sm disabled:opacity-40"
            >
              <Download size={15} aria-hidden="true" />
              {t({ id: "personalization.export", message: "Export" })}
            </button>
            <button
              type="button"
              onClick={handleImportStyles}
              disabled={importingBackup}
              className="ui-button-ghost h-9 gap-2 rounded-full border border-border-primary px-4 ui-text-button-sm disabled:opacity-40"
            >
              <Upload size={15} aria-hidden="true" />
              {importingBackup
                ? t({ id: "personalization.importing", message: "Importing" })
                : t({ id: "personalization.import", message: "Import" })}
            </button>
            <button
              type="button"
              onClick={handleAddMode}
              aria-label={t({
                id: "personalization.new_mode",
                message: "New style",
              })}
              className="group inline-flex shrink-0 self-start items-center gap-1.5 rounded-lg border border-border-primary bg-surface-secondary px-3 py-1.5 ui-text-button ui-color-secondary transition-colors hover:border-border-hover hover:bg-surface-elevated hover:text-content-primary"
            >
              <Plus
                size={13}
                aria-hidden="true"
                className="text-content-muted transition-colors group-hover:text-content-primary"
              />
              {t({
                id: "personalization.new_mode",
                message: "New style",
              })}
            </button>
          </div>
        </div>
      </div>

      {(backupStatus || backupError) && (
        <div
          className={`mb-4 rounded-lg border px-3 py-2 ui-text-body-sm ${
            backupError
              ? "border-red-500/30 bg-red-500/10 ui-color-error-soft"
              : "border-border-primary bg-surface-surface ui-color-secondary"
          }`}
          role="status"
        >
          {backupError ?? backupStatus}
        </div>
      )}

      {!loading && personalities.length > 0 && (
        <>
          <ActiveStylePreviewPanel
            preview={activeStylePreview}
            loading={activeStylePreviewLoading}
            error={activeStylePreviewError}
            onRefresh={loadActiveStylePreview}
            onOpenStyle={(id) => setActivePersonalityId(id)}
          />
          <StyleCoveragePanel coverage={styleCoverage} />
        </>
      )}

      <div className="mb-5 flex flex-wrap items-center gap-2">
        <div className="mr-1 flex items-center gap-1.5 ui-text-meta-strong ui-color-muted">
          <Sparkles size={13} aria-hidden="true" />
          {t({
            id: "personalization.templates.label",
            message: "Starter styles",
          })}
        </div>
        {styleTemplates.map((template) => (
          <button
            key={template.name}
            type="button"
            onClick={() => handleAddTemplate(template)}
            title={template.description}
            className="rounded-full border border-border-primary bg-surface-surface px-3 py-1.5 ui-text-button-sm ui-color-secondary transition-colors hover:border-border-hover hover:bg-surface-elevated hover:text-content-primary"
          >
            {template.name}
          </button>
        ))}
      </div>

      {loading ? (
        <div className="flex items-center justify-center py-12">
          <DotMatrix
            rows={2}
            cols={6}
            activeDots={[0, 1, 2, 3, 4, 5]}
            dotSize={3}
            gap={3}
            color="var(--color-content-muted)"
            animated
            className="opacity-60"
          />
        </div>
      ) : personalities.length === 0 ? (
        <div className="rounded-xl border border-border-primary bg-surface-secondary px-6 py-8 ui-color-muted">
          <p className="ui-text-body-lg-strong">
            {t({
              id: "personalization.empty.title",
              message: "No styles yet",
            })}
          </p>
          <p className="ui-text-body-sm ui-color-muted">
            {t({
              id: "personalization.empty.description",
              message: "Create a style to customize writing behavior for apps and websites.",
            })}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
          {personalities.map((personality, index) => {
            const appsPreview = personality.apps.slice(0, 3);
            const sitesPreview = personality.websites.slice(0, 2);
            const moreApps = Math.max(0, personality.apps.length - appsPreview.length);
            const moreSites = Math.max(0, personality.websites.length - sitesPreview.length);
            return (
              <div
                key={personality.id || `personality-${index}`}
                onClick={() => {
                  if (shiftHeld) {
                    requestDeleteModeConfirm(personality);
                    return;
                  }
                  setActivePersonalityId(personality.id);
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    if (shiftHeld) {
                      requestDeleteModeConfirm(personality);
                      return;
                    }
                    setActivePersonalityId(personality.id);
                  }
                }}
                role="button"
                tabIndex={0}
                className={`ui-card-liftable group relative p-2.5 text-left ${
                  shiftHeld ? "!border-red-500/30 hover:!border-red-500/60 hover:!bg-red-500/5" : ""
                }`}
              >
                <div className="relative space-y-2">
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <p className="ui-text-body-lg-strong ui-color-primary">{personality.name}</p>
                    </div>
                    <div className="flex items-start gap-2">
                      <div
                        data-no-press
                        className="-mt-0.5"
                        onPointerDown={(event) => event.stopPropagation()}
                        onClick={(event) => event.stopPropagation()}
                        onKeyDown={(event) => {
                          if (
                            event.key === "Enter" ||
                            event.key === " " ||
                            event.key === "Spacebar"
                          ) {
                            event.stopPropagation();
                          }
                        }}
                      >
                        <ToggleSwitch
                          enabled={personality.enabled}
                          onToggle={() =>
                            updatePersonality(personality.id, {
                              enabled: !personality.enabled,
                            })
                          }
                          ariaLabel={
                            personality.enabled
                              ? t({
                                  id: "personalization.disable_mode",
                                  message: "Disable style",
                                })
                              : t({
                                  id: "personalization.enable_mode",
                                  message: "Enable style",
                                })
                          }
                        />
                      </div>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-2">
                    <div>
                      <p className="ui-text-uppercase-micro ui-color-disabled">
                        {t({
                          id: "personalization.apps",
                          message: "Apps",
                        })}
                      </p>
                      <div className="mt-1.5 flex items-center gap-1.5 flex-wrap">
                        {appsPreview.length === 0 ? (
                          <span className="ui-text-meta ui-color-disabled">
                            {t({
                              id: "personalization.no_apps",
                              message: "No apps yet",
                            })}
                          </span>
                        ) : (
                          appsPreview.map((app, index) => (
                            <div key={`app-preview-${index}-${app || "empty"}`} title={app}>
                              <AppIconBadge
                                appName={app}
                                iconPath={installedAppByName.get(app.toLowerCase())?.icon_path}
                                size="chip"
                              />
                            </div>
                          ))
                        )}
                        {moreApps > 0 && (
                          <span className="ui-text-meta font-mono ui-color-muted">+{moreApps}</span>
                        )}
                      </div>
                    </div>

                    <div>
                      <p className="ui-text-uppercase-micro ui-color-disabled">
                        {t({
                          id: "personalization.websites",
                          message: "Websites",
                        })}
                      </p>
                      <div className="mt-1.5 flex items-center gap-1.5 min-w-0 flex-nowrap">
                        {sitesPreview.length === 0 ? (
                          <span className="ui-text-meta ui-color-disabled">
                            {t({
                              id: "personalization.no_sites",
                              message: "No sites yet",
                            })}
                          </span>
                        ) : (
                          sitesPreview.map((site, index) => (
                            <span
                              key={`site-preview-${index}-${site || "empty"}`}
                              className="min-w-0 max-w-[118px] rounded-md border border-border-primary bg-surface-overlay px-2 py-1 ui-text-micro ui-color-secondary inline-flex items-center gap-1"
                            >
                              <WebsiteFavicon
                                site={site}
                                iconPath={websiteIconBySite[normalizeWebsite(site)]}
                                size="chip"
                              />
                              <span className="min-w-0 truncate font-mono">
                                {formatWebsitePreview(site)}
                              </span>
                            </span>
                          ))
                        )}
                        {moreSites > 0 && (
                          <span className="shrink-0 ui-text-meta font-mono ui-color-muted">
                            +{moreSites}
                          </span>
                        )}
                      </div>
                    </div>
                  </div>

                  <div className="mt-2 pt-2 border-t border-border-primary ui-text-meta ui-color-muted flex items-center gap-2 min-w-0">
                    <span className="ui-text-uppercase-micro ui-color-disabled">
                      {t({
                        id: "personalization.notes",
                        message: "Notes:",
                      })}
                    </span>
                    <span className="font-mono truncate flex-1">
                      {personality.instructions.length > 0
                        ? personality.instructions[0]
                        : t({
                            id: "personalization.no_notes",
                            message: "No notes yet",
                          })}
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {error && <div className="mt-4 ui-text-body-sm ui-color-error-soft">{error}</div>}

      {activePersonality && (
        <PersonalityModal
          personality={activePersonality}
          installedApps={installedApps}
          websiteIconBySite={websiteIconBySite}
          onClose={() => setActivePersonalityId(null)}
          onUpdate={(patch) => updatePersonality(activePersonality.id, patch)}
          onUpdateList={(updater) => updatePersonalityList(activePersonality.id, updater)}
          onDelete={() => requestDeleteModeConfirm(activePersonality)}
        />
      )}

      <AnimatePresence>
        {pendingDeletePersonality && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.15 }}
            className="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 backdrop-blur-xs"
            onClick={() => setPendingDeletePersonality(null)}
          >
            <motion.div
              initial={{ opacity: 0, scale: 0.96, y: 14 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.96, y: 14 }}
              transition={{ duration: 0.2, ease: "easeOut" }}
              onClick={(event) => event.stopPropagation()}
              className="w-[380px] max-w-[92vw] rounded-2xl border border-border-secondary bg-surface-overlay p-5 shadow-2xl"
              role="dialog"
              aria-modal="true"
              aria-labelledby="delete-mode-title"
            >
              <h3 id="delete-mode-title" className="ui-text-title-strong ui-color-primary">
                {t({
                  id: "personalization.delete_mode.title",
                  message: "Delete style?",
                })}
              </h3>
              <p className="mt-2 ui-text-body-sm ui-color-secondary">
                {t({
                  id: "personalization.delete_mode.description",
                  message: `Delete "${pendingDeletePersonality.name}"? This cannot be undone.`,
                })}
              </p>
              <div className="mt-4 flex items-center justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setPendingDeletePersonality(null)}
                  className="rounded-lg border border-border-primary bg-surface-surface px-3 py-1.5 ui-text-button ui-color-primary hover:bg-surface-elevated transition-colors"
                >
                  {t({
                    id: "personalization.cancel",
                    message: "Cancel",
                  })}
                </button>
                <button
                  type="button"
                  onClick={confirmDeleteMode}
                  className="rounded-lg border border-red-500/40 bg-red-500/10 px-3 py-1.5 ui-text-button font-semibold ui-color-error-soft hover:bg-red-500/15 transition-colors"
                >
                  {t({
                    id: "personalization.delete",
                    message: "Delete",
                  })}
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

const ActiveStylePreviewPanel = ({
  preview,
  loading,
  error,
  onRefresh,
  onOpenStyle,
}: {
  preview: ActiveStylePreview | null;
  loading: boolean;
  error: string | null;
  onRefresh: () => void | Promise<void>;
  onOpenStyle: (id: string) => void;
}) => {
  const { t } = useLingui();
  const matchCount = preview?.matches.length ?? 0;
  const primaryContext =
    preview?.url || preview?.window_title || preview?.app_name || "No target detected";
  const statusText = error
    ? error
    : loading
      ? t({
          id: "personalization.active_preview.loading",
          message: "Checking active target...",
        })
      : !preview?.permission_granted
        ? t({
            id: "personalization.active_preview.permission_needed",
            message: "Accessibility permission is needed to inspect the active app.",
          })
        : !preview?.context_available
          ? t({
              id: "personalization.active_preview.context_missing",
              message: "No active app context is available yet.",
            })
          : matchCount > 0
            ? t({
                id: "personalization.active_preview.matched",
                message: "Friday will apply matched style guidance.",
              })
            : t({
                id: "personalization.active_preview.no_match",
                message: "No style matches this target yet.",
              });

  return (
    <div className="mb-3 rounded-xl border border-border-primary bg-surface-secondary px-3 py-3">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2 ui-text-body-sm-strong ui-color-primary">
            <Crosshair size={14} aria-hidden="true" />
            {t({
              id: "personalization.active_preview.title",
              message: "Active target preview",
            })}
          </div>
          <p className="mt-1 truncate ui-text-meta ui-color-muted" title={primaryContext}>
            {statusText}
          </p>
          {preview?.context_available && (
            <div className="mt-2 flex flex-wrap gap-1.5">
              {preview.app_name && (
                <span className="inline-flex max-w-full items-center gap-1 rounded-full border border-border-primary bg-surface-surface px-2 py-1 ui-text-meta ui-color-secondary">
                  <AppWindow size={11} aria-hidden="true" />
                  <span className="max-w-[150px] truncate font-mono">{preview.app_name}</span>
                </span>
              )}
              {preview.url && (
                <span className="inline-flex max-w-full items-center gap-1 rounded-full border border-border-primary bg-surface-surface px-2 py-1 ui-text-meta ui-color-secondary">
                  <Globe2 size={11} aria-hidden="true" />
                  <span className="max-w-[210px] truncate font-mono">{preview.url}</span>
                </span>
              )}
              {preview.window_title && (
                <span className="inline-flex max-w-full items-center rounded-full border border-border-primary bg-surface-surface px-2 py-1 ui-text-meta ui-color-muted">
                  <span className="max-w-[280px] truncate">{preview.window_title}</span>
                </span>
              )}
            </div>
          )}
        </div>

        <button
          type="button"
          onClick={() => void onRefresh()}
          disabled={loading}
          className="inline-flex h-8 items-center gap-1.5 rounded-full border border-border-primary bg-surface-surface px-3 ui-text-button-sm ui-color-secondary transition-colors hover:border-border-hover hover:bg-surface-elevated hover:text-content-primary disabled:opacity-50"
        >
          <RefreshCw size={13} aria-hidden="true" className={loading ? "animate-spin" : ""} />
          {t({
            id: "personalization.active_preview.refresh",
            message: "Refresh",
          })}
        </button>
      </div>

      {matchCount > 0 && (
        <div className="mt-3 flex flex-wrap gap-1.5">
          {preview?.matches.map((match, index) => (
            <button
              key={match.id}
              type="button"
              onClick={() => onOpenStyle(match.id)}
              className="inline-flex max-w-full items-center gap-1.5 rounded-full border border-border-primary bg-surface-surface px-2 py-1 ui-text-meta ui-color-secondary transition-colors hover:border-border-hover hover:bg-surface-elevated hover:text-content-primary"
              title={`${match.name} - ${match.instruction_count} instructions`}
            >
              <Sparkles size={11} aria-hidden="true" />
              <span className="max-w-[150px] truncate">{match.name}</span>
              {index === 0 && (
                <span className="ui-color-disabled">
                  {t({
                    id: "personalization.active_preview.primary",
                    message: "primary",
                  })}
                </span>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

const StyleCoveragePanel = ({ coverage }: { coverage: StyleCoverage }) => {
  const { t } = useLingui();
  const hasConflicts = coverage.conflicts.length > 0;
  const visibleConflicts = coverage.conflicts.slice(0, 3);
  const hiddenConflictCount = Math.max(0, coverage.conflicts.length - visibleConflicts.length);
  const metrics = [
    {
      label: t({
        id: "personalization.coverage.enabled",
        message: "Enabled",
      }),
      value: coverage.enabledStyles,
      icon: <Sparkles size={13} aria-hidden="true" />,
    },
    {
      label: t({
        id: "personalization.coverage.apps",
        message: "Apps",
      }),
      value: coverage.assignedApps,
      icon: <AppWindow size={13} aria-hidden="true" />,
    },
    {
      label: t({
        id: "personalization.coverage.sites",
        message: "Sites",
      }),
      value: coverage.assignedSites,
      icon: <Globe2 size={13} aria-hidden="true" />,
    },
    {
      label: t({
        id: "personalization.coverage.conflicts",
        message: "Conflicts",
      }),
      value: coverage.conflicts.length,
      icon: hasConflicts ? (
        <AlertTriangle size={13} aria-hidden="true" />
      ) : (
        <Route size={13} aria-hidden="true" />
      ),
    },
  ];

  return (
    <div className="mb-5 rounded-xl border border-border-primary bg-surface-secondary px-3 py-3">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex items-center gap-2 ui-text-body-sm-strong ui-color-primary">
            <Route size={14} aria-hidden="true" />
            {t({
              id: "personalization.coverage.title",
              message: "Style routing",
            })}
          </div>
          <p className="mt-1 ui-text-meta ui-color-muted">
            {hasConflicts
              ? t({
                  id: "personalization.coverage.conflict_status",
                  message: "Some app/site assignments overlap.",
                })
              : t({
                  id: "personalization.coverage.clean_status",
                  message: "Assignments are unambiguous.",
                })}
          </p>
        </div>

        <div className="grid min-w-[360px] flex-1 grid-cols-4 gap-1.5">
          {metrics.map((metric) => (
            <div
              key={metric.label}
              className="min-w-0 rounded-lg border border-border-primary bg-surface-surface px-2 py-1.5"
            >
              <div className="flex items-center gap-1.5 ui-text-micro-strong ui-color-muted">
                {metric.icon}
                <span className="truncate">{metric.label}</span>
              </div>
              <div className="mt-0.5 font-mono ui-text-body-sm-strong ui-color-primary">
                {metric.value}
              </div>
            </div>
          ))}
        </div>
      </div>

      {hasConflicts && (
        <div className="mt-3 flex flex-wrap gap-1.5">
          {visibleConflicts.map((conflict) => (
            <span
              key={`${conflict.kind}-${conflict.label}`}
              className="inline-flex max-w-full items-center gap-1 rounded-full border border-border-primary bg-surface-surface px-2 py-1 ui-text-meta ui-color-secondary"
              title={conflict.styles.join(", ")}
            >
              {conflict.kind === "app" ? (
                <AppWindow size={11} aria-hidden="true" />
              ) : (
                <Globe2 size={11} aria-hidden="true" />
              )}
              <span className="max-w-[120px] truncate font-mono">{conflict.label}</span>
              <span className="ui-color-disabled">in</span>
              <span className="max-w-[180px] truncate">{conflict.styles.join(", ")}</span>
            </span>
          ))}
          {hiddenConflictCount > 0 && (
            <span className="inline-flex items-center rounded-full border border-border-primary bg-surface-surface px-2 py-1 ui-text-meta ui-color-muted">
              +{hiddenConflictCount}
            </span>
          )}
        </div>
      )}
    </div>
  );
};

export default PersonalizationView;
