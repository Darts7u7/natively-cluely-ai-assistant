use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use anyhow::{Result, anyhow};
use std::env;

#[derive(Clone)]
pub struct LLMClient {
    http_client: Client,
    google_api_key: String,
    groq_api_key: String,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContentResponse>,
}

#[derive(Deserialize)]
struct GeminiContentResponse {
    parts: Option<Vec<GeminiPartResponse>>,
}

#[derive(Deserialize)]
struct GeminiPartResponse {
    text: Option<String>,
}

impl LLMClient {
    pub fn new() -> Result<Self> {
        let google_key = env::var("GOOGLE_API_KEY").map_err(|_| anyhow!("GOOGLE_API_KEY not set"))?;
        let groq_key = env::var("GROQ_API_KEY").unwrap_or_default(); // Optional for now
        
        Ok(Self {
            http_client: Client::new(),
            google_api_key: google_key,
            groq_api_key: groq_key,
        })
    }

    pub async fn generate_gemini(&self, prompt: &str, system_instruction: Option<&str>) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            self.google_api_key
        );

        let mut contents = vec![
            GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart { text: prompt.to_string() }],
            }
        ];
        
        // Note: System instruction structure for Gemini 1.5 is slightly different (in 'system_instruction' field), 
        // but for simplicity in this port we can prepend it or use the correct field. 
        // Let's prepend for now to keep JSON simple or implement correct struct later.
        
        let body = json!({
            "contents": contents,
            "generationConfig": {
                "temperature": 0.7,
                "maxOutputTokens": 1024,
            }
        });

        let res = self.http_client.post(&url)
            .json(&body)
            .send()
            .await?;
            
        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow!("Gemini API error: {}", error_text));
        }

        let response: GeminiResponse = res.json().await?;
        
        let text = response.candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .and_then(|p| p.into_iter().next())
            .and_then(|p| p.text)
            .ok_or_else(|| anyhow!("No text in confirmation response"))?;

        Ok(text)
    }

    // Streaming implementation would go here (using reqwest::RequestBuilder::send() -> BytesStream)
}
