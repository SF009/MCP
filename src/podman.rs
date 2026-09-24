use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

/// Direct command runner. The MCP process itself runs inside Distrobox.
pub struct PodmanRunner {
    shell: String,
    timeout_secs: u64,
    workspace: String,
}

pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl PodmanRunner {
    pub fn new(shell: String, timeout_secs: u64, workspace: String) -> Self {
        Self { shell, timeout_secs, workspace }
    }

    pub async fn exec(&self, command: &str, workdir: Option<&str>) -> Result<ExecResult> {
        let cwd = workdir.unwrap_or(&self.workspace);
        let mut cmd = Command::new(&self.shell);
        cmd.arg("-c")
            .arg(command)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = match timeout(Duration::from_secs(self.timeout_secs), cmd.output()).await {
            Ok(res) => res.context("failed to spawn shell")?,
            Err(_) => return Ok(ExecResult {
                stdout: String::new(),
                stderr: format!("[timeout] command exceeded {}s", self.timeout_secs),
                exit_code: 124,
            }),
        };

        Ok(ExecResult {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    pub async fn info(&self) -> Result<serde_json::Value> {
        let result = self.exec(
            "printf '{\"uid\":\"'; id -u; printf '\",\"user\":\"'; id -un; printf '\",\"gid\":\"'; id -g; printf '\",\"cwd\":\"'; pwd; printf '\",\"hostname\":\"'; hostname; printf '\",\"kernel\":\"'; uname -sr; printf '\"}\\n'",
            None,
        ).await?;

        if result.exit_code != 0 {
            return Err(anyhow::anyhow!(result.stderr));
        }

        serde_json::from_str(result.stdout.trim()).context("runtime info command returned invalid JSON")
    }
}
