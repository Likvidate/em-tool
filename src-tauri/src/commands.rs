use crate::{migrations, state::AppState, vault};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<vault::VaultError> for CommandError {
    fn from(e: vault::VaultError) -> Self {
        let code = match &e {
            vault::VaultError::InvalidPassword => "invalid_password",
            vault::VaultError::AlreadyExists(_) => "already_exists",
            vault::VaultError::NotFound(_) => "not_found",
            vault::VaultError::MissingSalt => "missing_salt",
            vault::VaultError::Sqlite(_) => "sqlite",
            vault::VaultError::Kdf(_) => "kdf",
            vault::VaultError::Io(_) => "io",
        };
        CommandError { code: code.to_string(), message: e.to_string() }
    }
}

impl From<rusqlite::Error> for CommandError {
    fn from(e: rusqlite::Error) -> Self {
        CommandError { code: "sqlite".to_string(), message: e.to_string() }
    }
}

fn now_secs() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tauri::command]
pub fn vault_exists(state: State<AppState>) -> bool {
    let guard = state.inner.lock().unwrap();
    vault::vault_exists(&guard.db_path)
}

#[tauri::command]
pub fn is_unlocked(state: State<AppState>) -> bool {
    let guard = state.inner.lock().unwrap();
    guard.connection.is_some()
}

#[tauri::command]
pub fn create_vault(state: State<AppState>, password: String) -> Result<(), CommandError> {
    let mut guard = state.inner.lock().unwrap();
    let conn = vault::create(&guard.db_path, &password)?;
    migrations::run(&conn)?;
    guard.connection = Some(conn);
    guard.last_activity_at = now_secs();
    Ok(())
}

#[tauri::command]
pub fn unlock_vault(state: State<AppState>, password: String) -> Result<(), CommandError> {
    let mut guard = state.inner.lock().unwrap();
    let conn = vault::open(&guard.db_path, &password)?;
    migrations::run(&conn)?;
    guard.connection = Some(conn);
    guard.last_activity_at = now_secs();
    Ok(())
}

#[tauri::command]
pub fn lock_vault(state: State<AppState>) {
    let mut guard = state.inner.lock().unwrap();
    guard.connection = None;
    guard.last_activity_at = 0;
}

#[tauri::command]
pub fn touch_activity(state: State<AppState>) {
    let mut guard = state.inner.lock().unwrap();
    guard.last_activity_at = now_secs();
}

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

