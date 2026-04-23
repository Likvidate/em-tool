use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionItem {
    pub id: i64,
    pub one_on_one_id: Option<i64>,
    pub report_id: i64,
    pub text: String,
    pub owner: String,          // "me" | "them"
    pub due_date: Option<String>,
    pub completed_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInput {
    pub one_on_one_id: Option<i64>,
    pub report_id: i64,
    pub text: String,
    pub owner: String,
    pub due_date: Option<String>,
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<ActionItem> {
    Ok(ActionItem {
        id: r.get(0)?,
        one_on_one_id: r.get(1)?,
        report_id: r.get(2)?,
        text: r.get(3)?,
        owner: r.get(4)?,
        due_date: r.get(5)?,
        completed_at: r.get(6)?,
        created_at: r.get(7)?,
    })
}

const SELECT: &str =
    "SELECT id, one_on_one_id, report_id, text, owner, due_date, completed_at, created_at \
     FROM action_item";

pub fn list_by_meeting(conn: &Connection, one_on_one_id: i64) -> rusqlite::Result<Vec<ActionItem>> {
    let sql = format!("{} WHERE one_on_one_id = ?1 ORDER BY created_at ASC", SELECT);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([one_on_one_id], row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_by_report(conn: &Connection, report_id: i64) -> rusqlite::Result<Vec<ActionItem>> {
    let sql = format!("{} WHERE report_id = ?1 ORDER BY created_at DESC", SELECT);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([report_id], row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_open_for_report(conn: &Connection, report_id: i64) -> rusqlite::Result<Vec<ActionItem>> {
    let sql = format!("{} WHERE report_id = ?1 AND completed_at IS NULL ORDER BY due_date ASC, created_at ASC", SELECT);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([report_id], row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn create(conn: &Connection, i: NewInput, now: i64) -> rusqlite::Result<ActionItem> {
    conn.execute(
        "INSERT INTO action_item (one_on_one_id, report_id, text, owner, due_date, completed_at, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6)",
        params![i.one_on_one_id, i.report_id, i.text, i.owner, i.due_date, now],
    )?;
    let id = conn.last_insert_rowid();
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<ActionItem>> {
    let sql = format!("{} WHERE id = ?1", SELECT);
    conn.query_row(&sql, [id], row).optional()
}

pub fn toggle_complete(conn: &Connection, id: i64, now: i64) -> rusqlite::Result<ActionItem> {
    let cur = get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    if cur.completed_at.is_none() {
        conn.execute("UPDATE action_item SET completed_at = ?1 WHERE id = ?2", params![now, id])?;
    } else {
        conn.execute("UPDATE action_item SET completed_at = NULL WHERE id = ?1", [id])?;
    }
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute("DELETE FROM action_item WHERE id = ?1", [id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{migrations, reports};

    fn setup() -> (Connection, i64) {
        let c = Connection::open_in_memory().unwrap();
        migrations::run(&c).unwrap();
        let alice = reports::create(&c, reports::NewReportInput {
            name: "Alice".into(), role: None, start_date: None,
            one_on_one_cadence_days: 14, notes: None,
        }, 1000).unwrap();
        (c, alice.id)
    }

    #[test]
    fn create_defaults_to_open() {
        let (c, alice) = setup();
        let a = create(&c, NewInput {
            one_on_one_id: None, report_id: alice,
            text: "RFC due".into(), owner: "them".into(), due_date: Some("2026-05-01".into()),
        }, 1000).unwrap();
        assert!(a.completed_at.is_none());
    }

    #[test]
    fn toggle_completes_and_uncompletes() {
        let (c, alice) = setup();
        let a = create(&c, NewInput {
            one_on_one_id: None, report_id: alice,
            text: "t".into(), owner: "me".into(), due_date: None,
        }, 1000).unwrap();

        let done = toggle_complete(&c, a.id, 2000).unwrap();
        assert_eq!(done.completed_at, Some(2000));

        let reopened = toggle_complete(&c, a.id, 3000).unwrap();
        assert!(reopened.completed_at.is_none());
    }

    #[test]
    fn list_open_excludes_completed() {
        let (c, alice) = setup();
        let a = create(&c, NewInput {
            one_on_one_id: None, report_id: alice,
            text: "open".into(), owner: "me".into(), due_date: None,
        }, 1000).unwrap();
        let b = create(&c, NewInput {
            one_on_one_id: None, report_id: alice,
            text: "done".into(), owner: "me".into(), due_date: None,
        }, 1000).unwrap();
        toggle_complete(&c, b.id, 2000).unwrap();

        let open = list_open_for_report(&c, alice).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, a.id);
    }
}
