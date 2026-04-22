# EM Tool — Design Spec

**Date:** 2026-04-22
**Status:** Approved — ready for implementation planning
**Owner:** Irmantas Čiuldis

---

## 1. Problem

Engineering managers keep a personal tracker (typically Excel) to record per-report weekly observations, prep for 1:1s, and accumulate evidence for performance reviews. Jira and GitLab show work; they do not show people. HR tools are too heavy or unavailable. Spreadsheets are fragile, unstructured, and make it painful to retrieve months of history when review season arrives.

This tool replaces the personal spreadsheet with a structured, searchable, encrypted notebook that also **generates** 1:1 and performance-review plans from its own accumulated notes.

## 2. Goals (v1)

- **Weekly capture** of color-coded per-report and team ratings, optionally with free-form notes — under ~60 seconds per week total.
- **Long-range visibility** into each report's history (timeline) and the whole team's history (heatmap).
- **Plan generation** for next 1:1 and next performance review, with either a Claude-API-powered summary or a deterministic template — user picks at generation time.
- **Encrypted local storage** with password auth. No data leaves the laptop except to the Anthropic API when the user explicitly generates with Claude.
- **Runs as a native desktop app** on Linux and Windows, easy to launch.

## 3. Non-goals (v1)

- Jira and GitLab integrations (deferred to v2).
- Performance metrics — delivery velocity, review turnaround, code-review stats, ticket throughput (deferred to v2).
- Mobile access, web access, multi-user, team sharing, cloud sync, cloud backup.
- Mac build.
- Password reset / forgotten-password recovery (would defeat the encryption — not offered).
- HR-system-of-record features (formal reviews, compensation, headcount).

## 4. Architecture

### 4.1 Stack

| Layer | Choice |
|---|---|
| Desktop wrapper | **Tauri 2** |
| Backend (Tauri) | Rust — thin glue layer only (SQLCipher open/close, Anthropic HTTP, file ops, backup rotation) |
| Frontend | **Vue 3** + **TypeScript** + Vite |
| UI component library | **shadcn-vue** (or Naive UI — final choice at implementation time; both work) |
| Storage | **SQLCipher** (AES-256-encrypted SQLite) |
| KDF | **Argon2id** (memory-hard, resistant to GPU/ASIC brute force) |
| HTTP (to Anthropic) | Rust `reqwest` |
| LLM | **Anthropic Claude** via user-supplied API key (BYOK) |

Rationale: Tauri gives a small installer (~10–15 MB), lower memory footprint than Electron, and a narrower attack surface — appropriate for an app storing sensitive notes about real people. The Rust surface is intentionally small (≈ a dozen commands). All app logic lives in Vue/TypeScript.

### 4.2 Vault location

- Linux: `~/.local/share/em-tool/vault.db`
- Windows: `%APPDATA%\em-tool\vault.db`

The path is overridable via Settings. Backup rotation keeps the last 7 daily copies next to the vault in a `backups/` subfolder.

### 4.3 Auth flow

1. **First launch** — onboarding creates the vault: user picks a password; password goes through Argon2id to derive a 256-bit key; SQLCipher DB is created with that key. A prominent warning at setup states there is no password recovery.
2. **Subsequent launches** — prompt for password → derive key → open vault. Wrong password = vault won't open.
3. **Session** — vault stays open while the app runs. Auto-lock after a configurable idle period (default 15 min; minimum 1 min, maximum 8 hours).
4. **Wrong-password rate limit** — 5 failed attempts within 5 minutes triggers a 60-second cooldown before the next attempt is accepted.
5. **Anthropic API key** — encrypted with the same vault key and stored in the `setting` table. Never written plaintext to disk.

## 5. Data model

All entities live in one SQLCipher-encrypted SQLite database. Column types below are SQLite types; `*_at` fields are Unix timestamps (seconds) unless stated otherwise.

### 5.1 `report`

| Column | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| name | TEXT NOT NULL | |
| role | TEXT | e.g. "Senior Backend" |
| start_date | TEXT | ISO date (YYYY-MM-DD); weeks before this render grey in heatmap |
| one_on_one_cadence_days | INTEGER NOT NULL DEFAULT 14 | 7, 14, 21, 30, or custom |
| notes | TEXT | free-form profile notes |
| active | INTEGER NOT NULL DEFAULT 1 | soft-archive flag |
| created_at | INTEGER NOT NULL | |

### 5.2 `week_rating`

| Column | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| report_id | INTEGER | FK to `report`; NULL = team-overall rating |
| iso_week | TEXT NOT NULL | e.g. `"2026-W17"` (ISO 8601 week) |
| color | TEXT NOT NULL | one of: `red`, `yellow`, `grey`, `green`, `blue` |
| notes | TEXT | optional |
| created_at | INTEGER NOT NULL | |
| updated_at | INTEGER NOT NULL | |

Unique on `(report_id, iso_week)` — one rating per person per week, and one team-overall rating per week (report_id IS NULL).

### 5.3 `one_on_one`

