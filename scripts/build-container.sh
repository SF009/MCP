#!/usr/bin/env bash
set -euo pipefail
CONTAINER_NAME="${1:-ai-agent-lab}"
IMAGE="${CONTAINER_NAME}:latest"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

echo ">> building image ${IMAGE}"
podman build -t "${IMAGE}" -f "${ROOT}/Containerfile" "${ROOT}"
podman rm -f "${CONTAINER_NAME}" 2>/dev/null || true
mkdir -p "${ROOT}/workspace"

podman run -d \
  --name "${CONTAINER_NAME}" \
  --network none \
  --cap-drop all \
  --security-opt no-new-privileges \
  --pids-limit 512 \
  --memory 1g \
  --workdir /workspace \
  -v "${ROOT}/workspace:/workspace:Z" \
  "${IMAGE}"

echo ">> done: podman exec -it ${CONTAINER_NAME} bash"
