use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub struct PodmanRunner {
    container: String,
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
    pub fn new(container: String, shell: String, timeout_secs: u64, workspace: String) -> Self {
        Self { container, shell, timeout_secs, workspace }
    }

    pub async fn exec(&self, command: &str, workdir: Option<&str>) -> Result<ExecResult> {
        let cwd = workdir.unwrap_or(&self.workspace);
        let mut cmd = Command::new("podman");
        cmd.arg("exec").arg("-w").arg(cwd).arg(&self.container)
            .arg(&self.shell).arg("-c").arg(command)
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = match timeout(Duration::from_secs(self.timeout_secs), cmd.output()).await {
            Ok(res) => res.context("failed to spawn podman exec")?,
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
}
