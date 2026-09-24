#!/usr/bin/env bash
set -euo pipefail

NAME="${MCP_DISTROBOX_NAME:-ubuntu}"

command -v distrobox >/dev/null || {
  echo "error: distrobox is required" >&2
  exit 1
}

if ! distrobox list --no-color 2>/dev/null | grep -Eq "(^|[[:space:]])${NAME}([[:space:]]|$)"; then
  echo "error: existing Distrobox '${NAME}' was not found." >&2
  echo "This project does not create a Distrobox automatically." >&2
  exit 1
fi

echo ">> using existing Distrobox: ${NAME}"
echo ">> configuring Podman inside ${NAME}"

distrobox enter --name "${NAME}" --no-tty --   bash -lc '
    command -v podman >/dev/null || {
      echo "error: podman is not installed inside the Distrobox." >&2
      exit 1
    }

    mkdir -p "$HOME/.config/containers"
    cat > "$HOME/.config/containers/containers.conf" <<'EOF'
[engine]
cgroup_manager = "cgroupfs"
events_logger = "file"
EOF

    podman info >/dev/null
  '

echo
echo "Distrobox '${NAME}' is ready for the MCP bridge."
echo
echo "Manual shell:"
echo "  distrobox enter ${NAME}"
echo
echo "The MCP bridge is started from the host with:"
echo "  bash scripts/run-mcp-distrobox.sh"
echo
echo "The script does not create or manage ai-agent-lab."
