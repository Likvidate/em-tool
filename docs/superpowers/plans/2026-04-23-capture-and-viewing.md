# EM Tool — Plan 2: Capture & Viewing

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the daily-use loop — add reports, capture weekly colors + notes in the grid, view per-person timelines and the team heatmap. End state: a genuinely useful EM notebook. No AI, no 1:1/review logging yet (those come in Plan 3).

**Architecture:** Rust gets two pure data modules (`reports`, `week_ratings`) plus a `db` helper that locks the `AppState` mutex and hands out `&Connection` to closures. All mutations go through the existing `commands.rs` → typed `invoke.ts` → Pinia stores chain established in Plan 1. The 3 new views share a `ColorSwatches` component (picker) and a `ColorStrip` component (heatmap bar); the color palette lives in a single `colors.ts` module so nothing drifts out of the red→yellow→grey→green→blue order.

**Tech Stack:** (unchanged from Plan 1) Tauri 2 + Vue 3 + TypeScript + Pinia + Vue Router + rusqlite/SQLCipher + chrono.

---

## File structure added by this plan

```
em-tool/
├── src/
│   ├── lib/
│   │   ├── iso-week.ts                 (Task 1)
│   │   ├── iso-week.test.ts            (Task 1)
│   │   ├── colors.ts                   (Task 2)
│   │   └── invoke.ts                   (Task 5, extend)
│   ├── types/
│   │   ├── report.ts                   (Task 2)
│   │   └── week-rating.ts              (Task 2)
│   ├── composables/
│   │   └── useCurrentWeek.ts           (Task 11)
│   ├── stores/
│   │   ├── reports.ts                  (Task 6)
│   │   └── week-ratings.ts             (Task 7)
│   ├── components/
│   │   ├── ColorSwatches.vue           (Task 8)
│   │   ├── ColorSwatches.test.ts       (Task 8)
│   │   ├── ColorStrip.vue              (Task 13)
│   │   ├── AddReportModal.vue          (Task 9)
│   │   └── WeekNav.vue                 (Task 11)
│   ├── router.ts                       (Task 14, extend)
│   └── views/
│       ├── ReportsView.vue             (Task 10, replace stub)
│       ├── WeeklyCaptureView.vue       (Task 12, replace stub)
│       ├── ReportTimelineView.vue      (Task 13, new)
│       └── TeamHeatmapView.vue         (Task 14, replace stub)
└── src-tauri/
    └── src/
        ├── db.rs                       (Task 3)
        ├── reports.rs                  (Task 3)
        ├── week_ratings.rs             (Task 4)
        ├── commands.rs                 (Task 5, extend)
        └── lib.rs                      (Tasks 3, 4, 5, extend)
```

**Responsibilities:**
- `db.rs` — one helper: `with_conn(state, f)` locks the mutex, fails if no connection, invokes the closure, returns the result. Avoids every command relitigating lock semantics.
- `reports.rs` / `week_ratings.rs` — pure functions over `&Connection`, no Tauri types, no mutex awareness. Testable against `Connection::open_in_memory()` without touching `AppState`.
- `commands.rs` — thin wrappers: lock state, call data layer, serialize result. Errors converted to the existing `CommandError` shape.
- Stores — own the canonical cached list. Views subscribe; mutations go through the store so the cache updates optimistically.
- `iso-week.ts` — `currentIsoWeek()`, `parseIsoWeek("2026-W17")`, `addWeeks`, `weeksInRange` — used by capture + heatmap + timeline.
- `colors.ts` — single source of truth for `COLORS`, `COLOR_ORDER`, label, hex value.

---

## Task 1 — ISO week utility (TDD)

**Files:**
- Create: `src/lib/iso-week.ts`, `src/lib/iso-week.test.ts`

- [ ] **Step 1: Create the failing test file**

`src/lib/iso-week.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { currentIsoWeek, parseIsoWeek, formatIsoWeek, addWeeks, weeksInRange } from "./iso-week";

describe("iso-week", () => {
  describe("formatIsoWeek", () => {
    it("formats year + week to YYYY-Www with zero padding", () => {
      expect(formatIsoWeek({ year: 2026, week: 17 })).toBe("2026-W17");
      expect(formatIsoWeek({ year: 2026, week: 1 })).toBe("2026-W01");
      expect(formatIsoWeek({ year: 2025, week: 52 })).toBe("2025-W52");
    });
  });

  describe("parseIsoWeek", () => {
    it("parses YYYY-Www into year + week", () => {
      expect(parseIsoWeek("2026-W17")).toEqual({ year: 2026, week: 17 });
      expect(parseIsoWeek("2025-W01")).toEqual({ year: 2025, week: 1 });
    });

    it("throws on malformed input", () => {
      expect(() => parseIsoWeek("2026-17")).toThrow();
      expect(() => parseIsoWeek("not-a-week")).toThrow();
      expect(() => parseIsoWeek("2026-W00")).toThrow();
      expect(() => parseIsoWeek("2026-W54")).toThrow();
    });
  });

  describe("addWeeks", () => {
    it("moves forward within a year", () => {
      expect(addWeeks({ year: 2026, week: 10 }, 4)).toEqual({ year: 2026, week: 14 });
    });

    it("moves backward within a year", () => {
      expect(addWeeks({ year: 2026, week: 10 }, -3)).toEqual({ year: 2026, week: 7 });
    });

    it("crosses year boundary forward", () => {
      expect(addWeeks({ year: 2025, week: 51 }, 3)).toEqual({ year: 2026, week: 2 });
    });

    it("crosses year boundary backward", () => {
      expect(addWeeks({ year: 2026, week: 2 }, -3)).toEqual({ year: 2025, week: 51 });
    });

    it("handles 53-week ISO years (2020 had W53)", () => {
      // 2020 is a 53-week year. Going from 2020-W52 +1 should land on 2020-W53.
      expect(addWeeks({ year: 2020, week: 52 }, 1)).toEqual({ year: 2020, week: 53 });
      expect(addWeeks({ year: 2020, week: 53 }, 1)).toEqual({ year: 2021, week: 1 });
    });
  });

  describe("weeksInRange", () => {
    it("yields every week from start to end inclusive", () => {
      const range = weeksInRange({ year: 2026, week: 10 }, { year: 2026, week: 13 });
      expect(range).toEqual([
        { year: 2026, week: 10 },
        { year: 2026, week: 11 },
        { year: 2026, week: 12 },
        { year: 2026, week: 13 },
      ]);
    });

    it("crosses year boundaries", () => {
      const range = weeksInRange({ year: 2025, week: 52 }, { year: 2026, week: 2 });
      expect(range).toEqual([
        { year: 2025, week: 52 },
        { year: 2026, week: 1 },
        { year: 2026, week: 2 },
      ]);
    });

    it("yields a single week when start == end", () => {
      expect(weeksInRange({ year: 2026, week: 17 }, { year: 2026, week: 17 })).toEqual([
        { year: 2026, week: 17 },
      ]);
    });
  });

  describe("currentIsoWeek", () => {
    it("returns a plausible year + week from the system clock", () => {
      const w = currentIsoWeek();
      expect(w.year).toBeGreaterThanOrEqual(2024);
      expect(w.year).toBeLessThanOrEqual(2100);
      expect(w.week).toBeGreaterThanOrEqual(1);
      expect(w.week).toBeLessThanOrEqual(53);
    });
  });
});
```

- [ ] **Step 2: Create the implementation**

`src/lib/iso-week.ts`:

