export type TransformPreset = {
  id: string;
  label: string;
  instruction: string;
};

export type TransformResult = {
  history_id?: string | null;
  preset_id?: string | null;
  label: string;
  original: string;
  transformed: string;
  instruction?: string | null;
  created_at?: string | null;
};

export type TransformSource = {
  source: "selection" | "clipboard";
  text: string;
};

export type TransformHistoryEntry = {
  id: string;
  label: string;
  preset_id?: string | null;
  instruction?: string | null;
  original: string;
  transformed: string;
  created_at: string;
};
