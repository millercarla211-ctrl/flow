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
  total_words: number;
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
  daily: DailyInsight[];
  top_modes: InsightBreakdown[];
  top_models: InsightBreakdown[];
};
