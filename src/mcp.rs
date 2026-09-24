use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::Config;
use crate::rag::RagStore;
use crate::terminal::{shell_escape, Terminal};

pub const PROTOCOL_VERSION: &str = "2024-11-05";
pub const SERVER_NAME: &str = "mcp-terminal-bridge";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
pub struct Request {
    #[serde(default)] pub jsonrpc: Option<String>,
    #[serde(default)] pub id: Option<Value>,
    pub method: String,
    #[serde(default)] pub params: Value,
}
#[derive(Debug, Serialize)]
pub struct Response {
    pub jsonrpc: &'static str,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")] pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")] pub error: Option<RpcError>,
}
#[derive(Debug, Serialize)]
pub struct RpcError { pub code: i32, pub message: String }
impl Response {
    pub fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self { jsonrpc: "2.0", id, result: None, error: Some(RpcError { code, message: message.into() }) }
    }
    fn ok(id: Value, result: Value) -> Self {
        Self { jsonrpc: "2.0", id, result: Some(result), error: None }
    }
}

pub struct Server { cfg: Config, terminal: Terminal, runner: Arc<crate::podman::PodmanRunner>, rag: Arc<Mutex<RagStore>> }
impl Server {
    pub fn new(cfg: Config, terminal: Terminal, runner: Arc<crate::podman::PodmanRunner>, rag: Arc<Mutex<RagStore>>) -> Self {
        Self { cfg, terminal, runner, rag }
    }
    pub async fn handle(&mut self, req: Request) -> Option<Response> {
        let id = req.id.clone()?;
        let result = match req.method.as_str() {
            "initialize" => Ok(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": SERVER_NAME, "version": SERVER_VERSION}
            })),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(self.tools_list()),
            "tools/call" => self.tools_call(req.params).await,
            other => Err(anyhow!("Method not found: {other}")),
        };
        Some(match result {
            Ok(v) => Response::ok(id, v),
            Err(e) => Response::error(id, -32603, e.to_string()),
        })
    }

    fn tools_list(&self) -> Value {
        json!({"tools":[
            {"name":"terminal_exec","description":"Execute a shell command in the configured Podman container.","inputSchema":{"type":"object","properties":{"command":{"type":"string"},"workdir":{"type":"string"}},"required":["command"]}},
            {"name":"terminal_read","description":"Read a bounded file from the container.","inputSchema":{"type":"object","properties":{"path":{"type":"string"},"max_bytes":{"type":"integer"}},"required":["path"]}},
            {"name":"fs_read","description":"Read a text file in the container workspace.","inputSchema":{"type":"object","properties":{"path":{"type":"string"},"max_bytes":{"type":"integer"}},"required":["path"]}},
            {"name":"fs_write","description":"Write UTF-8 text to a file in the container workspace.","inputSchema":{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}},
            {"name":"fs_list","description":"List a directory in the container workspace.","inputSchema":{"type":"object","properties":{"path":{"type":"string"}}}},
            {"name":"git_status","description":"Return git status for a repository inside the workspace.","inputSchema":{"type":"object","properties":{"path":{"type":"string"}}}},
            {"name":"git_diff","description":"Return git diff for a repository inside the workspace.","inputSchema":{"type":"object","properties":{"path":{"type":"string"},"staged":{"type":"boolean"}}}},
            {"name":"git_commit","description":"Create a git commit with a supplied message.","inputSchema":{"type":"object","properties":{"path":{"type":"string"},"message":{"type":"string"}},"required":["message"]}},
            {"name":"rag_store","description":"Persist a memory/note in local JSONL RAG storage.","inputSchema":{"type":"object","properties":{"text":{"type":"string"},"metadata":{"type":"object"}},"required":["text"]}},
            {"name":"rag_search","description":"Search local RAG memory using embeddings when configured, otherwise keywords.","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"top_k":{"type":"integer"}},"required":["query"]}},
            {"name":"container_info","description":"Return bridge, Podman, and sandbox runtime information.","inputSchema":{"type":"object","properties":{}}}
        ]})
    }

    async fn tools_call(&mut self, params: Value) -> Result<Value> {
        let name = params.get("name").and_then(Value::as_str)
            .ok_or_else(|| anyhow!("missing tool 'name'"))?;
        let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
        let (text, is_error) = match name {
            "terminal_exec" => self.terminal_exec(&args).await?,
            "terminal_read" => self.terminal_read(&args).await?,
            "fs_read" => self.fs_read(&args).await?,
            "fs_write" => self.fs_write(&args).await?,
            "fs_list" => self.fs_list(&args).await?,
            "git_status" => self.git_status(&args).await?,
            "git_diff" => self.git_diff(&args).await?,
            "git_commit" => self.git_commit(&args).await?,
            "rag_store" => self.rag_store(&args).await?,
            "rag_search" => self.rag_search(&args).await?,
            "container_info" => self.container_info().await?,
            other => (format!("Unknown tool: {other}"), true),
        };
        Ok(json!({"content":[{"type":"text","text":text}],"isError":is_error}))
    }

    async fn terminal_exec(&self, a: &Value) -> Result<(String,bool)> {
        let c = required_str(a,"command","terminal_exec")?;
        self.terminal.exec(c, a.get("workdir").and_then(Value::as_str)).await
    }
    async fn terminal_read(&self, a: &Value) -> Result<(String,bool)> {
        let p = required_str(a,"path","terminal_read")?;
        let n = a.get("max_bytes").and_then(Value::as_u64).unwrap_or(65536) as usize;
        self.terminal.read_file(p,n).await
    }
    async fn fs_read(&self, a: &Value) -> Result<(String,bool)> {
        let p = required_str(a,"path","fs_read")?;
        let n = a.get("max_bytes").and_then(Value::as_u64).unwrap_or(65536) as usize;
        self.terminal.read_file(p,n).await
    }
    async fn fs_write(&self, a: &Value) -> Result<(String,bool)> {
        let p = required_str(a,"path","fs_write")?;
        let c = required_str(a,"content","fs_write")?;
        let mut body = c.replace("\nMCP_EOF\n", "\nMCP_EOF_\n");
        body.push('\n');
        let cmd = format!("mkdir -p -- \"$(dirname -- {})\" && cat > {} <<'MCP_EOF'\n{}MCP_EOF",
            shell_escape(p), shell_escape(p), body);
        self.terminal.exec(&cmd,None).await
    }
    async fn fs_list(&self, a: &Value) -> Result<(String,bool)> {
        let p = a.get("path").and_then(Value::as_str).unwrap_or(".");
        self.terminal.exec(&format!("find {} -maxdepth 1 -mindepth 1 -printf '%y %p\\n' | sort", shell_escape(p)),None).await
    }
    async fn git_status(&self, a: &Value) -> Result<(String,bool)> {
        let p = a.get("path").and_then(Value::as_str).unwrap_or(".");
        self.terminal.exec(&format!("git -C {} status --short --branch", shell_escape(p)),None).await
    }
    async fn git_diff(&self, a: &Value) -> Result<(String,bool)> {
        let p = a.get("path").and_then(Value::as_str).unwrap_or(".");
        let flag = if a.get("staged").and_then(Value::as_bool).unwrap_or(false) {"--cached"} else {""};
        self.terminal.exec(&format!("git -C {} diff {} --", shell_escape(p), flag),None).await
    }
    async fn git_commit(&self, a: &Value) -> Result<(String,bool)> {
        let p = a.get("path").and_then(Value::as_str).unwrap_or(".");
        let m = required_str(a,"message","git_commit")?;
        self.terminal.exec(&format!("git -C {} commit -m {}", shell_escape(p), shell_escape(m)),None).await
    }
    async fn rag_store(&self, a: &Value) -> Result<(String,bool)> {
        let text = required_str(a,"text","rag_store")?.to_owned();
        let metadata = a.get("metadata").cloned().unwrap_or_else(|| json!({}));
        let store = self.rag.lock().await;
        if !store.enabled() { return Ok(("RAG is disabled in config.".into(),true)); }
        let e = store.add(text,metadata).await?;
        Ok((format!("stored memory id={} ts={}\n{}",e.id,e.timestamp,trunc(&e.text,500)),false))
    }
    async fn rag_search(&self, a: &Value) -> Result<(String,bool)> {
        let q = required_str(a,"query","rag_search")?;
        let k: usize = a.get("top_k")
            .and_then(Value::as_u64)
            .map(|v| v.min(100) as usize)
            .unwrap_or(self.cfg.rag.top_k.min(100));
        let store = self.rag.lock().await;
        if !store.enabled() { return Ok(("RAG is disabled in config.".into(),true)); }
        let results = store.search(q,k).await?;
        if results.is_empty() { return Ok(("no memories found".into(),false)); }
        let mut out=String::new();
        for (i,(s,e)) in results.iter().enumerate() {
            out.push_str(&format!("[{}] score={:.4} ts={}\n{}\n---\n",i+1,s,e.timestamp,trunc(&e.text,800)));
        }
        Ok((out,false))
    }
    async fn container_info(&self) -> Result<(String,bool)> {
        let inspect = self.runner.info().await?;
        let state = inspect.get(0).cloned().unwrap_or_else(|| json!({}));
        Ok((serde_json::to_string_pretty(&json!({
            "container": self.cfg.container,
            "podman": self.cfg.podman,
            "auto_start": self.cfg.auto_start,
            "shell": self.cfg.shell,
            "workspace": self.cfg.workspace,
            "timeout": self.cfg.timeout,
            "max_output": self.cfg.max_output,
            "container_state": state.get("State").cloned().unwrap_or_else(|| json!({})),
            "container_config": {
                "image": state.get("Config").and_then(|v| v.get("Image")),
                "user": state.get("Config").and_then(|v| v.get("User")),
                "working_dir": state.get("Config").and_then(|v| v.get("WorkingDir"))
            },
            "rag":{"enabled":self.cfg.rag.enabled,"provider":self.cfg.rag.embedding_provider,"path":self.cfg.rag.storage_path},
            "server":{"name":SERVER_NAME,"version":SERVER_VERSION,"protocol":PROTOCOL_VERSION}
        }))?,false))
    }
}
fn required_str<'a>(v:&'a Value,key:&str,tool:&str)->Result<&'a str>{
    v.get(key).and_then(Value::as_str).ok_or_else(|| anyhow!("{tool}: '{key}' is required"))
}
fn trunc(s:&str,n:usize)->String{
    if s.len()<=n{return s.to_owned();}
    let mut e=n;while e>0&&!s.is_char_boundary(e){e-=1;}format!("{}…",&s[..e])
}
