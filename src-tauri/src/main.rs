use journal::commands;
use journal::db::DbConnection;
use journal::models::{self, AppError};
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use std::sync::MutexGuard;
use tauri::Manager;
use tauri::State;

/// Application state holding the database connection
struct AppState {
    db: Mutex<DbConnection>,
}

fn lock_db<'a>(state: &'a State<'_, AppState>) -> Result<MutexGuard<'a, DbConnection>, AppError> {
    state
        .db
        .lock()
        .map_err(|e| AppError::StateLock(e.to_string()))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CreateEntryPayload {
    title: String,
    body: String,
    mood: Option<i32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UpdateEntryPayload {
    id: String,
    title: String,
    body: String,
    mood: Option<i32>,
    created_at: Option<i64>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CreateTagPayload {
    name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RenameTagPayload {
    id: String,
    name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SetEntryPinnedPayload {
    id: String,
    pinned: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SearchEntriesPayload {
    query: String,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(serde::Serialize)]
struct OkResponse {
    ok: bool,
}

#[tauri::command]
fn create_entry(
    payload: CreateEntryPayload,
    state: State<'_, AppState>,
) -> Result<models::Entry, AppError> {
    let db_guard = lock_db(&state)?;

    commands::entries::create_entry(&db_guard, payload.title, payload.body, payload.mood)
}

#[tauri::command]
fn get_entries(state: State<'_, AppState>) -> Result<Vec<models::Entry>, AppError> {
    let db_guard = lock_db(&state)?;

    commands::entries::get_entries(&db_guard)
}

#[tauri::command]
fn get_entry(id: String, state: State<'_, AppState>) -> Result<models::Entry, AppError> {
    let db_guard = lock_db(&state)?;

    match commands::entries::get_entry(&db_guard, id)? {
        Some(entry) => Ok(entry),
        None => Err(AppError::NotFound("Entry not found".to_string())),
    }
}

#[tauri::command]
fn update_entry(
    payload: UpdateEntryPayload,
    state: State<'_, AppState>,
) -> Result<models::Entry, AppError> {
    let db_guard = lock_db(&state)?;

    commands::entries::update_entry(
        &db_guard,
        payload.id,
        payload.title,
        payload.body,
        payload.mood,
        payload.created_at,
    )
}

#[tauri::command]
fn delete_entry(id: String, state: State<'_, AppState>) -> Result<OkResponse, AppError> {
    let db_guard = lock_db(&state)?;

    commands::entries::delete_entry(&db_guard, id)?;

    Ok(OkResponse { ok: true })
}

#[tauri::command]
fn set_entry_pinned(
    payload: SetEntryPinnedPayload,
    state: State<'_, AppState>,
) -> Result<models::Entry, AppError> {
    let db_guard = lock_db(&state)?;

    commands::entries::set_pinned(&db_guard, payload.id, payload.pinned)
}

#[tauri::command]
fn create_tag(
    payload: CreateTagPayload,
    state: State<'_, AppState>,
) -> Result<models::Tag, AppError> {
    let db_guard = lock_db(&state)?;

    commands::tags::create_tag(&db_guard, payload.name)
}

#[tauri::command]
fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<models::Tag>, AppError> {
    let db_guard = lock_db(&state)?;

    commands::tags::get_all_tags(&db_guard)
}

#[tauri::command]
fn list_tags(state: State<'_, AppState>) -> Result<Vec<models::Tag>, AppError> {
    let db_guard = lock_db(&state)?;

    commands::tags::get_all_tags(&db_guard)
}

#[tauri::command]
fn rename_tag(
    payload: RenameTagPayload,
    state: State<'_, AppState>,
) -> Result<models::Tag, AppError> {
    let db_guard = lock_db(&state)?;

    commands::tags::rename_tag(&db_guard, payload.id, payload.name)
}

#[tauri::command]
fn delete_tag(id: String, state: State<'_, AppState>) -> Result<OkResponse, AppError> {
    let db_guard = lock_db(&state)?;

    commands::tags::delete_tag(&db_guard, id)?;

    Ok(OkResponse { ok: true })
}

#[tauri::command]
fn get_tags_for_entry(
    entry_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<models::Tag>, AppError> {
    let db_guard = lock_db(&state)?;

    commands::tags::get_tags_for_entry(&db_guard, entry_id)
}

#[tauri::command]
fn assign_tag_to_entry(
    entry_id: String,
    tag_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let db_guard = lock_db(&state)?;

    commands::tags::assign_tag_to_entry(&db_guard, entry_id, tag_id)
}

#[tauri::command]
fn remove_tag_from_entry(
    entry_id: String,
    tag_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let db_guard = lock_db(&state)?;

    commands::tags::remove_tag_from_entry(&db_guard, entry_id, tag_id)
}

#[tauri::command]
fn get_all_entry_tags(
    state: State<'_, AppState>,
) -> Result<HashMap<String, Vec<models::Tag>>, AppError> {
    let db_guard = lock_db(&state)?;

    commands::tags::get_all_entry_tags(&db_guard)
}

#[tauri::command]
fn search_entries(
    payload: SearchEntriesPayload,
    state: State<'_, AppState>,
) -> Result<commands::search::SearchEntriesResponse, AppError> {
    let db_guard = lock_db(&state)?;

    commands::search::search_entries(&db_guard, payload.query, payload.limit, payload.offset)
}

#[tauri::command]
fn get_summary_stats(state: State<'_, AppState>) -> Result<models::SummaryStats, AppError> {
    let db_guard = lock_db(&state)?;
    commands::stats::get_summary_stats(&db_guard)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

            fs::create_dir_all(&app_data_dir)
                .map_err(|e| format!("Failed to create app data directory: {}", e))?;

            let db_path = app_data_dir.join("journal.db");
            let db_path_string = db_path.to_string_lossy().to_string();
            let db = DbConnection::new(&db_path_string)
                .map_err(|e| format!("Failed to initialize database {}: {}", db_path_string, e))?;

            app.manage(AppState { db: Mutex::new(db) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_entry,
            get_entries,
            get_entry,
            update_entry,
            delete_entry,
            set_entry_pinned,
            create_tag,
            get_all_tags,
            list_tags,
            rename_tag,
            delete_tag,
            get_tags_for_entry,
            assign_tag_to_entry,
            remove_tag_from_entry,
            get_all_entry_tags,
            search_entries,
            get_summary_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