#[tauri::command]
pub fn delete_report(state: State<AppState>, id: i64) -> Result<(), CommandError> {
    db::with_conn(&state, |c| reports::delete(c, id)).map_err(Into::into)
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

use crate::{one_on_ones, action_items, performance_reviews, plan_generation, secure_settings};

impl From<plan_generation::GenError> for CommandError {
    fn from(e: plan_generation::GenError) -> Self {
        let code = match &e {
            plan_generation::GenError::ReportNotFound => "not_found",
            plan_generation::GenError::NoApiKey => "no_api_key",
            plan_generation::GenError::Anthropic(_) => "anthropic",
            plan_generation::GenError::Ollama(_) => "ollama",
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
pub fn update_review(state: State<AppState>, input: performance_reviews::UpdateInput)
    -> Result<performance_reviews::PerformanceReview, CommandError> {
    db::with_conn(&state, |c| performance_reviews::update(c, input)).map_err(Into::into)
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
    let guard = state.inner.lock().unwrap();
    let conn = guard.connection.as_ref().ok_or(CommandError {
        code: "locked".into(), message: "vault is locked".into(),
    })?;
    plan_generation::generate_sync(conn, &input, now).map_err(CommandError::from)
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
        let prompt = plan_generation::gather_prompt(conn, &input)
            .map_err(CommandError::from)?;
        (key, prompt)
    };

    // Phase 2 (no lock held): async HTTP call
    let output = plan_generation::call_claude(&api_key, &prompt).await
        .map_err(CommandError::from)?;

    // Phase 3 (re-lock): persist result
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

// --- Ollama settings + generation ---

#[derive(Debug, serde::Serialize)]
pub struct OllamaModelInfo {
    pub name: String,
    pub size: i64,
}

#[tauri::command]
pub fn get_ollama_settings(state: State<AppState>)
    -> Result<serde_json::Value, CommandError> {
    db::with_conn(&state, |c| {
        let url = secure_settings::get_ollama_url(c)?;
        let model = secure_settings::get_ollama_model(c)?;
        Ok(serde_json::json!({ "url": url, "model": model }))
    }).map_err(Into::into)
}

#[tauri::command]
pub fn set_ollama_url(state: State<AppState>, value: String) -> Result<(), CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| secure_settings::set_ollama_url(c, &value, now)).map_err(Into::into)
}

#[tauri::command]
pub fn set_ollama_model(state: State<AppState>, value: Option<String>) -> Result<(), CommandError> {
    let now = now_secs();
    db::with_conn(&state, |c| secure_settings::set_ollama_model(c, value.as_deref(), now)).map_err(Into::into)
}

#[tauri::command]
pub async fn list_ollama_models(state: State<'_, AppState>)
    -> Result<Vec<OllamaModelInfo>, CommandError> {
    let url = {
        let guard = state.inner.lock().unwrap();
        let conn = guard.connection.as_ref().ok_or(CommandError {
            code: "locked".into(), message: "vault is locked".into(),
        })?;
        secure_settings::get_ollama_url(conn).map_err(CommandError::from)?
    };

    let endpoint = format!("{}/api/tags", url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| CommandError { code: "ollama_unreachable".into(), message: e.to_string() })?;

    let resp = client.get(&endpoint).send().await
        .map_err(|e| CommandError { code: "ollama_unreachable".into(), message: e.to_string() })?;
    if !resp.status().is_success() {
        return Err(CommandError {
            code: "ollama_error".into(),
            message: format!("status {}", resp.status()),
        });
    }
    let json: serde_json::Value = resp.json().await
        .map_err(|e| CommandError { code: "ollama_error".into(), message: e.to_string() })?;

    let models = json["models"].as_array()
        .ok_or(CommandError { code: "ollama_error".into(), message: "no models field".into() })?
        .iter()
        .filter_map(|m| Some(OllamaModelInfo {
            name: m["name"].as_str()?.to_string(),
            size: m["size"].as_i64().unwrap_or(0),
        }))
        .collect();
    Ok(models)
}

#[tauri::command]
pub async fn generate_plan_ollama(
    state: State<'_, AppState>,
    input: plan_generation::GenerateInput,
) -> Result<plan_generation::GeneratedPlan, CommandError> {
    let (url, model, prompt) = {
        let guard = state.inner.lock().unwrap();
        let conn = guard.connection.as_ref().ok_or(CommandError {
            code: "locked".into(), message: "vault is locked".into(),
        })?;
        let url = secure_settings::get_ollama_url(conn).map_err(CommandError::from)?;
        let model = secure_settings::get_ollama_model(conn).map_err(CommandError::from)?
            .ok_or(CommandError {
                code: "no_model".into(),
                message: "no Ollama model selected in Settings".into(),
            })?;
        let prompt = plan_generation::gather_prompt(conn, &input).map_err(CommandError::from)?;
        (url, model, prompt)
    };

    let output = plan_generation::call_ollama(&url, &model, &prompt).await
        .map_err(CommandError::from)?;

    let now = now_secs();
    let guard = state.inner.lock().unwrap();
    let conn = guard.connection.as_ref().ok_or(CommandError {
        code: "locked".into(), message: "vault is locked (after ollama call)".into(),
    })?;
    plan_generation::save_ollama_plan(conn, &input, &prompt, &output, now).map_err(CommandError::from)
}
