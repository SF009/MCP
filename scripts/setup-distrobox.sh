#!/usr/bin/env bash
set -euo pipefail

NAME="${MCP_DISTROBOX_NAME:-mcp}"
IMAGE="${MCP_DISTROBOX_IMAGE:-docker.io/library/debian:bookworm}"
USER_NAME="${USER:?USER must be set}"

command -v distrobox >/dev/null 2>&1 || {
  echo "error: distrobox is not installed on the host" >&2
  exit 1
}

if distrobox list --no-color 2>/dev/null | grep -Eq "(^|[[:space:]])${NAME}([[:space:]]|$)"; then
  echo "distrobox '${NAME}' already exists"
  exit 0
fi

echo "creating rootful, unshared Distrobox '${NAME}'"
echo "this follows Distrobox's documented pattern for Podman inside Distrobox."

sudo -v
distrobox create --root --yes   --name "${NAME}"   --image "${IMAGE}"   --additional-packages "podman fuse-overlayfs uidmap git curl ca-certificates build-essential sudo"   --unshare-all

echo
echo "Distrobox created."
echo "Enter it with:"
echo "  distrobox enter --root ${NAME}"
echo
echo "Then run:"
echo "  sudo usermod --add-subuids 10000-65536 ${USER_NAME}"
echo "  sudo usermod --add-subgids 10000-65536 ${USER_NAME}"
echo
echo "After that, configure Podman and build the sandbox with:"
echo "  bash scripts/bootstrap-distrobox.sh"
