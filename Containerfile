FROM docker.io/library/debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends bash coreutils findutils grep sed gawk git curl wget ca-certificates python3 python3-pip ripgrep jq tree nano vim sudo procps && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /workspace && chmod 1777 /tmp
USER root
WORKDIR /workspace
CMD ["sleep","infinity"]
