#!/usr/bin/env bash
set -euo pipefail

NAME="${MCP_DISTROBOX_NAME:-mcp}"
PROJECT_DIR="${MCP_PROJECT_DIR:-$HOME/mcp/MCP}"
CONFIG="${MCP_CONFIG:-$PROJECT_DIR/config.toml}"
BIN="${MCP_BINARY:-$PROJECT_DIR/target/release/mcp-terminal-bridge}"

command -v distrobox >/dev/null 2>&1 || {
  echo "error: distrobox is not installed" >&2
  exit 1
}

[[ -f "$CONFIG" ]] || { echo "error: config not found: $CONFIG" >&2; exit 1; }
[[ -x "$BIN" ]] || { echo "error: binary not found/executable: $BIN" >&2; exit 1; }

# Rootful Distrobox is required for the documented nested-Podman setup.
# The MCP process itself is deliberately executed as the normal user so
# Podman remains rootless inside the Distrobox.
USER_NAME="${MCP_USER:-$USER}"

exec sudo distrobox enter --root --name "$NAME" --no-tty --   /bin/bash -lc "exec runuser -u '$USER_NAME' -- env MCP_CONFIG='$CONFIG' RUST_LOG='${RUST_LOG:-info}' '$BIN'"