```ts
export interface IsoWeek {
  year: number;
  week: number;
}

/**
 * Returns the Monday date (UTC) of the ISO 8601 week containing `date`.
 */
function mondayOfIsoWeek(date: Date): Date {
  const d = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate()));
  // ISO: Monday=1, Sunday=7
  const dow = d.getUTCDay() || 7;
  d.setUTCDate(d.getUTCDate() - (dow - 1));
  return d;
}

/**
 * Returns the ISO 8601 week number and week-year for the given date.
 * The week-year can differ from the calendar year near Jan 1 / Dec 31.
 */
function isoWeekOf(date: Date): IsoWeek {
  const monday = mondayOfIsoWeek(date);
  const thursday = new Date(monday);
  thursday.setUTCDate(monday.getUTCDate() + 3); // Thursday of this ISO week
  const year = thursday.getUTCFullYear();
  const firstThursday = new Date(Date.UTC(year, 0, 4));
  const firstMonday = mondayOfIsoWeek(firstThursday);
  const diffMs = monday.getTime() - firstMonday.getTime();
  const week = 1 + Math.round(diffMs / (7 * 24 * 60 * 60 * 1000));
  return { year, week };
}

/**
 * Returns true if `year` has 53 ISO weeks.
 * An ISO year has 53 weeks iff Jan 1 falls on Thursday, or Jan 1 falls on
 * Wednesday in a leap year.
 */
function weeksInIsoYear(year: number): 52 | 53 {
  const jan1 = new Date(Date.UTC(year, 0, 1)).getUTCDay(); // 0=Sun..6=Sat
  const isLeap = (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0;
  if (jan1 === 4 || (jan1 === 3 && isLeap)) return 53;
  return 52;
}

export function currentIsoWeek(): IsoWeek {
  return isoWeekOf(new Date());
}

export function formatIsoWeek({ year, week }: IsoWeek): string {
  const ww = String(week).padStart(2, "0");
  return `${year}-W${ww}`;
}

export function parseIsoWeek(s: string): IsoWeek {
  const match = /^(\d{4})-W(\d{2})$/.exec(s);
  if (!match) throw new Error(`Invalid ISO week string: ${s}`);
  const year = Number(match[1]);
  const week = Number(match[2]);
  if (week < 1 || week > weeksInIsoYear(year)) {
    throw new Error(`Invalid ISO week number: ${s}`);
  }
  return { year, week };
}

export function addWeeks(w: IsoWeek, delta: number): IsoWeek {
  let { year, week } = w;
  week += delta;
  while (week < 1) {
    year -= 1;
    week += weeksInIsoYear(year);
  }
  while (week > weeksInIsoYear(year)) {
    week -= weeksInIsoYear(year);
    year += 1;
  }
  return { year, week };
}

function compareIsoWeek(a: IsoWeek, b: IsoWeek): number {
  if (a.year !== b.year) return a.year - b.year;
  return a.week - b.week;
}

export function weeksInRange(start: IsoWeek, end: IsoWeek): IsoWeek[] {
  if (compareIsoWeek(start, end) > 0) {
    throw new Error("start must be <= end");
  }
  const out: IsoWeek[] = [];
  let cur = start;
  while (compareIsoWeek(cur, end) <= 0) {
    out.push(cur);
    cur = addWeeks(cur, 1);
  }
  return out;
}
```

- [ ] **Step 3: Run the tests**

Run: `cd /home/net-irmantasci/em-tool && npm run test 2>&1 | tail -20`

Expected: all tests pass including the new 14 iso-week tests + the 3 existing idle-timer tests (17 total).

- [ ] **Step 4: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add src/lib/iso-week.ts src/lib/iso-week.test.ts
git commit -m "feat(lib): ISO 8601 week utilities (format/parse/add/range)

currentIsoWeek / formatIsoWeek / parseIsoWeek / addWeeks / weeksInRange.
Handles 53-week ISO years (2020, 2026 is 53-week too) and year-boundary
crossings. 14 unit tests cover the edge cases."
```

---

## Task 2 — Colors palette + types

**Files:**
- Create: `src/lib/colors.ts`
- Create: `src/types/report.ts`, `src/types/week-rating.ts`

- [ ] **Step 1: Create `src/lib/colors.ts`**

```ts
export const COLOR_ORDER = ["red", "yellow", "grey", "green", "blue"] as const;
export type Color = typeof COLOR_ORDER[number];

export interface ColorDef {
  key: Color;
  label: string;
  hex: string;
  description: string;
}

export const COLORS: Record<Color, ColorDef> = {
  red:    { key: "red",    label: "Red",    hex: "#ef4444", description: "Serious issue, flag for next 1:1" },
  yellow: { key: "yellow", label: "Yellow", hex: "#facc15", description: "Concern, keep an eye on" },
  grey:   { key: "grey",   label: "Grey",   hex: "#6b7280", description: "No meaningful signal this week" },
  green:  { key: "green",  label: "Green",  hex: "#4ade80", description: "Good week, delivered well" },
  blue:   { key: "blue",   label: "Blue",   hex: "#3b82f6", description: "Growth milestone / big win" },
};

export function colorHex(c: Color): string {
  return COLORS[c].hex;
}
```

- [ ] **Step 2: Create `src/types/report.ts`**

```ts
export interface Report {
  id: number;
  name: string;
  role: string | null;
  startDate: string | null;            // ISO date YYYY-MM-DD
  oneOnOneCadenceDays: number;
  notes: string | null;
  active: boolean;
  createdAt: number;                   // unix seconds
}

export interface NewReportInput {
  name: string;
  role?: string | null;
  startDate?: string | null;
  oneOnOneCadenceDays: number;
  notes?: string | null;
}

export interface UpdateReportInput {
  id: number;
  name?: string;
  role?: string | null;
  startDate?: string | null;
  oneOnOneCadenceDays?: number;
  notes?: string | null;
  active?: boolean;
}
```

- [ ] **Step 3: Create `src/types/week-rating.ts`**

```ts
import type { Color } from "../lib/colors";

export interface WeekRating {
  id: number;
  reportId: number | null;             // null = team-overall
  isoWeek: string;                     // "YYYY-Www"
  color: Color;
  notes: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface UpsertWeekRatingInput {
  reportId: number | null;
  isoWeek: string;
  color: Color;
  notes: string | null;
}
```

- [ ] **Step 4: Typecheck**

Run: `cd /home/net-irmantasci/em-tool && npx vue-tsc --noEmit`

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add src/lib/colors.ts src/types/
git commit -m "feat(lib): color palette + Report/WeekRating types

Single source of truth for COLOR_ORDER (red, yellow, grey, green, blue)
and hex values. Types mirror the v1 schema with camelCase field names."
```

---

## Task 3 — Rust `db` helper + `reports` module

**Files:**
- Create: `src-tauri/src/db.rs`, `src-tauri/src/reports.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create `src-tauri/src/db.rs`**

```rust
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
```

- [ ] **Step 2: Create `src-tauri/src/reports.rs`**

```rust
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
```

- [ ] **Step 3: Register modules in `lib.rs`**

Replace `src-tauri/src/lib.rs` with:

```rust
mod kdf;
mod vault;
mod migrations;
mod state;
mod commands;
mod db;
mod reports;

use state::{AppState, default_db_path};

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new(default_db_path()))
        .invoke_handler(tauri::generate_handler![
            commands::vault_exists,
            commands::is_unlocked,
            commands::create_vault,
            commands::unlock_vault,
            commands::lock_vault,
            commands::touch_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

(`week_ratings` module added in Task 4; invoke handlers expanded in Task 5.)

- [ ] **Step 4: Run Rust tests**

Run: `cd /home/net-irmantasci/em-tool/src-tauri && cargo test --lib reports 2>&1 | tail -15`

Expected: 3 tests pass (`create_and_list`, `archive_hides_from_list`, `update_patches_only_specified_fields`).

- [ ] **Step 5: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add src-tauri/src/db.rs src-tauri/src/reports.rs src-tauri/src/lib.rs
git commit -m "feat(rust): reports module with CRUD + archive, plus db::with_conn

reports.rs has pure functions over &Connection (list/get/create/update/
archive); db.rs is a thin helper that locks AppState and hands the
connection to a closure, so every tauri command doesn't relitigate
mutex semantics. Update is PATCH-style: fields are Option<Option<T>>
so None means 'don't touch' and Some(None) means 'clear'."
```

---

## Task 4 — Rust `week_ratings` module

**Files:**
- Create: `src-tauri/src/week_ratings.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create `src-tauri/src/week_ratings.rs`**

```rust
use rusqlite::{params, Connection, OptionalExtension};
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
/// Used by the team heatmap to pull a block of data once.
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
```

- [ ] **Step 2: Register module in `lib.rs`**

Replace `src-tauri/src/lib.rs`:

```rust
mod kdf;
mod vault;
mod migrations;
mod state;
mod commands;
mod db;
mod reports;
mod week_ratings;

use state::{AppState, default_db_path};

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new(default_db_path()))
        .invoke_handler(tauri::generate_handler![
            commands::vault_exists,
            commands::is_unlocked,
            commands::create_vault,
            commands::unlock_vault,
            commands::lock_vault,
            commands::touch_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Run tests**

Run: `cd /home/net-irmantasci/em-tool/src-tauri && cargo test --lib week_ratings 2>&1 | tail -15`

Expected: 4 tests pass.

- [ ] **Step 4: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add src-tauri/src/week_ratings.rs src-tauri/src/lib.rs
git commit -m "feat(rust): week_ratings module — upsert, list-by-week/report/range

Upsert is keyed on COALESCE(report_id, -1) + iso_week to match the
unique index installed by Plan 1 migrations. Team-overall ratings are
represented as report_id = NULL so they coexist with per-report rows."
```

---

## Task 5 — Tauri commands + invoke.ts extension

**Files:**
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`
- Modify: `src/lib/invoke.ts`

- [ ] **Step 1: Append to `src-tauri/src/commands.rs`**

Append the following at the end of the existing file (keep everything from Plan 1 intact):

```rust
use crate::{db, reports, week_ratings};

impl From<db::DbError> for CommandError {
    fn from(e: db::DbError) -> Self {
        let code = match &e {
            db::DbError::Locked => "locked",
            db::DbError::Sqlite(_) => "sqlite",
        };
        CommandError { code: code.to_string(), message: e.to_string() }
    }
}

// --- Reports ---

#[tauri::command]
pub fn list_reports(
    state: State<AppState>,
    include_archived: bool,
) -> Result<Vec<reports::Report>, CommandError> {
    db::with_conn(&state, |c| reports::list(c, include_archived)).map_err(Into::into)
}

#[tauri::command]
pub fn get_report(
    state: State<AppState>,
    id: i64,
) -> Result<Option<reports::Report>, CommandError> {
    db::with_conn(&state, |c| reports::get(c, id)).map_err(Into::into)
}

#[tauri::command]
pub fn create_report(
    state: State<AppState>,
    input: reports::NewReportInput,
) -> Result<reports::Report, CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| reports::create(c, input, now)).map_err(Into::into)
}

