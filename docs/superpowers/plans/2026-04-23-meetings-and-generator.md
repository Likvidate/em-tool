# EM Tool — Plan 3: Meetings & Plan Generator

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Log 1:1s with action items, log performance reviews, and generate plans (template-based or Claude-powered via a user-supplied API key). Add a minimal Settings screen so the API key can be entered. End state: the capture-and-view tool becomes a full EM thinking tool.

**Architecture:** Three new pure-data Rust modules (`one_on_ones`, `action_items`, `performance_reviews`) follow the same pattern as Plan 2's `reports` and `week_ratings`. A new `plan_generation` module handles both the deterministic template and the HTTP call to Anthropic. A tiny `secure_settings` module wraps the vault key so the API key can be AES-encrypted on top of the already-encrypted SQLCipher DB (defense in depth — key stays only in RAM after decrypt).

**Tech stack:** No new direct deps — `reqwest` was already added in Plan 1's Cargo.toml. Anthropic messages API used directly (no SDK needed).

---

## File structure added by this plan

```
em-tool/
├── src/
│   ├── types/
│   │   ├── one-on-one.ts                 (Task 1)
│   │   ├── action-item.ts                (Task 1)
│   │   ├── performance-review.ts         (Task 1)
│   │   └── generated-plan.ts             (Task 1)
│   ├── stores/
│   │   ├── one-on-ones.ts                (Task 7)
│   │   ├── action-items.ts               (Task 7)
│   │   ├── reviews.ts                    (Task 7)
│   │   └── generated-plans.ts            (Task 7)
│   ├── components/
│   │   ├── LogOneOnOneModal.vue          (Task 8)
│   │   ├── LogReviewModal.vue            (Task 9)
│   │   └── ActionItemList.vue            (Task 10)
│   ├── views/
│   │   ├── ReportTimelineView.vue        (Task 11, extend)
│   │   ├── PlanGeneratorView.vue         (Task 12, replace stub)
│   │   └── SettingsView.vue              (Task 13, replace stub)
│   ├── router.ts                         (Task 12, extend)
│   └── lib/
│       └── invoke.ts                     (Task 6, extend)
└── src-tauri/
    └── src/
        ├── one_on_ones.rs                (Task 2)
        ├── action_items.rs               (Task 3)
        ├── performance_reviews.rs        (Task 4)
        ├── plan_generation.rs            (Task 5)
        ├── secure_settings.rs            (Task 5)
        ├── commands.rs                   (Task 6, extend)
        └── lib.rs                        (Tasks 2-6, extend)
```

---

## Task 1 — TS types

**Files:** Create `src/types/one-on-one.ts`, `action-item.ts`, `performance-review.ts`, `generated-plan.ts`.

- [ ] **Step 1** — `src/types/one-on-one.ts`:

```ts
export interface OneOnOne {
  id: number;
  reportId: number;
  occurredAt: number;       // unix seconds
  agendaMd: string | null;
  notesMd: string | null;
  createdAt: number;
}

export interface NewOneOnOneInput {
  reportId: number;
  occurredAt: number;
  agendaMd?: string | null;
  notesMd?: string | null;
}

export interface UpdateOneOnOneInput {
  id: number;
  occurredAt?: number;
  agendaMd?: string | null;
  notesMd?: string | null;
}
```

- [ ] **Step 2** — `src/types/action-item.ts`:

```ts
export type ActionItemOwner = "me" | "them";

export interface ActionItem {
  id: number;
  oneOnOneId: number | null;
  reportId: number;
  text: string;
  owner: ActionItemOwner;
  dueDate: string | null;    // YYYY-MM-DD
  completedAt: number | null;
  createdAt: number;
}

export interface NewActionItemInput {
  oneOnOneId?: number | null;
  reportId: number;
  text: string;
  owner: ActionItemOwner;
  dueDate?: string | null;
}
```

- [ ] **Step 3** — `src/types/performance-review.ts`:

```ts
export interface PerformanceReview {
  id: number;
  reportId: number;
  period: string;            // e.g. "Q1 2026"
  rating: string | null;
  strengthsMd: string | null;
  devAreasMd: string | null;
  goalsMd: string | null;
  occurredAt: number;
  createdAt: number;
}

export interface NewPerformanceReviewInput {
  reportId: number;
  period: string;
  rating?: string | null;
  strengthsMd?: string | null;
  devAreasMd?: string | null;
  goalsMd?: string | null;
  occurredAt: number;
}
```

- [ ] **Step 4** — `src/types/generated-plan.ts`:

```ts
export type PlanKind = "one_on_one" | "review";
export type PlanSource = "claude" | "template";

export type WindowSpec =
  | { type: "since_last_one_on_one" }
  | { type: "last_n_weeks"; n: number }
  | { type: "since_last_review" }
  | { type: "custom"; from: string; to: string };

export interface GeneratedPlan {
  id: number;
  kind: PlanKind;
  targetReportId: number;
  windowSpec: string;        // JSON string of WindowSpec
  source: PlanSource;
  promptMd: string | null;
  outputMd: string;
  savedToMeetingId: number | null;
  createdAt: number;
}

export interface GeneratePlanInput {
  kind: PlanKind;
  targetReportId: number;
  windowSpec: WindowSpec;
  source: PlanSource;
}
```

- [ ] **Step 5** — typecheck + commit:

```bash
npx vue-tsc --noEmit
git add src/types/
git commit -m "feat(types): OneOnOne / ActionItem / PerformanceReview / GeneratedPlan"
git push origin main
```

---

## Task 2 — Rust `one_on_ones` module

**Files:** Create `src-tauri/src/one_on_ones.rs`; add `mod one_on_ones;` in `lib.rs`.

- [ ] **Step 1** — module (pattern matches `reports.rs` from Plan 2):

```rust
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
    stmt.query_map([report_id], row)?.collect()
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
        create(&c, NewInput {
            report_id: alice, occurred_at: 1000, agenda_md: None, notes_md: None,
        }, 0).unwrap();
        create(&c, NewInput {
            report_id: alice, occurred_at: 3000, agenda_md: Some("newer".into()), notes_md: None,
        }, 0).unwrap();
        create(&c, NewInput {
            report_id: alice, occurred_at: 2000, agenda_md: None, notes_md: None,
        }, 0).unwrap();

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
```

- [ ] **Step 2** — register in `lib.rs` (add `mod one_on_ones;` near other `mod` lines).

- [ ] **Step 3** — test and commit:

