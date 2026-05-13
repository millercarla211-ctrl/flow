export const MAX_CONTEXT_FILE_CHARS = 40000;

const SUPPORTED_TEXT_EXTENSIONS = new Set([
  "txt",
  "md",
  "mdx",
  "json",
  "csv",
  "ts",
  "tsx",
  "js",
  "jsx",
  "css",
  "html",
  "rs",
  "toml",
  "yaml",
  "yml",
  "log",
]);

export const CONTEXT_FILE_ACCEPT =
  ".txt,.md,.mdx,.json,.csv,.ts,.tsx,.js,.jsx,.css,.html,.rs,.toml,.yaml,.yml,.log,text/*";

export function isSupportedContextFile(file: File) {
  if (file.type.startsWith("text/")) return true;
  const extension = file.name.split(".").pop()?.toLowerCase();
  return extension ? SUPPORTED_TEXT_EXTENSIONS.has(extension) : false;
}

export async function readContextFile(file: File): Promise<string> {
  const text = await file.text();
  if (text.length <= MAX_CONTEXT_FILE_CHARS) return text;
  return `${text.slice(0, MAX_CONTEXT_FILE_CHARS)}\n\n[Friday clipped this file to ${MAX_CONTEXT_FILE_CHARS.toLocaleString()} characters for local context speed.]`;
}