#[tauri::command]
pub fn update_report(
    state: State<AppState>,
    input: reports::UpdateReportInput,
) -> Result<reports::Report, CommandError> {
    db::with_conn(&state, |c| reports::update(c, input)).map_err(Into::into)
}

#[tauri::command]
pub fn archive_report(state: State<AppState>, id: i64) -> Result<(), CommandError> {
    db::with_conn(&state, |c| reports::archive(c, id)).map_err(Into::into)
}

// --- Week ratings ---

#[tauri::command]
pub fn list_week_ratings_by_week(
    state: State<AppState>,
    iso_week: String,
) -> Result<Vec<week_ratings::WeekRating>, CommandError> {
    db::with_conn(&state, |c| week_ratings::list_by_week(c, &iso_week)).map_err(Into::into)
}

#[tauri::command]
pub fn list_week_ratings_by_report(
    state: State<AppState>,
    report_id: i64,
) -> Result<Vec<week_ratings::WeekRating>, CommandError> {
    db::with_conn(&state, |c| week_ratings::list_by_report(c, report_id)).map_err(Into::into)
}

#[tauri::command]
pub fn list_week_ratings_team_overall(
    state: State<AppState>,
) -> Result<Vec<week_ratings::WeekRating>, CommandError> {
    db::with_conn(&state, |c| week_ratings::list_team_overall(c)).map_err(Into::into)
}

#[tauri::command]
pub fn list_week_ratings_in_range(
    state: State<AppState>,
    from_iso_week: String,
    to_iso_week: String,
) -> Result<Vec<week_ratings::WeekRating>, CommandError> {
    db::with_conn(&state, |c| week_ratings::list_in_range(c, &from_iso_week, &to_iso_week))
        .map_err(Into::into)
}

#[tauri::command]
pub fn upsert_week_rating(
    state: State<AppState>,
    input: week_ratings::UpsertInput,
) -> Result<week_ratings::WeekRating, CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| week_ratings::upsert(c, input, now)).map_err(Into::into)
}

#[tauri::command]
pub fn delete_week_rating(
    state: State<AppState>,
    report_id: Option<i64>,
    iso_week: String,
) -> Result<(), CommandError> {
    db::with_conn(&state, |c| week_ratings::delete(c, report_id, &iso_week)).map_err(Into::into)
}
```

- [ ] **Step 2: Register all new handlers in `lib.rs`**

Replace the `invoke_handler!` block in `src-tauri/src/lib.rs` to include the new commands:

```rust
        .invoke_handler(tauri::generate_handler![
            commands::vault_exists,
            commands::is_unlocked,
            commands::create_vault,
            commands::unlock_vault,
            commands::lock_vault,
            commands::touch_activity,
            commands::list_reports,
            commands::get_report,
            commands::create_report,
            commands::update_report,
            commands::archive_report,
            commands::list_week_ratings_by_week,
            commands::list_week_ratings_by_report,
            commands::list_week_ratings_team_overall,
            commands::list_week_ratings_in_range,
            commands::upsert_week_rating,
            commands::delete_week_rating,
        ])
```

- [ ] **Step 3: Extend `src/lib/invoke.ts`**

Replace the file (keep the existing header + `vaultApi`, add `reportsApi` and `weekRatingsApi`):

```ts
import { invoke as rawInvoke } from "@tauri-apps/api/core";
import type { Report, NewReportInput, UpdateReportInput } from "../types/report";
import type { WeekRating, UpsertWeekRatingInput } from "../types/week-rating";

export type CommandError = { code: string; message: string };

export class InvokeError extends Error {
  code: string;
  constructor(err: CommandError) {
    super(err.message);
    this.code = err.code;
  }
}

export async function invoke<T = void>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await rawInvoke<T>(command, args);
  } catch (err) {
    if (err && typeof err === "object" && "code" in err && "message" in err) {
      throw new InvokeError(err as CommandError);
    }
    throw err;
  }
}

export const vaultApi = {
  exists: () => invoke<boolean>("vault_exists"),
  isUnlocked: () => invoke<boolean>("is_unlocked"),
  create: (password: string) => invoke<void>("create_vault", { password }),
  unlock: (password: string) => invoke<void>("unlock_vault", { password }),
  lock: () => invoke<void>("lock_vault"),
  touchActivity: () => invoke<void>("touch_activity"),
};

export const reportsApi = {
  list: (includeArchived = false) =>
    invoke<Report[]>("list_reports", { includeArchived }),
  get: (id: number) => invoke<Report | null>("get_report", { id }),
  create: (input: NewReportInput) => invoke<Report>("create_report", { input }),
  update: (input: UpdateReportInput) => invoke<Report>("update_report", { input }),
  archive: (id: number) => invoke<void>("archive_report", { id }),
};

export const weekRatingsApi = {
  listByWeek: (isoWeek: string) =>
    invoke<WeekRating[]>("list_week_ratings_by_week", { isoWeek }),
  listByReport: (reportId: number) =>
    invoke<WeekRating[]>("list_week_ratings_by_report", { reportId }),
  listTeamOverall: () =>
    invoke<WeekRating[]>("list_week_ratings_team_overall"),
  listInRange: (fromIsoWeek: string, toIsoWeek: string) =>
    invoke<WeekRating[]>("list_week_ratings_in_range", { fromIsoWeek, toIsoWeek }),
  upsert: (input: UpsertWeekRatingInput) =>
    invoke<WeekRating>("upsert_week_rating", { input }),
  delete: (reportId: number | null, isoWeek: string) =>
    invoke<void>("delete_week_rating", { reportId, isoWeek }),
};
```

- [ ] **Step 4: Typecheck**

Run: `cd /home/net-irmantasci/em-tool && npx vue-tsc --noEmit`

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src/lib/invoke.ts
git commit -m "feat: 11 new Tauri commands for reports + week ratings

Rust side converts DbError to CommandError (new 'locked' code for
callers that accidentally invoke while the vault is locked). TS side
extends invoke.ts with reportsApi and weekRatingsApi surfaces, typed
against the shared Report/WeekRating interfaces."
```

---

## Task 6 — Pinia `reports` store

**Files:**
- Create: `src/stores/reports.ts`

- [ ] **Step 1: Create `src/stores/reports.ts`**

```ts
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { reportsApi, InvokeError } from "../lib/invoke";
import type { Report, NewReportInput, UpdateReportInput } from "../types/report";

export const useReportsStore = defineStore("reports", () => {
  const items = ref<Report[]>([]);
  const loaded = ref(false);
  const loading = ref(false);
  const lastError = ref<string | null>(null);

  const active = computed(() => items.value.filter((r) => r.active));

  async function load(includeArchived = false) {
    loading.value = true;
    lastError.value = null;
    try {
      items.value = await reportsApi.list(includeArchived);
      loaded.value = true;
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function create(input: NewReportInput): Promise<Report> {
    const created = await reportsApi.create(input);
    items.value = [...items.value, created].sort((a, b) =>
      a.name.localeCompare(b.name, undefined, { sensitivity: "base" }),
    );
    return created;
  }

  async function update(input: UpdateReportInput): Promise<Report> {
    const updated = await reportsApi.update(input);
    items.value = items.value.map((r) => (r.id === updated.id ? updated : r));
    return updated;
  }

  async function archive(id: number) {
    await reportsApi.archive(id);
    items.value = items.value.map((r) => (r.id === id ? { ...r, active: false } : r));
  }

  function byId(id: number): Report | undefined {
    return items.value.find((r) => r.id === id);
  }

  return { items, active, loaded, loading, lastError, load, create, update, archive, byId };
});
```

