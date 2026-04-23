use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceReview {
    pub id: i64,
    pub report_id: i64,
    pub period: String,
    pub rating: Option<String>,
    pub strengths_md: Option<String>,
    pub dev_areas_md: Option<String>,
    pub goals_md: Option<String>,
    pub occurred_at: i64,
    pub created_at: i64,
    pub notes_md: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInput {
    pub report_id: i64,
    pub period: String,
    pub rating: Option<String>,
    pub strengths_md: Option<String>,
    pub dev_areas_md: Option<String>,
    pub goals_md: Option<String>,
    pub notes_md: Option<String>,
    pub occurred_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInput {
    pub id: i64,
    pub period: Option<String>,
    pub rating: Option<Option<String>>,
    pub strengths_md: Option<Option<String>>,
    pub dev_areas_md: Option<Option<String>>,
    pub goals_md: Option<Option<String>>,
    pub notes_md: Option<Option<String>>,
    pub occurred_at: Option<i64>,
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<PerformanceReview> {
    Ok(PerformanceReview {
        id: r.get(0)?, report_id: r.get(1)?, period: r.get(2)?, rating: r.get(3)?,
        strengths_md: r.get(4)?, dev_areas_md: r.get(5)?, goals_md: r.get(6)?,
        occurred_at: r.get(7)?, created_at: r.get(8)?, notes_md: r.get(9)?,
    })
}

const SELECT: &str =
    "SELECT id, report_id, period, rating, strengths_md, dev_areas_md, goals_md, occurred_at, created_at, notes_md \
     FROM performance_review";

pub fn list_by_report(conn: &Connection, report_id: i64) -> rusqlite::Result<Vec<PerformanceReview>> {
    let sql = format!("{} WHERE report_id = ?1 ORDER BY occurred_at DESC", SELECT);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([report_id], row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn latest_for_report(conn: &Connection, report_id: i64) -> rusqlite::Result<Option<PerformanceReview>> {
    let sql = format!("{} WHERE report_id = ?1 ORDER BY occurred_at DESC LIMIT 1", SELECT);
    conn.query_row(&sql, [report_id], row).optional()
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<PerformanceReview>> {
    let sql = format!("{} WHERE id = ?1", SELECT);
    conn.query_row(&sql, [id], row).optional()
}

pub fn create(conn: &Connection, i: NewInput, now: i64) -> rusqlite::Result<PerformanceReview> {
    conn.execute(
        "INSERT INTO performance_review (report_id, period, rating, strengths_md, dev_areas_md, goals_md, occurred_at, created_at, notes_md) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![i.report_id, i.period, i.rating, i.strengths_md, i.dev_areas_md, i.goals_md, i.occurred_at, now, i.notes_md],
    )?;
    let id = conn.last_insert_rowid();
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn update(conn: &Connection, i: UpdateInput) -> rusqlite::Result<PerformanceReview> {
    let mut sets: Vec<&str> = Vec::new();
    let mut vals: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(v) = i.period { sets.push("period = ?"); vals.push(Box::new(v)); }
    if let Some(v) = i.rating { sets.push("rating = ?"); vals.push(Box::new(v)); }
    if let Some(v) = i.strengths_md { sets.push("strengths_md = ?"); vals.push(Box::new(v)); }
    if let Some(v) = i.dev_areas_md { sets.push("dev_areas_md = ?"); vals.push(Box::new(v)); }
    if let Some(v) = i.goals_md { sets.push("goals_md = ?"); vals.push(Box::new(v)); }
    if let Some(v) = i.notes_md { sets.push("notes_md = ?"); vals.push(Box::new(v)); }
    if let Some(v) = i.occurred_at { sets.push("occurred_at = ?"); vals.push(Box::new(v)); }
    if sets.is_empty() {
        return get(conn, i.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows);
    }
    let sql = format!("UPDATE performance_review SET {} WHERE id = ?", sets.join(", "));
    vals.push(Box::new(i.id));
    let refs: Vec<&dyn rusqlite::ToSql> = vals.iter().map(|b| &**b).collect();
    conn.execute(&sql, refs.as_slice())?;
    get(conn, i.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute("DELETE FROM performance_review WHERE id = ?1", [id])?;
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
    fn create_and_latest() {
        let (c, alice) = setup();
        create(&c, NewInput {
            report_id: alice, period: "Q1 2026".into(), rating: Some("Exceeds".into()),
            strengths_md: Some("shipped".into()), dev_areas_md: None, goals_md: None,
            notes_md: None, occurred_at: 1000,
        }, 0).unwrap();
        create(&c, NewInput {
            report_id: alice, period: "H1 2026".into(), rating: Some("Meets".into()),
            strengths_md: None, dev_areas_md: None, goals_md: None,
            notes_md: None, occurred_at: 2000,
        }, 0).unwrap();

        let latest = latest_for_report(&c, alice).unwrap().unwrap();
        assert_eq!(latest.period, "H1 2026");
    }

    #[test]
    fn update_patches_notes_md() {
        let (c, alice) = setup();
        let r = create(&c, NewInput {
            report_id: alice, period: "Q1 2026".into(), rating: None,
            strengths_md: Some("shipped".into()), dev_areas_md: None, goals_md: None,
            notes_md: None, occurred_at: 1000,
        }, 0).unwrap();
        assert!(r.notes_md.is_none());

        update(&c, UpdateInput {
            id: r.id, period: None, rating: None,
            strengths_md: None, dev_areas_md: None, goals_md: None,
            notes_md: Some(Some("Went well; Alex was open to feedback.".into())),
            occurred_at: None,
        }).unwrap();

        let after = get(&c, r.id).unwrap().unwrap();
        assert_eq!(after.notes_md, Some("Went well; Alex was open to feedback.".into()));
        assert_eq!(after.strengths_md, Some("shipped".into())); // unchanged
    }
}
