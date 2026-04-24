mod kdf;
mod vault;
mod migrations;
mod state;
mod commands;
mod db;
mod reports;
mod week_ratings;
mod one_on_ones;
mod action_items;
mod performance_reviews;
mod secure_settings;
mod plan_generation;

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
            commands::list_reports,
            commands::get_report,
            commands::create_report,
            commands::update_report,
            commands::archive_report,
            commands::delete_report,
            commands::list_week_ratings_by_week,
            commands::list_week_ratings_by_report,
            commands::list_week_ratings_team_overall,
            commands::list_week_ratings_in_range,
            commands::upsert_week_rating,
            commands::delete_week_rating,
            commands::list_one_on_ones,
            commands::create_one_on_one,
            commands::update_one_on_one,
            commands::delete_one_on_one,
            commands::list_action_items_by_meeting,
            commands::list_action_items_by_report,
            commands::list_open_action_items,
            commands::create_action_item,
            commands::toggle_action_item,
            commands::delete_action_item,
            commands::list_reviews,
            commands::create_review,
            commands::update_review,
            commands::delete_review,
            commands::list_generated_plans,
            commands::generate_plan_template,
            commands::generate_plan_claude,
            commands::attach_plan_to_meeting,
            commands::get_api_key_set,
            commands::set_api_key,
            commands::get_ollama_settings,
            commands::set_ollama_url,
            commands::set_ollama_model,
            commands::list_ollama_models,
            commands::generate_plan_ollama,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