| Column | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| report_id | INTEGER NOT NULL | FK to `report` |
| occurred_at | INTEGER NOT NULL | |
| agenda_md | TEXT | markdown, often the saved plan from the generator |
| notes_md | TEXT | meeting notes taken during/after |
| created_at | INTEGER NOT NULL | |

### 5.4 `action_item`

| Column | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| one_on_one_id | INTEGER | FK to `one_on_one`; nullable if item created standalone |
| report_id | INTEGER NOT NULL | FK to `report` |
| text | TEXT NOT NULL | |
| owner | TEXT NOT NULL | `me` or `them` |
| due_date | TEXT | ISO date |
| completed_at | INTEGER | NULL = open |
| created_at | INTEGER NOT NULL | |

### 5.5 `performance_review`

| Column | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| report_id | INTEGER NOT NULL | FK to `report` |
| period | TEXT NOT NULL | e.g. `"Q1 2026"`, `"H1 2026"`, `"Annual 2025"` |
| rating | TEXT | free-text label; org-specific |
| strengths_md | TEXT | |
| dev_areas_md | TEXT | |
| goals_md | TEXT | goals agreed for next cycle |
| occurred_at | INTEGER NOT NULL | |
| created_at | INTEGER NOT NULL | |

### 5.6 `generated_plan`

| Column | Type | Notes |
|---|---|---|
| id | INTEGER PK | |
| kind | TEXT NOT NULL | `one_on_one` or `review` |
| target_report_id | INTEGER NOT NULL | FK to `report` |
| window_spec | TEXT NOT NULL | JSON describing the context window. One of: `{"type":"since_last_one_on_one"}`, `{"type":"last_n_weeks","n":8}`, `{"type":"since_last_review"}`, `{"type":"custom","from":"2026-01-01","to":"2026-04-01"}` |
| source | TEXT NOT NULL | `claude` or `template` |
| prompt_md | TEXT | the prompt sent to Claude (or the template definition used) |
| output_md | TEXT NOT NULL | the plan itself, markdown |
| saved_to_meeting_id | INTEGER | FK to `one_on_one` once the user clicks "Save plan to 1:1 record" |
| created_at | INTEGER NOT NULL | |

### 5.7 `setting`

| Column | Type | Notes |
|---|---|---|
| key | TEXT PK | e.g. `anthropic_api_key_encrypted`, `color_palette`, `auto_lock_minutes`, `theme` |
| value | TEXT | JSON or scalar string |
| updated_at | INTEGER NOT NULL | |

### 5.8 `app_meta`

| Column | Type | Notes |
|---|---|---|
| id | INTEGER PK CHECK (id = 1) | single-row table |
| schema_version | INTEGER NOT NULL | |
| created_at | INTEGER NOT NULL | |
| last_unlocked_at | INTEGER | |

## 6. UI / screens

Sidebar-driven single-window app with five top-level destinations:

### 6.1 Weekly capture (default landing)

Grid layout. One row per report, one row for team-overall. Each row: name + role, 5 color swatches in order **🔴 Red · 🟡 Yellow · ⚪ Grey · 🟢 Green · 🔵 Blue**, optional note field, "last 1:1 Nd ago" indicator (with ⚠ when overdue per cadence).

Week navigation (previous / next / current). Autosaves on each change.

### 6.2 Reports → per-person timeline

Header: name, role, stats row (weeks tracked, color counts, last 1:1, last review).
Color strip: one cell per week as a heatmap bar.
Chronological feed mixing three entry types with tags: **Week** (rating + notes), **1:1** (agenda, notes, action items), **Q/H/Annual Review**.
Buttons: "Generate 1:1 plan", "Generate review plan".

### 6.3 Team heatmap

Rows = reports (plus a team-overall row on top, dashed-separated).
Columns = weeks (configurable range: last 26, last 52, YTD, Q1/Q2/…).
Cells = colors; hover shows notes; click jumps to that week.

### 6.4 Plan generator

Header: target report, next meeting date, selected context window.
Context-window selector (pills): **Since last 1:1** (default, uses cadence), Last 4/8/12 weeks, Since last review, Custom date range.
Left panel: what's being fed in (week ratings + notes, last 1:1, open action items, last review's dev areas) — visible for transparency.
Right panel: generated plan, organized into sections (Suggested talking points, Growth/career, Things to listen for) with a "Why:" trace under each bullet linking back to the source entry.
Action bar: "Generate with Claude" (disabled if no API key), "Generate from template".
Footer: Regenerate, Edit, Copy markdown, **Save plan to 1:1 record**.
Same screen is used for review plans — section headings change to Strengths / Development areas / Goals for next cycle, and the default window becomes "Since last review".

### 6.5 Settings

- Anthropic API key (write-only field; stored encrypted)
- Color palette (labels and hex codes for each of the 5 colors — defaults are editable)
- Auto-lock minutes
- Vault path (read-only display + "Move vault…" action)
- Backups folder + "Restore from backup…" action
- Theme (light / dark / system)
- Danger zone: "Change password" and "Export all data to markdown"

### 6.6 Add-report modal

Fields: name, role, start date on team, 1:1 cadence, optional last-1:1 import date, free-form notes. Triggered from the Reports list header or the empty-state on first launch.

