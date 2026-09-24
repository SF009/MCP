use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub container: String,
    #[serde(default = "default_shell")] pub shell: String,
    #[serde(default = "default_timeout")] pub timeout: u64,
    #[serde(default = "default_workspace")] pub workspace: String,
    #[serde(default = "default_max_output")] pub max_output: usize,
    #[serde(default)] pub rag: RagConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RagConfig {
    #[serde(default = "default_true")] pub enabled: bool,
    #[serde(default = "default_rag_path")] pub storage_path: String,
    #[serde(default = "default_top_k")] pub top_k: usize,
    #[serde(default = "default_max_text_bytes")] pub max_text_bytes: usize,
    #[serde(default = "default_embedding_provider")] pub embedding_provider: String,
    #[serde(default = "default_ollama_url")] pub ollama_url: String,
    #[serde(default = "default_ollama_model")] pub ollama_model: String,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            enabled: true, storage_path: default_rag_path(), top_k: default_top_k(),
            max_text_bytes: default_max_text_bytes(), embedding_provider: default_embedding_provider(),
            ollama_url: default_ollama_url(), ollama_model: default_ollama_model(),
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read config file: {path}"))?;
        toml::from_str(&raw).with_context(|| format!("invalid TOML in {path}"))
    }
}
fn default_shell() -> String { "/bin/bash".into() }
fn default_timeout() -> u64 { 300 }
fn default_workspace() -> String { "/workspace".into() }
fn default_max_output() -> usize { 65536 }
fn default_true() -> bool { true }
fn default_rag_path() -> String { "./data/rag.jsonl".into() }
fn default_top_k() -> usize { 5 }
fn default_max_text_bytes() -> usize { 32768 }
fn default_embedding_provider() -> String { "none".into() }
fn default_ollama_url() -> String { "http://127.0.0.1:11434".into() }
fn default_ollama_model() -> String { "nomic-embed-text".into() }
