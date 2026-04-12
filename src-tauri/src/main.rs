use journal::commands;
use journal::db::DbConnection;
use journal::models;
use std::sync::Mutex;
use tauri::State;

/// Application state holding the database connection
struct AppState {
    db: Mutex<DbConnection>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CreateEntryPayload {
    body: String,
    mood: Option<i32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UpdateEntryPayload {
    id: String,
    body: String,
    mood: Option<i32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CreateTagPayload {
    name: String,
}

#[tauri::command]
fn create_entry(
    payload: CreateEntryPayload,
    state: State<'_, AppState>,
) -> Result<models::Entry, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    commands::entries::create_entry(&db_guard, payload.body, payload.mood)
        .map_err(|e| format!("Failed to create entry: {}", e))
}

#[tauri::command]
fn get_entries(state: State<'_, AppState>) -> Result<Vec<models::Entry>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    commands::entries::get_entries(&db_guard).map_err(|e| format!("Failed to get entries: {}", e))
}

#[tauri::command]
fn update_entry(
    payload: UpdateEntryPayload,
    state: State<'_, AppState>,
) -> Result<models::Entry, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    commands::entries::update_entry(&db_guard, payload.id, payload.body, payload.mood)
        .map_err(|e| format!("Failed to update entry: {}", e))
}

#[tauri::command]
fn delete_entry(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    commands::entries::delete_entry(&db_guard, id).map_err(|e| format!("Failed to delete entry: {}", e))
}

#[tauri::command]
fn create_tag(
    payload: CreateTagPayload,
    state: State<'_, AppState>,
) -> Result<models::Tag, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    commands::tags::create_tag(&db_guard, payload.name).map_err(|e| format!("Failed to create tag: {}", e))
}

#[tauri::command]
fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<models::Tag>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    commands::tags::get_all_tags(&db_guard).map_err(|e| format!("Failed to get tags: {}", e))
}

#[tauri::command]
fn get_tags_for_entry(
    entry_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<models::Tag>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    commands::tags::get_tags_for_entry(&db_guard, entry_id)
        .map_err(|e| format!("Failed to get tags: {}", e))
}

#[tauri::command]
fn assign_tag_to_entry(
    entry_id: String,
    tag_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    commands::tags::assign_tag_to_entry(&db_guard, entry_id, tag_id)
        .map_err(|e| format!("Failed to assign tag: {}", e))
}

#[tauri::command]
fn remove_tag_from_entry(
    entry_id: String,
    tag_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    commands::tags::remove_tag_from_entry(&db_guard, entry_id, tag_id)
        .map_err(|e| format!("Failed to remove tag: {}", e))
}

fn main() {
    let db = DbConnection::new("journal.db").expect("Failed to initialize database");
    let app_state = AppState { db: Mutex::new(db) };

    tauri::Builder::default()
        .manage(app_state)
        .setup(|_| Ok(()))
        .invoke_handler(tauri::generate_handler![
            create_entry,
            get_entries,
            update_entry,
            delete_entry,
            create_tag,
            get_all_tags,
            get_tags_for_entry,
            assign_tag_to_entry,
            remove_tag_from_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