```bash
(cd src-tauri && cargo test --lib one_on_ones 2>&1 | tail -15)
git add src-tauri/src/one_on_ones.rs src-tauri/src/lib.rs
git commit -m "feat(rust): one_on_ones module — list/latest/get/create/update/delete + 4 tests"
git push origin main
```

---

## Task 3 — Rust `action_items` module

**Files:** Create `src-tauri/src/action_items.rs`; add `mod action_items;` in `lib.rs`.

- [ ] **Step 1** — module:

```rust
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
    stmt.query_map([one_on_one_id], row)?.collect()
}

pub fn list_by_report(conn: &Connection, report_id: i64) -> rusqlite::Result<Vec<ActionItem>> {
    let sql = format!("{} WHERE report_id = ?1 ORDER BY created_at DESC", SELECT);
    let mut stmt = conn.prepare(&sql)?;
    stmt.query_map([report_id], row)?.collect()
}

pub fn list_open_for_report(conn: &Connection, report_id: i64) -> rusqlite::Result<Vec<ActionItem>> {
    let sql = format!("{} WHERE report_id = ?1 AND completed_at IS NULL ORDER BY due_date ASC, created_at ASC", SELECT);
    let mut stmt = conn.prepare(&sql)?;
    stmt.query_map([report_id], row)?.collect()
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
```

- [ ] **Step 2** — register + test + commit:

```bash
# lib.rs: add `mod action_items;`
(cd src-tauri && cargo test --lib action_items 2>&1 | tail -15)
git add src-tauri/src/action_items.rs src-tauri/src/lib.rs
git commit -m "feat(rust): action_items module — list/create/toggle/delete + 3 tests"
git push origin main
```

---

## Task 4 — Rust `performance_reviews` module

**Files:** Create `src-tauri/src/performance_reviews.rs`; add `mod performance_reviews;` in `lib.rs`.

- [ ] **Step 1** — module (CRUD shape matches `one_on_ones`):

```rust
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
    pub occurred_at: i64,
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<PerformanceReview> {
    Ok(PerformanceReview {
        id: r.get(0)?, report_id: r.get(1)?, period: r.get(2)?, rating: r.get(3)?,
        strengths_md: r.get(4)?, dev_areas_md: r.get(5)?, goals_md: r.get(6)?,
        occurred_at: r.get(7)?, created_at: r.get(8)?,
    })
}

const SELECT: &str =
    "SELECT id, report_id, period, rating, strengths_md, dev_areas_md, goals_md, occurred_at, created_at \
     FROM performance_review";

pub fn list_by_report(conn: &Connection, report_id: i64) -> rusqlite::Result<Vec<PerformanceReview>> {
    let sql = format!("{} WHERE report_id = ?1 ORDER BY occurred_at DESC", SELECT);
    let mut stmt = conn.prepare(&sql)?;
    stmt.query_map([report_id], row)?.collect()
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
        "INSERT INTO performance_review (report_id, period, rating, strengths_md, dev_areas_md, goals_md, occurred_at, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![i.report_id, i.period, i.rating, i.strengths_md, i.dev_areas_md, i.goals_md, i.occurred_at, now],
    )?;
    let id = conn.last_insert_rowid();
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
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
            occurred_at: 1000,
        }, 0).unwrap();
        create(&c, NewInput {
            report_id: alice, period: "H1 2026".into(), rating: Some("Meets".into()),
            strengths_md: None, dev_areas_md: None, goals_md: None,
            occurred_at: 2000,
        }, 0).unwrap();

        let latest = latest_for_report(&c, alice).unwrap().unwrap();
        assert_eq!(latest.period, "H1 2026");
    }
}
```

- [ ] **Step 2** — register + test + commit:

```bash
(cd src-tauri && cargo test --lib performance_reviews 2>&1 | tail -15)
git add src-tauri/src/performance_reviews.rs src-tauri/src/lib.rs
git commit -m "feat(rust): performance_reviews module — list/latest/create/delete + 1 test"
git push origin main
```

---

## Task 5 — Rust plan generation + secure settings

**Files:**
- Create: `src-tauri/src/secure_settings.rs`
- Create: `src-tauri/src/plan_generation.rs`
- Extend: Cargo.toml (add `reqwest` with `rustls-tls` feature — not already present as a full dep)
- Extend: `lib.rs`

- [ ] **Step 1** — add `reqwest` to Cargo.toml `[dependencies]`:

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

Rationale: we already have tokio in the tree. Using rustls (not system openssl) keeps the build portable.

- [ ] **Step 2** — create `src-tauri/src/secure_settings.rs` — AES-GCM wrapper for the API key using the vault's Argon2 key:

```rust
// Store the Anthropic API key encrypted-at-rest in the `setting` table.
// The key used to encrypt is derived from the vault password every unlock;
// we regenerate a per-write random nonce and prepend it to the ciphertext.
//
// For MVP we keep this simple: encrypt with the vault key (SQLCipher already
// protects the DB file, this adds a second layer so raw row dumps won't
// leak the key). If/when a separate Settings key-management UI is added,
// we can pivot to a dedicated key-derivation path.
use rusqlite::Connection;

// MVP: just store the API key as plaintext inside the SQLCipher-encrypted
// DB. The DB is already AES-256 encrypted at rest; a second layer would be
// defense-in-depth but adds complexity we don't need for a single-user
// local app. Revisit if/when the key-storage threat model changes.

pub const ANTHROPIC_KEY_SETTING: &str = "anthropic_api_key";

pub fn get_anthropic_key(conn: &Connection) -> rusqlite::Result<Option<String>> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT value FROM setting WHERE key = ?1",
        [ANTHROPIC_KEY_SETTING],
        |r| r.get::<_, Option<String>>(0),
    ).optional().map(|o| o.flatten())
}

pub fn set_anthropic_key(conn: &Connection, value: Option<&str>, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO setting (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        rusqlite::params![ANTHROPIC_KEY_SETTING, value, now],
    )?;
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
    fn roundtrip_api_key() {
        let c = mem();
        assert!(get_anthropic_key(&c).unwrap().is_none());
        set_anthropic_key(&c, Some("sk-ant-test"), 1000).unwrap();
        assert_eq!(get_anthropic_key(&c).unwrap(), Some("sk-ant-test".into()));
        set_anthropic_key(&c, None, 2000).unwrap();
        assert!(get_anthropic_key(&c).unwrap().is_none());
    }
}
```

- [ ] **Step 3** — create `src-tauri/src/plan_generation.rs`:

