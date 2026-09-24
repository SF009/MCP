mod config;
mod mcp;
mod podman;
mod rag;
mod terminal;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing_subscriber::EnvFilter;
use crate::config::Config;
use crate::mcp::{Request, Response, Server};
use crate::podman::PodmanRunner;
use crate::rag::RagStore;
use crate::terminal::Terminal;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_writer(std::io::stderr)
        .init();

    let cfg_path = std::env::var("MCP_CONFIG").unwrap_or_else(|_| "config.toml".into());
    let cfg = Config::load(&cfg_path)?;
    tracing::info!(config=%cfg_path, container=%cfg.container, podman=%cfg.podman, "MCP bridge started");

    let runner = Arc::new(PodmanRunner::new(
        cfg.podman.clone(), cfg.container.clone(), cfg.shell.clone(),
        cfg.timeout, cfg.workspace.clone(), cfg.auto_start
    ));
    let terminal = Terminal::new(runner.clone(), cfg.max_output);
    let rag = Arc::new(Mutex::new(RagStore::new(&cfg.rag).await?));
    let mut server = Server::new(cfg, terminal, runner, rag);

    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() { continue; }
        let req: Request = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                write_response(&mut stdout, &Response::error(serde_json::Value::Null, -32700, format!("Parse error: {e}"))).await?;
                continue;
            }
        };
        if let Some(resp) = server.handle(req).await { write_response(&mut stdout, &resp).await?; }
    }
    Ok(())
}

async fn write_response(stdout: &mut tokio::io::Stdout, resp: &Response) -> Result<()> {
    let s = serde_json::to_string(resp)?;
    stdout.write_all(s.as_bytes()).await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;
    Ok(())
}
