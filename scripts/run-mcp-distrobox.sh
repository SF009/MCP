#!/usr/bin/env bash
set -euo pipefail

# Use the existing Distrobox by default. Override with MCP_DISTROBOX_NAME if needed.
NAME="${MCP_DISTROBOX_NAME:-ubuntu}"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${MCP_CONFIG:-$ROOT/config.toml}"
BIN="${MCP_BINARY:-$ROOT/target/release/mcp-terminal-bridge}"

command -v distrobox >/dev/null || {
  echo "error: distrobox is required" >&2
  exit 1
}

# This launcher intentionally uses the existing rootless Distrobox.
# It does not create/manage Distrobox or ai-agent-lab containers.
exec distrobox enter --name "${NAME}" --no-tty --   bash -lc "
    cd '${ROOT}'
    if [ ! -x '${BIN}' ]; then
      cargo build --release >&2
    fi
    exec env MCP_CONFIG='${CONFIG}' RUST_LOG='${RUST_LOG:-info}' '${BIN}'
  "
