import { Discord } from "@/liquidglass/www/components/ui/svgs/discord";
import { Figma } from "@/liquidglass/www/components/ui/svgs/figma";
import { GithubDark } from "@/liquidglass/www/components/ui/svgs/githubDark";
import { NextjsIconDark } from "@/liquidglass/www/components/ui/svgs/nextjsIconDark";
import { Notion } from "@/liquidglass/www/components/ui/svgs/notion";
import { ReactLight } from "@/liquidglass/www/components/ui/svgs/reactLight";
import { Slack } from "@/liquidglass/www/components/ui/svgs/slack";
import { Stripe } from "@/liquidglass/www/components/ui/svgs/stripe";
import { Supabase } from "@/liquidglass/www/components/ui/svgs/supabase";
import { Typescript } from "@/liquidglass/www/components/ui/svgs/typescript";
import { Vercel } from "@/liquidglass/www/components/ui/svgs/vercel";
import { Youtube } from "@/liquidglass/www/components/ui/svgs/youtube";
import type { Tab } from "./types";

export function getLogoComponentForTab(
  tab: Tab,
): React.ComponentType<React.SVGProps<SVGSVGElement>> {
  const title = tab.title.toLowerCase();
  const url = tab.url.toLowerCase();

  if (title.includes("react") || url.includes("react")) return ReactLight;
  if (title.includes("next") || url.includes("next")) return NextjsIconDark;
  if (title.includes("typescript") || url.includes("typescript"))
    return Typescript;
  if (title.includes("github") || url.includes("github")) return GithubDark;
  if (title.includes("vercel") || url.includes("vercel")) return Vercel;
  if (title.includes("figma") || url.includes("figma")) return Figma;
  if (title.includes("notion") || url.includes("notion")) return Notion;
  if (title.includes("slack") || url.includes("slack")) return Slack;
  if (title.includes("discord") || url.includes("discord")) return Discord;
  if (title.includes("youtube") || url.includes("youtube")) return Youtube;
  if (title.includes("supabase") || url.includes("supabase")) return Supabase;
  if (title.includes("stripe") || url.includes("stripe")) return Stripe;

  if (url.includes(".com")) return GithubDark;
  if (url.includes(".io")) return Vercel;
  if (url.includes(".dev")) return Typescript;

  return ReactLight;
}
