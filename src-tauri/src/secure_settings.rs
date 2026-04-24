// MVP: store the Anthropic API key as a value in the `setting` table
// (which already lives inside the SQLCipher-encrypted DB file). A second
// encryption layer isn't worth the complexity for a single-user local
// app given SQLCipher's AES-256 already protects the file at rest.
use rusqlite::{Connection, OptionalExtension};

pub const ANTHROPIC_KEY_SETTING: &str = "anthropic_api_key";
pub const OLLAMA_URL_SETTING: &str = "ollama_url";
pub const OLLAMA_MODEL_SETTING: &str = "ollama_model";

pub fn get_setting(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM setting WHERE key = ?1",
        [key],
        |r| r.get::<_, Option<String>>(0),
    )
    .optional()
    .map(|o| o.flatten())
}

pub fn set_setting(conn: &Connection, key: &str, value: Option<&str>, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO setting (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        rusqlite::params![key, value, now],
    )?;
    Ok(())
}

pub fn get_anthropic_key(conn: &Connection) -> rusqlite::Result<Option<String>> {
    get_setting(conn, ANTHROPIC_KEY_SETTING)
}

pub fn set_anthropic_key(conn: &Connection, value: Option<&str>, now: i64) -> rusqlite::Result<()> {
    set_setting(conn, ANTHROPIC_KEY_SETTING, value, now)
}

pub fn get_ollama_url(conn: &Connection) -> rusqlite::Result<String> {
    Ok(get_setting(conn, OLLAMA_URL_SETTING)?.unwrap_or_else(|| "http://localhost:11434".to_string()))
}

pub fn set_ollama_url(conn: &Connection, value: &str, now: i64) -> rusqlite::Result<()> {
    set_setting(conn, OLLAMA_URL_SETTING, Some(value), now)
}

pub fn get_ollama_model(conn: &Connection) -> rusqlite::Result<Option<String>> {
    get_setting(conn, OLLAMA_MODEL_SETTING)
}

pub fn set_ollama_model(conn: &Connection, value: Option<&str>, now: i64) -> rusqlite::Result<()> {
    set_setting(conn, OLLAMA_MODEL_SETTING, value, now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations;

    fn mem() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        migrations::run(&c).unwrap();
        c
    }

    #[test]
    fn roundtrip_api_key() {
        let c = mem();
        assert!(get_anthropic_key(&c).unwrap().is_none());
        set_anthropic_key(&c, Some("sk-ant-test"), 1000).unwrap();
        assert_eq!(get_anthropic_key(&c).unwrap(), Some("sk-ant-test".into()));
        set_anthropic_key(&c, None, 2000).unwrap();
        assert!(get_anthropic_key(&c).unwrap().is_none());
    }
}
