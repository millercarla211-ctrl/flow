use crate::{storage, AppState};

#[tauri::command]
pub(crate) fn get_insights(
    state: tauri::State<AppState>,
    days: Option<usize>,
) -> Result<storage::InsightsSummary, String> {
    state
        .storage()
        .get_insights(days.unwrap_or(30))
        .map_err(|err| format!("Failed to load insights: {err}"))
}
