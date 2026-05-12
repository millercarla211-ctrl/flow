export interface EyeState {
  w: number;
  h: number;
  r: number | string;
  x: number;
  y: number;
  sx: number;
  sy: number;
  rot: number;
  color: string;
}

export interface Keyframe {
  at: number;
  eyes: Partial<EyeState>;
  left?: Partial<EyeState>;
  right?: Partial<EyeState>;
  gap?: number;
  ease?: "linear" | "ease" | "ease-in" | "ease-out" | "spring";
}

export interface Animation {
  name: string;
  description: string;
  duration: number;
  keyframes: Keyframe[];
  trigger?: "click" | "dblclick" | "contextmenu" | "mouseenter" | "mouseleave" | "wheel" | "idle" | "keypress" | "manual";
  key?: string;
  priority?: number;
  tags?: string[];
  author?: string;
}

export const DEFAULT_EYE_STATE: EyeState = {
  w: 80,
  h: 80,
  r: 16,
  x: 0,
  y: 0,
  sx: 1,
  sy: 1,
  rot: 0,
  color: "currentColor",
};
