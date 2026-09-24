#!/usr/bin/env bash
set -euo pipefail
NAME="${MCP_DISTROBOX_NAME:-mcp}"
IMAGE="${MCP_DISTROBOX_IMAGE:-docker.io/library/debian:bookworm}"
USER_NAME="${SUDO_USER:-$USER}"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
command -v distrobox >/dev/null || { echo "distrobox is required" >&2; exit 1; }
command -v sudo >/dev/null || { echo "sudo is required" >&2; exit 1; }
sudo -v
if ! distrobox list --no-color 2>/dev/null | grep -Eq "(^|[[:space:]])${NAME}([[:space:]]|$)"; then
  distrobox create --root --yes --name "${NAME}" --image "${IMAGE}" --additional-packages "podman fuse-overlayfs uidmap git curl ca-certificates build-essential sudo" --unshare-all
fi
# Give the normal user subordinate IDs for rootless Podman inside the Distrobox.
sudo distrobox enter --root --name "${NAME}" --no-tty -- bash -lc "usermod --add-subuids 10000-65536 '${USER_NAME}' || true; usermod --add-subgids 10000-65536 '${USER_NAME}' || true"
# Build everything as the normal user; the sandbox itself is root.
sudo distrobox enter --root --name "${NAME}" --no-tty -- bash -lc "runuser -u '${USER_NAME}' -- bash -lc 'cd "${ROOT}" && cargo build --release && bash scripts/build-container.sh'"
echo "MCP environment ready."
