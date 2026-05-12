import { i18n, type Messages } from "@lingui/core";
import type { AppLocaleSetting } from "./types";
import {
  DEFAULT_APP_LOCALE,
  DEFAULT_LOCALE,
  SUPPORTED_APP_LOCALES,
  normalizeSupportedAppLocale,
} from "./shared/lib/appLocales";
import enCatalog from "./locales/en/messages.js";

export type AppLocale = string;

const catalogs: Record<string, Messages> = {
  en: enCatalog.messages,
};

for (const locale of SUPPORTED_APP_LOCALES) {
  if (!catalogs[locale]) {
    throw new Error(`Missing locale catalog for ${locale}`);
  }
}

function resolveRequestedLocale(localeSetting?: AppLocaleSetting | string | null): string | null {
  if (!localeSetting || localeSetting === DEFAULT_APP_LOCALE) {
    return typeof navigator !== "undefined" ? navigator.language : DEFAULT_LOCALE;
  }
  return localeSetting;
}

export function activateLocale(localeSetting?: AppLocaleSetting | string | null): AppLocale {
  const nextLocale = normalizeSupportedAppLocale(resolveRequestedLocale(localeSetting));

  i18n.loadAndActivate({
    locale: nextLocale,
    messages: catalogs[nextLocale],
  });

  if (typeof document !== "undefined") {
    document.documentElement.lang = nextLocale;
  }

  return nextLocale;
}

activateLocale(DEFAULT_APP_LOCALE);

export { i18n };
