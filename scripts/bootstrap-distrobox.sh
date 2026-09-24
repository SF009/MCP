#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${CONTAINER_ID:-}" ]]; then
  echo "error: this script must run inside the MCP Distrobox" >&2
  exit 1
fi

command -v podman >/dev/null 2>&1 || {
  echo "error: podman is not installed" >&2
  exit 1
}

USER_NAME="${USER:?USER must be set}"

echo "== configuring subordinate IDs =="
if command -v sudo >/dev/null 2>&1; then
  sudo usermod --add-subuids 10000-65536 "$USER_NAME" || true
  sudo usermod --add-subgids 10000-65536 "$USER_NAME" || true
fi

mkdir -p "$HOME/.config/containers"
cat > "$HOME/.config/containers/containers.conf" <<'EOF'
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

echo "== checking Podman =="
podman info >/dev/null
podman version

echo "== building MCP sandbox =="
bash scripts/build-container.sh

echo
echo "== sandbox status =="
podman ps --filter name=ai-agent-lab --format 'table {{.Names}}\t{{.Status}}\t{{.Image}}'
echo
echo "Distrobox + nested Podman are ready."
