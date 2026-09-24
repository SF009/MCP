# MCP Terminal Bridge

A Rust MCP server that lets an external AI model such as Bionic control the existing Ubuntu Distrobox through MCP stdio.

## Architecture

```
Host
┌──────────────────────────────────────────────────────────┐
│ Bionic                                                   │
│   │                                                      │
│   │ MCP stdio                                             │
│   ▼                                                      │
│ run-mcp-distrobox.sh                                     │
│   │                                                      │
│   │ distrobox enter ubuntu                               │
└───┼──────────────────────────────────────────────────────┘
    ▼
Ubuntu Distrobox
┌──────────────────────────────────────────────────────────┐
│ mcp-terminal-bridge                                      │
│   │                                                      │
│   ├── terminal_exec                                      │
│   ├── terminal_read                                      │
│   ├── fs_read / fs_write / fs_list                       │
│   ├── git_status / git_diff / git_commit                 │
│   └── rag_* / runtime info                               │
│                                                          │
│ Commands execute directly in Ubuntu.                     │
└──────────────────────────────────────────────────────────┘
```

Bionic stays on the host. The MCP process is launched inside the existing Distrobox named `ubuntu`. Tool calls execute directly in that Ubuntu environment.

There is no `ai-agent-lab`, no nested Podman sandbox, and no container lifecycle management in the MCP bridge.

## Existing Ubuntu Distrobox

The default Distrobox name is:

```text
ubuntu
```

Manual entry:

```bash
distrobox enter ubuntu
```

The project does not create or replace this Distrobox.

## Start MCP

From the host:

```bash
bash scripts/run-mcp-distrobox.sh
```

For Bionic, register that script as a local stdio MCP server. The launcher uses non-interactive Distrobox entry so stdout remains available for MCP JSON-RPC.

## MCP tools

- `terminal_exec` — execute a shell command directly in Ubuntu
- `terminal_read` — read a file directly in Ubuntu
- `fs_read`
- `fs_write`
- `fs_list`
- `git_status`
- `git_diff`
- `git_commit`
- `rag_store`
- `rag_search`
- `container_info` — report Ubuntu/MCP runtime information

## Configuration

```toml
shell = "/bin/bash"
timeout = 300
workspace = "/workspace"
max_output = 65536

[rag]
enabled = true
storage_path = "./data/rag.jsonl"
top_k = 5
max_text_bytes = 32768
embedding_provider = "none"
ollama_url = "http://127.0.0.1:11434"
ollama_model = "nomic-embed-text"
```

## Security

Bionic can execute commands exposed through `terminal_exec` with the privileges of the user running the MCP process inside Ubuntu. Distrobox is not a host security sandbox.

## License

MIT
