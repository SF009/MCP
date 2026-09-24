#!/usr/bin/env bash
set -euo pipefail

NAME="${MCP_DISTROBOX_NAME:-ubuntu}"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${MCP_CONFIG:-$ROOT/config.toml}"

# The release archive contains the compiled bridge beside this scripts/
# directory. A source checkout may instead have target/release/.
if [[ -n "${MCP_BINARY:-}" ]]; then
  BIN="${MCP_BINARY}"
elif [[ -x "$ROOT/mcp-terminal-bridge" ]]; then
  BIN="$ROOT/mcp-terminal-bridge"
else
  BIN="$ROOT/target/release/mcp-terminal-bridge"
fi

command -v distrobox >/dev/null || {
  printf '%s\n' "error: distrobox is required" >&2
  exit 1
}

if [[ ! -f "$CONFIG" ]]; then
  printf 'error: config not found: %s\n' "$CONFIG" >&2
  exit 1
fi

if [[ ! -x "$BIN" ]]; then
  printf 'error: MCP binary not found or not executable: %s\n' "$BIN" >&2
  printf '%s\n' "This launcher does not install Cargo or build the release archive automatically." >&2
  exit 1
fi

# Bionic uses MCP stdio, so stdout must contain only JSON-RPC messages.
# Avoid an interactive/login shell and therefore avoid user .bashrc output.
exec distrobox enter --name "${NAME}" --no-tty --   env MCP_CONFIG="${CONFIG}" RUST_LOG="${RUST_LOG:-info}"   "$BIN"
