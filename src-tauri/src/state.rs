use rusqlite::Connection;
use std::sync::Mutex;
use std::path::PathBuf;

pub struct AppState {
    pub inner: Mutex<VaultState>,
}

pub struct VaultState {
    pub connection: Option<Connection>,
    pub db_path: PathBuf,
    pub last_activity_at: i64,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            inner: Mutex::new(VaultState {
                connection: None,
                db_path,
                last_activity_at: 0,
            }),
        }
    }
}

pub fn default_db_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").expect("HOME not set");
                PathBuf::from(home).join(".local/share")
            });
        base.join("em-tool").join("vault.db")
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").expect("APPDATA not set");
        PathBuf::from(appdata).join("em-tool").join("vault.db")
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        PathBuf::from("./vault.db")
    }
}