```rust
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use crate::{one_on_ones, performance_reviews, reports, week_ratings, action_items, secure_settings};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedPlan {
    pub id: i64,
    pub kind: String,               // "one_on_one" | "review"
    pub target_report_id: i64,
    pub window_spec: String,        // JSON
    pub source: String,             // "claude" | "template"
    pub prompt_md: Option<String>,
    pub output_md: String,
    pub saved_to_meeting_id: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum WindowSpec {
    #[serde(rename = "since_last_one_on_one")]
    SinceLastOneOnOne,
    #[serde(rename = "last_n_weeks")]
    LastNWeeks { n: i64 },
    #[serde(rename = "since_last_review")]
    SinceLastReview,
    #[serde(rename = "custom")]
    Custom { from: String, to: String },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateInput {
    pub kind: String,
    pub target_report_id: i64,
    pub window_spec: WindowSpec,
    pub source: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GenError {
    #[error("report not found")]
    ReportNotFound,
    #[error("no api key configured")]
    NoApiKey,
    #[error("anthropic: {0}")]
    Anthropic(String),
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Gather the context (week ratings, last 1:1, open action items, last review)
/// needed to feed either the template or Claude generator.
struct Context {
    report: reports::Report,
    week_ratings: Vec<week_ratings::WeekRating>,
    latest_one_on_one: Option<one_on_ones::OneOnOne>,
    open_action_items: Vec<action_items::ActionItem>,
    latest_review: Option<performance_reviews::PerformanceReview>,
}

fn collect_context(conn: &Connection, input: &GenerateInput) -> Result<Context, GenError> {
    let report = reports::get(conn, input.target_report_id)?
        .ok_or(GenError::ReportNotFound)?;

    let all = week_ratings::list_by_report(conn, report.id)?;
    let week_ratings: Vec<_> = match &input.window_spec {
        WindowSpec::SinceLastOneOnOne => {
            // Everything since most recent 1:1 (or all of it if none yet).
            let latest = one_on_ones::latest_for_report(conn, report.id)?;
            match latest {
                Some(m) => all.into_iter()
                    .filter(|r| r.created_at >= m.occurred_at)
                    .collect(),
                None => all,
            }
        }
        WindowSpec::LastNWeeks { n } => {
            all.into_iter().rev().take(*n as usize).rev().collect()
        }
        WindowSpec::SinceLastReview => {
            let latest = performance_reviews::latest_for_report(conn, report.id)?;
            match latest {
                Some(rv) => all.into_iter()
                    .filter(|r| r.created_at >= rv.occurred_at)
                    .collect(),
                None => all,
            }
        }
        WindowSpec::Custom { from, to } => {
            all.into_iter().filter(|r| &r.iso_week >= from && &r.iso_week <= to).collect()
        }
    };

    Ok(Context {
        report,
        week_ratings,
        latest_one_on_one: one_on_ones::latest_for_report(conn, input.target_report_id)?,
        open_action_items: action_items::list_open_for_report(conn, input.target_report_id)?,
        latest_review: performance_reviews::latest_for_report(conn, input.target_report_id)?,
    })
}

fn format_context_md(ctx: &Context) -> String {
    let mut s = String::new();
    s.push_str(&format!("## Context for {}\n\n", ctx.report.name));

    s.push_str("### Weekly ratings (in window)\n");
    if ctx.week_ratings.is_empty() {
        s.push_str("_None._\n");
    } else {
        for r in &ctx.week_ratings {
            s.push_str(&format!(
                "- **{}** — `{}`{}\n",
                r.iso_week, r.color,
                r.notes.as_ref().map(|n| format!(": {}", n)).unwrap_or_default()
            ));
        }
    }

    s.push_str("\n### Last 1:1\n");
    match &ctx.latest_one_on_one {
        Some(m) => {
            s.push_str(&format!("- occurred_at (unix): {}\n", m.occurred_at));
            if let Some(a) = &m.agenda_md { s.push_str(&format!("- agenda: {}\n", a)); }
            if let Some(n) = &m.notes_md { s.push_str(&format!("- notes: {}\n", n)); }
        }
        None => s.push_str("_No 1:1 logged yet._\n"),
    }

    s.push_str("\n### Open action items\n");
    if ctx.open_action_items.is_empty() {
        s.push_str("_None._\n");
    } else {
        for a in &ctx.open_action_items {
            s.push_str(&format!(
                "- [{}] {} — owner: {}{}\n",
                if a.completed_at.is_some() { "x" } else { " " },
                a.text, a.owner,
                a.due_date.as_ref().map(|d| format!(" (due {})", d)).unwrap_or_default()
            ));
        }
    }

    s.push_str("\n### Latest performance review\n");
    match &ctx.latest_review {
        Some(r) => {
            s.push_str(&format!("- {}: {}\n", r.period, r.rating.as_deref().unwrap_or("—")));
            if let Some(d) = &r.dev_areas_md { s.push_str(&format!("- dev areas: {}\n", d)); }
        }
        None => s.push_str("_No review on file yet._\n"),
    }

    s
}

fn template_plan(ctx: &Context, kind: &str) -> String {
    let mut s = String::new();
    let heading = if kind == "review" { "Review prep" } else { "1:1 prep" };
    s.push_str(&format!("# {} — {}\n\n", heading, ctx.report.name));

    s.push_str("## Suggested talking points\n");
    let reds: Vec<_> = ctx.week_ratings.iter().filter(|r| r.color == "red").collect();
    let yellows: Vec<_> = ctx.week_ratings.iter().filter(|r| r.color == "yellow").collect();
    let blues: Vec<_> = ctx.week_ratings.iter().filter(|r| r.color == "blue").collect();
    let greens: Vec<_> = ctx.week_ratings.iter().filter(|r| r.color == "green").collect();

    for r in &reds {
        s.push_str(&format!("- 🔴 **{}**: {}\n", r.iso_week, r.notes.as_deref().unwrap_or("(no notes)")));
    }
    for r in &yellows {
        s.push_str(&format!("- 🟡 {}: {}\n", r.iso_week, r.notes.as_deref().unwrap_or("(no notes)")));
    }

    s.push_str("\n## Wins to acknowledge\n");
    if greens.is_empty() && blues.is_empty() {
        s.push_str("_Nothing strongly positive in this window._\n");
    } else {
        for r in &blues { s.push_str(&format!("- 🔵 **{}**: {}\n", r.iso_week, r.notes.as_deref().unwrap_or("(no notes)"))); }
        for r in &greens { s.push_str(&format!("- 🟢 {}: {}\n", r.iso_week, r.notes.as_deref().unwrap_or("(no notes)"))); }
    }

    if !ctx.open_action_items.is_empty() {
        s.push_str("\n## Open action items to follow up on\n");
        for a in &ctx.open_action_items {
            s.push_str(&format!(
                "- {} — owner: {}{}\n",
                a.text, a.owner,
                a.due_date.as_ref().map(|d| format!(" (due {})", d)).unwrap_or_default()
            ));
        }
    }

    if let Some(r) = &ctx.latest_review {
        if let Some(d) = &r.dev_areas_md {
            s.push_str(&format!("\n## Development areas from {}\n{}\n", r.period, d));
        }
    }

    s
}

async fn claude_plan(ctx: &Context, kind: &str, api_key: &str) -> Result<(String, String), GenError> {
    let ctx_md = format_context_md(ctx);
    let task = if kind == "review" {
        "Draft a performance review prep document with sections: Strengths, Development areas, Goals for next cycle."
    } else {
        "Draft a 1:1 agenda with sections: Suggested talking points, Growth / career, Things to listen for. For each bullet, add a 'Why:' line pointing back to the specific notes or colors above."
    };
    let prompt = format!(
        "You are helping an engineering manager prepare for a meeting. Use ONLY the context below — don't invent facts.\n\n{}\n\n---\n\n{}\n",
        ctx_md, task
    );

    let body = serde_json::json!({
        "model": "claude-opus-4-7",
        "max_tokens": 2000,
        "messages": [{"role": "user", "content": prompt}]
    });

    let client = reqwest::Client::new();
    let resp = client.post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| GenError::Anthropic(e.to_string()))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await
        .map_err(|e| GenError::Anthropic(e.to_string()))?;

    if !status.is_success() {
        return Err(GenError::Anthropic(format!("status {}: {}", status, json)));
    }

    let text = json["content"][0]["text"].as_str()
        .ok_or_else(|| GenError::Anthropic(format!("unexpected shape: {}", json)))?
        .to_string();
    Ok((prompt, text))
}

pub async fn generate(
    conn_getter: impl Fn() -> Option<std::sync::MutexGuard<'static, ()>>, // placeholder to avoid lifetime issues
    _: &GenerateInput,
) -> Result<GeneratedPlan, GenError> {
    // NOTE: The caller is responsible for locking the DB — this function is
    // invoked from the commands layer which holds the `AppState` lock while
    // collecting context synchronously, then releases it before the async
    // Claude call.
    let _ = conn_getter;
    unimplemented!("see commands::generate_plan which drives the pieces below")
}

/// Sync portion: gather context + run template generator. Must be called
/// while holding the AppState connection lock.
pub fn generate_sync(conn: &Connection, input: &GenerateInput, now: i64) -> Result<GeneratedPlan, GenError> {
    if input.source != "template" {
        return Err(GenError::Anthropic("use generate_with_claude for source=claude".into()));
    }
    let ctx = collect_context(conn, input)?;
    let output = template_plan(&ctx, &input.kind);
    let window_spec_json = serde_json::to_string(&input.window_spec)?;

    conn.execute(
        "INSERT INTO generated_plan (kind, target_report_id, window_spec, source, prompt_md, output_md, created_at)
         VALUES (?1, ?2, ?3, 'template', NULL, ?4, ?5)",
        rusqlite::params![input.kind, input.target_report_id, window_spec_json, output, now],
    )?;
    let id = conn.last_insert_rowid();
    get_plan(conn, id)?.ok_or(GenError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn get_plan(conn: &Connection, id: i64) -> rusqlite::Result<Option<GeneratedPlan>> {
    conn.query_row(
        "SELECT id, kind, target_report_id, window_spec, source, prompt_md, output_md, saved_to_meeting_id, created_at
         FROM generated_plan WHERE id = ?1",
        [id],
        |r| Ok(GeneratedPlan {
            id: r.get(0)?, kind: r.get(1)?, target_report_id: r.get(2)?,
            window_spec: r.get(3)?, source: r.get(4)?, prompt_md: r.get(5)?,
            output_md: r.get(6)?, saved_to_meeting_id: r.get(7)?, created_at: r.get(8)?,
        }),
    ).optional()
}

pub fn list_plans_for_report(conn: &Connection, report_id: i64) -> rusqlite::Result<Vec<GeneratedPlan>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, target_report_id, window_spec, source, prompt_md, output_md, saved_to_meeting_id, created_at
         FROM generated_plan WHERE target_report_id = ?1 ORDER BY created_at DESC"
    )?;
    stmt.query_map([report_id], |r| Ok(GeneratedPlan {
        id: r.get(0)?, kind: r.get(1)?, target_report_id: r.get(2)?,
        window_spec: r.get(3)?, source: r.get(4)?, prompt_md: r.get(5)?,
        output_md: r.get(6)?, saved_to_meeting_id: r.get(7)?, created_at: r.get(8)?,
    }))?.collect()
}

/// Async portion used by the commands layer: collects context + prompt
/// under lock, releases lock, runs Claude call, inserts the resulting plan.
/// We expose this as two helper functions for the commands file to orchestrate.
pub fn gather_prompt(conn: &Connection, input: &GenerateInput) -> Result<(Context, String), GenError> {
    let ctx = collect_context(conn, input)?;
    let task = if input.kind == "review" {
        "Draft a performance review prep document with sections: Strengths, Development areas, Goals for next cycle."
    } else {
        "Draft a 1:1 agenda with sections: Suggested talking points, Growth / career, Things to listen for. For each bullet, add a 'Why:' line pointing back to the specific notes or colors above."
    };
    let ctx_md = format_context_md(&ctx);
    let prompt = format!(
        "You are helping an engineering manager prepare for a meeting. Use ONLY the context below — don't invent facts.\n\n{}\n\n---\n\n{}\n",
        ctx_md, task
    );
    Ok((ctx, prompt))
}

pub async fn call_claude(api_key: &str, prompt: &str) -> Result<String, GenError> {
    let body = serde_json::json!({
        "model": "claude-opus-4-7",
        "max_tokens": 2000,
        "messages": [{"role": "user", "content": prompt}]
    });

    let client = reqwest::Client::new();
    let resp = client.post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| GenError::Anthropic(e.to_string()))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await
        .map_err(|e| GenError::Anthropic(e.to_string()))?;

    if !status.is_success() {
        return Err(GenError::Anthropic(format!("status {}: {}", status, json)));
    }

    let text = json["content"][0]["text"].as_str()
        .ok_or_else(|| GenError::Anthropic(format!("unexpected shape: {}", json)))?
        .to_string();
    Ok(text)
}

pub fn save_claude_plan(conn: &Connection, input: &GenerateInput, prompt: &str, output: &str, now: i64) -> rusqlite::Result<GeneratedPlan> {
    let window_spec_json = serde_json::to_string(&input.window_spec)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    conn.execute(
        "INSERT INTO generated_plan (kind, target_report_id, window_spec, source, prompt_md, output_md, created_at)
         VALUES (?1, ?2, ?3, 'claude', ?4, ?5, ?6)",
        rusqlite::params![input.kind, input.target_report_id, window_spec_json, prompt, output, now],
    )?;
    let id = conn.last_insert_rowid();
    get_plan(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn attach_to_meeting(conn: &Connection, plan_id: i64, one_on_one_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE generated_plan SET saved_to_meeting_id = ?1 WHERE id = ?2",
        rusqlite::params![one_on_one_id, plan_id],
    )?;
    Ok(())
}

pub fn read_api_key(conn: &Connection) -> rusqlite::Result<Option<String>> {
    secure_settings::get_anthropic_key(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{migrations, reports, week_ratings};

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
    fn template_plan_renders_from_week_ratings() {
        let (c, alice) = setup();
        week_ratings::upsert(&c, week_ratings::UpsertInput {
            report_id: Some(alice), iso_week: "2026-W17".into(),
            color: "green".into(), notes: Some("shipped auth refactor".into()),
        }, 1000).unwrap();

        let input = GenerateInput {
            kind: "one_on_one".into(),
            target_report_id: alice,
            window_spec: WindowSpec::LastNWeeks { n: 4 },
            source: "template".into(),
        };
        let plan = generate_sync(&c, &input, 2000).unwrap();
        assert!(plan.output_md.contains("shipped auth refactor"));
        assert_eq!(plan.source, "template");
    }
}
```

