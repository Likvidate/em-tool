use crate::state::AppState;
use rusqlite::Connection;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("vault is locked")]
    Locked,
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Acquire the vault connection (fails if locked) and run `f` against it.
/// Held for the duration of `f` — keep the closure quick.
pub fn with_conn<T>(
    state: &AppState,
    f: impl FnOnce(&Connection) -> rusqlite::Result<T>,
) -> Result<T, DbError> {
    let guard = state.inner.lock().unwrap();
    let conn = guard.connection.as_ref().ok_or(DbError::Locked)?;
    Ok(f(conn)?)
}
