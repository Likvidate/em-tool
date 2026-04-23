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
