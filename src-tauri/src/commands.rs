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
