# MCP Terminal Bridge

A Rust MCP server that exposes terminal, filesystem, Git, container, and optional RAG tools to an external AI model over stdio JSON-RPC.

## Architecture

```
Bionic / AI model (host)
        |
        | MCP stdio / JSON-RPC
        v
run-mcp-distrobox.sh
        |
        v
distrobox enter --root mcp
        |
        v
rootless Podman inside Distrobox
        |
        v
ai-agent-lab (existing container, UID 0 inside)
```

The MCP process runs as the normal user inside the rootful Distrobox. Nested Podman also runs rootless as that user. The existing `ai-agent-lab` container is the execution boundary and runs as UID 0 inside the container.

## Important behavior

The MCP launcher does **not** manage the Podman container lifecycle. It does not create, rebuild, inspect, start, stop, or replace `ai-agent-lab`.

The existing container is configured by you and is accessed by the Rust MCP server through the `podman` command inside the Distrobox.

## Distrobox

The intended command is exactly:

```bash
distrobox enter --root mcp
```

Do not replace this with `sudo distrobox enter --root mcp`. Distrobox documents `--root` as the preferred mechanism for rootful Distrobox. citeturn0search1

## One-time Distrobox setup

If the `mcp` Distrobox has not been configured yet:

```bash
bash scripts/setup-distrobox.sh
```

This creates/configures only the Distrobox and its nested rootless Podman environment. It does **not** create or build `ai-agent-lab`.

## Start MCP

For manual interactive use:

```bash
distrobox enter --root mcp
```

For Bionic/MCP stdio, use:

```bash
bash scripts/run-mcp-distrobox.sh
```

The launcher performs only the equivalent of entering the existing rootful Distrobox and starting the MCP process inside it. It does not open an interactive shell and does not manage `ai-agent-lab`.

## Existing sandbox requirements


The user-managed container must be named:

```text
ai-agent-lab
```

and must already be running.

Check it from inside the Distrobox:

```bash
distrobox enter --root mcp
podman container exists ai-agent-lab
podman inspect --format '{{.State.Running}}' ai-agent-lab
podman exec ai-agent-lab id
```

The expected UID from the last command is:

```text
0
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

All terminal/container execution is performed through the existing Podman sandbox.

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

## License

MIT
