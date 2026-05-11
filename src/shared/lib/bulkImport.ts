import type { Replacement } from "../../types";

export const BULK_IMPORT_MAX_BYTES = 3 * 1024 * 1024;
export const BULK_IMPORT_MAX_ITEMS = 1000;
export const SNIPPET_TRIGGER_MAX_LENGTH = 59;
const DICTIONARY_ENTRY_MAX_LENGTH = 160;
const REPLACEMENT_FROM_MAX_LENGTH = 100;
const REPLACEMENT_TO_MAX_LENGTH = 200;
const SNIPPET_EXPANSION_MAX_LENGTH = 4000;

type CsvParseResult = {
  rows: string[][];
  malformedRows: number;
};

export type DictionaryImportPayload = {
  entries: string[];
  replacements: Replacement[];
  skipped: number;
};

export type SnippetImportItem = {
  trigger: string;
  expansion: string;
};

export type SnippetImportPayload = {
  snippets: SnippetImportItem[];
  skipped: number;
};

const isObject = (value: unknown): value is Record<string, unknown> =>
  Boolean(value && typeof value === "object" && !Array.isArray(value));

const extensionFor = (name: string) => name.split(".").pop()?.toLowerCase() ?? "";

export function assertBulkImportFile(file: File, supportedExtensions: string[]) {
  if (file.size > BULK_IMPORT_MAX_BYTES) {
    throw new Error("Import files must be 3 MB or smaller.");
  }

  const extension = extensionFor(file.name);
  if (!supportedExtensions.includes(extension)) {
    throw new Error(`Use a ${supportedExtensions.map((ext) => `.${ext}`).join(" or ")} file.`);
  }
}

function normalizeSingleLine(value: string) {
  return value.split(/\s+/).join(" ").trim();
}

function pushUnique<T>(items: T[], seen: Set<string>, key: string, item: T): boolean {
  const normalizedKey = key.toLowerCase();
  if (seen.has(normalizedKey)) {
    return false;
  }
  seen.add(normalizedKey);
  items.push(item);
  return true;
}

function parseCsvRows(input: string): CsvParseResult {
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let inQuotes = false;
  let malformedRow = false;
  let malformedRows = 0;

  const pushField = () => {
    row.push(field.trim());
    field = "";
  };

  const pushRow = () => {
    pushField();
    if (malformedRow) {
      malformedRows += 1;
    } else if (row.some((cell) => cell.trim().length > 0)) {
      rows.push(row);
    }
    row = [];
    malformedRow = false;
  };

  for (let index = 0; index < input.length; index += 1) {
    const char = input[index];
    const next = input[index + 1];

    if (inQuotes) {
      if (char === '"' && next === '"') {
        field += '"';
        index += 1;
      } else if (char === '"') {
        inQuotes = false;
      } else {
        field += char;
      }
      continue;
    }

    if (char === '"') {
      if (field.trim().length > 0) {
        malformedRow = true;
      }
      inQuotes = true;
    } else if (char === ",") {
      pushField();
    } else if (char === "\n") {
      pushRow();
    } else if (char !== "\r") {
      field += char;
    }
  }

  if (inQuotes) {
    malformedRows += 1;
  } else if (field.length > 0 || row.length > 0) {
    pushRow();
  }

  return { rows, malformedRows };
}

