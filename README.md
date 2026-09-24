# MCP Terminal Bridge

A Rust MCP server that exposes terminal, filesystem, Git, container, and optional RAG tools to an external AI model over stdio JSON-RPC.

## Architecture

```
Bionic / AI model (host)
        |
        | MCP stdio / JSON-RPC
        v
mcp-terminal-bridge
        |
        v
rootful + unshared Distrobox: mcp
        |
        v
rootless Podman inside Distrobox
        |
        v
ai-agent-lab (UID 0 inside this nested container)
```

The MCP process runs as the normal Distrobox user. The nested Podman engine also runs rootless. The sandbox `ai-agent-lab` runs as `root` (UID 0) inside its own Podman container.

Distrobox officially documents rootful + `--unshare-all` as the pattern for running a separate Podman instance inside Distrobox. It also documents configuring subordinate IDs and `containers.conf` for rootless Podman inside that Distrobox.

## Important security note

This is **not** a host security sandbox. Distrobox is designed for tight host integration, and the Distrobox documentation explicitly warns that rootful Distrobox containers use real root privileges and can modify host system state. The nested `ai-agent-lab` is the intended execution boundary for model-generated commands, but the outer rootful Distrobox must still be treated as privileged infrastructure.

Inside `ai-agent-lab`, the current restrictions are:

- UID 0 / root
- network disabled
- all Linux capabilities dropped
- `no-new-privileges`
- PID limit 512
- memory limit 1 GiB
- workspace mounted at `/workspace`

## One-command setup

From the repository:

```bash
bash scripts/setup-distrobox.sh
```

This performs the rootful/unshared Distrobox creation, nested rootless Podman configuration, Rust release build, sandbox build, and a runtime verification.

## Start MCP

For normal use:

```bash
bash scripts/run-mcp-distrobox.sh
```

The launcher automatically repairs a missing MCP binary or missing `ai-agent-lab` container.

For a manual interactive shell:

```bash
distrobox enter --root mcp
```

The `--root` option is intentional: the Distrobox itself is rootful. Distrobox normally uses sudo to access rootful containers, so a host authentication step may be required when the sudo timestamp has expired. For non-interactive MCP/stdio use, authenticate before launching when required. Distrobox documents that rootful entry uses sudo (or a configured alternative such as pkexec/doas).

## Build manually

```bash
cargo build --release
bash scripts/build-container.sh
```

## MCP tools

- `terminal_exec`
- `terminal_read`
- `fs_read`
- `fs_write`
- `fs_list`
- `git_status`
- `git_diff`
- `git_commit`
- `rag_store`
- `rag_search`
- `container_info`

All terminal/container execution is performed through the configured Podman sandbox.

## Configuration

```toml
container = "ai-agent-lab"
podman = "podman"
shell = "/bin/bash"
timeout = 300
workspace = "/workspace"
max_output = 65536
auto_start = true

[rag]
enabled = true
storage_path = "./data/rag.jsonl"
top_k = 5
max_text_bytes = 32768
embedding_provider = "none"
ollama_url = "http://127.0.0.1:11434"
ollama_model = "nomic-embed-text"
```

## CI artifact

GitHub Actions runs tests and a release build on pushes to `main`, then packages the Linux x86_64 binary, README, configuration, Containerfile, and scripts.

The artifact is named:

```
mcp-terminal-bridge-linux-x86_64
```

## License

MIT
