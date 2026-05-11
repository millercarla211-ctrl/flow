use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Datelike, Local, NaiveDate, TimeZone};
use parking_lot::Mutex;
use regex::Regex;
use rusqlite::{params, types::Type, Connection, OptionalExtension, Row, ToSql};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::library::{LibraryFilter, LibraryItem, LibraryItemPatch};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionRecord {
    pub id: String,
    pub timestamp: DateTime<Local>,
    /// The final text (cleaned if LLM was used, otherwise raw)
    pub text: String,
    /// The raw transcription before LLM cleanup (if applicable)
    #[serde(default)]
    pub raw_text: Option<String>,
    pub audio_path: String,
    #[serde(default)]
    pub audio_available: bool,
    pub status: TranscriptionStatus,
    pub error_message: Option<String>,
    /// Whether LLM cleanup was applied
    #[serde(default)]
    pub llm_cleaned: bool,
    #[serde(default)]
    pub speech_model: String,
    #[serde(default)]
    pub llm_model: Option<String>,
    #[serde(default)]
    pub word_count: u32,
    #[serde(default)]
    pub audio_duration_seconds: f32,
    #[serde(default)]
    pub synced: bool,
    #[serde(default)]
    pub mode_id: Option<String>,
    #[serde(default)]
    pub mode_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptionStatus {
    Success,
    Error,
}

impl TranscriptionStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
        }
    }

    fn from_str(value: &str) -> std::result::Result<Self, &'static str> {
        match value.to_ascii_lowercase().as_str() {
            "success" => Ok(Self::Success),
            "error" => Ok(Self::Error),
            _ => Err("Unknown transcription status"),
        }
    }
}

pub struct StorageManager {
    connection: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchpadEntry {
    pub id: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchpadVersion {
    pub id: String,
    pub entry_id: String,
    pub body: String,
    pub created_at: DateTime<Local>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub trigger: String,
    pub expansion: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowFetchLink {
    pub id: String,
    pub url: String,
    pub label: String,
    pub created_at: DateTime<Local>,
    pub last_seen_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyInsight {
    pub date: String,
    pub label: String,
    pub words: u32,
    pub transcriptions: u32,
    pub audio_duration_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightBreakdown {
    pub label: String,
    pub count: u32,
    pub words: u32,
    pub audio_duration_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsSummary {
    pub days: usize,
    pub total_transcriptions: u32,
    pub total_words: u32,
    pub words_today: u32,
    pub words_this_week: u32,
    pub total_audio_seconds: f32,
    pub average_words_per_minute: f32,
    pub average_words_per_day: f32,
    pub current_streak_days: u32,
    pub best_day_words: u32,
    pub best_day_label: String,
    pub local_percent: f32,
    pub cleanup_percent: f32,
    pub daily: Vec<DailyInsight>,
    pub top_modes: Vec<InsightBreakdown>,
    pub top_models: Vec<InsightBreakdown>,
}

#[derive(Debug, Clone)]
pub struct TranscriptionMetadata {
    pub speech_model: String,
    pub llm_model: Option<String>,
    pub word_count: u32,
    pub audio_duration_seconds: f32,
    pub synced: bool,
    pub mode_id: Option<String>,
    pub mode_name: Option<String>,
}

impl Default for TranscriptionMetadata {
    fn default() -> Self {
        Self {
            speech_model: String::new(),
            llm_model: None,
            word_count: 0,
            audio_duration_seconds: 0.0,
            synced: false,
            mode_id: None,
            mode_name: None,
        }
    }
}

impl StorageManager {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create storage directory at {}", parent.display())
            })?;
        }

        let connection = Connection::open(&db_path).with_context(|| {
            format!(
                "Failed to open transcription database at {}",
                db_path.display()
            )
        })?;

        Self::configure_connection(&connection)?;
        Self::apply_migrations(&connection)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn save_transcription(
        &self,
        text: String,
        audio_path: String,
        status: TranscriptionStatus,
        error_message: Option<String>,
        metadata: TranscriptionMetadata,
        id_override: Option<String>,
    ) -> Result<TranscriptionRecord> {
        let record = TranscriptionRecord {
            id: id_override.unwrap_or_else(|| Uuid::new_v4().to_string()),
            timestamp: Local::now(),
            text,
            raw_text: None,
            audio_available: !audio_path.is_empty(),
            audio_path,
            status,
            error_message,
            llm_cleaned: false,
            speech_model: metadata.speech_model,
            llm_model: metadata.llm_model,
            word_count: metadata.word_count,
            audio_duration_seconds: metadata.audio_duration_seconds,
            synced: metadata.synced,
            mode_id: metadata.mode_id,
            mode_name: metadata.mode_name,
        };

        let conn = self.connection.lock();
        Self::insert_record(&conn, &record)?;
        Ok(record)
    }

    pub fn save_transcription_with_cleanup(
        &self,
        raw_text: String,
        cleaned_text: String,
        audio_path: String,
        metadata: TranscriptionMetadata,
        id_override: Option<String>,
    ) -> Result<TranscriptionRecord> {
        let record = TranscriptionRecord {
            id: id_override.unwrap_or_else(|| Uuid::new_v4().to_string()),
            timestamp: Local::now(),
            text: cleaned_text,
            raw_text: Some(raw_text),
            audio_available: !audio_path.is_empty(),
            audio_path,
            status: TranscriptionStatus::Success,
            error_message: None,
            llm_cleaned: true,
            speech_model: metadata.speech_model,
            llm_model: metadata.llm_model,
            word_count: metadata.word_count,
            audio_duration_seconds: metadata.audio_duration_seconds,
            synced: metadata.synced,
            mode_id: metadata.mode_id,
            mode_name: metadata.mode_name,
        };

        let conn = self.connection.lock();
        Self::insert_record(&conn, &record)?;
        Ok(record)
    }

    pub fn create_scratchpad_entry(&self, body: String, source: String) -> Result<ScratchpadEntry> {
        let body = body.trim().to_string();
        if body.is_empty() {
            bail!("Scratchpad body cannot be empty");
        }

        let now = Local::now();
        let now_ms = now.timestamp_millis();
        let entry = ScratchpadEntry {
            id: Uuid::new_v4().to_string(),
            title: derive_scratchpad_title(&body),
            body,
            source: normalize_source(&source),
            created_at: now,
            updated_at: now,
            version: 1,
        };

        let conn = self.connection.lock();
        conn.execute(
            "INSERT INTO scratchpad_entries (id, title, body, source, created_at, updated_at, version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &entry.id,
                &entry.title,
                &entry.body,
                &entry.source,
                now_ms,
                now_ms,
                entry.version as i64,
            ],
        )?;
        conn.execute(
            "INSERT INTO scratchpad_versions (id, entry_id, body, created_at, version)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                Uuid::new_v4().to_string(),
                &entry.id,
                &entry.body,
                now_ms,
                entry.version as i64,
            ],
        )?;

        Ok(entry)
    }

    pub fn get_scratchpad_entries(
        &self,
        search_query: Option<&str>,
    ) -> Result<Vec<ScratchpadEntry>> {
        let conn = self.connection.lock();
        let search_query = search_query
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if let Some(query) = search_query {
            let needle = format!("%{query}%");
            let mut stmt = conn.prepare(
                "SELECT id, title, body, source, created_at, updated_at, version
                 FROM scratchpad_entries
                 WHERE title LIKE ?1 COLLATE NOCASE OR body LIKE ?1 COLLATE NOCASE
                 ORDER BY updated_at DESC",
            )?;
            let rows = stmt.query_map(params![needle], Self::scratchpad_entry_from_row)?;
            return rows
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(Into::into);
        }

        let mut stmt = conn.prepare(
            "SELECT id, title, body, source, created_at, updated_at, version
             FROM scratchpad_entries
             ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], Self::scratchpad_entry_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn get_scratchpad_versions(&self, entry_id: &str) -> Result<Vec<ScratchpadVersion>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            "SELECT id, entry_id, body, created_at, version
             FROM scratchpad_versions
             WHERE entry_id = ?1
             ORDER BY version DESC, created_at DESC",
        )?;
        let rows = stmt.query_map(params![entry_id], Self::scratchpad_version_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn update_scratchpad_entry(
        &self,
        id: String,
        title: Option<String>,
        body: String,
    ) -> Result<Option<ScratchpadEntry>> {
        let body = body.trim().to_string();
        if body.is_empty() {
            bail!("Scratchpad body cannot be empty");
        }

        let conn = self.connection.lock();
        let current = conn
            .query_row(
                "SELECT id, title, body, source, created_at, updated_at, version
                 FROM scratchpad_entries WHERE id = ?1",
                params![&id],
                Self::scratchpad_entry_from_row,
            )
            .optional()?;

        let Some(current) = current else {
            return Ok(None);
        };

        let next_version = if current.body == body {
            current.version
        } else {
            current.version.saturating_add(1)
        };
        let now = Local::now();
        let now_ms = now.timestamp_millis();
        let title = title
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| derive_scratchpad_title(&body));

        conn.execute(
            "UPDATE scratchpad_entries
             SET title = ?1, body = ?2, updated_at = ?3, version = ?4
             WHERE id = ?5",
            params![&title, &body, now_ms, next_version as i64, &current.id],
        )?;

        if next_version != current.version {
            conn.execute(
                "INSERT INTO scratchpad_versions (id, entry_id, body, created_at, version)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    Uuid::new_v4().to_string(),
                    &current.id,
                    &body,
                    now_ms,
                    next_version as i64,
                ],
            )?;
        }

        Self::get_scratchpad_entry(&conn, &id)
    }

