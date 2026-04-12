mod error;
mod config;
mod db;
mod adapter;
mod llm;
mod agents;
mod events;
mod state;
mod commands;
mod noderepo;

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

            // Get app handle for EventBridge
            let app_handle = app.handle().clone();
            let state = AppState::new(db, config, app_handle);
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
            commands::agents::agents_list_running,
            commands::config_cmd::config_get_openclaw,
            commands::config_cmd::config_set_openclaw,
            commands::config_cmd::config_get_llm,
            commands::config_cmd::config_set_llm,
            commands::config_cmd::config_test_openclaw,
            commands::config_cmd::config_test_llm,
            commands::chat::chat_send,
            commands::chat::chat_history,
            // NodeSpace & WorkNode commands
            commands::noderepo_cmd::nodespace_init,
            commands::noderepo_cmd::nodespace_get,
            commands::noderepo_cmd::nodespace_get_or_create,
            commands::noderepo_cmd::nodespace_update_plan,
            commands::noderepo_cmd::nodespace_get_plan,
            commands::noderepo_cmd::worknode_create_initial,
            commands::noderepo_cmd::worknode_get_current,
            commands::noderepo_cmd::worknode_get,
            commands::noderepo_cmd::worknode_list,
            commands::noderepo_cmd::worknode_get_bugs,
            commands::noderepo_cmd::worknode_add_bug,
            commands::noderepo_cmd::worknode_get_conclusion,
            commands::noderepo_cmd::worknode_get_user_manual,
            commands::noderepo_cmd::worknode_goal_achieved,
            commands::noderepo_cmd::worknode_achieved,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
