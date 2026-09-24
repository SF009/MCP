use anyhow::Result;
use std::sync::Arc;
use crate::podman::PodmanRunner;

pub struct Terminal {
    runner: Arc<PodmanRunner>,
    max_output: usize,
}
impl Terminal {
    pub fn new(runner: Arc<PodmanRunner>, max_output: usize) -> Self { Self { runner, max_output } }

    pub async fn exec(&self, command: &str, workdir: Option<&str>) -> Result<(String, bool)> {
        let res = self.runner.exec(command, workdir).await?;
        let mut out = String::new();
        if !res.stdout.is_empty() {
            out.push_str("--- stdout ---\n");
            out.push_str(&truncate(&res.stdout, self.max_output));
            out.push('\n');
        }
        if !res.stderr.is_empty() {
            out.push_str("--- stderr ---\n");
            out.push_str(&truncate(&res.stderr, self.max_output));
            out.push('\n');
        }
        out.push_str(&format!("--- exit code: {} ---", res.exit_code));
        Ok((out, res.exit_code != 0))
    }

    pub async fn read_file(&self, path: &str, max_bytes: usize) -> Result<(String, bool)> {
        let n = max_bytes.min(self.max_output.max(1));
        self.exec(&format!("head -c {} -- {}", n, shell_escape(path)), None).await
    }
}
pub fn shell_escape(s: &str) -> String { format!("'{}'", s.replace('\'', "'\\''")) }
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { return s.to_owned(); }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) { end -= 1; }
    format!("{}\n[...truncated {} bytes...]", &s[..end], s.len() - end)
}