    pub fn delete_scratchpad_entry(&self, id: &str) -> Result<bool> {
        let conn = self.connection.lock();
        let affected = conn.execute("DELETE FROM scratchpad_entries WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn get_snippets(&self) -> Result<Vec<Snippet>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            "SELECT id, trigger, expansion, created_at, updated_at
             FROM snippets
             ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], Self::snippet_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn create_snippet(&self, trigger: String, expansion: String) -> Result<Snippet> {
        let trigger = normalize_snippet_trigger(trigger)?;
        let expansion = normalize_snippet_expansion(expansion)?;
        let now = Local::now();
        let now_ms = now.timestamp_millis();
        let snippet = Snippet {
            id: Uuid::new_v4().to_string(),
            trigger,
            expansion,
            created_at: now,
            updated_at: now,
        };

        let conn = self.connection.lock();
        conn.execute(
            "INSERT INTO snippets (id, trigger, expansion, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &snippet.id,
                &snippet.trigger,
                &snippet.expansion,
                now_ms,
                now_ms,
            ],
        )?;

        Ok(snippet)
    }

    pub fn update_snippet(
        &self,
        id: String,
        trigger: String,
        expansion: String,
    ) -> Result<Option<Snippet>> {
        let trigger = normalize_snippet_trigger(trigger)?;
        let expansion = normalize_snippet_expansion(expansion)?;
        let now_ms = Local::now().timestamp_millis();
        let conn = self.connection.lock();
        let affected = conn.execute(
            "UPDATE snippets
             SET trigger = ?1, expansion = ?2, updated_at = ?3
             WHERE id = ?4",
            params![&trigger, &expansion, now_ms, &id],
        )?;

        if affected == 0 {
            return Ok(None);
        }

        Self::get_snippet(&conn, &id)
    }

