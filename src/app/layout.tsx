import type { Metadata, Viewport } from "next";
import type { ReactNode } from "react";
import "./App.css";

export const metadata: Metadata = {
  title: "Flow",
  description: "Free, unlimited local dictation for every app.",
  icons: {
    icon: "/favicon.svg",
  },
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html lang="en" className="dark" data-theme="dark" suppressHydrationWarning>
      <body>{children}</body>
    </html>
  );
}