- [ ] **Step 2: Typecheck**

Run: `cd /home/net-irmantasci/em-tool && npx vue-tsc --noEmit`

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add src/stores/reports.ts
git commit -m "feat(ui): Pinia reports store (load/create/update/archive)"
```

---

## Task 7 — Pinia `week-ratings` store

**Files:**
- Create: `src/stores/week-ratings.ts`

- [ ] **Step 1: Create `src/stores/week-ratings.ts`**

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { weekRatingsApi, InvokeError } from "../lib/invoke";
import type { WeekRating, UpsertWeekRatingInput } from "../types/week-rating";

/**
 * Keyed cache of ratings for fast access from multiple views.
 * Key format: `${reportId ?? "team"}:${isoWeek}` — unique per rating.
 */
function keyOf(reportId: number | null, isoWeek: string): string {
  return `${reportId ?? "team"}:${isoWeek}`;
}

export const useWeekRatingsStore = defineStore("weekRatings", () => {
  const byKey = ref<Record<string, WeekRating>>({});
  const loading = ref(false);
  const lastError = ref<string | null>(null);

  function indexMany(rows: WeekRating[]) {
    const next = { ...byKey.value };
    for (const r of rows) {
      next[keyOf(r.reportId, r.isoWeek)] = r;
    }
    byKey.value = next;
  }

  async function loadWeek(isoWeek: string) {
    loading.value = true;
    lastError.value = null;
    try {
      indexMany(await weekRatingsApi.listByWeek(isoWeek));
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadForReport(reportId: number) {
    loading.value = true;
    lastError.value = null;
    try {
      indexMany(await weekRatingsApi.listByReport(reportId));
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadRange(fromIsoWeek: string, toIsoWeek: string) {
    loading.value = true;
    lastError.value = null;
    try {
      indexMany(await weekRatingsApi.listInRange(fromIsoWeek, toIsoWeek));
      indexMany(await weekRatingsApi.listTeamOverall()); // ensure team row covered too
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function upsert(input: UpsertWeekRatingInput): Promise<WeekRating> {
    const saved = await weekRatingsApi.upsert(input);
    byKey.value = { ...byKey.value, [keyOf(saved.reportId, saved.isoWeek)]: saved };
    return saved;
  }

  async function remove(reportId: number | null, isoWeek: string) {
    await weekRatingsApi.delete(reportId, isoWeek);
    const next = { ...byKey.value };
    delete next[keyOf(reportId, isoWeek)];
    byKey.value = next;
  }

  function get(reportId: number | null, isoWeek: string): WeekRating | undefined {
    return byKey.value[keyOf(reportId, isoWeek)];
  }

  return { byKey, loading, lastError, loadWeek, loadForReport, loadRange, upsert, remove, get };
});
```

- [ ] **Step 2: Typecheck + commit**

```bash
cd /home/net-irmantasci/em-tool
npx vue-tsc --noEmit
git add src/stores/week-ratings.ts
git commit -m "feat(ui): Pinia week-ratings store with keyed cache"
```

---

## Task 8 — `ColorSwatches` component (with test)

**Files:**
- Create: `src/components/ColorSwatches.vue`, `src/components/ColorSwatches.test.ts`

- [ ] **Step 1: Create `src/components/ColorSwatches.vue`**

```vue
<script setup lang="ts">
import { COLOR_ORDER, COLORS, type Color } from "../lib/colors";

const props = defineProps<{ modelValue: Color | null }>();
const emit = defineEmits<{ "update:modelValue": [value: Color | null] }>();

function pick(color: Color) {
  emit("update:modelValue", props.modelValue === color ? null : color);
}
</script>

<template>
  <div class="swatches" role="radiogroup" aria-label="Color rating">
    <button
      v-for="c in COLOR_ORDER"
      :key="c"
      type="button"
      class="sw"
      :class="[c, { active: modelValue === c }]"
      :aria-label="COLORS[c].label"
      :title="COLORS[c].description"
      role="radio"
      :aria-checked="modelValue === c"
      @click="pick(c)"
    />
  </div>
</template>

<style scoped>
.swatches { display: inline-flex; gap: 5px; }
.sw {
  width: 22px; height: 22px;
  border-radius: 4px;
  cursor: pointer;
  border: 2px solid transparent;
  padding: 0;
}
.sw.red { background: #ef4444; }
.sw.yellow { background: #facc15; }
.sw.grey { background: #6b7280; }
.sw.green { background: #4ade80; }
.sw.blue { background: #3b82f6; }
.sw.active { border-color: #fff; box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.2); }
.sw:focus-visible { outline: 2px solid #7c3aed; outline-offset: 2px; }
</style>
```

- [ ] **Step 2: Create `src/components/ColorSwatches.test.ts`**

```ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import ColorSwatches from "./ColorSwatches.vue";

describe("ColorSwatches", () => {
  it("renders 5 swatches in canonical order", () => {
    const w = mount(ColorSwatches, { props: { modelValue: null } });
    const buttons = w.findAll("button.sw");
    expect(buttons).toHaveLength(5);
    expect(buttons[0].classes()).toContain("red");
    expect(buttons[1].classes()).toContain("yellow");
    expect(buttons[2].classes()).toContain("grey");
    expect(buttons[3].classes()).toContain("green");
    expect(buttons[4].classes()).toContain("blue");
  });

  it("marks the current selection active", () => {
    const w = mount(ColorSwatches, { props: { modelValue: "yellow" } });
    const yellow = w.findAll("button.sw")[1];
    expect(yellow.classes()).toContain("active");
  });

  it("emits update:modelValue on click", async () => {
    const w = mount(ColorSwatches, { props: { modelValue: null } });
    await w.findAll("button.sw")[3].trigger("click");   // green
    expect(w.emitted("update:modelValue")?.[0]).toEqual(["green"]);
  });

  it("clicking the active color clears the selection", async () => {
    const w = mount(ColorSwatches, { props: { modelValue: "red" } });
    await w.findAll("button.sw")[0].trigger("click");
    expect(w.emitted("update:modelValue")?.[0]).toEqual([null]);
  });
});
```

- [ ] **Step 3: Run tests**

Run: `cd /home/net-irmantasci/em-tool && npm run test 2>&1 | tail -15`

Expected: all tests pass (idle-timer, iso-week, ColorSwatches — 21 total).

- [ ] **Step 4: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add src/components/ColorSwatches.vue src/components/ColorSwatches.test.ts
git commit -m "feat(ui): ColorSwatches component with radiogroup semantics

