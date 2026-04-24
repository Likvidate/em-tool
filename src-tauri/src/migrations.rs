use rusqlite::Connection;

pub const CURRENT_SCHEMA_VERSION: i64 = 3;

pub fn run(conn: &Connection) -> rusqlite::Result<()> {
    let version = current_version(conn)?;
    if version < 1 {
        apply_v1(conn)?;
    }
    if version < 2 {
        apply_v2(conn)?;
    }
    if version < 3 {
        apply_v3(conn)?;
    }
    Ok(())
}

fn current_version(conn: &Connection) -> rusqlite::Result<i64> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_meta (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            schema_version INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            last_unlocked_at INTEGER
        );
        INSERT OR IGNORE INTO app_meta (id, schema_version) VALUES (1, 0);",
    )?;
    conn.query_row(
        "SELECT schema_version FROM app_meta WHERE id = 1",
        [],
        |r| r.get(0),
    )
}

fn apply_v1(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(r#"
        CREATE TABLE report (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            role TEXT,
            start_date TEXT,
            one_on_one_cadence_days INTEGER NOT NULL DEFAULT 14,
            notes TEXT,
            active INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE week_rating (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_id INTEGER REFERENCES report(id) ON DELETE CASCADE,
            iso_week TEXT NOT NULL,
            color TEXT NOT NULL CHECK (color IN ('red','yellow','grey','green','blue')),
            notes TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE UNIQUE INDEX idx_week_rating_unique
            ON week_rating(COALESCE(report_id, -1), iso_week);
        CREATE INDEX idx_week_rating_report ON week_rating(report_id);
        CREATE INDEX idx_week_rating_week ON week_rating(iso_week);

        CREATE TABLE one_on_one (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_id INTEGER NOT NULL REFERENCES report(id) ON DELETE CASCADE,
            occurred_at INTEGER NOT NULL,
            agenda_md TEXT,
            notes_md TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX idx_one_on_one_report ON one_on_one(report_id);

        CREATE TABLE action_item (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            one_on_one_id INTEGER REFERENCES one_on_one(id) ON DELETE SET NULL,
            report_id INTEGER NOT NULL REFERENCES report(id) ON DELETE CASCADE,
            text TEXT NOT NULL,
            owner TEXT NOT NULL CHECK (owner IN ('me','them')),
            due_date TEXT,
            completed_at INTEGER,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX idx_action_item_report ON action_item(report_id);
        CREATE INDEX idx_action_item_open ON action_item(report_id) WHERE completed_at IS NULL;

        CREATE TABLE performance_review (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_id INTEGER NOT NULL REFERENCES report(id) ON DELETE CASCADE,
            period TEXT NOT NULL,
            rating TEXT,
            strengths_md TEXT,
            dev_areas_md TEXT,
            goals_md TEXT,
            occurred_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX idx_performance_review_report ON performance_review(report_id);

        CREATE TABLE generated_plan (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL CHECK (kind IN ('one_on_one','review')),
            target_report_id INTEGER NOT NULL REFERENCES report(id) ON DELETE CASCADE,
            window_spec TEXT NOT NULL,
            source TEXT NOT NULL CHECK (source IN ('claude','template')),
            prompt_md TEXT,
            output_md TEXT NOT NULL,
            saved_to_meeting_id INTEGER REFERENCES one_on_one(id) ON DELETE SET NULL,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE setting (
            key TEXT PRIMARY KEY,
            value TEXT,
            updated_at INTEGER NOT NULL
        );

        UPDATE app_meta SET schema_version = 1 WHERE id = 1;
    "#)
}

fn apply_v2(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(r#"
        ALTER TABLE performance_review ADD COLUMN notes_md TEXT;
        UPDATE app_meta SET schema_version = 2 WHERE id = 1;
    "#)
}

fn apply_v3(conn: &Connection) -> rusqlite::Result<()> {
    // Widen generated_plan.source CHECK to include 'ollama'. SQLite can't
    // drop a CHECK constraint in place, so copy-rename.
    conn.execute_batch(r#"
        CREATE TABLE generated_plan_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL CHECK (kind IN ('one_on_one','review')),
            target_report_id INTEGER NOT NULL REFERENCES report(id) ON DELETE CASCADE,
            window_spec TEXT NOT NULL,
            source TEXT NOT NULL CHECK (source IN ('claude','template','ollama')),
            prompt_md TEXT,
            output_md TEXT NOT NULL,
            saved_to_meeting_id INTEGER REFERENCES one_on_one(id) ON DELETE SET NULL,
            created_at INTEGER NOT NULL
        );
        INSERT INTO generated_plan_new SELECT * FROM generated_plan;
        DROP TABLE generated_plan;
        ALTER TABLE generated_plan_new RENAME TO generated_plan;
        UPDATE app_meta SET schema_version = 3 WHERE id = 1;
    "#)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn open_in_memory() -> Connection {
        // In-memory unencrypted connection — enough to test schema migrations
        // because SQLCipher's PRAGMA-level encryption is transparent above
        // the schema level.
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn first_run_reaches_current_version() {
        let c = open_in_memory();
        run(&c).unwrap();
        let v: i64 = c.query_row(
            "SELECT schema_version FROM app_meta WHERE id = 1",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(v, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn run_is_idempotent() {
        let c = open_in_memory();
        run(&c).unwrap();
        run(&c).unwrap();
        run(&c).unwrap();
        let v: i64 = c.query_row(
            "SELECT schema_version FROM app_meta WHERE id = 1",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(v, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn all_tables_exist_after_v1() {
        let c = open_in_memory();
        run(&c).unwrap();
        for table in [
            "report", "week_rating", "one_on_one", "action_item",
            "performance_review", "generated_plan", "setting", "app_meta",
        ] {
            let count: i64 = c
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "table {} missing", table);
        }
    }

    #[test]
    fn color_check_constraint_rejects_invalid() {
        let c = open_in_memory();
        run(&c).unwrap();
        c.execute(
            "INSERT INTO report (name, created_at) VALUES ('A', 0)",
            [],
        ).unwrap();
        let err = c.execute(
            "INSERT INTO week_rating (report_id, iso_week, color, created_at, updated_at) \
             VALUES (1, '2026-W17', 'purple', 0, 0)",
            [],
        );
        assert!(err.is_err());
    }

    #[test]
    fn unique_week_rating_per_report_per_week() {
        let c = open_in_memory();
        run(&c).unwrap();
        c.execute("INSERT INTO report (name, created_at) VALUES ('A', 0)", []).unwrap();
        c.execute(
            "INSERT INTO week_rating (report_id, iso_week, color, created_at, updated_at) \
             VALUES (1, '2026-W17', 'green', 0, 0)",
            [],
        ).unwrap();
        let err = c.execute(
            "INSERT INTO week_rating (report_id, iso_week, color, created_at, updated_at) \
             VALUES (1, '2026-W17', 'red', 0, 0)",
            [],
        );
        assert!(err.is_err());
    }

    #[test]
    fn v2_adds_notes_md_column() {
        let c = open_in_memory();
        run(&c).unwrap();
        // Column should exist and accept inserts
        c.execute("INSERT INTO report (name, created_at) VALUES ('A', 0)", []).unwrap();
        c.execute(
            "INSERT INTO performance_review (report_id, period, occurred_at, created_at, notes_md) \
             VALUES (1, 'Q1 2026', 0, 0, 'post-review reflection')",
            [],
        ).unwrap();
        let notes: String = c.query_row(
            "SELECT notes_md FROM performance_review WHERE id = 1",
            [], |r| r.get(0),
        ).unwrap();
        assert_eq!(notes, "post-review reflection");
    }

    #[test]
    fn v2_is_idempotent_on_existing_v1_db() {
        let c = open_in_memory();
        // Simulate a v1-only DB by ensuring app_meta exists then running apply_v1 directly
        let _ = current_version(&c).unwrap();
        apply_v1(&c).unwrap();
        // Now run() should notice version=1 and apply v2
        run(&c).unwrap();
        let v: i64 = c.query_row("SELECT schema_version FROM app_meta WHERE id = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(v, 3);
    }

    #[test]
    fn v3_allows_ollama_source() {
        let c = open_in_memory();
        run(&c).unwrap();
        c.execute("INSERT INTO report (name, created_at) VALUES ('A', 0)", []).unwrap();
        c.execute(
            "INSERT INTO generated_plan (kind, target_report_id, window_spec, source, output_md, created_at) \
             VALUES ('one_on_one', 1, '{}', 'ollama', 'out', 0)",
            [],
        ).unwrap();
    }
}
