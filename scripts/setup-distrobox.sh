#!/usr/bin/env bash
set -euo pipefail

NAME="${MCP_DISTROBOX_NAME:-mcp}"
IMAGE="${MCP_DISTROBOX_IMAGE:-docker.io/library/debian:bookworm}"
USER_NAME="${USER}"
HOST_UID="$(id -u "${USER_NAME}")"
HOST_GID="$(id -g "${USER_NAME}")"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

command -v distrobox >/dev/null || { echo "error: distrobox is required" >&2; exit 1; }

if ! distrobox list --root --no-color 2>/dev/null | grep -Eq "(^|[[:space:]])${NAME}([[:space:]]|$)"; then
  echo ">> creating rootful/unshared Distrobox: ${NAME}"
  distrobox create --root --yes --name "${NAME}" --image "${IMAGE}"     --additional-packages "podman fuse-overlayfs uidmap git curl ca-certificates build-essential sudo"     --unshare-all
fi

echo ">> configuring rootless Podman inside Distrobox"

distrobox enter --root --name "${NAME}" --no-tty -- bash -lc "
  if ! id '${USER_NAME}' >/dev/null 2>&1; then
    getent group ${HOST_GID} >/dev/null 2>&1 || groupadd -g ${HOST_GID} '${USER_NAME}'
    useradd -m -u ${HOST_UID} -g ${HOST_GID} -s /bin/bash '${USER_NAME}'
  fi
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

echo
echo "Distrobox '${NAME}' is ready."
echo "The existing ai-agent-lab container is NOT created, rebuilt, replaced, or started."
echo
echo "Enter it with:"
echo "  distrobox enter --root ${NAME}"
echo
echo "Then verify your existing sandbox with:"
echo "  podman container exists ai-agent-lab"
echo "  podman inspect --format '{{.State.Running}}' ai-agent-lab"
