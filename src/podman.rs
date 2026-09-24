use anyhow::{anyhow, Context, Result};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub struct PodmanRunner {
    podman: String,
    container: String,
    shell: String,
    timeout_secs: u64,
    workspace: String,
    auto_start: bool,
}

pub struct ExecResult { pub stdout: String, pub stderr: String, pub exit_code: i32 }

impl PodmanRunner {
    pub fn new(podman: String, container: String, shell: String, timeout_secs: u64, workspace: String, auto_start: bool) -> Self {
        Self { podman, container, shell, timeout_secs, workspace, auto_start }
    }

    async fn container_running(&self) -> Result<bool> {
        let output = Command::new(&self.podman)
            .args(["inspect", "--format", "{{.State.Running}}", &self.container])
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).output().await
            .with_context(|| format!("failed to run {}", self.podman))?;
        if !output.status.success() { return Ok(false); }
        Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
    }

    async fn ensure_running(&self) -> Result<()> {
        if self.container_running().await? { return Ok(()); }
        if !self.auto_start { return Err(anyhow!("container '{}' is not running", self.container)); }
        let output = Command::new(&self.podman)
            .args(["start", &self.container])
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).output().await
            .with_context(|| format!("failed to start container '{}'", self.container))?;
        if !output.status.success() {
            return Err(anyhow!("cannot start container '{}': {}", self.container, String::from_utf8_lossy(&output.stderr).trim()));
        }
        Ok(())
    }

    pub async fn exec(&self, command: &str, workdir: Option<&str>) -> Result<ExecResult> {
        self.ensure_running().await?;
        let cwd = workdir.unwrap_or(&self.workspace);
        let mut cmd = Command::new(&self.podman);
        cmd.arg("exec").arg("-w").arg(cwd).arg(&self.container)
            .arg(&self.shell).arg("-c").arg(command)
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let output = match timeout(Duration::from_secs(self.timeout_secs), cmd.output()).await {
            Ok(res) => res.context("failed to spawn podman exec")?,
            Err(_) => return Ok(ExecResult { stdout: String::new(), stderr: format!("[timeout] command exceeded {}s", self.timeout_secs), exit_code: 124 }),
        };
        Ok(ExecResult {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    pub async fn info(&self) -> Result<serde_json::Value> {
        let output = Command::new(&self.podman)
            .args(["inspect", &self.container])
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).output().await
            .with_context(|| format!("failed to inspect '{}'", self.container))?;
        if !output.status.success() {
            return Err(anyhow!("cannot inspect container '{}': {}", self.container, String::from_utf8_lossy(&output.stderr).trim()));
        }
        serde_json::from_slice(&output.stdout).context("podman inspect returned invalid JSON")
    }
}
