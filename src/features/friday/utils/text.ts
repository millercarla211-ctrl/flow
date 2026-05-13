export function textFromMessage(message: { parts: Array<{ type: string; text?: string }> }) {
  return message.parts
    .filter((part) => part.type === "text" && typeof part.text === "string")
    .map((part) => part.text)
    .join("\n")
    .trim();
}

export function titleFromText(text: string, fallback: string) {
  const firstLine = text
    .split("\n")
    .map((line) => line.replace(/^#+\s*/, "").trim())
    .find(Boolean);

  if (!firstLine) return fallback;
  return firstLine.length > 64 ? `${firstLine.slice(0, 61)}...` : firstLine;
}
