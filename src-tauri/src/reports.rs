use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub id: i64,
    pub name: String,
    pub role: Option<String>,
    pub start_date: Option<String>,
    pub one_on_one_cadence_days: i64,
    pub notes: Option<String>,
    pub active: bool,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReportInput {
    pub name: String,
    pub role: Option<String>,
    pub start_date: Option<String>,
    pub one_on_one_cadence_days: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReportInput {
    pub id: i64,
    pub name: Option<String>,
    pub role: Option<Option<String>>,
    pub start_date: Option<Option<String>>,
    pub one_on_one_cadence_days: Option<i64>,
    pub notes: Option<Option<String>>,
    pub active: Option<bool>,
}

fn row_to_report(row: &rusqlite::Row) -> rusqlite::Result<Report> {
    Ok(Report {
        id: row.get(0)?,
        name: row.get(1)?,
        role: row.get(2)?,
        start_date: row.get(3)?,
        one_on_one_cadence_days: row.get(4)?,
        notes: row.get(5)?,
        active: row.get::<_, i64>(6)? != 0,
        created_at: row.get(7)?,
    })
}

pub fn list(conn: &Connection, include_archived: bool) -> rusqlite::Result<Vec<Report>> {
    let sql = if include_archived {
        "SELECT id, name, role, start_date, one_on_one_cadence_days, notes, active, created_at \
         FROM report ORDER BY name COLLATE NOCASE"
    } else {
        "SELECT id, name, role, start_date, one_on_one_cadence_days, notes, active, created_at \
         FROM report WHERE active = 1 ORDER BY name COLLATE NOCASE"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], row_to_report)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<Report>> {
    conn.query_row(
        "SELECT id, name, role, start_date, one_on_one_cadence_days, notes, active, created_at \
         FROM report WHERE id = ?1",
        [id],
        row_to_report,
    ).optional()
}

pub fn create(conn: &Connection, input: NewReportInput, now: i64) -> rusqlite::Result<Report> {
    conn.execute(
        "INSERT INTO report (name, role, start_date, one_on_one_cadence_days, notes, active, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
        params![input.name, input.role, input.start_date, input.one_on_one_cadence_days, input.notes, now],
    )?;
    let id = conn.last_insert_rowid();
    Ok(get(conn, id)?.expect("row just inserted"))
}

pub fn update(conn: &Connection, input: UpdateReportInput) -> rusqlite::Result<Report> {
    // Build dynamic UPDATE from Option-set fields.
    let mut sets: Vec<&str> = Vec::new();
    let mut vals: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(v) = input.name { sets.push("name = ?"); vals.push(Box::new(v)); }
    if let Some(v) = input.role { sets.push("role = ?"); vals.push(Box::new(v)); }
    if let Some(v) = input.start_date { sets.push("start_date = ?"); vals.push(Box::new(v)); }
    if let Some(v) = input.one_on_one_cadence_days { sets.push("one_on_one_cadence_days = ?"); vals.push(Box::new(v)); }
    if let Some(v) = input.notes { sets.push("notes = ?"); vals.push(Box::new(v)); }
    if let Some(v) = input.active { sets.push("active = ?"); vals.push(Box::new(if v { 1i64 } else { 0 })); }

    if sets.is_empty() {
        return get(conn, input.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows);
    }

    let sql = format!("UPDATE report SET {} WHERE id = ?", sets.join(", "));
    vals.push(Box::new(input.id));
    let refs: Vec<&dyn rusqlite::ToSql> = vals.iter().map(|b| &**b).collect();
    conn.execute(&sql, refs.as_slice())?;

    get(conn, input.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn archive(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("UPDATE report SET active = 0 WHERE id = ?1", [id])?;
    Ok(())
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
    fn create_and_list() {
        let c = mem();
        create(&c, NewReportInput {
            name: "Alice".into(), role: Some("Senior BE".into()),
            start_date: Some("2026-01-01".into()), one_on_one_cadence_days: 14, notes: None,
        }, 1000).unwrap();
        create(&c, NewReportInput {
            name: "Bohdan".into(), role: Some("Mid FE".into()),
            start_date: None, one_on_one_cadence_days: 7, notes: None,
        }, 1001).unwrap();

        let reports = list(&c, false).unwrap();
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].name, "Alice"); // sorted by name
        assert_eq!(reports[1].name, "Bohdan");
    }

    #[test]
    fn archive_hides_from_list() {
        let c = mem();
        let a = create(&c, NewReportInput {
            name: "Alice".into(), role: None, start_date: None,
            one_on_one_cadence_days: 14, notes: None,
        }, 1000).unwrap();
        archive(&c, a.id).unwrap();

        assert_eq!(list(&c, false).unwrap().len(), 0);
        assert_eq!(list(&c, true).unwrap().len(), 1);
        assert!(!list(&c, true).unwrap()[0].active);
    }

    #[test]
    fn update_patches_only_specified_fields() {
        let c = mem();
        let r = create(&c, NewReportInput {
            name: "Alice".into(), role: Some("Senior BE".into()),
            start_date: Some("2026-01-01".into()), one_on_one_cadence_days: 14,
            notes: Some("original".into()),
        }, 1000).unwrap();

        update(&c, UpdateReportInput {
            id: r.id,
            name: None,
            role: None,
            start_date: None,
            one_on_one_cadence_days: Some(7),
            notes: Some(None),   // explicit null
            active: None,
        }).unwrap();

        let after = get(&c, r.id).unwrap().unwrap();
        assert_eq!(after.name, "Alice");                       // unchanged
        assert_eq!(after.role, Some("Senior BE".into()));      // unchanged
        assert_eq!(after.one_on_one_cadence_days, 7);          // changed
        assert_eq!(after.notes, None);                         // cleared
    }
}
