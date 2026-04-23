use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekRating {
    pub id: i64,
    pub report_id: Option<i64>,  // None = team-overall
    pub iso_week: String,
    pub color: String,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertInput {
    pub report_id: Option<i64>,
    pub iso_week: String,
    pub color: String,
    pub notes: Option<String>,
}

fn row_to_rating(row: &rusqlite::Row) -> rusqlite::Result<WeekRating> {
    Ok(WeekRating {
        id: row.get(0)?,
        report_id: row.get(1)?,
        iso_week: row.get(2)?,
        color: row.get(3)?,
        notes: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

/// All ratings for a given ISO week, including the team-overall row if present.
pub fn list_by_week(conn: &Connection, iso_week: &str) -> rusqlite::Result<Vec<WeekRating>> {
    let mut stmt = conn.prepare(
        "SELECT id, report_id, iso_week, color, notes, created_at, updated_at \
         FROM week_rating WHERE iso_week = ?1",
    )?;
    let rows = stmt.query_map([iso_week], row_to_rating)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// All ratings for a single report, ordered by iso_week ascending.
pub fn list_by_report(conn: &Connection, report_id: i64) -> rusqlite::Result<Vec<WeekRating>> {
    let mut stmt = conn.prepare(
        "SELECT id, report_id, iso_week, color, notes, created_at, updated_at \
         FROM week_rating WHERE report_id = ?1 ORDER BY iso_week ASC",
    )?;
    let rows = stmt.query_map([report_id], row_to_rating)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// All team-overall ratings (report_id IS NULL), ordered by iso_week ascending.
pub fn list_team_overall(conn: &Connection) -> rusqlite::Result<Vec<WeekRating>> {
    let mut stmt = conn.prepare(
        "SELECT id, report_id, iso_week, color, notes, created_at, updated_at \
         FROM week_rating WHERE report_id IS NULL ORDER BY iso_week ASC",
    )?;
    let rows = stmt.query_map([], row_to_rating)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Every rating within a week range, any report or team-overall.
pub fn list_in_range(
    conn: &Connection,
    from_iso_week: &str,
    to_iso_week: &str,
) -> rusqlite::Result<Vec<WeekRating>> {
    let mut stmt = conn.prepare(
        "SELECT id, report_id, iso_week, color, notes, created_at, updated_at \
         FROM week_rating WHERE iso_week >= ?1 AND iso_week <= ?2 \
         ORDER BY iso_week ASC, report_id IS NOT NULL, report_id ASC",
    )?;
    let rows = stmt
        .query_map([from_iso_week, to_iso_week], row_to_rating)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Upsert by (report_id, iso_week). SQLCipher enforces uniqueness via the
/// `idx_week_rating_unique` index (COALESCE(report_id,-1), iso_week).
pub fn upsert(conn: &Connection, input: UpsertInput, now: i64) -> rusqlite::Result<WeekRating> {
    // Try UPDATE first, fall back to INSERT if no row was touched.
    let rows = conn.execute(
        "UPDATE week_rating SET color = ?1, notes = ?2, updated_at = ?3 \
         WHERE COALESCE(report_id, -1) = COALESCE(?4, -1) AND iso_week = ?5",
        params![input.color, input.notes, now, input.report_id, input.iso_week],
    )?;
    if rows == 0 {
        conn.execute(
            "INSERT INTO week_rating (report_id, iso_week, color, notes, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![input.report_id, input.iso_week, input.color, input.notes, now],
        )?;
    }

    conn.query_row(
        "SELECT id, report_id, iso_week, color, notes, created_at, updated_at \
         FROM week_rating \
         WHERE COALESCE(report_id, -1) = COALESCE(?1, -1) AND iso_week = ?2",
        params![input.report_id, input.iso_week],
        row_to_rating,
    )
}

pub fn delete(conn: &Connection, report_id: Option<i64>, iso_week: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM week_rating \
         WHERE COALESCE(report_id, -1) = COALESCE(?1, -1) AND iso_week = ?2",
        params![report_id, iso_week],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{migrations, reports};

    fn mem() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        migrations::run(&c).unwrap();
        c
    }

    fn add_report(c: &Connection, name: &str) -> i64 {
        reports::create(c, reports::NewReportInput {
            name: name.into(), role: None, start_date: None,
            one_on_one_cadence_days: 14, notes: None,
        }, 1000).unwrap().id
    }

    #[test]
    fn upsert_inserts_then_updates() {
        let c = mem();
        let alice = add_report(&c, "Alice");

        let a = upsert(&c, UpsertInput {
            report_id: Some(alice), iso_week: "2026-W17".into(),
            color: "green".into(), notes: Some("shipped".into()),
        }, 2000).unwrap();

        let b = upsert(&c, UpsertInput {
            report_id: Some(alice), iso_week: "2026-W17".into(),
            color: "yellow".into(), notes: None,
        }, 3000).unwrap();

        assert_eq!(a.id, b.id);                // same row
        assert_eq!(b.color, "yellow");
        assert_eq!(b.notes, None);
        assert_eq!(b.created_at, 2000);        // preserved
        assert_eq!(b.updated_at, 3000);        // bumped
    }

    #[test]
    fn team_overall_and_per_report_are_distinct() {
        let c = mem();
        let alice = add_report(&c, "Alice");

        upsert(&c, UpsertInput {
            report_id: None, iso_week: "2026-W17".into(),
            color: "yellow".into(), notes: None,
        }, 1000).unwrap();

        upsert(&c, UpsertInput {
            report_id: Some(alice), iso_week: "2026-W17".into(),
            color: "green".into(), notes: None,
        }, 1000).unwrap();

        let by_week = list_by_week(&c, "2026-W17").unwrap();
        assert_eq!(by_week.len(), 2);
    }

    #[test]
    fn list_in_range_spans_multiple_weeks() {
        let c = mem();
        let alice = add_report(&c, "Alice");

        for (w, color) in [("2026-W15", "green"), ("2026-W16", "yellow"), ("2026-W17", "green")] {
            upsert(&c, UpsertInput {
                report_id: Some(alice), iso_week: w.into(),
                color: color.into(), notes: None,
            }, 1000).unwrap();
        }

        let got = list_in_range(&c, "2026-W15", "2026-W17").unwrap();
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].iso_week, "2026-W15");
        assert_eq!(got[2].iso_week, "2026-W17");
    }

    #[test]
    fn delete_removes_by_composite_key() {
        let c = mem();
        let alice = add_report(&c, "Alice");
        upsert(&c, UpsertInput {
            report_id: Some(alice), iso_week: "2026-W17".into(),
            color: "red".into(), notes: None,
        }, 1000).unwrap();

        delete(&c, Some(alice), "2026-W17").unwrap();
        assert_eq!(list_by_report(&c, alice).unwrap().len(), 0);
    }
}
