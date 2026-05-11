export type DailyInsight = {
  date: string;
  label: string;
  words: number;
  transcriptions: number;
  audio_duration_seconds: number;
};

export type InsightBreakdown = {
  label: string;
  count: number;
  words: number;
  audio_duration_seconds: number;
};

export type InsightsSummary = {
  days: number;
  total_transcriptions: number;
  pinned_transcriptions: number;
  total_words: number;
  pinned_words: number;
  words_today: number;
  words_this_week: number;
  total_audio_seconds: number;
  average_words_per_minute: number;
  average_words_per_day: number;
  current_streak_days: number;
  best_day_words: number;
  best_day_label: string;
  local_percent: number;
  cleanup_percent: number;
  timed_transcriptions: number;
  average_stt_elapsed_ms: number;
  average_cleanup_elapsed_ms: number;
  average_paste_elapsed_ms: number;
  average_total_elapsed_ms: number;
  auto_paste_attempts: number;
  auto_paste_success_percent: number;
  paste_fallback_count: number;
  daily: DailyInsight[];
  top_modes: InsightBreakdown[];
  top_models: InsightBreakdown[];
  top_transforms: InsightBreakdown[];
};
