use rusqlite::Connection;
use std::path::Path;

use crate::kdf::{derive_key, random_salt, SALT_LEN};

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("kdf: {0}")]
    Kdf(#[from] crate::kdf::KdfError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid password or corrupted vault")]
    InvalidPassword,
    #[error("vault already exists at {0}")]
    AlreadyExists(String),
    #[error("vault not found at {0}")]
    NotFound(String),
    #[error("missing salt — vault was not created by this app")]
    MissingSalt,
}

/// Set the SQLCipher key for this connection using the raw key bytes
/// encoded as `x'…'` — SQLCipher's "raw hex key" form — which skips its
/// own PBKDF2-over-the-passphrase and uses our Argon2-derived key directly.
fn apply_key(conn: &Connection, key: &[u8; 32]) -> rusqlite::Result<()> {
    let hex_key = hex::encode(key);
    conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", hex_key))?;
    Ok(())
}

pub fn salt_path_for(db_path: &Path) -> std::path::PathBuf {
    db_path.with_extension("salt")
}

/// True if the file exists at path (independent of whether it's a valid vault).
pub fn vault_exists(path: &Path) -> bool {
    path.exists()
}

/// Create a new vault at `path`, encrypted with a key derived from `password`.
/// Writes the salt to a sidecar file at `path.with_extension("salt")`.
/// Returns an opened connection.
pub fn create(path: &Path, password: &str) -> Result<Connection, VaultError> {
    if path.exists() {
        return Err(VaultError::AlreadyExists(path.display().to_string()));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let salt = random_salt();
    let key = derive_key(password, &salt)?;

    let conn = Connection::open(path)?;
    apply_key(&conn, &key)?;
    // Force SQLCipher to actually encrypt the empty DB by doing a write.
    conn.execute_batch("CREATE TABLE IF NOT EXISTS _probe (v INTEGER); DROP TABLE _probe;")?;
    std::fs::write(salt_path_for(path), salt)?;
    Ok(conn)
}

/// Open an existing vault by deriving the key from `password`.
/// Returns `InvalidPassword` if decryption fails.
pub fn open(path: &Path, password: &str) -> Result<Connection, VaultError> {
    if !path.exists() {
        return Err(VaultError::NotFound(path.display().to_string()));
    }

    let salt_path = salt_path_for(path);
    let salt_bytes = std::fs::read(&salt_path).map_err(|_| VaultError::MissingSalt)?;
    if salt_bytes.len() != SALT_LEN {
        return Err(VaultError::MissingSalt);
    }
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&salt_bytes);

    let key = derive_key(password, &salt)?;
    let conn = Connection::open(path)?;
    apply_key(&conn, &key)?;
    // Validate key by forcing a read from an encrypted page.
    match conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0)) {
        Ok(_) => Ok(conn),
        Err(rusqlite::Error::SqliteFailure(_, _)) => Err(VaultError::InvalidPassword),
        Err(e) => Err(VaultError::Sqlite(e)),
    }
}

/// Rotate the vault's encryption key by re-deriving from `new_password`
/// and writing a fresh salt to the sidecar.
pub fn change_password(conn: &Connection, new_password: &str, salt_path: &Path) -> Result<(), VaultError> {
    let new_salt = random_salt();
    let new_key = derive_key(new_password, &new_salt)?;
    let hex_key = hex::encode(new_key);
    conn.execute_batch(&format!("PRAGMA rekey = \"x'{}'\";", hex_key))?;
    std::fs::write(salt_path, new_salt)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_then_open_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("vault.db");

        {
            let conn = create(&path, "hunter2").unwrap();
            conn.execute_batch("CREATE TABLE t (x INTEGER); INSERT INTO t VALUES (42);").unwrap();
        }

        let conn = open(&path, "hunter2").unwrap();
        let v: i64 = conn.query_row("SELECT x FROM t", [], |r| r.get(0)).unwrap();
        assert_eq!(v, 42);
    }

    #[test]
    fn wrong_password_fails() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("vault.db");

        {
            let _ = create(&path, "hunter2").unwrap();
        }

        let err = open(&path, "wrong-password").unwrap_err();
        assert!(matches!(err, VaultError::InvalidPassword));
    }

    #[test]
    fn create_rejects_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let _ = create(&path, "pw").unwrap();

        let err = create(&path, "pw").unwrap_err();
        assert!(matches!(err, VaultError::AlreadyExists(_)));
    }
}