- [ ] **Step 4** — register modules in `lib.rs`: add `mod secure_settings;` and `mod plan_generation;`.

- [ ] **Step 5** — test + commit:

```bash
(cd src-tauri && cargo test --lib secure_settings plan_generation 2>&1 | tail -20)
git add -A
git commit -m "feat(rust): plan_generation (template + Claude) + secure_settings

Template generator uses pure context assembly from weeks/1:1s/actions/
reviews. Claude generator calls Anthropic messages API via reqwest
with rustls. API key stored in the 'setting' table inside SQLCipher —
no second encryption layer for MVP (single-user local app)."
git push origin main
```

---

## Task 6 — Tauri commands + TS invoke

**Files:**
- Extend: `src-tauri/src/commands.rs`
- Extend: `src-tauri/src/lib.rs` (register new handlers)
- Extend: `src/lib/invoke.ts`

- [ ] **Step 1** — append to `commands.rs` (import the new modules at top-of-file if not done yet):

```rust
use crate::{one_on_ones, action_items, performance_reviews, plan_generation, secure_settings};

impl From<plan_generation::GenError> for CommandError {
    fn from(e: plan_generation::GenError) -> Self {
        let code = match &e {
            plan_generation::GenError::ReportNotFound => "not_found",
            plan_generation::GenError::NoApiKey => "no_api_key",
            plan_generation::GenError::Anthropic(_) => "anthropic",
            plan_generation::GenError::Sqlite(_) => "sqlite",
            plan_generation::GenError::Json(_) => "json",
        };
        CommandError { code: code.to_string(), message: e.to_string() }
    }
}

// --- one_on_ones ---

#[tauri::command]
pub fn list_one_on_ones(state: State<AppState>, report_id: i64)
    -> Result<Vec<one_on_ones::OneOnOne>, CommandError> {
    db::with_conn(&state, |c| one_on_ones::list_by_report(c, report_id)).map_err(Into::into)
}

#[tauri::command]
pub fn create_one_on_one(state: State<AppState>, input: one_on_ones::NewInput)
    -> Result<one_on_ones::OneOnOne, CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| one_on_ones::create(c, input, now)).map_err(Into::into)
}

#[tauri::command]
pub fn update_one_on_one(state: State<AppState>, input: one_on_ones::UpdateInput)
    -> Result<one_on_ones::OneOnOne, CommandError> {
    db::with_conn(&state, |c| one_on_ones::update(c, input)).map_err(Into::into)
}

#[tauri::command]
pub fn delete_one_on_one(state: State<AppState>, id: i64) -> Result<(), CommandError> {
    db::with_conn(&state, |c| one_on_ones::delete(c, id)).map_err(Into::into)
}

// --- action_items ---

#[tauri::command]
pub fn list_action_items_by_meeting(state: State<AppState>, one_on_one_id: i64)
    -> Result<Vec<action_items::ActionItem>, CommandError> {
    db::with_conn(&state, |c| action_items::list_by_meeting(c, one_on_one_id)).map_err(Into::into)
}

#[tauri::command]
pub fn list_action_items_by_report(state: State<AppState>, report_id: i64)
    -> Result<Vec<action_items::ActionItem>, CommandError> {
    db::with_conn(&state, |c| action_items::list_by_report(c, report_id)).map_err(Into::into)
}

#[tauri::command]
pub fn list_open_action_items(state: State<AppState>, report_id: i64)
    -> Result<Vec<action_items::ActionItem>, CommandError> {
    db::with_conn(&state, |c| action_items::list_open_for_report(c, report_id)).map_err(Into::into)
}

#[tauri::command]
pub fn create_action_item(state: State<AppState>, input: action_items::NewInput)
    -> Result<action_items::ActionItem, CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| action_items::create(c, input, now)).map_err(Into::into)
}

#[tauri::command]
pub fn toggle_action_item(state: State<AppState>, id: i64)
    -> Result<action_items::ActionItem, CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| action_items::toggle_complete(c, id, now)).map_err(Into::into)
}

#[tauri::command]
pub fn delete_action_item(state: State<AppState>, id: i64) -> Result<(), CommandError> {
    db::with_conn(&state, |c| action_items::delete(c, id)).map_err(Into::into)
}

// --- performance_reviews ---

#[tauri::command]
pub fn list_reviews(state: State<AppState>, report_id: i64)
    -> Result<Vec<performance_reviews::PerformanceReview>, CommandError> {
    db::with_conn(&state, |c| performance_reviews::list_by_report(c, report_id)).map_err(Into::into)
}

#[tauri::command]
pub fn create_review(state: State<AppState>, input: performance_reviews::NewInput)
    -> Result<performance_reviews::PerformanceReview, CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| performance_reviews::create(c, input, now)).map_err(Into::into)
}

#[tauri::command]
pub fn delete_review(state: State<AppState>, id: i64) -> Result<(), CommandError> {
    db::with_conn(&state, |c| performance_reviews::delete(c, id)).map_err(Into::into)
}

// --- plan generation ---

#[tauri::command]
pub fn list_generated_plans(state: State<AppState>, report_id: i64)
    -> Result<Vec<plan_generation::GeneratedPlan>, CommandError> {
    db::with_conn(&state, |c| plan_generation::list_plans_for_report(c, report_id)).map_err(Into::into)
}

#[tauri::command]
pub fn generate_plan_template(state: State<AppState>, input: plan_generation::GenerateInput)
    -> Result<plan_generation::GeneratedPlan, CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| {
        plan_generation::generate_sync(c, &input, now).map_err(|e| match e {
            plan_generation::GenError::Sqlite(s) => s,
            other => rusqlite::Error::InvalidQuery, // promoted below
        })
    }).map_err(|_| CommandError {
        code: "generation_failed".into(),
        message: "template generation failed".into(),
    })
}

#[tauri::command]
pub async fn generate_plan_claude(
    state: State<'_, AppState>,
    input: plan_generation::GenerateInput,
) -> Result<plan_generation::GeneratedPlan, CommandError> {
    // Phase 1 (under lock): read API key + gather prompt
    let (api_key, prompt) = {
        let guard = state.inner.lock().unwrap();
        let conn = guard.connection.as_ref().ok_or(CommandError {
            code: "locked".into(), message: "vault is locked".into(),
        })?;
        let key = plan_generation::read_api_key(conn)
            .map_err(CommandError::from)?
            .ok_or(CommandError { code: "no_api_key".into(), message: "no api key configured".into() })?;
        let (_ctx, prompt) = plan_generation::gather_prompt(conn, &input)
            .map_err(CommandError::from)?;
        (key, prompt)
    };

    // Phase 2 (no lock): async HTTP call
    let output = plan_generation::call_claude(&api_key, &prompt).await
        .map_err(CommandError::from)?;

    // Phase 3 (under lock again): persist result
    let now = now_secs();
    let guard = state.inner.lock().unwrap();
    let conn = guard.connection.as_ref().ok_or(CommandError {
        code: "locked".into(), message: "vault is locked (after claude call)".into(),
    })?;
    plan_generation::save_claude_plan(conn, &input, &prompt, &output, now)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn attach_plan_to_meeting(state: State<AppState>, plan_id: i64, one_on_one_id: i64)
    -> Result<(), CommandError> {
    db::with_conn(&state, |c| plan_generation::attach_to_meeting(c, plan_id, one_on_one_id)).map_err(Into::into)
}

// --- API key settings ---

#[tauri::command]
pub fn get_api_key_set(state: State<AppState>) -> Result<bool, CommandError> {
    db::with_conn(&state, |c| secure_settings::get_anthropic_key(c).map(|o| o.is_some())).map_err(Into::into)
}

#[tauri::command]
pub fn set_api_key(state: State<AppState>, value: Option<String>) -> Result<(), CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| secure_settings::set_anthropic_key(c, value.as_deref(), now)).map_err(Into::into)
}
```