Locked red→yellow→grey→green→blue order. v-model exposes a Color |
null (null = cleared). Click-active-to-clear behavior. 4 Vitest tests."
```

---

## Task 9 — `AddReportModal` component

**Files:**
- Create: `src/components/AddReportModal.vue`

- [ ] **Step 1: Create `src/components/AddReportModal.vue`**

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
import { useReportsStore } from "../stores/reports";

const emit = defineEmits<{ close: []; created: [id: number] }>();
const reports = useReportsStore();

const name = ref("");
const role = ref("");
const startDate = ref(new Date().toISOString().slice(0, 10));
const cadence = ref(14);
const notes = ref("");
const submitting = ref(false);
const error = ref<string | null>(null);

const canSubmit = computed(() => name.value.trim().length > 0 && !submitting.value);

async function submit() {
  if (!canSubmit.value) return;
  submitting.value = true;
  error.value = null;
  try {
    const created = await reports.create({
      name: name.value.trim(),
      role: role.value.trim() || null,
      startDate: startDate.value || null,
      oneOnOneCadenceDays: cadence.value,
      notes: notes.value.trim() || null,
    });
    emit("created", created.id);
    emit("close");
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="backdrop" @click.self="emit('close')">
    <div class="modal">
      <header>
        <h3>Add a report</h3>
        <button class="close" @click="emit('close')">✕</button>
      </header>

      <form @submit.prevent="submit">
        <label><span>Name</span>
          <input v-model="name" type="text" autofocus placeholder="e.g. Fatima Al-Sayed" />
        </label>
        <div class="row">
          <label><span>Role</span>
            <input v-model="role" type="text" placeholder="e.g. Senior Backend" />
          </label>
          <label><span>Start date on team</span>
            <input v-model="startDate" type="date" />
          </label>
        </div>
        <label><span>1:1 cadence (days)</span>
          <select v-model.number="cadence">
            <option :value="7">Weekly</option>
            <option :value="14">Bi-weekly</option>
            <option :value="21">Every 3 weeks</option>
            <option :value="30">Monthly</option>
          </select>
        </label>
        <label><span>Notes</span>
          <textarea v-model="notes" rows="3" placeholder="Anything you want to remember..."></textarea>
        </label>

        <div v-if="error" class="error">{{ error }}</div>

        <footer>
          <button type="button" class="secondary" @click="emit('close')">Cancel</button>
          <button type="submit" class="primary" :disabled="!canSubmit">
            {{ submitting ? "Adding…" : "Add report" }}
          </button>
        </footer>
      </form>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(0, 0, 0, 0.55);
  display: flex; align-items: center; justify-content: center; padding: 24px;
}
.modal {
  background: var(--surface); border: 1px solid var(--border);
  border-radius: 8px; max-width: 520px; width: 100%;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
}
header { display: flex; justify-content: space-between; align-items: center; padding: 16px 18px; border-bottom: 1px solid var(--border); }
header h3 { margin: 0; font-size: 16px; }
.close { background: none; border: none; color: var(--text-dim); font-size: 16px; cursor: pointer; }
form { display: grid; gap: 14px; padding: 18px; }
.row { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
label { display: grid; gap: 4px; font-size: 12px; color: var(--text-dim); }
input, textarea, select {
  background: var(--bg); border: 1px solid var(--border); color: var(--text);
  padding: 8px 10px; border-radius: 4px; font-family: inherit; font-size: 13px;
}
textarea { resize: vertical; }
.error { color: #f87171; font-size: 12px; }
footer { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
button { padding: 7px 14px; border: none; border-radius: 4px; font-size: 13px; cursor: pointer; }
.primary { background: var(--accent); color: #fff; }
.primary:disabled { opacity: 0.4; cursor: not-allowed; }
.secondary { background: #374151; color: var(--text); }
</style>
```

- [ ] **Step 2: Commit**

```bash
cd /home/net-irmantasci/em-tool
npx vue-tsc --noEmit
git add src/components/AddReportModal.vue
git commit -m "feat(ui): AddReportModal — name/role/start-date/cadence/notes

Pinia-backed: on submit, creates the report via the store (which
optimistically updates the list) and emits the new id. Click-outside
and ✕ close the modal. Default cadence = 14 days (bi-weekly)."
```

---

## Task 10 — `ReportsView`

**Files:**
- Modify: `src/views/ReportsView.vue` (replace stub)

- [ ] **Step 1: Replace `src/views/ReportsView.vue`**

```vue
<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useRouter } from "vue-router";
import { useReportsStore } from "../stores/reports";
import AddReportModal from "../components/AddReportModal.vue";

const reports = useReportsStore();
const router = useRouter();
const showAdd = ref(false);
const showArchived = ref(false);

const visible = computed(() =>
  showArchived.value ? reports.items : reports.active,
);

onMounted(() => {
  if (!reports.loaded) reports.load(true);
});

function openTimeline(id: number) {
  router.push({ name: "report-timeline", params: { id: String(id) } });
}
</script>

<template>
  <div class="reports-view">
    <header class="page-head">
      <h2>Reports</h2>
      <div class="actions">
        <label class="archived-toggle">
          <input v-model="showArchived" type="checkbox" />
          <span>Show archived</span>
        </label>
        <button class="primary" @click="showAdd = true">+ Add report</button>
      </div>
    </header>

    <div v-if="reports.loading && !reports.loaded" class="empty">Loading…</div>

    <div v-else-if="visible.length === 0" class="empty">
      <p>No reports yet.</p>
      <button class="primary" @click="showAdd = true">Add your first report</button>
    </div>

    <table v-else class="list">
      <thead>
        <tr>
          <th>Name</th>
          <th>Role</th>
          <th>Cadence</th>
          <th>Started</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="r in visible"
          :key="r.id"
          :class="{ archived: !r.active }"
          @click="openTimeline(r.id)"
        >
          <td class="name">{{ r.name }}</td>
          <td>{{ r.role ?? "—" }}</td>
          <td>every {{ r.oneOnOneCadenceDays }}d</td>
          <td>{{ r.startDate ?? "—" }}</td>
          <td class="status">
            <span v-if="!r.active" class="badge">archived</span>
          </td>
        </tr>
      </tbody>
    </table>

    <AddReportModal v-if="showAdd" @close="showAdd = false" @created="(id) => openTimeline(id)" />
  </div>
</template>

<style scoped>
.reports-view { max-width: 900px; }
.page-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px; }
h2 { margin: 0; font-size: 20px; }
.actions { display: flex; gap: 12px; align-items: center; }
.archived-toggle { display: inline-flex; gap: 6px; align-items: center; font-size: 12px; color: var(--text-dim); }
.primary {
  background: var(--accent); color: #fff; border: none;
  padding: 7px 14px; border-radius: 4px; font-size: 13px; cursor: pointer;
}
.empty { padding: 48px 0; text-align: center; color: var(--text-dim); }
.empty .primary { margin-top: 12px; }
.list { width: 100%; border-collapse: collapse; font-size: 13px; }
.list th {
  text-align: left; padding: 8px 12px;
  font-size: 10px; text-transform: uppercase; letter-spacing: 0.08em;
  opacity: 0.55; border-bottom: 1px solid var(--border);
}
.list td { padding: 10px 12px; border-bottom: 1px solid var(--border); cursor: pointer; }
.list tr:hover td { background: var(--surface-2); }
.list .name { font-weight: 600; }
.list tr.archived td { opacity: 0.5; }
.badge {
  display: inline-block; padding: 2px 6px; border-radius: 3px;
  background: #374151; font-size: 10px; text-transform: uppercase; letter-spacing: 0.06em;
}
.status { text-align: right; }
</style>
```

- [ ] **Step 2: Typecheck + commit**

```bash
cd /home/net-irmantasci/em-tool
npx vue-tsc --noEmit
git add src/views/ReportsView.vue
git commit -m "feat(ui): ReportsView — list + archive toggle + row→timeline nav

Empty state on first launch points to the Add button. Clicking a row
routes to /reports/:id/timeline (Task 14 wires the route)."
```

---

## Task 11 — `useCurrentWeek` composable + `WeekNav` component

**Files:**
- Create: `src/composables/useCurrentWeek.ts`
- Create: `src/components/WeekNav.vue`

- [ ] **Step 1: Create `src/composables/useCurrentWeek.ts`**

```ts
import { ref, computed } from "vue";
import { currentIsoWeek, formatIsoWeek, addWeeks, type IsoWeek } from "../lib/iso-week";

/**
 * Reactive "week cursor" for capture-style screens.
 * Starts at the system's current ISO week; advancing/rewinding updates
 * all consumers of `isoWeek` and `week`.
 */
export function useCurrentWeek() {
  const week = ref<IsoWeek>(currentIsoWeek());
  const isoWeek = computed(() => formatIsoWeek(week.value));
  const label = computed(() => `Week ${week.value.week}, ${week.value.year}`);

  function prev() { week.value = addWeeks(week.value, -1); }
  function next() { week.value = addWeeks(week.value, 1); }
  function toCurrent() { week.value = currentIsoWeek(); }
  function set(w: IsoWeek) { week.value = w; }

  return { week, isoWeek, label, prev, next, toCurrent, set };
}
```

- [ ] **Step 2: Create `src/components/WeekNav.vue`**

```vue
<script setup lang="ts">
defineProps<{
  label: string;
  isCurrent?: boolean;
}>();
defineEmits<{ prev: []; next: []; jumpToCurrent: [] }>();
</script>

<template>
  <div class="weeknav">
    <button @click="$emit('prev')" aria-label="Previous week">◀</button>
    <div class="label">
      <span>{{ label }}</span>
      <button v-if="!isCurrent" class="today" @click="$emit('jumpToCurrent')">Today</button>
    </div>
    <button @click="$emit('next')" aria-label="Next week">▶</button>
  </div>
</template>

<style scoped>
.weeknav {
  display: flex; align-items: center; gap: 12px;
  padding: 8px 12px;
  background: var(--surface-2); border: 1px solid var(--border);
  border-radius: 6px;
}
.weeknav > button {
  background: none; border: 1px solid var(--border); color: var(--text);
  width: 28px; height: 28px; border-radius: 4px; cursor: pointer;
  font-size: 11px;
}
.label { flex: 1; display: flex; justify-content: center; align-items: center; gap: 10px; font-weight: 600; }
.today {
  background: none; border: 1px solid var(--border); color: var(--text-dim);
  padding: 2px 8px; border-radius: 3px; font-size: 11px; cursor: pointer;
}
</style>
```

