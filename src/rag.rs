use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use crate::config::RagConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub text: String,
    #[serde(default)] pub metadata: serde_json::Value,
    pub timestamp: String,
    #[serde(default)] pub embedding: Option<Vec<f32>>,
}

pub struct RagStore {
    cfg: RagConfig,
    path: PathBuf,
    client: reqwest::Client,
}
impl RagStore {
    pub async fn new(cfg: &RagConfig) -> Result<Self> {
        let path = PathBuf::from(&cfg.storage_path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() { tokio::fs::create_dir_all(parent).await?; }
        }
        Ok(Self { cfg: cfg.clone(), path, client: reqwest::Client::new() })
    }
    pub fn enabled(&self) -> bool { self.cfg.enabled }

    pub async fn add(&self, text: String, metadata: serde_json::Value) -> Result<MemoryEntry> {
        let text = truncate_bytes(&text, self.cfg.max_text_bytes);
        let embedding = self.embed(&text).await.ok().flatten();
        let entry = MemoryEntry {
            id: Uuid::new_v4().to_string(), text, metadata,
            timestamp: Utc::now().to_rfc3339(), embedding,
        };
        let line = serde_json::to_string(&entry)?;
        let mut f = OpenOptions::new().create(true).append(true).open(&self.path).await?;
        f.write_all(line.as_bytes()).await?;
        f.write_all(b"\n").await?;
        f.flush().await?;
        Ok(entry)
    }

    async fn load_all(&self) -> Result<Vec<MemoryEntry>> {
        let data = match tokio::fs::read_to_string(&self.path).await {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        };
        Ok(data.lines().filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str::<MemoryEntry>(l).ok()).collect())
    }

    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<(f32, MemoryEntry)>> {
        let entries = self.load_all().await?;
        if entries.is_empty() { return Ok(vec![]); }
        let query_emb = self.embed(query).await.ok().flatten();
        let mut scored = entries.into_iter().map(|e| {
            let score = match (&query_emb, &e.embedding) {
                (Some(q), Some(v)) if q.len() == v.len() => cosine(q, v),
                _ => keyword_score(query, &e.text),
            };
            (score, e)
        }).collect::<Vec<_>>();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0));
        scored.truncate(top_k.min(100));
        Ok(scored)
    }

    async fn embed(&self, text: &str) -> Result<Option<Vec<f32>>> {
        if self.cfg.embedding_provider != "ollama" { return Ok(None); }
        let url = format!("{}/api/embeddings", self.cfg.ollama_url.trim_end_matches('/'));
        let resp = self.client.post(url).json(&serde_json::json!({
            "model": self.cfg.ollama_model, "prompt": text
        })).send().await?;
        if !resp.status().is_success() {
            tracing::warn!("ollama embedding failed: {}", resp.status());
            return Ok(None);
        }
        let v: serde_json::Value = resp.json().await?;
        Ok(v.get("embedding").and_then(|x| x.as_array()).map(|a|
            a.iter().filter_map(|x| x.as_f64()).map(|x| x as f32).collect()))
    }
}
fn truncate_bytes(s: &str, max: usize) -> String {
    if s.len() <= max { return s.to_owned(); }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) { end -= 1; }
    s[..end].to_owned()
}
fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() { return 0.0; }
    let (mut dot, mut na, mut nb) = (0.0, 0.0, 0.0);
    for i in 0..a.len() { dot += a[i]*b[i]; na += a[i]*a[i]; nb += b[i]*b[i]; }
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na.sqrt() * nb.sqrt()) }
}
fn keyword_score(query: &str, text: &str) -> f32 {
    let tl = text.to_lowercase();
    let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    if terms.is_empty() { return 0.0; }
    terms.iter().filter(|t| tl.contains(t.as_str())).count() as f32 / terms.len() as f32
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn keyword_score_matches_terms() {
        assert!(keyword_score("rust memory", "Rust has memory tools") > 0.9);
        assert_eq!(keyword_score("", "anything"), 0.0);
    }
    #[test] fn cosine_identical_vectors_is_one() {
        assert!((cosine(&[1.0, 2.0], &[1.0, 2.0]) - 1.0).abs() < 1e-5);
    }
}