Note: the `generate_plan_template` function above has a rough error mapping. Simpler, cleaner version:

```rust
#[tauri::command]
pub fn generate_plan_template(state: State<AppState>, input: plan_generation::GenerateInput)
    -> Result<plan_generation::GeneratedPlan, CommandError> {
    let now = now_secs();
    let guard = state.inner.lock().unwrap();
    let conn = guard.connection.as_ref().ok_or(CommandError {
        code: "locked".into(), message: "vault is locked".into(),
    })?;
    plan_generation::generate_sync(conn, &input, now).map_err(CommandError::from)
}
```

Use this simpler version instead.

- [ ] **Step 2** — register all new handlers in `lib.rs` invoke_handler (17 new commands).

- [ ] **Step 3** — extend `src/lib/invoke.ts` with:

```ts
import type { OneOnOne, NewOneOnOneInput, UpdateOneOnOneInput } from "../types/one-on-one";
import type { ActionItem, NewActionItemInput } from "../types/action-item";
import type { PerformanceReview, NewPerformanceReviewInput } from "../types/performance-review";
import type { GeneratedPlan, GeneratePlanInput } from "../types/generated-plan";

export const oneOnOnesApi = {
  list: (reportId: number) => invoke<OneOnOne[]>("list_one_on_ones", { reportId }),
  create: (input: NewOneOnOneInput) => invoke<OneOnOne>("create_one_on_one", { input }),
  update: (input: UpdateOneOnOneInput) => invoke<OneOnOne>("update_one_on_one", { input }),
  delete: (id: number) => invoke<void>("delete_one_on_one", { id }),
};

export const actionItemsApi = {
  listByMeeting: (oneOnOneId: number) => invoke<ActionItem[]>("list_action_items_by_meeting", { oneOnOneId }),
  listByReport: (reportId: number) => invoke<ActionItem[]>("list_action_items_by_report", { reportId }),
  listOpen: (reportId: number) => invoke<ActionItem[]>("list_open_action_items", { reportId }),
  create: (input: NewActionItemInput) => invoke<ActionItem>("create_action_item", { input }),
  toggle: (id: number) => invoke<ActionItem>("toggle_action_item", { id }),
  delete: (id: number) => invoke<void>("delete_action_item", { id }),
};

export const reviewsApi = {
  list: (reportId: number) => invoke<PerformanceReview[]>("list_reviews", { reportId }),
  create: (input: NewPerformanceReviewInput) => invoke<PerformanceReview>("create_review", { input }),
  delete: (id: number) => invoke<void>("delete_review", { id }),
};

export const plansApi = {
  list: (reportId: number) => invoke<GeneratedPlan[]>("list_generated_plans", { reportId }),
  generateTemplate: (input: GeneratePlanInput) => invoke<GeneratedPlan>("generate_plan_template", { input }),
  generateClaude: (input: GeneratePlanInput) => invoke<GeneratedPlan>("generate_plan_claude", { input }),
  attachToMeeting: (planId: number, oneOnOneId: number) =>
    invoke<void>("attach_plan_to_meeting", { planId, oneOnOneId }),
};

export const settingsApi = {
  hasApiKey: () => invoke<boolean>("get_api_key_set"),
  setApiKey: (value: string | null) => invoke<void>("set_api_key", { value }),
};
```

