use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use crate::llm::LLMClient;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TranscriptSegment {
    pub speaker: String,
    pub text: String,
    pub timestamp: u64,
    pub is_final: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub role: String, // "interviewer", "user", "assistant"
    pub text: String,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct IntelligenceManager {
    llm_client: LLMClient,
    context: Arc<Mutex<VecDeque<ContextItem>>>,
    app_handle: AppHandle,
}

impl IntelligenceManager {
    pub fn new(app_handle: AppHandle, llm_client: LLMClient) -> Self {
        Self {
            llm_client,
            context: Arc::new(Mutex::new(VecDeque::new())),
            app_handle,
        }
    }

    pub fn add_transcript(&self, segment: TranscriptSegment) {
        if !segment.is_final { return; }
        
        let mut ctx = self.context.lock().unwrap();
        
        // Map speaker to role
        let role = if segment.speaker == "user" { "user" } else { "interviewer" };
        
        ctx.push_back(ContextItem {
            role: role.to_string(),
            text: segment.text.clone(),
            timestamp: segment.timestamp,
        });
        
        // Prune old context (simple count based for now, time-based is better)
        if ctx.len() > 100 {
            ctx.pop_front();
        }
        
        // Trigger generic analysis? (e.g. passive assist)
    }
    
    pub async fn run_what_should_i_say(&self) -> anyhow::Result<String> {
        let ctx_snapshot = {
            let ctx = self.context.lock().unwrap();
            let mut s = String::new();
            for item in ctx.iter() {
                s.push_str(&format!("{}: {}\n", item.role.to_uppercase(), item.text));
            }
            s
        };
        
        // Simplified Prompt
        let prompt = format!(
            "You are an interview assistant. Based on the transcript below, suggest what the user (candidate) should say next.\n\nTRANSCRIPT:\n{}", 
            ctx_snapshot
        );
        
        let response = self.llm_client.generate_gemini(&prompt, None).await?;
        
        // Add to context
        {
            let mut ctx = self.context.lock().unwrap();
            ctx.push_back(ContextItem {
                role: "assistant".to_string(),
                text: response.clone(),
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64,
            });
        }
        
        // Emit event to frontend
        self.app_handle.emit("suggested_answer", &response)?;
        
        Ok(response)
    }
}