function normalizeDictionaryJson(value: unknown): DictionaryImportPayload {
  if (!isObject(value)) {
    throw new Error("JSON does not contain a Flow dictionary backup.");
  }

  const rawEntries = Array.isArray(value.entries) ? value.entries : [];
  const rawReplacements = Array.isArray(value.replacements) ? value.replacements : [];
  const entries: string[] = [];
  const replacements: Replacement[] = [];
  const seenEntries = new Set<string>();
  const seenReplacements = new Set<string>();
  let skipped = 0;

  for (const item of rawEntries) {
    const entry = typeof item === "string" ? normalizeSingleLine(item) : "";
    if (!entry || entry.length > DICTIONARY_ENTRY_MAX_LENGTH) {
      skipped += 1;
      continue;
    }
    if (!pushUnique(entries, seenEntries, entry, entry)) {
      skipped += 1;
    }
    if (entries.length + replacements.length >= BULK_IMPORT_MAX_ITEMS) break;
  }

  for (const item of rawReplacements) {
    if (!isObject(item)) {
      skipped += 1;
      continue;
    }
    const from = typeof item.from === "string" ? normalizeSingleLine(item.from) : "";
    const to = typeof item.to === "string" ? normalizeSingleLine(item.to) : "";
    if (
      !from ||
      from.length > REPLACEMENT_FROM_MAX_LENGTH ||
      to.length > REPLACEMENT_TO_MAX_LENGTH
    ) {
      skipped += 1;
      continue;
    }
    if (!pushUnique(replacements, seenReplacements, from, { from, to })) {
      skipped += 1;
    }
    if (entries.length + replacements.length >= BULK_IMPORT_MAX_ITEMS) break;
  }

  if (entries.length === 0 && replacements.length === 0) {
    throw new Error("No usable dictionary entries or replacements were found.");
  }

  return { entries, replacements, skipped };
}

export function parseDictionaryImport(name: string, text: string): DictionaryImportPayload {
  if (extensionFor(name) === "json") {
    return normalizeDictionaryJson(JSON.parse(text));
  }

  const parsed = parseCsvRows(text);
  const entries: string[] = [];
  const replacements: Replacement[] = [];
  const seenEntries = new Set<string>();
  const seenReplacements = new Set<string>();
  let skipped = parsed.malformedRows;

  for (const row of parsed.rows) {
    if (
      row.length < 1 ||
      row.length > 2 ||
      entries.length + replacements.length >= BULK_IMPORT_MAX_ITEMS
    ) {
      skipped += 1;
      continue;
    }

    const from = normalizeSingleLine(row[0] ?? "");
    const to = row.length === 2 ? normalizeSingleLine(row[1] ?? "") : "";

    if (row.length === 1) {
      if (!from || from.length > DICTIONARY_ENTRY_MAX_LENGTH) {
        skipped += 1;
        continue;
      }
      if (!pushUnique(entries, seenEntries, from, from)) {
        skipped += 1;
      }
      continue;
    }

    if (
      !from ||
      from.length > REPLACEMENT_FROM_MAX_LENGTH ||
      to.length > REPLACEMENT_TO_MAX_LENGTH
    ) {
      skipped += 1;
      continue;
    }
    if (!pushUnique(replacements, seenReplacements, from, { from, to })) {
      skipped += 1;
    }
  }

  if (entries.length === 0 && replacements.length === 0) {
    throw new Error("No usable dictionary entries or replacements were found.");
  }

  return { entries, replacements, skipped };
}

export function parseSnippetImport(text: string): SnippetImportPayload {
  const value = JSON.parse(text);
  const source = Array.isArray(value)
    ? value
    : isObject(value) && Array.isArray(value.snippets)
      ? value.snippets
      : null;

  if (!source) {
    throw new Error("JSON does not contain Flow snippets.");
  }

  const snippets: SnippetImportItem[] = [];
  const seen = new Set<string>();
  let skipped = 0;

  for (const item of source) {
    if (snippets.length >= BULK_IMPORT_MAX_ITEMS) {
      skipped += 1;
      continue;
    }
    if (!isObject(item)) {
      skipped += 1;
      continue;
    }
    const trigger =
      typeof item.trigger === "string"
        ? normalizeSingleLine(item.trigger)
        : typeof item.name === "string"
          ? normalizeSingleLine(item.name)
          : "";
    const expansion =
      typeof item.expansion === "string"
        ? item.expansion.trim()
        : typeof item.text === "string"
          ? item.text.trim()
          : "";

    if (
      !trigger ||
      !expansion ||
      trigger.length > SNIPPET_TRIGGER_MAX_LENGTH ||
      expansion.length > SNIPPET_EXPANSION_MAX_LENGTH
    ) {
      skipped += 1;
      continue;
    }

    if (!pushUnique(snippets, seen, trigger, { trigger, expansion })) {
      skipped += 1;
    }
  }

  if (snippets.length === 0) {
    throw new Error("No usable snippets were found.");
  }

  return { snippets, skipped };
}