- [ ] **Step 4** — typecheck + commit:

```bash
npx vue-tsc --noEmit
(cd src-tauri && cargo build --lib 2>&1 | tail -10)
git add -A
git commit -m "feat: 17 new Tauri commands + TS invoke surfaces for meetings + plans"
git push origin main
```

---

## Task 7 — Pinia stores (one-on-ones, action-items, reviews, generated-plans)

**Files:** Create `src/stores/one-on-ones.ts`, `action-items.ts`, `reviews.ts`, `generated-plans.ts`.

- [ ] **Step 1** — one-on-ones store (pattern follows `reports.ts`):

```ts
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { oneOnOnesApi, InvokeError } from "../lib/invoke";
import type { OneOnOne, NewOneOnOneInput } from "../types/one-on-one";

export const useOneOnOnesStore = defineStore("oneOnOnes", () => {
  const byReport = ref<Record<number, OneOnOne[]>>({});
  const loading = ref(false);
  const lastError = ref<string | null>(null);

  async function loadForReport(reportId: number) {
    loading.value = true;
    lastError.value = null;
    try {
      byReport.value = { ...byReport.value, [reportId]: await oneOnOnesApi.list(reportId) };
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function create(input: NewOneOnOneInput): Promise<OneOnOne> {
    const created = await oneOnOnesApi.create(input);
    const cur = byReport.value[input.reportId] ?? [];
    byReport.value = {
      ...byReport.value,
      [input.reportId]: [created, ...cur].sort((a, b) => b.occurredAt - a.occurredAt),
    };
    return created;
  }

  async function remove(id: number, reportId: number) {
    await oneOnOnesApi.delete(id);
    byReport.value = {
      ...byReport.value,
      [reportId]: (byReport.value[reportId] ?? []).filter((m) => m.id !== id),
    };
  }

  function forReport(reportId: number): OneOnOne[] {
    return byReport.value[reportId] ?? [];
  }

  return { byReport, loading, lastError, loadForReport, create, remove, forReport };
});
```

