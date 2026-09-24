#!/usr/bin/env bash
set -euo pipefail

NAME="\${MCP_DISTROBOX_NAME:-ubuntu}"

command -v distrobox >/dev/null || { echo "error: distrobox is required" >&2; exit 1; }

if ! distrobox list --no-color 2>/dev/null | grep -Eq "(^|[[:space:]])\${NAME}([[:space:]]|$)"; then
  echo "error: existing Distrobox '\${NAME}' was not found." >&2
  exit 1
fi

echo "Using existing Distrobox: \${NAME}"
echo "No Distrobox or Podman container is created by this project."
