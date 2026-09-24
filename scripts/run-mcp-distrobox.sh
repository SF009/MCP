#!/usr/bin/env bash
set -euo pipefail

NAME="${MCP_DISTROBOX_NAME:-mcp}"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${MCP_CONFIG:-$ROOT/config.toml}"
BIN="${MCP_BINARY:-$ROOT/target/release/mcp-terminal-bridge}"

command -v distrobox >/dev/null || {
  echo "error: distrobox is required" >&2
  exit 1
}

# Bionic/other MCP clients need a non-interactive stdio process.
# Enter the already-existing rootful Distrobox and execute MCP directly.
# This launcher deliberately does NOT create, inspect, start, stop, rebuild,
# or otherwise manage ai-agent-lab. Podman/container lifecycle is outside
# the launcher's responsibility.
exec distrobox enter --root --name "${NAME}" --no-tty --   bash -lc "
    cd '${ROOT}'
    if [ ! -x '${BIN}' ]; then
      cargo build --release >&2
    fi
    exec env MCP_CONFIG='${CONFIG}' RUST_LOG='${RUST_LOG:-info}' '${BIN}'
  "