    pub fn delete_snippet(&self, id: &str) -> Result<bool> {
        let conn = self.connection.lock();
        let affected = conn.execute("DELETE FROM snippets WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn upsert_flow_fetch_link(&self, url: String) -> Result<FlowFetchLink> {
        let url = normalize_flow_fetch_url(url)?;
        let label = derive_flow_fetch_label(&url);
        let now = Local::now();
        let now_ms = now.timestamp_millis();
        let conn = self.connection.lock();

        let existing_id = conn
            .query_row(
                "SELECT id FROM flow_fetch_links WHERE url = ?1",
                params![&url],
                |row| row.get::<_, String>("id"),
            )
            .optional()?;

        if let Some(id) = existing_id {
            conn.execute(
                "UPDATE flow_fetch_links
                 SET label = ?1, last_seen_at = ?2
                 WHERE id = ?3",
                params![&label, now_ms, &id],
            )?;
            return Self::get_flow_fetch_link(&conn, &id)
                .and_then(|value| value.context("Flow Fetch link disappeared after update"));
        }

        let link = FlowFetchLink {
            id: Uuid::new_v4().to_string(),
            url,
            label,
            created_at: now,
            last_seen_at: now,
        };

        conn.execute(
            "INSERT INTO flow_fetch_links (id, url, label, created_at, last_seen_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&link.id, &link.url, &link.label, now_ms, now_ms],
        )?;
        Ok(link)
    }

    pub fn get_flow_fetch_links(&self, limit: usize) -> Result<Vec<FlowFetchLink>> {
        let limit = limit.clamp(1, 100) as i64;
        let conn = self.connection.lock();
        Self::prune_flow_fetch_links_inner(&conn)?;

        let mut stmt = conn.prepare(
            "SELECT id, url, label, created_at, last_seen_at
             FROM flow_fetch_links
             ORDER BY last_seen_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], Self::flow_fetch_link_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn delete_flow_fetch_link(&self, id: &str) -> Result<bool> {
        let conn = self.connection.lock();
        let affected = conn.execute("DELETE FROM flow_fetch_links WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn prune_flow_fetch_links(&self) -> Result<usize> {
        let conn = self.connection.lock();
        Self::prune_flow_fetch_links_inner(&conn)
    }

    pub fn update_with_llm_cleanup(
        &self,
        id: &str,
        cleaned_text: String,
        llm_model: Option<String>,
    ) -> Result<Option<TranscriptionRecord>> {
        let conn = self.connection.lock();
        Self::apply_llm_cleanup(&conn, id, &cleaned_text, llm_model.as_deref())
    }

    pub fn revert_to_raw(&self, id: &str) -> Result<Option<TranscriptionRecord>> {
        let conn = self.connection.lock();
        Self::revert_to_raw_internal(&conn, id)
    }

    pub fn update_transcription_result(
        &self,
        id: &str,
        text: String,
        raw_text: Option<String>,
        status: TranscriptionStatus,
        error_message: Option<String>,
        metadata: TranscriptionMetadata,
    ) -> Result<Option<TranscriptionRecord>> {
        let conn = self.connection.lock();
        let existing = Self::get_record(&conn, id)?;
        if existing.is_none() {
            return Ok(None);
        }

        conn.execute(
            "UPDATE transcriptions SET 
                text = ?1,
                raw_text = ?2,
                status = ?3,
                error_message = ?4,
                llm_cleaned = ?5,
                speech_model = ?6,
                llm_model = ?7,
                word_count = ?8,
                audio_duration_seconds = ?9,
                synced = ?10,
                mode_id = ?11,
                mode_name = ?12
             WHERE id = ?13",
            params![
                text,
                raw_text,
                status.as_str(),
                error_message,
                raw_text.is_some(),
                metadata.speech_model,
                metadata.llm_model,
                metadata.word_count,
                metadata.audio_duration_seconds,
                metadata.synced,
                metadata.mode_id,
                metadata.mode_name,
                id,
            ],
        )?;

        Self::get_record(&conn, id)
    }

    pub fn get_all_filtered(&self, search_query: Option<&str>) -> Result<Vec<TranscriptionRecord>> {
        let conn = self.connection.lock();
        let (where_clause, params) = Self::build_search_query(search_query);

        let sql = format!(
            "SELECT id, timestamp, text, raw_text, audio_path, status, error_message, llm_cleaned,
                    speech_model, llm_model, word_count, audio_duration_seconds, synced, mode_id, mode_name
             FROM transcriptions
             {}
             ORDER BY timestamp DESC",
            where_clause
        );

        let mut stmt = conn.prepare(&sql)?;
        let records = stmt
            .query_map(
                rusqlite::params_from_iter(params.iter()),
                Self::record_from_row,
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(records)
    }

    pub fn get_recent_transcriptions(&self, limit: usize) -> Result<Vec<TranscriptionRecord>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, text, raw_text, audio_path, status, error_message, llm_cleaned,
                    speech_model, llm_model, word_count, audio_duration_seconds, synced, mode_id, mode_name
             FROM transcriptions
             WHERE status = ?1 AND text <> ''
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        let records = stmt
            .query_map(
                params![TranscriptionStatus::Success.as_str(), limit as i64],
                Self::record_from_row,
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(records)
    }

    pub fn get_insights(&self, days: usize) -> Result<InsightsSummary> {
        let days = days.clamp(7, 90);
        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, text, raw_text, audio_path, status, error_message, llm_cleaned,
                    speech_model, llm_model, word_count, audio_duration_seconds, synced, mode_id, mode_name
             FROM transcriptions
             WHERE status = ?1 AND text <> ''
             ORDER BY timestamp DESC",
        )?;

        let records = stmt
            .query_map(
                params![TranscriptionStatus::Success.as_str()],
                Self::record_from_row,
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(build_insights_summary(
            &records,
            days,
            Local::now().date_naive(),
        ))
    }

    pub fn delete(&self, id: &str) -> Result<Option<String>> {
        let conn = self.connection.lock();
        let record = Self::get_record(&conn, id)?;
        if record.is_some() {
            conn.execute("DELETE FROM transcriptions WHERE id = ?1", params![id])?;
        }
        Ok(record.map(|r| r.audio_path))
    }

    /// Delete all transcription records and return their audio paths
    pub fn delete_all(&self) -> Result<Vec<String>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare("SELECT audio_path FROM transcriptions")?;
        let paths = stmt
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;
        conn.execute("DELETE FROM transcriptions", [])?;
        Ok(paths)
    }

    pub fn get_by_id(&self, id: &str) -> Option<TranscriptionRecord> {
        let conn = self.connection.lock();
        match Self::get_record(&conn, id) {
            Ok(record) => record,
            Err(err) => {
                eprintln!("Failed to read transcription {id}: {err}");
                None
            }
        }
    }

    fn build_search_query(search_query: Option<&str>) -> (String, Vec<Box<dyn ToSql>>) {
        if let Some(query) = search_query {
            if !query.trim().is_empty() {
                let like_query = format!("%{}%", query.trim());
                return (
                    "WHERE text LIKE ?1 OR raw_text LIKE ?1".to_string(),
                    vec![Box::new(like_query)],
                );
            }
        }
        ("".to_string(), Vec::new())
    }

    fn insert_record(conn: &Connection, record: &TranscriptionRecord) -> Result<()> {
        let timestamp = record.timestamp.timestamp_millis();
        conn.execute(
            "INSERT INTO transcriptions (
                id,
                timestamp,
                text,
                raw_text,
                audio_path,
                status,
                error_message,
                llm_cleaned,
                speech_model,
                llm_model,
                word_count,
                audio_duration_seconds,
                synced,
                mode_id,
                mode_name
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                record.id,
                timestamp,
                record.text,
                record.raw_text,
                record.audio_path,
                record.status.as_str(),
                record.error_message,
                if record.llm_cleaned { 1 } else { 0 },
                record.speech_model,
                record.llm_model,
                record.word_count as i64,
                record.audio_duration_seconds as f64,
                if record.synced { 1 } else { 0 },
                record.mode_id,
                record.mode_name,
            ],
        )?;
        Ok(())
    }

    fn apply_llm_cleanup(
        conn: &Connection,
        id: &str,
        cleaned_text: &str,
        llm_model: Option<&str>,
    ) -> Result<Option<TranscriptionRecord>> {
        if let Some(mut record) = Self::get_record(conn, id)? {
            if record.raw_text.is_none() {
                record.raw_text = Some(record.text.clone());
            }
            record.text = cleaned_text.to_string();
            record.llm_cleaned = true;
            record.llm_model = llm_model.map(|value| value.to_string());
            record.word_count = count_words(&record.text);
            record.synced = false;

            conn.execute(
                "UPDATE transcriptions
                 SET text = ?1, raw_text = ?2, llm_cleaned = 1, llm_model = ?3, word_count = ?4, synced = 0
                 WHERE id = ?5",
                params![
                    record.text,
                    record.raw_text,
                    record.llm_model,
                    record.word_count as i64,
                    id
                ],
            )?;

            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    fn revert_to_raw_internal(conn: &Connection, id: &str) -> Result<Option<TranscriptionRecord>> {
        if let Some(mut record) = Self::get_record(conn, id)? {
            if let Some(raw) = record.raw_text.take() {
                record.text = raw;
                record.llm_cleaned = false;
                record.word_count = count_words(&record.text);
                record.llm_model = None;
                record.synced = false;
                conn.execute(
                    "UPDATE transcriptions
                     SET text = ?1, raw_text = NULL, llm_cleaned = 0, llm_model = NULL, word_count = ?2, synced = 0
                     WHERE id = ?3",
                    params![record.text, record.word_count as i64, id],
                )?;
                return Ok(Some(record));
            }
        }
        Ok(None)
    }

    fn get_record(conn: &Connection, id: &str) -> Result<Option<TranscriptionRecord>> {
        conn.query_row(
            "SELECT id, timestamp, text, raw_text, audio_path, status, error_message, llm_cleaned,
                    speech_model, llm_model, word_count, audio_duration_seconds, synced, mode_id, mode_name
             FROM transcriptions WHERE id = ?1",
            params![id],
            Self::record_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    fn get_scratchpad_entry(conn: &Connection, id: &str) -> Result<Option<ScratchpadEntry>> {
        conn.query_row(
            "SELECT id, title, body, source, created_at, updated_at, version
             FROM scratchpad_entries WHERE id = ?1",
            params![id],
            Self::scratchpad_entry_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    fn get_snippet(conn: &Connection, id: &str) -> Result<Option<Snippet>> {
        conn.query_row(
            "SELECT id, trigger, expansion, created_at, updated_at
             FROM snippets WHERE id = ?1",
            params![id],
            Self::snippet_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    fn get_flow_fetch_link(conn: &Connection, id: &str) -> Result<Option<FlowFetchLink>> {
        conn.query_row(
            "SELECT id, url, label, created_at, last_seen_at
             FROM flow_fetch_links WHERE id = ?1",
            params![id],
            Self::flow_fetch_link_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    fn record_from_row(row: &Row<'_>) -> rusqlite::Result<TranscriptionRecord> {
        let timestamp_ms: i64 = row.get("timestamp")?;
        let timestamp = local_datetime_from_millis(timestamp_ms, 0)?;

        let status_value: String = row.get("status")?;
        let status = TranscriptionStatus::from_str(&status_value).map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                Type::Text,
                Box::new(io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
                    as Box<dyn std::error::Error + Send + Sync + 'static>,
            )
        })?;

        let audio_path: String = row.get("audio_path")?;
        let audio_available = if audio_path.is_empty() {
            false
        } else {
            PathBuf::from(&audio_path).exists()
        };

        Ok(TranscriptionRecord {
            id: row.get("id")?,
            timestamp,
            text: row.get("text")?,
            raw_text: row.get("raw_text")?,
            audio_path,
            audio_available,
            status,
            error_message: row.get("error_message")?,
            llm_cleaned: row.get::<_, i64>("llm_cleaned")? == 1,
            speech_model: row.get("speech_model")?,
            llm_model: row.get("llm_model")?,
            word_count: row.get::<_, i64>("word_count")? as u32,
            audio_duration_seconds: row.get::<_, f64>("audio_duration_seconds")? as f32,
            synced: row.get::<_, i64>("synced").unwrap_or(0) == 1,
            mode_id: row.get("mode_id").unwrap_or(None),
            mode_name: row.get("mode_name").unwrap_or(None),
        })
    }

    fn scratchpad_entry_from_row(row: &Row<'_>) -> rusqlite::Result<ScratchpadEntry> {
        let created_ms: i64 = row.get("created_at")?;
        let updated_ms: i64 = row.get("updated_at")?;

        Ok(ScratchpadEntry {
            id: row.get("id")?,
            title: row.get("title")?,
            body: row.get("body")?,
            source: row.get("source")?,
            created_at: local_datetime_from_millis(created_ms, 0)?,
            updated_at: local_datetime_from_millis(updated_ms, 0)?,
            version: row.get::<_, i64>("version")? as u32,
        })
    }

    fn scratchpad_version_from_row(row: &Row<'_>) -> rusqlite::Result<ScratchpadVersion> {
        let created_ms: i64 = row.get("created_at")?;

        Ok(ScratchpadVersion {
            id: row.get("id")?,
            entry_id: row.get("entry_id")?,
            body: row.get("body")?,
            created_at: local_datetime_from_millis(created_ms, 0)?,
            version: row.get::<_, i64>("version")? as u32,
        })
    }

    fn snippet_from_row(row: &Row<'_>) -> rusqlite::Result<Snippet> {
        let created_ms: i64 = row.get("created_at")?;
        let updated_ms: i64 = row.get("updated_at")?;

        Ok(Snippet {
            id: row.get("id")?,
            trigger: row.get("trigger")?,
            expansion: row.get("expansion")?,
            created_at: local_datetime_from_millis(created_ms, 0)?,
            updated_at: local_datetime_from_millis(updated_ms, 0)?,
        })
    }

    fn flow_fetch_link_from_row(row: &Row<'_>) -> rusqlite::Result<FlowFetchLink> {
        let created_ms: i64 = row.get("created_at")?;
        let last_seen_ms: i64 = row.get("last_seen_at")?;

        Ok(FlowFetchLink {
            id: row.get("id")?,
            url: row.get("url")?,
            label: row.get("label")?,
            created_at: local_datetime_from_millis(created_ms, 0)?,
            last_seen_at: local_datetime_from_millis(last_seen_ms, 0)?,
        })
    }

    fn configure_connection(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;\nPRAGMA synchronous = NORMAL;\nPRAGMA foreign_keys = ON;",
        )?;
        Ok(())
    }

    fn apply_migrations(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                text TEXT NOT NULL,
                raw_text TEXT NULL,
                audio_path TEXT NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT NULL,
                llm_cleaned INTEGER NOT NULL DEFAULT 0,
                speech_model TEXT NOT NULL DEFAULT '',
                llm_model TEXT NULL,
                word_count INTEGER NOT NULL DEFAULT 0,
                audio_duration_seconds REAL NOT NULL DEFAULT 0,
                synced INTEGER NOT NULL DEFAULT 0,
                mode_id TEXT NULL,
                mode_name TEXT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_transcriptions_timestamp ON transcriptions(timestamp);
            CREATE INDEX IF NOT EXISTS idx_transcriptions_status ON transcriptions(status);
            CREATE INDEX IF NOT EXISTS idx_transcriptions_speech_model ON transcriptions(speech_model);

            CREATE TABLE IF NOT EXISTS library_items (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                audio_path TEXT NOT NULL,
                source_path TEXT NOT NULL DEFAULT '',
                store_original INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'pending',
                progress REAL DEFAULT 0,
                error_message TEXT,
                transcript TEXT,
                segments TEXT,
                duration_seconds REAL NOT NULL,
                file_size_bytes INTEGER NOT NULL,
                original_format TEXT NOT NULL,
                created_at TEXT NOT NULL,
                transcribed_at TEXT,
                tags TEXT NOT NULL DEFAULT '[]',
                llm_cleanup_enabled INTEGER NOT NULL DEFAULT 0,
                speech_model TEXT NOT NULL,
                show_timestamps INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_library_items_created_at ON library_items(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_library_items_status ON library_items(status);

            CREATE TABLE IF NOT EXISTS scratchpad_entries (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                source TEXT NOT NULL DEFAULT 'manual',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE INDEX IF NOT EXISTS idx_scratchpad_entries_updated_at ON scratchpad_entries(updated_at DESC);

            CREATE TABLE IF NOT EXISTS scratchpad_versions (
                id TEXT PRIMARY KEY,
                entry_id TEXT NOT NULL,
                body TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                version INTEGER NOT NULL,
                FOREIGN KEY(entry_id) REFERENCES scratchpad_entries(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_scratchpad_versions_entry_id ON scratchpad_versions(entry_id);

            CREATE TABLE IF NOT EXISTS snippets (
                id TEXT PRIMARY KEY,
                trigger TEXT NOT NULL COLLATE NOCASE UNIQUE,
                expansion TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_snippets_updated_at ON snippets(updated_at DESC);

            CREATE TABLE IF NOT EXISTS flow_fetch_links (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL UNIQUE,
                label TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_seen_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_flow_fetch_links_last_seen_at ON flow_fetch_links(last_seen_at DESC);",
        )?;

        Self::ensure_column(
            conn,
            "transcriptions",
            "speech_model",
            "ALTER TABLE transcriptions ADD COLUMN speech_model TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            conn,
            "transcriptions",
            "llm_model",
            "ALTER TABLE transcriptions ADD COLUMN llm_model TEXT NULL",
        )?;
        Self::ensure_column(
            conn,
            "transcriptions",
            "word_count",
            "ALTER TABLE transcriptions ADD COLUMN word_count INTEGER NOT NULL DEFAULT 0",
        )?;
        Self::ensure_column(
            conn,
            "transcriptions",
            "audio_duration_seconds",
            "ALTER TABLE transcriptions ADD COLUMN audio_duration_seconds REAL NOT NULL DEFAULT 0",
        )?;
        Self::ensure_column(
            conn,
            "transcriptions",
            "synced",
            "ALTER TABLE transcriptions ADD COLUMN synced INTEGER NOT NULL DEFAULT 0",
        )?;
        Self::ensure_column(
            conn,
            "transcriptions",
            "mode_id",
            "ALTER TABLE transcriptions ADD COLUMN mode_id TEXT NULL",
        )?;
        Self::ensure_column(
            conn,
            "transcriptions",
            "mode_name",
            "ALTER TABLE transcriptions ADD COLUMN mode_name TEXT NULL",
        )?;
        Self::ensure_column(
            conn,
            "library_items",
            "show_timestamps",
            "ALTER TABLE library_items ADD COLUMN show_timestamps INTEGER NOT NULL DEFAULT 0",
        )?;
        Self::ensure_column(
            conn,
            "library_items",
            "source_path",
            "ALTER TABLE library_items ADD COLUMN source_path TEXT NOT NULL DEFAULT ''",
        )?;
        Self::ensure_column(
            conn,
            "library_items",
            "store_original",
            "ALTER TABLE library_items ADD COLUMN store_original INTEGER NOT NULL DEFAULT 0",
        )?;
        Ok(())
    }

    fn prune_flow_fetch_links_inner(conn: &Connection) -> Result<usize> {
        let cutoff_ms = (Local::now() - chrono::Duration::days(14)).timestamp_millis();
        conn.execute(
            "DELETE FROM flow_fetch_links WHERE last_seen_at < ?1",
            params![cutoff_ms],
        )
        .map_err(Into::into)
    }

    fn ensure_column(conn: &Connection, table: &str, column: &str, add_sql: &str) -> Result<()> {
        if !Self::column_exists(conn, table, column)? {
            conn.execute(add_sql, [])?;
        }
        Ok(())
    }

    fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
        let query = format!("PRAGMA table_info({table})");
        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get("name")?;
            if name == column {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl StorageManager {
    pub fn insert_library_item(&self, item: LibraryItem) -> Result<LibraryItem> {
        let conn = self.connection.lock();
        crate::library::repo::insert_library_item(&conn, item)
    }

    pub fn get_library_item(&self, id: &str) -> Result<Option<LibraryItem>> {
        let conn = self.connection.lock();
        crate::library::repo::get_library_item(&conn, id)
    }

    pub fn get_library_items_page(
        &self,
        filter: LibraryFilter,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<LibraryItem>, bool)> {
        let conn = self.connection.lock();
        crate::library::repo::get_library_items_page(&conn, filter, limit, offset)
    }

    pub fn get_recoverable_library_items(&self) -> Result<Vec<LibraryItem>> {
        let conn = self.connection.lock();
        crate::library::repo::get_recoverable_library_items(&conn)
    }

    pub fn update_library_item(
        &self,
        id: &str,
        patch: LibraryItemPatch,
    ) -> Result<Option<LibraryItem>> {
        let mut conn = self.connection.lock();
        crate::library::repo::update_library_item(&mut conn, id, patch)
    }

    pub fn delete_library_item(&self, id: &str) -> Result<Option<String>> {
        let conn = self.connection.lock();
        crate::library::repo::delete_library_item(&conn, id)
    }

    pub fn get_library_tags(&self) -> Result<Vec<String>> {
        let conn = self.connection.lock();
        crate::library::repo::get_library_tags(&conn)
    }
}

fn count_words(text: &str) -> u32 {
    crate::transcribe::count_words(text)
}

pub fn apply_snippets_to_text(text: &str, snippets: &[Snippet]) -> String {
    if text.trim().is_empty() || snippets.is_empty() {
        return text.to_string();
    }

    let normalized_text = normalize_phrase_for_match(text);
    let mut ordered = snippets.to_vec();
    ordered.sort_by(|a, b| b.trigger.len().cmp(&a.trigger.len()));

    for snippet in &ordered {
        if normalize_phrase_for_match(&snippet.trigger) == normalized_text {
            return snippet.expansion.clone();
        }
    }

    let mut result = text.to_string();
    for snippet in ordered {
        let pattern = snippet_phrase_pattern(&snippet.trigger);
        if pattern.is_empty() {
            continue;
        }

        let Ok(regex) = Regex::new(&format!(
            r"(?i)(^|[^\p{{L}}\p{{N}}_])({pattern})($|[^\p{{L}}\p{{N}}_])"
        )) else {
            continue;
        };
        let expansion = snippet.expansion.clone();
        result = regex
            .replace_all(&result, |captures: &regex::Captures<'_>| {
                let prefix = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                let suffix = captures.get(3).map(|m| m.as_str()).unwrap_or("");
                format!("{prefix}{expansion}{suffix}")
            })
            .into_owned();
    }

    result
}

fn derive_scratchpad_title(body: &str) -> String {
    let title = body
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(body)
        .split_whitespace()
        .take(9)
        .collect::<Vec<_>>()
        .join(" ");

    if title.is_empty() {
        return "Untitled note".to_string();
    }

    let max_chars = 72;
    if title.chars().count() <= max_chars {
        return title;
    }

    let mut shortened = title
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>();
    shortened.push_str("...");
    shortened
}

fn normalize_source(source: &str) -> String {
    let normalized = source.trim();
    if normalized.is_empty() {
        "manual".to_string()
    } else {
        normalized.chars().take(80).collect()
    }
}

fn normalize_snippet_trigger(trigger: String) -> Result<String> {
    let normalized = trigger.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        bail!("Snippet trigger cannot be empty");
    }
    if normalized.chars().count() > 59 {
        bail!("Snippet trigger must be 59 characters or fewer");
    }
    Ok(normalized)
}

fn normalize_snippet_expansion(expansion: String) -> Result<String> {
    let normalized = expansion.trim().to_string();
    if normalized.is_empty() {
        bail!("Snippet expansion cannot be empty");
    }
    Ok(normalized)
}

fn normalize_phrase_for_match(value: &str) -> String {
    value
        .trim()
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn snippet_phrase_pattern(trigger: &str) -> String {
    trigger
        .split_whitespace()
        .map(regex::escape)
        .collect::<Vec<_>>()
        .join(r"\s+")
}

pub fn normalize_flow_fetch_url(url: String) -> Result<String> {
    let mut normalized = url
        .trim()
        .trim_matches(|ch: char| matches!(ch, '"' | '\'' | '<' | '>' | '(' | ')' | '[' | ']'))
        .trim_end_matches(|ch: char| matches!(ch, '.' | ',' | ';' | ':' | '!' | '?'))
        .to_string();

    if normalized.len() > 2048 {
        bail!("URL is too long for Flow Fetch");
    }

    if normalized.starts_with("http://") || normalized.starts_with("https://") {
        if normalized.contains(char::is_whitespace) {
            bail!("URL cannot contain whitespace");
        }
        return Ok(normalized);
    }

    normalized.clear();
    bail!("Flow Fetch only tracks http and https URLs")
}

fn derive_flow_fetch_label(url: &str) -> String {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let host = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(without_scheme)
        .trim_start_matches("www.");
    let path = without_scheme
        .split_once('/')
        .map(|(_, path)| path.split(['?', '#']).next().unwrap_or(""))
        .unwrap_or("");

    if path.is_empty() {
        return host.to_string();
    }

    let slug = path
        .rsplit('/')
        .find(|part| !part.trim().is_empty())
        .unwrap_or("")
        .replace(['-', '_'], " ");
    if slug.is_empty() {
        host.to_string()
    } else {
        format!("{host} - {slug}")
    }
}

#[derive(Debug, Clone, Default)]
struct InsightBucket {
    words: u32,
    transcriptions: u32,
    audio_duration_seconds: f32,
}

fn build_insights_summary(
    records: &[TranscriptionRecord],
    days: usize,
    today: NaiveDate,
) -> InsightsSummary {
    let days = days.clamp(7, 90);
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let window_start = today - chrono::Duration::days(days.saturating_sub(1) as i64);
    let mut daily_map: HashMap<NaiveDate, InsightBucket> = HashMap::new();
    let mut mode_map: HashMap<String, InsightBucket> = HashMap::new();
    let mut model_map: HashMap<String, InsightBucket> = HashMap::new();
    let mut active_days = HashSet::new();

    let mut total_words = 0u32;
    let mut words_today = 0u32;
    let mut words_this_week = 0u32;
    let mut total_audio_seconds = 0.0f32;
    let mut window_words = 0u32;
    let mut local_count = 0u32;
    let mut cleanup_count = 0u32;

    for record in records {
        let words = effective_word_count(record);
        let day = record.timestamp.date_naive();
        let audio_seconds = record.audio_duration_seconds.max(0.0);

        total_words = total_words.saturating_add(words);
        total_audio_seconds += audio_seconds;

        if day == today {
            words_today = words_today.saturating_add(words);
        }
        if day >= week_start && day <= today {
            words_this_week = words_this_week.saturating_add(words);
        }
        if day >= window_start && day <= today {
            window_words = window_words.saturating_add(words);
            let entry = daily_map.entry(day).or_default();
            entry.words = entry.words.saturating_add(words);
            entry.transcriptions = entry.transcriptions.saturating_add(1);
            entry.audio_duration_seconds += audio_seconds;
        }
        if words > 0 {
            active_days.insert(day);
        }
        if is_local_speech_model(&record.speech_model) {
            local_count = local_count.saturating_add(1);
        }
        if record.llm_cleaned {
            cleanup_count = cleanup_count.saturating_add(1);
        }

        let mode_label = record
            .mode_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .or(record.mode_id.as_deref())
            .unwrap_or("Default")
            .to_string();
        add_breakdown(&mut mode_map, mode_label, words, audio_seconds);

        let model_label = if record.speech_model.trim().is_empty() {
            "Unknown".to_string()
        } else {
            record.speech_model.clone()
        };
        add_breakdown(&mut model_map, model_label, words, audio_seconds);
    }

    let daily = (0..days)
        .rev()
        .map(|offset| {
            let day = today - chrono::Duration::days(offset as i64);
            let bucket = daily_map.remove(&day).unwrap_or_default();
            DailyInsight {
                date: day.to_string(),
                label: format!("{} {}", day.format("%b"), day.day()),
                words: bucket.words,
                transcriptions: bucket.transcriptions,
                audio_duration_seconds: bucket.audio_duration_seconds,
            }
        })
        .collect::<Vec<_>>();

    let (best_day_words, best_day_label) = daily
        .iter()
        .max_by_key(|day| day.words)
        .map(|day| (day.words, day.label.clone()))
        .unwrap_or((0, String::new()));

    let total_transcriptions = records.len() as u32;
    let average_words_per_minute = if total_audio_seconds > 0.0 {
        total_words as f32 / (total_audio_seconds / 60.0)
    } else {
        0.0
    };
    let average_words_per_day = window_words as f32 / days as f32;

    InsightsSummary {
        days,
        total_transcriptions,
        total_words,
        words_today,
        words_this_week,
        total_audio_seconds,
        average_words_per_minute,
        average_words_per_day,
        current_streak_days: compute_current_streak_days(&active_days, today),
        best_day_words,
        best_day_label,
        local_percent: percentage(local_count, total_transcriptions),
        cleanup_percent: percentage(cleanup_count, total_transcriptions),
        daily,
        top_modes: sorted_breakdowns(mode_map, 5),
        top_models: sorted_breakdowns(model_map, 5),
    }
}

fn effective_word_count(record: &TranscriptionRecord) -> u32 {
    if record.word_count > 0 {
        record.word_count
    } else {
        count_words(&record.text)
    }
}

fn add_breakdown(
    map: &mut HashMap<String, InsightBucket>,
    label: String,
    words: u32,
    audio_seconds: f32,
) {
    let entry = map.entry(label).or_default();
    entry.transcriptions = entry.transcriptions.saturating_add(1);
    entry.words = entry.words.saturating_add(words);
    entry.audio_duration_seconds += audio_seconds;
}

fn sorted_breakdowns(map: HashMap<String, InsightBucket>, limit: usize) -> Vec<InsightBreakdown> {
    let mut items = map
        .into_iter()
        .map(|(label, bucket)| InsightBreakdown {
            label,
            count: bucket.transcriptions,
            words: bucket.words,
            audio_duration_seconds: bucket.audio_duration_seconds,
        })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| {
        b.words
            .cmp(&a.words)
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.label.cmp(&b.label))
    });
    items.truncate(limit);
    items
}

fn compute_current_streak_days(active_days: &HashSet<NaiveDate>, today: NaiveDate) -> u32 {
    let mut streak = 0u32;
    loop {
        let day = today - chrono::Duration::days(streak as i64);
        if !active_days.contains(&day) {
            return streak;
        }
        streak = streak.saturating_add(1);
    }
}

fn percentage(count: u32, total: u32) -> f32 {
    if total == 0 {
        0.0
    } else {
        count as f32 * 100.0 / total as f32
    }
}

fn is_local_speech_model(model: &str) -> bool {
    let value = model.to_ascii_lowercase();
    !value.contains("cloud")
        && !value.contains("deepgram")
        && !value.contains("elevenlabs")
        && !value.contains("groq")
        && !value.contains("nim")
        && !value.contains("openai")
}

fn local_datetime_from_millis(
    timestamp_ms: i64,
    column_index: usize,
) -> rusqlite::Result<DateTime<Local>> {
    Local
        .timestamp_millis_opt(timestamp_ms)
        .single()
        .ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                column_index,
                Type::Integer,
                Box::new(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid timestamp stored in database: {timestamp_ms}"),
                )) as Box<dyn std::error::Error + Send + Sync + 'static>,
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record_on(
        day: NaiveDate,
        words: u32,
        model: &str,
        mode_name: Option<&str>,
    ) -> TranscriptionRecord {
        let timestamp = Local
            .with_ymd_and_hms(day.year(), day.month(), day.day(), 12, 0, 0)
            .single()
            .unwrap_or_else(Local::now);

        TranscriptionRecord {
            id: Uuid::new_v4().to_string(),
            timestamp,
            text: (0..words)
                .map(|index| format!("word{index}"))
                .collect::<Vec<_>>()
                .join(" "),
            raw_text: None,
            audio_path: String::new(),
            audio_available: false,
            status: TranscriptionStatus::Success,
            error_message: None,
            llm_cleaned: false,
            speech_model: model.to_string(),
            llm_model: None,
            word_count: words,
            audio_duration_seconds: 30.0,
            synced: false,
            mode_id: None,
            mode_name: mode_name.map(str::to_string),
        }
    }

    fn snippet(trigger: &str, expansion: &str) -> Snippet {
        let now = Local::now();
        Snippet {
            id: trigger.to_string(),
            trigger: trigger.to_string(),
            expansion: expansion.to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn snippets_expand_a_full_trigger_with_punctuation() {
        let text = apply_snippets_to_text(
            "my email address.",
            &[snippet("my email address", "me@example.com")],
        );
        assert_eq!(text, "me@example.com");
    }

    #[test]
    fn snippets_expand_inside_a_sentence_case_insensitively() {
        let text = apply_snippets_to_text(
            "Send it to My Email Address please",
            &[snippet("my email address", "me@example.com")],
        );
        assert_eq!(text, "Send it to me@example.com please");
    }

    #[test]
    fn scratchpad_title_uses_the_first_meaningful_words() {
        assert_eq!(
            derive_scratchpad_title("\n\nHello who are you and why are you here today"),
            "Hello who are you and why are you here today"
        );
    }

    #[test]
    fn flow_fetch_normalizes_simple_urls() {
        assert_eq!(
            normalize_flow_fetch_url(" https://example.com/path. ".to_string()).unwrap(),
            "https://example.com/path"
        );
    }

    #[test]
    fn insights_include_daily_history_and_current_streak() {
        let today = Local::now().date_naive();
        let records = vec![
            record_on(today, 10, "parakeet-tdt-0.6b-v3-int8", Some("Default")),
            record_on(
                today - chrono::Duration::days(1),
                20,
                "parakeet-tdt-0.6b-v3-int8",
                Some("Default"),
            ),
            record_on(
                today - chrono::Duration::days(3),
                30,
                "openai-whisper",
                Some("Meeting"),
            ),
        ];

        let summary = build_insights_summary(&records, 7, today);

        assert_eq!(summary.total_transcriptions, 3);
        assert_eq!(summary.total_words, 60);
        assert_eq!(summary.words_today, 10);
        assert_eq!(summary.current_streak_days, 2);
        assert_eq!(summary.best_day_words, 30);
        assert_eq!(summary.daily.len(), 7);
        assert_eq!(summary.top_modes[0].label, "Default");
        assert!(summary.local_percent > 60.0);
        assert!(summary.local_percent < 70.0);
    }
}
