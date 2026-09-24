#!/usr/bin/env bash
set -euo pipefail
NAME="${MCP_DISTROBOX_NAME:-mcp}"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${MCP_CONFIG:-$ROOT/config.toml}"
BIN="${MCP_BINARY:-$ROOT/target/release/mcp-terminal-bridge}"
USER_NAME="${SUDO_USER:-$USER}"
if ! command -v distrobox >/dev/null; then echo "distrobox is required" >&2; exit 1; fi
if ! distrobox list --no-color 2>/dev/null | grep -Eq "(^|[[:space:]])${NAME}([[:space:]]|$)"; then
  bash "${ROOT}/scripts/setup-distrobox.sh"
fi
sudo -v
exec sudo distrobox enter --root --name "${NAME}" --no-tty -- bash -lc "runuser -u '${USER_NAME}' -- env MCP_CONFIG='${CONFIG}' RUST_LOG='${RUST_LOG:-info}' '${BIN}'"
