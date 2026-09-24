#!/usr/bin/env bash
set -euo pipefail
NAME="${1:-ai-agent-lab}"
IMAGE="${NAME}:latest"
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
mkdir -p "${ROOT}/workspace"
podman build -t "${IMAGE}" -f "${ROOT}/Containerfile" "${ROOT}"
podman rm -f "${NAME}" >/dev/null 2>&1 || true
podman run -d --name "${NAME}" --network none --pids-limit 512 --memory 1g --user 0:0 --workdir /workspace -v "${ROOT}/workspace:/workspace:Z" "${IMAGE}" >/dev/null
echo "ready: ${NAME} (uid=$(podman exec "${NAME}" id -u))"
