use tauri::{command, State};
use crate::AppState;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse {
    success: bool,
    error: Option<String>,
}

#[command]
pub async fn start_meeting() -> Result<ApiResponse, String> {
    println!("Starting meeting...");
    // Logic to trigger audio capture stream -> STT -> IntelligenceManager
    Ok(ApiResponse { success: true, error: None })
}

#[command]
pub async fn stop_meeting() -> Result<ApiResponse, String> {
    println!("Stopping meeting...");
    Ok(ApiResponse { success: true, error: None })
}

#[command]
pub async fn rag_query(query: String) -> Result<String, String> {
    println!("Querying RAG: {}", query);
    Ok("RAG Result Placeholder".to_string())
}

#[command]
pub async fn get_recent_meetings() -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[command]
pub async fn start_screen_capture() -> Result<(), String> {
    println!("Starting screen capture...");
    Ok(())
}

#[command]
pub async fn what_should_i_say(state: State<'_, AppState>) -> Result<String, String> {
    let manager = {
        let intelligence = state.intelligence.lock().map_err(|e| e.to_string())?;
        intelligence.as_ref().ok_or("Intelligence Manager not initialized".to_string())?.clone()
    };
    
    manager.run_what_should_i_say().await.map_err(|e| e.to_string())
}
