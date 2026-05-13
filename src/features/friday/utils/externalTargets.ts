export function firstExplicitUrl(...values: Array<string | null | undefined>) {
  const text = values.filter(Boolean).join(" ");
  return (
    text
      .split(/\s+/)
      .map((token) =>
        token
          .trim()
          .replace(/^[`"'(<[{]+/, "")
          .replace(/[>`"')\]}.,;:!?]+$/, ""),
      )
      .find((token) => token.startsWith("https://") || token.startsWith("http://")) ?? null
  );
}
