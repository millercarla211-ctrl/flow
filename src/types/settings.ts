export type TranscriptionMode = "cloud" | "local";
export type TtsVoiceMode = "source_audio" | "preset";
export type TextSizeMode = "small" | "default" | "large";
export type ThemeMode = "system" | "light" | "dark";
export type AppLocaleSetting = "system" | string;

export type RecordingPrunePolicy =
  | "never"
  | "immediately"
  | "day"
  | "week"
  | "month"
  | "three_months"
  | "year";

export type LocalDataStoragePolicy = "store" | "day" | "never";

export type LlmProvider =
  | "none"
  | "local"
  | "lmstudio"
  | "ollama"
  | "openai"
  | "anthropic"
  | "google"
  | "xai"
  | "groq"
  | "cerebras"
  | "sambanova"
  | "together"
  | "openrouter"
  | "perplexity"
  | "deepseek"
  | "fireworks"
  | "mistral"
  | "custom";

export type Replacement = {
  from: string;
  to: string;
};

export type Personality = {
  id: string;
  name: string;
  enabled: boolean;
  apps: string[];
  websites: string[];
  instructions: string[];
};

export type WakeSpeakerProfile = {
  phrase: string;
  enrolled_at: string;
  duration_ms: number;
  sample_rate: number;
  sample_count: number;
  threshold: number;
  embedding: number[];
};

export type StoredSettings = {
  onboarding_completed: boolean;
  smart_shortcut: string;
  smart_shortcuts: string[];
  smart_enabled: boolean;
  hold_shortcut: string;
  hold_shortcuts: string[];
  hold_enabled: boolean;
  toggle_shortcut: string;
  toggle_shortcuts: string[];
  toggle_enabled: boolean;
  command_shortcut: string;
  command_shortcuts: string[];
  command_enabled: boolean;
  paste_last_transcript_shortcut: string;
  paste_last_transcript_shortcuts: string[];
  paste_last_transcript_enabled: boolean;
  cancel_shortcut: string;
  cancel_shortcuts: string[];
  cancel_enabled: boolean;
  wake_listening_enabled: boolean;
  wake_phrases: string[];
  wake_speaker_verification_enabled: boolean;
  wake_speaker_profile: WakeSpeakerProfile | null;
  transcription_mode: TranscriptionMode;
  local_model: string;
  tts_enabled: boolean;
  tts_auto_after_stt: boolean;
  tts_auto_play: boolean;
  tts_volume: number;
  tts_model: string;
  tts_voice_mode: TtsVoiceMode;
  tts_speaker: string;
  tts_instruction: string;
  microphone_device: string | null;
  language: string;
  app_locale: AppLocaleSetting;
  theme_mode: ThemeMode;
  llm_enabled: boolean;
  cleanup_enabled: boolean;
  llm_provider: LlmProvider;
  llm_endpoint: string;
  llm_api_key: string;
  llm_model: string;
  user_name: string;
  dictionary: string[];
  replacements: Replacement[];
  personalities: Personality[];
  edit_mode_enabled: boolean;
  auto_transform_enabled: boolean;
  auto_transform_preset_id: string;
  vibe_coding_enabled: boolean;
  vibe_coding_variable_recognition: boolean;
  vibe_coding_file_tagging: boolean;
  vibe_coding_include_window_context: boolean;
  media_control_enabled: boolean;
  auto_update_enabled: boolean;
  auto_launch_enabled: boolean;
  recording_prune_policy: RecordingPrunePolicy;
  local_data_storage_policy: LocalDataStoragePolicy;
  context_awareness_enabled: boolean;
  analytics_enabled: boolean;
  analytics_install_id: string;
};

export type AutoPasteStatus = {
  enabled: boolean;
  accessibility_granted: boolean;
};
