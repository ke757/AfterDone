mod error;
mod config;
mod db;
mod comm;
mod llm;
mod agents;
mod events;
mod state;
mod commands;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize config
            let config = config::load_config().unwrap_or_else(|e| {
                eprintln!("Failed to load config, using defaults: {}", e);
                config::AppConfig::default()
            });

            // Initialize tracing
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.app.log_level))
                )
                .init();

            // Initialize database
            let db_path = config::loader::database_path(&config)
                .unwrap_or_else(|_| std::path::PathBuf::from("rig-your-goal.db"));

            let runtime = tokio::runtime::Handle::current();
            let db = runtime.block_on(async { db::init_db(&db_path).await })
                .expect("Failed to initialize database");

            let state = AppState::new(db, config);
            app.manage(state);

            tracing::info!("RigYourGoal initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::goals::goals_create,
            commands::goals::goals_list,
            commands::goals::goals_get,
            commands::goals::goals_pin,
            commands::goals::goals_update,
            commands::goals::goals_delete,
            commands::goals::goals_tree,
            commands::milestones::milestones_list,
            commands::milestones::milestones_get,
            commands::milestones::milestones_fork,
            commands::skills::skills_list,
            commands::skills::skills_get,
            commands::skills::skills_invoke,
            commands::agents::agents_start,
            commands::agents::agents_stop,
            commands::agents::agents_status,
            commands::config_cmd::config_get_openclaw,
            commands::config_cmd::config_set_openclaw,
            commands::config_cmd::config_get_llm,
            commands::config_cmd::config_set_llm,
            commands::config_cmd::config_test_openclaw,
            commands::config_cmd::config_test_llm,
            commands::chat::chat_send,
            commands::chat::chat_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
