#!/usr/bin/env bash
set -euo pipefail

NAME="${MCP_DISTROBOX_NAME:-mcp}"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${MCP_CONFIG:-$ROOT/config.toml}"
BIN="${MCP_BINARY:-$ROOT/target/release/mcp-terminal-bridge}"
USER_NAME="${SUDO_USER:-$USER}"

command -v distrobox >/dev/null || { echo "error: distrobox is required" >&2; exit 1; }
command -v sudo >/dev/null || { echo "error: sudo is required for rootful Distrobox" >&2; exit 1; }

if ! sudo distrobox list --root --no-color 2>/dev/null | grep -Eq "(^|[[:space:]])${NAME}([[:space:]]|$)"; then
  bash "${ROOT}/scripts/setup-distrobox.sh"
fi

if ! sudo distrobox enter --root --name "${NAME}" --no-tty -- true 2>/dev/null; then
  echo "error: cannot enter rootful Distrobox '${NAME}'." >&2
  echo "Run once: sudo -v" >&2
  exit 1
fi

if ! sudo distrobox enter --root --name "${NAME}" --no-tty --   bash -lc "runuser -u '${USER_NAME}' -- test -x '${BIN}'"; then
  echo ">> MCP binary missing; rebuilding"
  sudo distrobox enter --root --name "${NAME}" --no-tty --     bash -lc "runuser -u '${USER_NAME}' -- bash -lc 'cd "${ROOT}" && cargo build --release'"
fi

if ! sudo distrobox enter --root --name "${NAME}" --no-tty --   bash -lc "runuser -u '${USER_NAME}' -- podman container exists ai-agent-lab"; then
  echo ">> sandbox missing; creating it"
  sudo distrobox enter --root --name "${NAME}" --no-tty --     bash -lc "runuser -u '${USER_NAME}' -- bash -lc 'cd "${ROOT}" && bash scripts/build-container.sh'"
fi

exec sudo distrobox enter --root --name "${NAME}" --no-tty --   bash -lc "exec runuser -u '${USER_NAME}' -- env MCP_CONFIG='${CONFIG}' RUST_LOG='${RUST_LOG:-info}' '${BIN}'"