- [ ] **Step 2** — action-items store (similar shape, keyed by report):

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { actionItemsApi, InvokeError } from "../lib/invoke";
import type { ActionItem, NewActionItemInput } from "../types/action-item";

export const useActionItemsStore = defineStore("actionItems", () => {
  const byReport = ref<Record<number, ActionItem[]>>({});
  const loading = ref(false);
  const lastError = ref<string | null>(null);

  async function loadForReport(reportId: number) {
    loading.value = true;
    try {
      byReport.value = { ...byReport.value, [reportId]: await actionItemsApi.listByReport(reportId) };
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function create(input: NewActionItemInput): Promise<ActionItem> {
    const created = await actionItemsApi.create(input);
    const cur = byReport.value[input.reportId] ?? [];
    byReport.value = { ...byReport.value, [input.reportId]: [created, ...cur] };
    return created;
  }

  async function toggle(id: number, reportId: number) {
    const updated = await actionItemsApi.toggle(id);
    byReport.value = {
      ...byReport.value,
      [reportId]: (byReport.value[reportId] ?? []).map((a) => (a.id === id ? updated : a)),
    };
  }

  async function remove(id: number, reportId: number) {
    await actionItemsApi.delete(id);
    byReport.value = {
      ...byReport.value,
      [reportId]: (byReport.value[reportId] ?? []).filter((a) => a.id !== id),
    };
  }

  function forReport(reportId: number): ActionItem[] {
    return byReport.value[reportId] ?? [];
  }

  return { byReport, loading, lastError, loadForReport, create, toggle, remove, forReport };
});
```

- [ ] **Step 3** — reviews store:

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { reviewsApi, InvokeError } from "../lib/invoke";
import type { PerformanceReview, NewPerformanceReviewInput } from "../types/performance-review";

export const useReviewsStore = defineStore("reviews", () => {
  const byReport = ref<Record<number, PerformanceReview[]>>({});
  const loading = ref(false);
  const lastError = ref<string | null>(null);

  async function loadForReport(reportId: number) {
    loading.value = true;
    try {
      byReport.value = { ...byReport.value, [reportId]: await reviewsApi.list(reportId) };
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function create(input: NewPerformanceReviewInput): Promise<PerformanceReview> {
    const created = await reviewsApi.create(input);
    const cur = byReport.value[input.reportId] ?? [];
    byReport.value = {
      ...byReport.value,
      [input.reportId]: [created, ...cur].sort((a, b) => b.occurredAt - a.occurredAt),
    };
    return created;
  }

  function forReport(reportId: number): PerformanceReview[] {
    return byReport.value[reportId] ?? [];
  }

  return { byReport, loading, lastError, loadForReport, create, forReport };
});
```

- [ ] **Step 4** — generated-plans store:

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { plansApi, InvokeError } from "../lib/invoke";
import type { GeneratedPlan, GeneratePlanInput } from "../types/generated-plan";

export const useGeneratedPlansStore = defineStore("generatedPlans", () => {
  const byReport = ref<Record<number, GeneratedPlan[]>>({});
  const generating = ref(false);
  const lastError = ref<string | null>(null);

  async function loadForReport(reportId: number) {
    byReport.value = { ...byReport.value, [reportId]: await plansApi.list(reportId) };
  }

  async function generate(input: GeneratePlanInput): Promise<GeneratedPlan> {
    generating.value = true;
    lastError.value = null;
    try {
      const fn = input.source === "claude" ? plansApi.generateClaude : plansApi.generateTemplate;
      const plan = await fn(input);
      const cur = byReport.value[input.targetReportId] ?? [];
      byReport.value = { ...byReport.value, [input.targetReportId]: [plan, ...cur] };
      return plan;
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    } finally {
      generating.value = false;
    }
  }

  async function attachToMeeting(planId: number, oneOnOneId: number, reportId: number) {
    await plansApi.attachToMeeting(planId, oneOnOneId);
    byReport.value = {
      ...byReport.value,
      [reportId]: (byReport.value[reportId] ?? []).map((p) =>
        p.id === planId ? { ...p, savedToMeetingId: oneOnOneId } : p,
      ),
    };
  }

  function forReport(reportId: number): GeneratedPlan[] {
    return byReport.value[reportId] ?? [];
  }

  return { byReport, generating, lastError, loadForReport, generate, attachToMeeting, forReport };
});
```

- [ ] **Step 5** — typecheck + commit:

```bash
npx vue-tsc --noEmit
git add src/stores/
git commit -m "feat(ui): Pinia stores for one-on-ones / action items / reviews / plans"
git push origin main
```

---

## Task 8 — `LogOneOnOneModal` component

**File:** Create `src/components/LogOneOnOneModal.vue`.

Shape: modal with `occurred_at` datetime-local, `agendaMd` textarea, `notesMd` textarea, inline "action items" list (add rows, each with text + owner + optional due date). On submit, creates the 1:1 first, then creates each action item with `oneOnOneId` = created 1:1's id.

See `AddReportModal.vue` for modal styling pattern. Omitting full template here for brevity — implementer should follow that pattern with these fields. **Add a note during implementation: datetime input should use `type="datetime-local"` if it works in WebKit, or fall back to two text inputs (YYYY-MM-DD and HH:MM) if broken.** Initial default: now.

### Acceptance
- Opens from the ReportTimelineView "Log 1:1" button
- Emits `created(oneOnOneId)` on success
- Creates action items in a single form flow
- Committed with message: `feat(ui): LogOneOnOneModal with inline action items`

---

## Task 9 — `LogReviewModal` component

**File:** Create `src/components/LogReviewModal.vue`.

Shape: modal with `period` (text, e.g. "Q1 2026"), `rating` (text, e.g. "Exceeds"), `strengthsMd` / `devAreasMd` / `goalsMd` textareas, `occurredAt` datetime (default: now). No special logic — creates a single review.

### Acceptance
- Opens from the ReportTimelineView "Log review" button
- Emits `created(reviewId)` on success
- Committed with message: `feat(ui): LogReviewModal`

---

## Task 10 — `ActionItemList` component

**File:** Create `src/components/ActionItemList.vue`.

Props:
- `reportId: number`
- `items: ActionItem[]`
- `showCompleted?: boolean` (default false — hide completed)

Emits:
- `toggle(id)`
- `delete(id)`

Shape: checkbox + text + owner badge + due-date (if set) + delete icon. Struck-through for completed items.

### Acceptance
- Reusable from the timeline view (future: from the plan-generator view too)
- Committed with message: `feat(ui): ActionItemList component`

---

## Task 11 — Extend `ReportTimelineView`

**File:** Modify `src/views/ReportTimelineView.vue`.

Changes:
- Load 1:1s, action items, and reviews on mount (alongside existing week_ratings)
- Add "Log 1:1" and "Log review" buttons in the header
- Integrate entries into the feed — sort by timestamp DESC, tag each entry with its type (Week / 1:1 / Review)
- Render week entries as before; 1:1 entries show date + agenda preview; review entries show period + rating
- Optional: show open action items in a side panel or above the feed

### Acceptance
- Timeline feed shows mixed entry types chronologically
- Buttons open the LogOneOnOneModal / LogReviewModal
- Committed with message: `feat(ui): timeline shows 1:1s + reviews + action items`

---

## Task 12 — `PlanGeneratorView` + route

**File:** Replace `src/views/PlanGeneratorView.vue` stub. Route `/plan/:reportId?` or `/plan` with a report picker.

Shape:
- Header: report picker (dropdown from `reports.active`), plan-kind toggle (1:1 / Review)
- Context-window pills: Since last 1:1 / Last 4 weeks / Last 8 weeks / Last 12 weeks / Since last review / Custom
- Two buttons: "Generate with Claude ✨" (disabled if no API key set) and "Generate from template"
- Output panel: rendered markdown of the resulting plan. Plain `<pre>` is fine for MVP (markdown renderer can come later)
- Footer actions: Copy, Save-to-1:1 (opens a small dropdown listing the report's recent 1:1s)

Also add to `router.ts`:

```ts
{ path: "/plan/:reportId?", name: "plan", component: () => import("./views/PlanGeneratorView.vue") },
```

### Acceptance
- Can generate a template plan with no API key set
- Can generate a Claude plan when key is present
- Output persists and is listed in the plan history
- Committed with message: `feat(ui): PlanGeneratorView — template + Claude generation`

---

## Task 13 — `SettingsView`

**File:** Replace `src/views/SettingsView.vue` stub.

Shape:
- "API key" section: masked input (type=password), "Save" button, "Clear" button. Show "Key is configured ✓" or "No key set" based on `settingsApi.hasApiKey()` on mount
- "Vault" section: readonly display of the vault path (read from a new `vault_path` Tauri command — OR skip and just describe it), "Change password" button (opens a small sub-modal with old + new + confirm fields)
- "About" section: small blurb about the app + version

For MVP the "Change password" button can be TODO-marked with a note — the `vault::change_password` function exists in Rust but isn't wired to a command yet. Wire it in a small additional Tauri command `change_vault_password(old, new)`.

### Acceptance
- API key save + clear works; the "Generate with Claude" button in PlanGeneratorView enables/disables based on this
- Password change works and the next unlock requires the new password
- Committed with message: `feat(ui): SettingsView — API key + change password`

---

## Task 14 — Rebuild, relaunch, smoke test

- [ ] **Step 1** — stop existing tauri dev, relaunch

- [ ] **Step 2** — manual smoke script:

1. Open Alice's timeline. Click "Log 1:1" → fill agenda + notes + add 2 action items → save. Returns to timeline with the 1:1 visible.
2. Toggle an action item to completed. Verify it renders struck-through.
3. Click "Log review" → Q1 2026, Exceeds, add strengths + dev areas → save. Appears in timeline.
4. Go to Plan generator. Pick Alice. Template source. Generate. Verify the markdown output includes Alice's week colors, the 1:1 note, open action item, and dev-area text.
5. Go to Settings. Enter an Anthropic API key (a real one) → save. Return to Plan generator. "Generate with Claude" button is now enabled.
6. Generate with Claude. Verify the output is a well-structured meeting prep doc that references the actual context.
7. Save the plan to the 1:1 record. Verify the 1:1's agenda_md now contains the plan text.

- [ ] **Step 3** — tag:

```bash
git tag -a plan3-meetings-generator-complete -m "Plan 3 — meetings logging + plan generator (template + Claude) + minimal settings"
git push origin main
git push origin plan3-meetings-generator-complete
```

---

## Deferrals beyond Plan 3

- Full-screen layout responsiveness (user-flagged earlier)
- Backup rotation (mentioned in spec but deferred)
- Full Settings screen — color palette editing, auto-lock minutes, vault path override, backup restore
- Markdown rendering in plan output (currently raw `<pre>`)
- 1:1 reminders / overdue badges in capture grid
