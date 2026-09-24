FROM docker.io/library/debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    bash coreutils findutils grep sed gawk git curl wget ca-certificates \
    python3 python3-pip ripgrep jq tree nano vim \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --uid 1000 --shell /bin/bash agent \
    && mkdir -p /workspace \
    && chown -R agent:agent /workspace

USER agent
WORKDIR /workspace
CMD ["sleep", "infinity"]
