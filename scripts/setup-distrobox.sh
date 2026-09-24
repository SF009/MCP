#!/usr/bin/env bash
set -euo pipefail

NAME="${MCP_DISTROBOX_NAME:-mcp}"
IMAGE="${MCP_DISTROBOX_IMAGE:-docker.io/library/debian:bookworm}"
USER_NAME="${SUDO_USER:-$USER}"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

command -v distrobox >/dev/null || { echo "error: distrobox is required" >&2; exit 1; }
command -v sudo >/dev/null || { echo "error: sudo is required for rootful Distrobox" >&2; exit 1; }

sudo -v

if ! sudo distrobox list --root --no-color 2>/dev/null | grep -Eq "(^|[[:space:]])${NAME}([[:space:]]|$)"; then
  echo ">> creating rootful/unshared Distrobox: ${NAME}"
  sudo distrobox create --root --yes --name "${NAME}" --image "${IMAGE}" --additional-packages "podman fuse-overlayfs uidmap git curl ca-certificates build-essential sudo" --unshare-all
fi

echo ">> configuring rootless Podman inside Distrobox"
sudo distrobox enter --root --name "${NAME}" --no-tty -- bash -lc "
  id '${USER_NAME}' >/dev/null 2>&1 || useradd -m -u $(id -u '${USER_NAME}' 2>/dev/null || echo 1000) -s /bin/bash '${USER_NAME}' || true
  usermod --add-subuids 10000-65536 '${USER_NAME}' || true
  usermod --add-subgids 10000-65536 '${USER_NAME}' || true
  mkdir -p /etc/containers
  cat > /etc/containers/containers.conf <<'EOF'
[containers]
netns="host"
userns="host"
ipcns="host"
utsns="host"
cgroupns="host"
log_driver="k8s-file"

[engine]
cgroup_manager="cgroupfs"
events_logger="file"
EOF
"

echo ">> building MCP and sandbox"
sudo distrobox enter --root --name "${NAME}" --no-tty -- bash -lc   "runuser -u '${USER_NAME}' -- bash -lc 'cd "${ROOT}" && cargo build --release && bash scripts/build-container.sh'"

echo ">> verifying nested Podman"
sudo distrobox enter --root --name "${NAME}" --no-tty -- bash -lc   "runuser -u '${USER_NAME}' -- podman info >/dev/null && runuser -u '${USER_NAME}' -- podman exec ai-agent-lab id"

echo
echo "MCP environment is ready."
echo "Manual shell:"
echo "  distrobox enter --root ${NAME}"
