#!/usr/bin/env bash
set -euo pipefail

NAME="${MCP_DISTROBOX_NAME:-ubuntu}"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${MCP_CONFIG:-$ROOT/config.toml}"

if [[ -n "${MCP_BINARY:-}" ]]; then
  BIN="${MCP_BINARY}"
elif [[ -x "$ROOT/mcp-terminal-bridge" ]]; then
  BIN="$ROOT/mcp-terminal-bridge"
else
  BIN="$ROOT/target/release/mcp-terminal-bridge"
fi

command -v distrobox >/dev/null || {
  echo "error: distrobox is required" >&2
  exit 1
}

[[ -f "$CONFIG" ]] || {
  echo "error: config not found: $CONFIG" >&2
  exit 1
}

[[ -x "$BIN" ]] || {
  echo "error: MCP binary not found or not executable: $BIN" >&2
  exit 1
}

# Bionic runs on the host. The MCP stdio server runs inside the
# already-existing Ubuntu Distrobox and executes tools directly there.
exec distrobox enter --name "$NAME" --no-tty --   env MCP_CONFIG="$CONFIG" RUST_LOG="${RUST_LOG:-info}" "$BIN"
