export type TransformPreset = {
  id: string;
  label: string;
  instruction: string;
};

export type TransformResult = {
  preset_id?: string | null;
  label: string;
  original: string;
  transformed: string;
};