- [ ] **Step 3: Typecheck + commit**

```bash
cd /home/net-irmantasci/em-tool
npx vue-tsc --noEmit
git add src/composables/useCurrentWeek.ts src/components/WeekNav.vue
git commit -m "feat(ui): useCurrentWeek composable + WeekNav component"
```

---

## Task 12 — `WeeklyCaptureView`

**Files:**
- Modify: `src/views/WeeklyCaptureView.vue` (replace stub)

- [ ] **Step 1: Replace `src/views/WeeklyCaptureView.vue`**

```vue
<script setup lang="ts">
import { onMounted, watch, computed } from "vue";
import { useReportsStore } from "../stores/reports";
import { useWeekRatingsStore } from "../stores/week-ratings";
import { useCurrentWeek } from "../composables/useCurrentWeek";
import { currentIsoWeek, formatIsoWeek } from "../lib/iso-week";
import ColorSwatches from "../components/ColorSwatches.vue";
import WeekNav from "../components/WeekNav.vue";
import type { Color } from "../lib/colors";

const reports = useReportsStore();
const ratings = useWeekRatingsStore();
const { week, isoWeek, label, prev, next, toCurrent } = useCurrentWeek();

const isCurrent = computed(() => isoWeek.value === formatIsoWeek(currentIsoWeek()));

onMounted(async () => {
  if (!reports.loaded) await reports.load(false);
  await ratings.loadWeek(isoWeek.value);
});

watch(isoWeek, async (w) => {
  await ratings.loadWeek(w);
});

function colorFor(reportId: number | null): Color | null {
  const r = ratings.get(reportId, isoWeek.value);
  return r ? (r.color as Color) : null;
}

function notesFor(reportId: number | null): string {
  return ratings.get(reportId, isoWeek.value)?.notes ?? "";
}

async function setColor(reportId: number | null, color: Color | null) {
  if (color === null) {
    await ratings.remove(reportId, isoWeek.value);
    return;
  }
  await ratings.upsert({
    reportId,
    isoWeek: isoWeek.value,
    color,
    notes: ratings.get(reportId, isoWeek.value)?.notes ?? null,
  });
}

async function setNotes(reportId: number | null, notes: string) {
  const cur = ratings.get(reportId, isoWeek.value);
  if (!cur) return; // no rating yet — notes without color unused for MVP
  if (cur.notes === (notes.trim() ? notes : null)) return;
  await ratings.upsert({
    reportId,
    isoWeek: isoWeek.value,
    color: cur.color as Color,
    notes: notes.trim() || null,
  });
}

function cadenceOverdue(report: { oneOnOneCadenceDays: number }) {
  return report.oneOnOneCadenceDays < 0; // placeholder — real "last 1:1" lookup comes in Plan 3
}
</script>

<template>
  <div class="capture">
    <header class="page-head">
      <h2>Weekly capture</h2>
      <WeekNav :label="label" :is-current="isCurrent" @prev="prev" @next="next" @jump-to-current="toCurrent" />
    </header>

    <div class="team-row">
      <div class="team-label">Team overall</div>
      <ColorSwatches
        :model-value="colorFor(null)"
        @update:model-value="(c) => setColor(null, c)"
      />
      <input
        type="text"
        :value="notesFor(null)"
        placeholder="Team note (optional)"
        class="note"
        @blur="setNotes(null, ($event.target as HTMLInputElement).value)"
      />
    </div>

    <div v-if="reports.active.length === 0" class="empty">
      <p>Add reports first to capture weekly ratings.</p>
      <router-link to="/reports" class="link">Go to Reports →</router-link>
    </div>

    <div v-else class="grid">
      <div v-for="r in reports.active" :key="r.id" class="row">
        <div class="person">
          <div class="name">{{ r.name }}</div>
          <div class="role">{{ r.role ?? "" }}</div>
        </div>
        <ColorSwatches
          :model-value="colorFor(r.id)"
          @update:model-value="(c) => setColor(r.id, c)"
        />
        <input
          type="text"
          :value="notesFor(r.id)"
          placeholder="Note (optional)"
          class="note"
          @blur="setNotes(r.id, ($event.target as HTMLInputElement).value)"
        />
        <div class="cadence">every {{ r.oneOnOneCadenceDays }}d</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.capture { max-width: 960px; }
.page-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px; gap: 18px; }
h2 { margin: 0; font-size: 20px; }
.team-row {
  display: grid;
  grid-template-columns: 180px auto 1fr;
  gap: 12px; align-items: center;
  padding: 12px 14px;
  background: #1f2937;
  border-left: 3px solid var(--blue);
  border-radius: 6px;
  margin-bottom: 16px;
}
.team-label { font-size: 11px; text-transform: uppercase; letter-spacing: 0.08em; opacity: 0.75; font-weight: 600; }

.grid { background: var(--surface); border: 1px solid var(--border); border-radius: 6px; }
.row {
  display: grid;
  grid-template-columns: 180px 180px 1fr 80px;
  gap: 12px; align-items: center;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
}
.row:last-child { border-bottom: none; }
.person .name { font-weight: 600; }
.person .role { font-size: 12px; opacity: 0.6; }
.note {
  background: var(--bg); border: 1px solid var(--border); color: var(--text);
  padding: 5px 8px; border-radius: 4px; font-size: 12px; font-family: inherit;
}
.cadence { font-size: 11px; opacity: 0.5; text-align: right; }
.empty { padding: 48px 0; text-align: center; color: var(--text-dim); }
.link { color: var(--accent); text-decoration: underline; }
</style>
```

- [ ] **Step 2: Commit**

```bash
cd /home/net-irmantasci/em-tool
npx vue-tsc --noEmit
git add src/views/WeeklyCaptureView.vue
git commit -m "feat(ui): Weekly capture grid — team row + per-report rows

Click a swatch to set color (upsert), click it again to clear (delete).
Notes save on blur to avoid per-keystroke IPC. Empty state routes to
the Reports page when no reports exist yet."
```

---

## Task 13 — `ColorStrip` + `ReportTimelineView`

**Files:**
- Create: `src/components/ColorStrip.vue`
- Create: `src/views/ReportTimelineView.vue`

- [ ] **Step 1: Create `src/components/ColorStrip.vue`**

```vue
<script setup lang="ts">
import type { Color } from "../lib/colors";

defineProps<{
  /** Cells in chronological order. `null` renders as empty grey. */
  cells: { isoWeek: string; color: Color | null; title?: string }[];
}>();
</script>

<template>
  <div class="strip">
    <div
      v-for="c in cells"
      :key="c.isoWeek"
      class="cell"
      :class="c.color ?? 'none'"
      :title="c.title ?? c.isoWeek"
    />
  </div>
</template>

<style scoped>
.strip { display: flex; gap: 2px; padding: 12px 14px; background: #141414; border: 1px solid var(--border); border-top: none; }
.cell { flex: 1; min-width: 6px; height: 22px; border-radius: 2px; background: #222; }
.cell.red    { background: #ef4444; }
.cell.yellow { background: #facc15; }
.cell.grey   { background: #6b7280; }
.cell.green  { background: #4ade80; }
.cell.blue   { background: #3b82f6; }
.cell.none   { background: #1a1a1a; }
</style>
```

- [ ] **Step 2: Create `src/views/ReportTimelineView.vue`**

