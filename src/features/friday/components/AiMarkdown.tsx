import ReactMarkdown from "react-markdown";
import remarkBreaks from "remark-breaks";

export function AiMarkdown({ children }: { children: string }) {
  return (
    <ReactMarkdown
      remarkPlugins={[remarkBreaks]}
      components={{
        p: ({ children: paragraph }) => (
          <p className="mb-2 last:mb-0 text-sm leading-6 text-[var(--foreground)]">{paragraph}</p>
        ),
        ul: ({ children: list }) => (
          <ul className="mb-2 list-disc space-y-1 pl-5 text-sm text-[var(--muted-foreground)]">
            {list}
          </ul>
        ),
        li: ({ children: item }) => <li className="leading-6">{item}</li>,
        code: ({ children: code }) => (
          <code className="rounded bg-[var(--secondary)] px-1 py-0.5 text-xs text-[var(--foreground)]">
            {code}
          </code>
        ),
      }}
    >
      {children}
    </ReactMarkdown>
  );
}
