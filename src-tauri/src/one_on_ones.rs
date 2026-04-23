use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneOnOne {
    pub id: i64,
    pub report_id: i64,
    pub occurred_at: i64,
    pub agenda_md: Option<String>,
    pub notes_md: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInput {
    pub report_id: i64,
    pub occurred_at: i64,
    pub agenda_md: Option<String>,
    pub notes_md: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInput {
    pub id: i64,
    pub occurred_at: Option<i64>,
    pub agenda_md: Option<Option<String>>,
    pub notes_md: Option<Option<String>>,
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<OneOnOne> {
    Ok(OneOnOne {
        id: r.get(0)?, report_id: r.get(1)?, occurred_at: r.get(2)?,
        agenda_md: r.get(3)?, notes_md: r.get(4)?, created_at: r.get(5)?,
    })
}

pub fn list_by_report(conn: &Connection, report_id: i64) -> rusqlite::Result<Vec<OneOnOne>> {
    let mut stmt = conn.prepare(
        "SELECT id, report_id, occurred_at, agenda_md, notes_md, created_at
         FROM one_on_one WHERE report_id = ?1 ORDER BY occurred_at DESC"
    )?;
    let rows = stmt.query_map([report_id], row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn latest_for_report(conn: &Connection, report_id: i64) -> rusqlite::Result<Option<OneOnOne>> {
    conn.query_row(
        "SELECT id, report_id, occurred_at, agenda_md, notes_md, created_at
         FROM one_on_one WHERE report_id = ?1 ORDER BY occurred_at DESC LIMIT 1",
        [report_id], row,
    ).optional()
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<OneOnOne>> {
    conn.query_row(
        "SELECT id, report_id, occurred_at, agenda_md, notes_md, created_at
         FROM one_on_one WHERE id = ?1",
        [id], row,
    ).optional()
}

pub fn create(conn: &Connection, i: NewInput, now: i64) -> rusqlite::Result<OneOnOne> {
    conn.execute(
        "INSERT INTO one_on_one (report_id, occurred_at, agenda_md, notes_md, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![i.report_id, i.occurred_at, i.agenda_md, i.notes_md, now],
    )?;
    let id = conn.last_insert_rowid();
    Ok(get(conn, id)?.expect("just inserted"))
}

pub fn update(conn: &Connection, i: UpdateInput) -> rusqlite::Result<OneOnOne> {
    let mut sets: Vec<&str> = Vec::new();
    let mut vals: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(v) = i.occurred_at { sets.push("occurred_at = ?"); vals.push(Box::new(v)); }
    if let Some(v) = i.agenda_md { sets.push("agenda_md = ?"); vals.push(Box::new(v)); }
    if let Some(v) = i.notes_md { sets.push("notes_md = ?"); vals.push(Box::new(v)); }
    if sets.is_empty() {
        return get(conn, i.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows);
    }
    let sql = format!("UPDATE one_on_one SET {} WHERE id = ?", sets.join(", "));
    vals.push(Box::new(i.id));
    let refs: Vec<&dyn rusqlite::ToSql> = vals.iter().map(|b| &**b).collect();
    conn.execute(&sql, refs.as_slice())?;
    get(conn, i.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute("DELETE FROM one_on_one WHERE id = ?1", [id])?;
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
    fn create_and_list_sorted_desc() {
        let (c, alice) = setup();
        create(&c, NewInput { report_id: alice, occurred_at: 1000, agenda_md: None, notes_md: None }, 0).unwrap();
        create(&c, NewInput { report_id: alice, occurred_at: 3000, agenda_md: Some("newer".into()), notes_md: None }, 0).unwrap();
        create(&c, NewInput { report_id: alice, occurred_at: 2000, agenda_md: None, notes_md: None }, 0).unwrap();

        let list = list_by_report(&c, alice).unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].occurred_at, 3000);
        assert_eq!(list[2].occurred_at, 1000);
    }

    #[test]
    fn latest_returns_most_recent() {
        let (c, alice) = setup();
        create(&c, NewInput { report_id: alice, occurred_at: 1000, agenda_md: None, notes_md: None }, 0).unwrap();
        create(&c, NewInput { report_id: alice, occurred_at: 2000, agenda_md: None, notes_md: None }, 0).unwrap();
        assert_eq!(latest_for_report(&c, alice).unwrap().unwrap().occurred_at, 2000);
    }

    #[test]
    fn latest_returns_none_when_no_meetings() {
        let (c, alice) = setup();
        assert!(latest_for_report(&c, alice).unwrap().is_none());
    }

    #[test]
    fn update_patches_fields() {
        let (c, alice) = setup();
        let o = create(&c, NewInput {
            report_id: alice, occurred_at: 1000, agenda_md: Some("a".into()), notes_md: None,
        }, 0).unwrap();

        update(&c, UpdateInput {
            id: o.id, occurred_at: None, agenda_md: None, notes_md: Some(Some("n".into())),
        }).unwrap();

        let after = get(&c, o.id).unwrap().unwrap();
        assert_eq!(after.agenda_md, Some("a".into()));
        assert_eq!(after.notes_md, Some("n".into()));
    }
}
