use rusqlite::{params, Connection, Result, Row};
use std::sync::{Arc, Mutex};

pub struct VectorStore {
    conn: Arc<Mutex<Connection>>,
}

impl VectorStore {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }

    pub fn save_chunk(&self, meeting_id: &str, text: &str, embedding: &[f32]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // Convert &[f32] to Vec<u8> (BLOB)
        let blob: Vec<u8> = embedding
            .iter()
            .flat_map(|f| f.to_le_bytes().to_vec())
            .collect();
            
        conn.execute(
            "INSERT INTO chunks (meeting_id, chunk_index, cleaned_text, token_count, embedding) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![meeting_id, 0, text, 0, blob], // Simplified indices
        )?;
        
        Ok(())
    }

    pub fn search_similar(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<(String, f32)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT cleaned_text, embedding FROM chunks WHERE embedding IS NOT NULL")?;
        
        let rows = stmt.query_map([], |row: &Row| -> Result<(String, Vec<f32>)> {
            let text: String = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            
            // Convert blob back to Vec<f32>
            let embedding: Vec<f32> = blob
                .chunks_exact(4)
                .map(|chunk: &[u8]| f32::from_le_bytes(chunk.try_into().unwrap()))
                .collect();
                
            Ok((text, embedding))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (text, embedding) = row?;
            let similarity = cosine_similarity(query_embedding, &embedding);
            results.push((text, similarity));
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results.into_iter().take(limit).collect())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}
