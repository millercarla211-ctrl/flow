export type ScreenType = "browser" | "terminal" | "code" | "welcome" | "custom";

export interface Screen {
  id: string;
  type: ScreenType;
  title: string;
  width: number;
  height: number;
  /** Lucide icon name for dock (custom screens only). */
  dockIcon?: string;
}

export interface ScreenDimensions {
  width: number;
  height: number;
}