```vue
<script setup lang="ts">
import { computed, onMounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useReportsStore } from "../stores/reports";
import { useWeekRatingsStore } from "../stores/week-ratings";
import ColorStrip from "../components/ColorStrip.vue";
import type { Color } from "../lib/colors";

const route = useRoute();
const router = useRouter();
const reports = useReportsStore();
const ratings = useWeekRatingsStore();

const reportId = computed(() => Number(route.params.id));

const report = computed(() => reports.byId(reportId.value));

const ratingsForReport = computed(() => {
  return Object.values(ratings.byKey)
    .filter((r) => r.reportId === reportId.value)
    .sort((a, b) => a.isoWeek.localeCompare(b.isoWeek));
});

const stripCells = computed(() =>
  ratingsForReport.value.map((r) => ({
    isoWeek: r.isoWeek,
    color: r.color as Color,
    title: `${r.isoWeek}: ${r.color}${r.notes ? " — " + r.notes : ""}`,
  })),
);

const counts = computed(() => {
  const out: Record<Color, number> = { red: 0, yellow: 0, grey: 0, green: 0, blue: 0 };
  for (const r of ratingsForReport.value) out[r.color as Color] += 1;
  return out;
});

onMounted(async () => {
  if (!reports.loaded) await reports.load(true);
  await ratings.loadForReport(reportId.value);
});

watch(reportId, async (id) => {
  await ratings.loadForReport(id);
});
</script>

<template>
  <div class="timeline" v-if="report">
    <header class="head">
      <div>
        <button class="back" @click="router.push('/reports')">← Reports</button>
        <h2>{{ report.name }}</h2>
        <p class="sub">
          {{ report.role ?? "—" }} ·
          1:1 every {{ report.oneOnOneCadenceDays }}d ·
          joined {{ report.startDate ?? "—" }}
        </p>
      </div>
    </header>

    <div class="stats">
      <div class="stat"><strong>{{ ratingsForReport.length }}</strong><span>weeks</span></div>
      <div class="stat"><span class="sw green"></span><strong>{{ counts.green }}</strong></div>
      <div class="stat"><span class="sw yellow"></span><strong>{{ counts.yellow }}</strong></div>
      <div class="stat"><span class="sw red"></span><strong>{{ counts.red }}</strong></div>
      <div class="stat"><span class="sw blue"></span><strong>{{ counts.blue }}</strong></div>
      <div class="stat"><span class="sw grey"></span><strong>{{ counts.grey }}</strong></div>
    </div>

    <ColorStrip :cells="stripCells" />

    <div class="feed">
      <div v-if="ratingsForReport.length === 0" class="empty">
        No ratings yet. Head to Weekly capture to start tracking.
      </div>
      <div v-for="r in [...ratingsForReport].reverse()" :key="r.id" class="entry">
        <div class="week">{{ r.isoWeek }}</div>
        <div class="sw" :class="r.color"></div>
        <div class="notes">{{ r.notes ?? "—" }}</div>
      </div>
    </div>
  </div>
  <div v-else class="loading">Loading…</div>
</template>

<style scoped>
.timeline { max-width: 900px; }
.head { padding: 14px; background: var(--surface); border: 1px solid var(--border); border-radius: 6px 6px 0 0; }
.back { background: none; border: none; color: var(--text-dim); font-size: 12px; cursor: pointer; margin-bottom: 4px; padding: 0; }
h2 { margin: 0; }
.sub { margin: 3px 0 0; font-size: 12px; opacity: 0.6; }
.stats {
  display: flex; gap: 18px;
  padding: 10px 14px;
  background: #141414; border-left: 1px solid var(--border); border-right: 1px solid var(--border);
  font-size: 12px; align-items: center;
}
.stat { display: flex; align-items: center; gap: 4px; }
.stat strong { color: var(--text); }
.sw { width: 14px; height: 14px; border-radius: 3px; display: inline-block; }
.sw.red    { background: #ef4444; }
.sw.yellow { background: #facc15; }
.sw.grey   { background: #6b7280; }
.sw.green  { background: #4ade80; }
.sw.blue   { background: #3b82f6; }

.feed { background: #141414; border: 1px solid var(--border); border-top: none; border-radius: 0 0 6px 6px; }
.entry {
  display: grid; grid-template-columns: 90px 20px 1fr; gap: 10px;
  padding: 10px 14px; border-bottom: 1px solid #222;
  font-size: 13px; align-items: center;
}
.entry:last-child { border-bottom: none; }
.week { font-family: monospace; opacity: 0.55; }
.empty, .loading { padding: 32px; text-align: center; color: var(--text-dim); }
</style>
```

- [ ] **Step 3: Commit**

```bash
cd /home/net-irmantasci/em-tool
npx vue-tsc --noEmit
git add src/components/ColorStrip.vue src/views/ReportTimelineView.vue
git commit -m "feat(ui): ColorStrip + per-report ReportTimelineView

Timeline pulls all ratings for a single report, computes color counts,
renders a horizontal color strip and a reverse-chrono feed below.
Week entries only (1:1 and review entries come in Plan 3)."
```

---

## Task 14 — `TeamHeatmapView` + route registration

**Files:**
- Modify: `src/views/TeamHeatmapView.vue` (replace stub)
- Modify: `src/router.ts` (add `/reports/:id/timeline`)

- [ ] **Step 1: Replace `src/views/TeamHeatmapView.vue`**

```vue
<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useReportsStore } from "../stores/reports";
import { useWeekRatingsStore } from "../stores/week-ratings";
import { currentIsoWeek, addWeeks, weeksInRange, formatIsoWeek, type IsoWeek } from "../lib/iso-week";
import type { Color } from "../lib/colors";

const reports = useReportsStore();
const ratings = useWeekRatingsStore();

const rangeWeeks = ref(26);

const range = computed<{ weeks: IsoWeek[]; from: string; to: string }>(() => {
  const end = currentIsoWeek();
  const start = addWeeks(end, -(rangeWeeks.value - 1));
  const weeks = weeksInRange(start, end);
  return { weeks, from: formatIsoWeek(start), to: formatIsoWeek(end) };
});

onMounted(async () => {
  if (!reports.loaded) await reports.load(false);
  await ratings.loadRange(range.value.from, range.value.to);
});

async function changeRange(n: number) {
  rangeWeeks.value = n;
  await ratings.loadRange(range.value.from, range.value.to);
}

function cellColor(reportId: number | null, isoWeek: string): Color | "none" {
  const r = ratings.get(reportId, isoWeek);
  return r ? (r.color as Color) : "none";
}

function cellTitle(reportId: number | null, isoWeek: string): string {
  const r = ratings.get(reportId, isoWeek);
  if (!r) return `${isoWeek} — no rating`;
  return `${isoWeek}: ${r.color}${r.notes ? " — " + r.notes : ""}`;
}
</script>

<template>
  <div class="heatmap">
    <header class="page-head">
      <h2>Team heatmap</h2>
      <div class="range">
        <label>Range:</label>
        <select :value="rangeWeeks" @change="(e) => changeRange(Number((e.target as HTMLSelectElement).value))">
          <option :value="13">Last 13 weeks</option>
          <option :value="26">Last 26 weeks</option>
          <option :value="52">Last 52 weeks</option>
        </select>
      </div>
    </header>

    <div class="grid-wrap">
      <div class="grid">
        <div class="row team">
          <div class="name">Team overall</div>
          <div
            v-for="w in range.weeks"
            :key="w.year + '-' + w.week"
            class="cell"
            :class="cellColor(null, formatIsoWeek(w))"
            :title="cellTitle(null, formatIsoWeek(w))"
          />
        </div>

        <div class="sep"></div>

        <div v-for="r in reports.active" :key="r.id" class="row">
          <div class="name">{{ r.name }}</div>
          <div
            v-for="w in range.weeks"
            :key="w.year + '-' + w.week"
            class="cell"
            :class="cellColor(r.id, formatIsoWeek(w))"
            :title="cellTitle(r.id, formatIsoWeek(w))"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.heatmap { max-width: 100%; }
.page-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; }
h2 { margin: 0; font-size: 20px; }
.range { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--text-dim); }
.range select { background: var(--bg); border: 1px solid var(--border); color: var(--text); padding: 4px 8px; border-radius: 4px; font-size: 12px; }

.grid-wrap { background: #141414; border: 1px solid var(--border); border-radius: 6px; padding: 14px; overflow-x: auto; }
.grid { min-width: 600px; display: flex; flex-direction: column; gap: 2px; }
.row {
  display: grid; grid-template-columns: 140px 1fr;
  gap: 10px; align-items: center;
}
.row > :last-child { display: flex; gap: 2px; }
.name { font-size: 12px; opacity: 0.8; padding-right: 8px; }
.row.team .name { font-weight: 700; color: #93c5fd; }
.sep { height: 1px; background: #333; margin: 6px 0; }
.cell { flex: 1; min-width: 6px; height: 22px; border-radius: 2px; }
.cell.red    { background: #ef4444; }
.cell.yellow { background: #facc15; }
.cell.grey   { background: #6b7280; }
.cell.green  { background: #4ade80; }
.cell.blue   { background: #3b82f6; }
.cell.none   { background: #1a1a1a; }
</style>
```

