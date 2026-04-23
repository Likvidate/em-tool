use crate::{migrations, state::AppState, vault};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<vault::VaultError> for CommandError {
    fn from(e: vault::VaultError) -> Self {
        let code = match &e {
            vault::VaultError::InvalidPassword => "invalid_password",
            vault::VaultError::AlreadyExists(_) => "already_exists",
            vault::VaultError::NotFound(_) => "not_found",
            vault::VaultError::MissingSalt => "missing_salt",
            vault::VaultError::Sqlite(_) => "sqlite",
            vault::VaultError::Kdf(_) => "kdf",
            vault::VaultError::Io(_) => "io",
        };
        CommandError { code: code.to_string(), message: e.to_string() }
    }
}

impl From<rusqlite::Error> for CommandError {
    fn from(e: rusqlite::Error) -> Self {
        CommandError { code: "sqlite".to_string(), message: e.to_string() }
    }
}

fn now_secs() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tauri::command]
pub fn vault_exists(state: State<AppState>) -> bool {
    let guard = state.inner.lock().unwrap();
    vault::vault_exists(&guard.db_path)
}

#[tauri::command]
pub fn is_unlocked(state: State<AppState>) -> bool {
    let guard = state.inner.lock().unwrap();
    guard.connection.is_some()
}

#[tauri::command]
pub fn create_vault(state: State<AppState>, password: String) -> Result<(), CommandError> {
    let mut guard = state.inner.lock().unwrap();
    let conn = vault::create(&guard.db_path, &password)?;
    migrations::run(&conn)?;
    guard.connection = Some(conn);
    guard.last_activity_at = now_secs();
    Ok(())
}

#[tauri::command]
pub fn unlock_vault(state: State<AppState>, password: String) -> Result<(), CommandError> {
    let mut guard = state.inner.lock().unwrap();
    let conn = vault::open(&guard.db_path, &password)?;
    migrations::run(&conn)?;
    guard.connection = Some(conn);
    guard.last_activity_at = now_secs();
    Ok(())
}

#[tauri::command]
pub fn lock_vault(state: State<AppState>) {
    let mut guard = state.inner.lock().unwrap();
    guard.connection = None;
    guard.last_activity_at = 0;
}

#[tauri::command]
pub fn touch_activity(state: State<AppState>) {
    let mut guard = state.inner.lock().unwrap();
    guard.last_activity_at = now_secs();
}

use crate::{db, reports, week_ratings};

impl From<db::DbError> for CommandError {
    fn from(e: db::DbError) -> Self {
        let code = match &e {
            db::DbError::Locked => "locked",
            db::DbError::Sqlite(_) => "sqlite",
        };
        CommandError { code: code.to_string(), message: e.to_string() }
    }
}

// --- Reports ---

#[tauri::command]
pub fn list_reports(
    state: State<AppState>,
    include_archived: bool,
) -> Result<Vec<reports::Report>, CommandError> {
    db::with_conn(&state, |c| reports::list(c, include_archived)).map_err(Into::into)
}

#[tauri::command]
pub fn get_report(
    state: State<AppState>,
    id: i64,
) -> Result<Option<reports::Report>, CommandError> {
    db::with_conn(&state, |c| reports::get(c, id)).map_err(Into::into)
}

#[tauri::command]
pub fn create_report(
    state: State<AppState>,
    input: reports::NewReportInput,
) -> Result<reports::Report, CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| reports::create(c, input, now)).map_err(Into::into)
}

#[tauri::command]
pub fn update_report(
    state: State<AppState>,
    input: reports::UpdateReportInput,
) -> Result<reports::Report, CommandError> {
    db::with_conn(&state, |c| reports::update(c, input)).map_err(Into::into)
}

#[tauri::command]
pub fn archive_report(state: State<AppState>, id: i64) -> Result<(), CommandError> {
    db::with_conn(&state, |c| reports::archive(c, id)).map_err(Into::into)
}

// --- Week ratings ---

#[tauri::command]
pub fn list_week_ratings_by_week(
    state: State<AppState>,
    iso_week: String,
) -> Result<Vec<week_ratings::WeekRating>, CommandError> {
    db::with_conn(&state, |c| week_ratings::list_by_week(c, &iso_week)).map_err(Into::into)
}

#[tauri::command]
pub fn list_week_ratings_by_report(
    state: State<AppState>,
    report_id: i64,
) -> Result<Vec<week_ratings::WeekRating>, CommandError> {
    db::with_conn(&state, |c| week_ratings::list_by_report(c, report_id)).map_err(Into::into)
}

#[tauri::command]
pub fn list_week_ratings_team_overall(
    state: State<AppState>,
) -> Result<Vec<week_ratings::WeekRating>, CommandError> {
    db::with_conn(&state, |c| week_ratings::list_team_overall(c)).map_err(Into::into)
}

#[tauri::command]
pub fn list_week_ratings_in_range(
    state: State<AppState>,
    from_iso_week: String,
    to_iso_week: String,
) -> Result<Vec<week_ratings::WeekRating>, CommandError> {
    db::with_conn(&state, |c| week_ratings::list_in_range(c, &from_iso_week, &to_iso_week))
        .map_err(Into::into)
}

#[tauri::command]
pub fn upsert_week_rating(
    state: State<AppState>,
    input: week_ratings::UpsertInput,
) -> Result<week_ratings::WeekRating, CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| week_ratings::upsert(c, input, now)).map_err(Into::into)
}

#[tauri::command]
pub fn delete_week_rating(
    state: State<AppState>,
    report_id: Option<i64>,
    iso_week: String,
) -> Result<(), CommandError> {
    db::with_conn(&state, |c| week_ratings::delete(c, report_id, &iso_week)).map_err(Into::into)
}
