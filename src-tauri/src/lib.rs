mod audio;
mod db;
mod rag;
mod speaker;
mod commands;
mod intelligence;
mod llm;

use tauri::Manager;
use std::sync::Mutex;
use intelligence::IntelligenceManager;
use llm::LLMClient;

pub struct AppState {
    pub intelligence: Mutex<Option<IntelligenceManager>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec![])))
        .setup(|app| {
            let llm_client = LLMClient::new().ok(); // Handle error gracefully or panic
            if let Some(client) = llm_client {
                let intelligence = IntelligenceManager::new(app.handle().clone(), client);
                app.manage(AppState {
                    intelligence: Mutex::new(Some(intelligence)),
                });
            } else {
                 app.manage(AppState {
                    intelligence: Mutex::new(None),
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_meeting,
            commands::stop_meeting,
            commands::rag_query,
            commands::get_recent_meetings,
            commands::start_screen_capture,
            commands::what_should_i_say, // NEW
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