Note: the `.row > :last-child` selector above is a simplified attempt — if the inner cells don't layout properly, change the row markup to wrap the cells in an explicit `<div class="cells">` and style `.cells` directly. Keep this in mind during implementation.

Actually — to avoid that ambiguity, here's the refined template block to paste (replace the two `.row` blocks in the template with this pattern):

```vue
    <div class="grid-wrap">
      <div class="grid">
        <div class="row team">
          <div class="name">Team overall</div>
          <div class="cells">
            <div
              v-for="w in range.weeks"
              :key="w.year + '-' + w.week"
              class="cell"
              :class="cellColor(null, formatIsoWeek(w))"
              :title="cellTitle(null, formatIsoWeek(w))"
            />
          </div>
        </div>

        <div class="sep"></div>

        <div v-for="r in reports.active" :key="r.id" class="row">
          <div class="name">{{ r.name }}</div>
          <div class="cells">
            <div
              v-for="w in range.weeks"
              :key="w.year + '-' + w.week"
              class="cell"
              :class="cellColor(r.id, formatIsoWeek(w))"
              :title="cellTitle(r.id, formatIsoWeek(w))"
            />
          </div>
        </div>
      </div>
    </div>
```

And replace `.row > :last-child` in the CSS with:

```css
.cells { display: flex; gap: 2px; flex: 1; }
```

### Step 2: Add the new timeline route

Edit `src/router.ts`. Insert this route in the `routes` array (after the existing `reports` route):

```ts
  { path: "/reports/:id/timeline", name: "report-timeline",
    component: () => import("./views/ReportTimelineView.vue") },
```

The final file should look like:

```ts
import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";
import { useVaultStore } from "./stores/vault";

const routes: RouteRecordRaw[] = [
  { path: "/", redirect: "/capture" },
  { path: "/onboard", name: "onboard", component: () => import("./views/OnboardingView.vue") },
  { path: "/unlock", name: "unlock", component: () => import("./views/UnlockView.vue") },
  { path: "/capture", name: "capture", component: () => import("./views/WeeklyCaptureView.vue") },
  { path: "/reports", name: "reports", component: () => import("./views/ReportsView.vue") },
  { path: "/reports/:id/timeline", name: "report-timeline",
    component: () => import("./views/ReportTimelineView.vue") },
  { path: "/heatmap", name: "heatmap", component: () => import("./views/TeamHeatmapView.vue") },
  { path: "/plan", name: "plan", component: () => import("./views/PlanGeneratorView.vue") },
  { path: "/settings", name: "settings", component: () => import("./views/SettingsView.vue") },
];

const router = createRouter({ history: createWebHistory(), routes });

router.beforeEach(async (to) => {
  const vault = useVaultStore();
  if (vault.status === "loading") {
    await vault.refresh();
  }

  if (vault.status === "needs-setup" && to.name !== "onboard") {
    return { name: "onboard" };
  }
  if (vault.status === "locked" && to.name !== "unlock") {
    return { name: "unlock" };
  }
  if (vault.status === "unlocked" && (to.name === "onboard" || to.name === "unlock")) {
    return { name: "capture" };
  }
});

export default router;
```

- [ ] **Step 3: Full build verification**

```bash
cd /home/net-irmantasci/em-tool
npx vue-tsc --noEmit
npm run test 2>&1 | tail -10
npm run build 2>&1 | tail -8
```

Expected: typecheck clean, all tests pass, Vite build succeeds.

- [ ] **Step 4: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add src/views/TeamHeatmapView.vue src/router.ts
git commit -m "feat(ui): TeamHeatmapView + timeline route registration

Heatmap shows per-report rows + team-overall row with a date-range
selector (13 / 26 / 52 weeks). Cell tooltips carry the notes. Adds
/reports/:id/timeline so clicking a ReportsView row drills into
per-person history."
```

---

## Task 15 — Rebuild, relaunch, smoke test

- [ ] **Step 1: Stop any running Tauri dev processes**

```bash
pkill -f "target/debug/em-tool" 2>/dev/null || true
pkill -f "tauri dev" 2>/dev/null || true
pkill -f "/vite$" 2>/dev/null || true
```

- [ ] **Step 2: Relaunch**

```bash
cd /home/net-irmantasci/em-tool
npm run tauri dev
```

First build after Plan 2 lands is ~30-90s (incremental over Plan 1's compiled artifacts).

- [ ] **Step 3: Manual smoke script**

1. Unlock the vault with the password you set in Plan 1.
2. Go to **Reports** → empty state. Click "Add your first report" → fill name "Alice", cadence = 14 → submit. Should route to her timeline (empty).
3. Back to Reports. Add a second report "Bohdan", cadence = 7.
4. Navigate to **Weekly capture**. Should show both Alice and Bohdan as rows, plus the Team overall row above.
5. Click green for Alice. Click yellow for Bohdan. Click grey for Team overall.
6. Add notes to each by typing and tabbing out. Verify they persist on reload.
7. Use WeekNav's ◀ to go back one week. The grid should load empty for that week (no ratings yet). Add a red for Bohdan in that week.
8. Click "Today" → returns to current week with your original entries intact.
9. Navigate to **Team heatmap**. See the two weeks of data rendered as colored cells. Change range to 13 weeks. Hover a cell — tooltip shows the rating + notes.
10. Back to Reports, click Alice's row → timeline shows her single week + color strip.
11. Lock the vault (wait 15 min, or add an explicit lock via settings in Plan 3). Re-unlock, confirm data persists.

- [ ] **Step 4: Tag completion**

```bash
cd /home/net-irmantasci/em-tool
git tag -a plan2-capture-viewing-complete -m "Plan 2 — reports CRUD, weekly capture, timeline, heatmap"
git push origin main
git push origin plan2-capture-viewing-complete
```

---

## Self-review

- **Spec coverage:**
  - Spec §5 data model → `reports` + `week_ratings` modules touch the two tables Plan 2 cares about. `one_on_one`, `action_item`, `performance_review`, `generated_plan` tables untouched — they're Plan 3's domain.
  - Spec §6.1 Weekly capture → Task 12 ✓ (5-color order, per-report rows, team row, week nav).
  - Spec §6.2 Per-person timeline → Task 13 ✓ (header, stats, color strip, feed). Timeline currently only shows week-rating entries; 1:1 and review entries come in Plan 3.
  - Spec §6.3 Team heatmap → Task 14 ✓ (rows = reports, cols = weeks, team-overall separated, range picker, tooltips).
  - Spec §6.6 Add-report modal → Task 9 ✓ (name, role, start date, cadence, notes).
  - Spec §7.1 Weekly capture flow → end-to-end via Tasks 7, 12 (upsert on swatch click; notes save on blur).
  - Spec §7.3 Adding a report → Tasks 6, 9, 10 end-to-end.

- **Placeholder scan:** No TBD/TODO/vague-language hits. Every step has complete code or a complete command.

- **Type consistency:**
  - Rust `Report` / TS `Report`: field casing handled by `#[serde(rename_all = "camelCase")]`. `active: bool` ↔ `active: boolean`. `createdAt: i64` ↔ `number`.
  - Rust `UpdateReportInput` uses `Option<Option<T>>` for patch semantics — the TS side uses single-level `| null` because the TS side only passes fields it wants to change; the Rust deserializer maps missing keys to outer `None` and `null` keys to `Some(None)`.
  - `WeekRating.color: String` in Rust, `Color` in TS — values constrained by SQL CHECK to match `COLOR_ORDER`.
  - `reportsApi.list(includeArchived?)` default `false` matches Rust's positional param.

- **Spec-level deferrals explicitly out of Plan 2** (will land in Plan 3):
  - 1:1 log, action items, performance reviews — no write surface yet.
  - Plan generator (template + Claude) — no view yet.
  - Settings screen — still a stub.
  - Backup rotation.
  - Stale-1:1 indicator in capture grid (the `cadenceOverdue` placeholder stub above is deliberately inert — needs `one_on_one` data which arrives in Plan 3).

---

## After Plan 2

When the user tags `plan2-capture-viewing-complete`, write **Plan 3: Meetings & plan generator** covering:
- 1:1 logging (agenda, notes, action items)
- Performance review logging
- Template-based plan generator (no Claude yet)
- Claude-powered plan generator (BYOK)
- Settings screen (API key storage, color palette customization, auto-lock config, vault path, change password, restore from backup)
- Backup rotation (7 rolling daily copies in `backups/`)
