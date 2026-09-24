#!/usr/bin/env bash
set -euo pipefail

NAME="${MCP_DISTROBOX_NAME:-mcp}"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${MCP_CONFIG:-$ROOT/config.toml}"
BIN="${MCP_BINARY:-$ROOT/target/release/mcp-terminal-bridge}"
USER_NAME="${USER}"

command -v distrobox >/dev/null || {
  echo "error: distrobox is required" >&2
  exit 1
}

# The Distrobox must already exist. Do not create or modify the sandbox
# container from the MCP launcher: ai-agent-lab is managed by the user.
if ! distrobox list --root --no-color 2>/dev/null | grep -Eq "(^|[[:space:]])${NAME}([[:space:]]|$)"; then
  echo "error: rootful Distrobox '${NAME}' does not exist." >&2
  echo "Create/configure it once with: bash scripts/setup-distrobox.sh" >&2
  exit 1
fi

if ! distrobox enter --root --name "${NAME}" --no-tty -- true 2>/dev/null; then
  echo "error: cannot enter rootful Distrobox '${NAME}'." >&2
  exit 1
fi

# Build only the MCP binary when needed. Never build, replace, or start
# ai-agent-lab here.
if ! distrobox enter --root --name "${NAME}" --no-tty --   bash -lc "test -x '${BIN}'"; then
  echo ">> MCP binary missing; building release binary"
  distrobox enter --root --name "${NAME}" --no-tty --     bash -lc "cd '${ROOT}' && cargo build --release"
fi

if ! distrobox enter --root --name "${NAME}" --no-tty --   bash -lc "runuser -u '${USER_NAME}' -- podman container exists ai-agent-lab"; then
  echo "error: Podman container 'ai-agent-lab' does not exist." >&2
  echo "Create the container yourself before starting MCP." >&2
  exit 1
fi

if ! distrobox enter --root --name "${NAME}" --no-tty --   bash -lc "runuser -u '${USER_NAME}' -- podman inspect --format '{{.State.Running}}' ai-agent-lab 2>/dev/null | grep -qx true"; then
  echo "error: Podman container 'ai-agent-lab' exists but is not running." >&2
  echo "Start your existing container before starting MCP." >&2
  exit 1
fi

exec distrobox enter --root --name "${NAME}" --no-tty --   bash -lc "exec runuser -u '${USER_NAME}' -- env MCP_CONFIG='${CONFIG}' RUST_LOG='${RUST_LOG:-info}' '${BIN}'"