## 7. Key flows

### 7.1 Weekly capture

1. User opens the app, unlocks vault, lands on Weekly Capture showing current ISO week.
2. Clicks color swatch per row → row is autosaved to `week_rating` (upsert on `(report_id, iso_week)`).
3. Optionally types a note → autosaves on blur or debounced while typing.
4. Sets team-overall row the same way.
5. Leaves screen. Done.

### 7.2 Plan generation

1. User opens Plan Generator for a report.
2. Picks context window (defaults to "Since last 1:1").
3. App computes inputs: week ratings in window, last 1:1 (id + agenda + notes), open action items, most recent `performance_review` (for dev-areas context).
4. User clicks **Generate with Claude** or **Generate from template**:
   - **Claude**: app builds prompt with the collected inputs, calls Claude API via Rust backend using the decrypted API key, receives markdown, displays in the right panel. Persists to `generated_plan`.
   - **Template**: deterministic Vue-side assembly — "Here are the last N weeks grouped by color, here are the open action items, here are the Q1 dev areas; structure them into Talking points / Growth / Listen-for sections." Persists to `generated_plan` with `source = "template"`.
5. User can Regenerate (same window, new call), Edit the output, Copy markdown, or **Save plan to 1:1 record** which creates/updates a `one_on_one` row with `agenda_md = output_md` and sets `generated_plan.saved_to_meeting_id`.

### 7.3 Adding a report

1. User clicks "Add report" on the Reports screen.
2. Fills modal, clicks Add.
3. Row inserted into `report`. Weekly Capture grid gains a row for this person starting from the next week after `start_date`. Weeks before `start_date` render grey on the heatmap.

### 7.4 Performance review

1. User opens a report's timeline, clicks "Log performance review".
2. Modal collects period, rating, strengths, dev areas, goals.
3. Inserted into `performance_review`. Appears as a tagged entry in the timeline feed.
4. Future plan generations can use "Since last review" as the context window.

## 8. Error handling

| Situation | Behavior |
|---|---|
| Wrong password | Inline error. After 5 failures in 5 min, 60s cooldown. |
| No Anthropic API key | "Generate with Claude" disabled with tooltip pointing to Settings; template generator always available. |
| Claude API network/429/5xx | Error toast with Retry button; offer "Try template instead" as a one-click fallback. |
| Claude API 401 (bad key) | Modal: "Your Anthropic key isn't working. Update it in Settings." Link directly to Settings. |
| Vault corrupted / schema mismatch | Recovery screen: show most recent backups with dates, let user pick one to restore. If all backups fail, surface the underlying SQLite error for diagnosis. |
| Clock skew on ISO week | Week is computed from local system clock. User can edit any `week_rating.iso_week` via the timeline entry. |
| Duplicate `(report_id, iso_week)` insert attempt | Upsert — later write wins; previous `notes` preserved in an undo snapshot kept in memory for the session. |

## 9. Security considerations

- **Encryption at rest**: SQLCipher with AES-256, key derived via Argon2id (time cost, memory cost, parallelism at the OWASP recommended values for desktop apps in 2026).
- **API key**: encrypted with the vault key; surfaced to the Rust backend only at moment of HTTP call; never logged.
- **Memory hygiene**: the derived key is held in Rust; TypeScript side never sees it. Zeroize on lock / app exit.
- **Tauri security**: CSP locked down; allowlist restricted to required `invoke` commands; no remote-origin loading.
- **Logs**: only structural events (vault opened, plan generated) — never note content or prompts.
- **Backup rotation**: 7 daily copies in `backups/` — all SQLCipher-encrypted (same key).

## 10. Testing (v1 — scaled to a solo personal tool)

- **Unit tests** in Vue/TS: pure logic — week computation, context-window resolution, template plan generation, color palette config.
- **Rust unit tests**: KDF, vault open/close, key-derivation roundtrip, SQLCipher key rotation on password change.
- **Integration smoke test**: create vault → add report → add 2 weeks of ratings → generate template plan → save plan to 1:1 → lock → reopen.
- **No E2E browser automation in v1.** The app is single-user and locally-run; manual verification is sufficient for the MVP. Revisit if distribution expands.

## 11. Deferred to v2+

- **Jira integration** — pull per-report ticket throughput, blocked-ticket counts, cycle time. Adds objective signal alongside the subjective colors.
- **GitLab integration** — MR counts merged, review turnaround, PR size distribution, comment-to-merge ratio.
- **Performance indexes** — derived metrics the user can surface in the plan generator (e.g. "Alice's review turnaround dropped from 6h to 22h this month").
- **Mobile / web access.**
- **Cloud sync** (encrypted, opt-in).
- **Team / multi-user**: never — keeping this a personal tool is a core scope decision.
- **Mac build** — only if demand appears.

## 12. Open questions (none blocking implementation)

- Final UI component library: shadcn-vue vs Naive UI. Pick during implementation based on which one feels right after a small build-out.
- Whether to support multiple vaults (e.g. one per company/manager role). Not planned for v1; would be a small additive feature.
