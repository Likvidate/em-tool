use chrono::{NaiveDate, Utc};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use crate::{one_on_ones, performance_reviews, reports, week_ratings, action_items, secure_settings};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedPlan {
    pub id: i64,
    pub kind: String,
    pub target_report_id: i64,
    pub window_spec: String,
    pub source: String,
    pub prompt_md: Option<String>,
    pub output_md: String,
    pub saved_to_meeting_id: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WindowSpec {
    SinceLastOneOnOne,
    LastNWeeks { n: i64 },
    SinceLastReview,
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
    #[error("ollama: {0}")]
    Ollama(String),
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct Context {
    pub report: reports::Report,
    pub week_ratings: Vec<week_ratings::WeekRating>,
    pub latest_one_on_one: Option<one_on_ones::OneOnOne>,
    pub open_action_items: Vec<action_items::ActionItem>,
    pub latest_review: Option<performance_reviews::PerformanceReview>,
}

pub fn collect_context(conn: &Connection, input: &GenerateInput) -> Result<Context, GenError> {
    let report = reports::get(conn, input.target_report_id)?
        .ok_or(GenError::ReportNotFound)?;

    let all = week_ratings::list_by_report(conn, report.id)?;
    let week_ratings: Vec<_> = match &input.window_spec {
        WindowSpec::SinceLastOneOnOne => {
            let latest = one_on_ones::latest_for_report(conn, report.id)?;
            match latest {
                Some(m) => all.into_iter().filter(|r| r.created_at >= m.occurred_at).collect(),
                None => all,
            }
        }
        WindowSpec::LastNWeeks { n } => {
            let n = *n as usize;
            if all.len() <= n { all } else { all[all.len() - n..].to_vec() }
        }
        WindowSpec::SinceLastReview => {
            let latest = performance_reviews::latest_for_report(conn, report.id)?;
            match latest {
                Some(rv) => all.into_iter().filter(|r| r.created_at >= rv.occurred_at).collect(),
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

pub fn format_context_md(ctx: &Context) -> String {
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
            if let Some(n) = &r.notes_md { s.push_str(&format!("- post-review reflection: {}\n", n)); }
        }
        None => s.push_str("_No review on file yet._\n"),
    }

    s
}

fn count_trailing(ratings: &[week_ratings::WeekRating], pred: impl Fn(&str) -> bool) -> usize {
    ratings.iter().rev().take_while(|r| pred(&r.color)).count()
}

fn days_overdue(due: &str, today: NaiveDate) -> Option<i64> {
    let d = NaiveDate::parse_from_str(due, "%Y-%m-%d").ok()?;
    let delta = (today - d).num_days();
    if delta > 14 { Some(delta) } else { None }
}

fn is_stopword(t: &str) -> bool {
    matches!(t,
        "the" | "and" | "with" | "this" | "that" | "when" | "where"
        | "them" | "they" | "will" | "your" | "have"
    )
}

fn substantive_tokens(s: &str) -> Vec<String> {
    let lower = s.to_lowercase();
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for ch in lower.chars() {
        if ch.is_ascii_alphanumeric() {
            cur.push(ch);
        } else {
            if !cur.is_empty() {
                if cur.len() >= 5 && !is_stopword(&cur) {
                    out.push(cur.clone());
                }
                cur.clear();
            }
        }
    }
    if !cur.is_empty() && cur.len() >= 5 && !is_stopword(&cur) {
        out.push(cur);
    }
    out
}

fn text_contains_any_token(text: &str, tokens: &[String]) -> bool {
    if tokens.is_empty() { return false; }
    let lower = text.to_lowercase();
    tokens.iter().any(|t| lower.contains(t.as_str()))
}

pub fn template_plan(ctx: &Context, kind: &str) -> String {
    let mut s = String::new();
    let heading = if kind == "review" { "Review prep" } else { "1:1 prep" };
    s.push_str(&format!("# {} — {}\n\n", heading, ctx.report.name));

    let reds: Vec<_> = ctx.week_ratings.iter().filter(|r| r.color == "red").collect();
    let yellows: Vec<_> = ctx.week_ratings.iter().filter(|r| r.color == "yellow").collect();
    let blues: Vec<_> = ctx.week_ratings.iter().filter(|r| r.color == "blue").collect();
    let greens: Vec<_> = ctx.week_ratings.iter().filter(|r| r.color == "green").collect();

    // 1. Wellbeing check — 3+ red weeks in window.
    if reds.len() >= 3 {
        s.push_str(&format!(
            "## 🚨 Wellbeing check\n\
             {} red weeks in this window suggests sustained pressure. Before problem-solving,\n\
             ask how they're feeling overall and whether something outside work is contributing.\n\n",
            reds.len()
        ));
    }

    // 2 & 3. Trend flags (declining OR sustained momentum).
    let trailing_decline = count_trailing(&ctx.week_ratings, |c| c == "yellow" || c == "red");
    let trailing_momentum = count_trailing(&ctx.week_ratings, |c| c == "green" || c == "blue");
    if trailing_decline >= 3 {
        s.push_str(&format!(
            "## 📉 Recent trend\n\
             Last {} consecutive weeks were yellow or red. Worth asking if something changed\n\
             recently — a project, a team dynamic, workload, external factors.\n\n",
            trailing_decline
        ));
    } else if trailing_momentum >= 4 {
        s.push_str(&format!(
            "## 🚀 Sustained momentum\n\
             {} weeks of strong signal. Good time to talk growth — stretch projects, new\n\
             responsibilities, or external visibility (conferences, internal talks, mentorship).\n\n",
            trailing_momentum
        ));
    }

    // 3. Reflection from last 1:1.
    if let Some(m) = &ctx.latest_one_on_one {
        if let Some(notes) = &m.notes_md {
            s.push_str(&format!(
                "## 🔁 From last 1:1\n{}\n\nAnything to follow up on based on that?\n\n",
                notes
            ));
        }
    }

    // 4. Suggested talking points (existing).
    s.push_str("## Suggested talking points\n");
    for r in &reds {
        s.push_str(&format!("- 🔴 **{}**: {}\n", r.iso_week, r.notes.as_deref().unwrap_or("(no notes)")));
    }
    for r in &yellows {
        s.push_str(&format!("- 🟡 {}: {}\n", r.iso_week, r.notes.as_deref().unwrap_or("(no notes)")));
    }

    // 5. Overdue action items — surfaced separately.
    let today = Utc::now().date_naive();
    let overdue: Vec<(&action_items::ActionItem, i64)> = ctx.open_action_items
        .iter()
        .filter_map(|a| a.due_date.as_ref().and_then(|d| days_overdue(d, today)).map(|n| (a, n)))
        .collect();
    if !overdue.is_empty() {
        s.push_str("\n## ⏰ Overdue action items\n");
        for (a, n) in &overdue {
            let due = a.due_date.as_deref().unwrap_or("");
            s.push_str(&format!(
                "- {} — owner: {} (due {}, {} days overdue)\n",
                a.text, a.owner, due, n
            ));
        }
    }

    // 6. Open action items to follow up on (existing — ALL open items).
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

    // 7. Revisit dev areas from review — if not referenced in recent notes.
    if let Some(r) = &ctx.latest_review {
        if let Some(dev) = &r.dev_areas_md {
            let tokens = substantive_tokens(dev);
            if !tokens.is_empty() {
                let mut mentioned = false;
                for wr in &ctx.week_ratings {
                    if let Some(n) = &wr.notes {
                        if text_contains_any_token(n, &tokens) { mentioned = true; break; }
                    }
                }
                if !mentioned {
                    if let Some(m) = &ctx.latest_one_on_one {
                        if let Some(a) = &m.agenda_md {
                            if text_contains_any_token(a, &tokens) { mentioned = true; }
                        }
                        if !mentioned {
                            if let Some(n) = &m.notes_md {
                                if text_contains_any_token(n, &tokens) { mentioned = true; }
                            }
                        }
                    }
                }
                if !mentioned {
                    s.push_str(&format!(
                        "\n## 🎯 Revisit development areas from {}\n\
                         They haven't come up in recent notes. Consider touching on:\n{}\n",
                        r.period, dev
                    ));
                }
            }
        }
    }

    // 8. Wins to acknowledge (existing).
    s.push_str("\n## Wins to acknowledge\n");
    if greens.is_empty() && blues.is_empty() {
        s.push_str("_Nothing strongly positive in this window._\n");
    } else {
        for r in &blues { s.push_str(&format!("- 🔵 **{}**: {}\n", r.iso_week, r.notes.as_deref().unwrap_or("(no notes)"))); }
        for r in &greens { s.push_str(&format!("- 🟢 {}: {}\n", r.iso_week, r.notes.as_deref().unwrap_or("(no notes)"))); }
    }

    // 9. Development areas from review (existing).
    if let Some(r) = &ctx.latest_review {
        if let Some(d) = &r.dev_areas_md {
            s.push_str(&format!("\n## Development areas from {}\n{}\n", r.period, d));
        }
    }

    s
}

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
    let rows = stmt.query_map([report_id], |r| Ok(GeneratedPlan {
        id: r.get(0)?, kind: r.get(1)?, target_report_id: r.get(2)?,
        window_spec: r.get(3)?, source: r.get(4)?, prompt_md: r.get(5)?,
        output_md: r.get(6)?, saved_to_meeting_id: r.get(7)?, created_at: r.get(8)?,
    }))?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn gather_prompt(conn: &Connection, input: &GenerateInput) -> Result<String, GenError> {
    let ctx = collect_context(conn, input)?;
    let task_md = if input.kind == "review" {
        "Draft a performance review prep document with sections: Strengths, Development areas, Goals for next cycle."
    } else {
        "Draft a 1:1 agenda with sections: Suggested talking points, Growth / career, Things to listen for. For each bullet, add a 'Why:' line pointing back to the specific notes or colors above."
    };
    let ctx_md = format_context_md(&ctx);
    let prompt = format!(
        "You are helping an engineering manager prepare for a meeting. Use ONLY the context below — don't invent facts.\n\n{}\n\n---\n\n{}\n",
        ctx_md, task_md
    );
    Ok(prompt)
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

pub fn save_claude_plan(
    conn: &Connection,
    input: &GenerateInput,
    prompt: &str,
    output: &str,
    now: i64,
) -> Result<GeneratedPlan, GenError> {
    let window_spec_json = serde_json::to_string(&input.window_spec)?;
    conn.execute(
        "INSERT INTO generated_plan (kind, target_report_id, window_spec, source, prompt_md, output_md, created_at)
         VALUES (?1, ?2, ?3, 'claude', ?4, ?5, ?6)",
        rusqlite::params![input.kind, input.target_report_id, window_spec_json, prompt, output, now],
    )?;
    let id = conn.last_insert_rowid();
    get_plan(conn, id)?.ok_or(GenError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub async fn call_ollama(base_url: &str, model: &str, prompt: &str) -> Result<String, GenError> {
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "stream": false,
        "keep_alive": 0
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| GenError::Ollama(e.to_string()))?;

    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let resp = client.post(&url)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| GenError::Ollama(format!("request failed: {}", e)))?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await
        .map_err(|e| GenError::Ollama(format!("response parse failed: {}", e)))?;
    if !status.is_success() {
        return Err(GenError::Ollama(format!("status {}: {}", status, json)));
    }
    let text = json["message"]["content"].as_str()
        .ok_or_else(|| GenError::Ollama(format!("unexpected shape: {}", json)))?
        .to_string();
    Ok(text)
}

pub fn save_ollama_plan(
    conn: &Connection,
    input: &GenerateInput,
    prompt: &str,
    output: &str,
    now: i64,
) -> Result<GeneratedPlan, GenError> {
    let window_spec_json = serde_json::to_string(&input.window_spec)?;
    conn.execute(
        "INSERT INTO generated_plan (kind, target_report_id, window_spec, source, prompt_md, output_md, created_at)
         VALUES (?1, ?2, ?3, 'ollama', ?4, ?5, ?6)",
        rusqlite::params![input.kind, input.target_report_id, window_spec_json, prompt, output, now],
    )?;
    let id = conn.last_insert_rowid();
    get_plan(conn, id)?.ok_or(GenError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
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

    #[test]
    fn wellbeing_prompt_fires_on_three_red_weeks() {
        let (c, alice) = setup();
        for (w, color) in [("2026-W14", "red"), ("2026-W15", "red"), ("2026-W16", "red")] {
            week_ratings::upsert(&c, week_ratings::UpsertInput {
                report_id: Some(alice), iso_week: w.into(),
                color: color.into(), notes: None,
            }, 1000).unwrap();
        }
        let input = GenerateInput {
            kind: "one_on_one".into(), target_report_id: alice,
            window_spec: WindowSpec::LastNWeeks { n: 10 },
            source: "template".into(),
        };
        let plan = generate_sync(&c, &input, 2000).unwrap();
        assert!(plan.output_md.contains("Wellbeing check"));
    }

    #[test]
    fn sustained_momentum_fires_on_four_greens() {
        let (c, alice) = setup();
        for (w, color) in [("2026-W13", "green"), ("2026-W14", "green"), ("2026-W15", "green"), ("2026-W16", "green")] {
            week_ratings::upsert(&c, week_ratings::UpsertInput {
                report_id: Some(alice), iso_week: w.into(),
                color: color.into(), notes: None,
            }, 1000).unwrap();
        }
        let input = GenerateInput {
            kind: "one_on_one".into(), target_report_id: alice,
            window_spec: WindowSpec::LastNWeeks { n: 10 },
            source: "template".into(),
        };
        let plan = generate_sync(&c, &input, 2000).unwrap();
        assert!(plan.output_md.contains("Sustained momentum"));
    }

    #[test]
    fn last_1on1_reflection_section_renders_when_notes_md_set() {
        use crate::one_on_ones;
        let (c, alice) = setup();
        one_on_ones::create(&c, one_on_ones::NewInput {
            report_id: alice, occurred_at: 1000,
            agenda_md: Some("talked goals".into()),
            notes_md: Some("Alex was tired. Wants more async days.".into()),
        }, 0).unwrap();
        let input = GenerateInput {
            kind: "one_on_one".into(), target_report_id: alice,
            window_spec: WindowSpec::LastNWeeks { n: 4 },
            source: "template".into(),
        };
        let plan = generate_sync(&c, &input, 2000).unwrap();
        assert!(plan.output_md.contains("From last 1:1"));
        assert!(plan.output_md.contains("async days"));
    }
}
