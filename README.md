# MCP Terminal Bridge + RAG Memory

Rust MCP server over stdio JSON-RPC for a Podman sandbox, filesystem/Git helpers, and local RAG memory.

## Tools

| Tool | Purpose |
|---|---|
| terminal_exec | Run a shell command inside the configured container |
| terminal_read | Bounded file read |
| fs_read / fs_write / fs_list | Workspace filesystem operations |
| git_status / git_diff / git_commit | Git helpers inside the workspace |
| rag_store / rag_search | JSONL memory with optional Ollama embeddings |
| container_info | Runtime and configuration information |

## Build

    cargo test
    cargo build --release

## Sandbox

    bash scripts/build-container.sh

The supplied container runs as an unprivileged user, has no network,
drops Linux capabilities, enables no-new-privileges, and limits PIDs/memory.
Only the repository's workspace directory is mounted into /workspace.

The bridge itself must be treated as a trusted local integration because
podman access is powerful. Do not expose the MCP process to untrusted clients.

## RAG

Keyword-only mode needs no external service:

    [rag]
    enabled = true
    embedding_provider = "none"

For Ollama embeddings:

    ollama pull nomic-embed-text

and configure:

    embedding_provider = "ollama"
    ollama_url = "http://127.0.0.1:11434"
    ollama_model = "nomic-embed-text"

The container remains network-isolated; the host-side bridge contacts Ollama.

## MCP client configuration

    {
      "mcpServers": {
        "terminal-bridge": {
          "command": "/absolute/path/to/mcp-terminal-bridge",
          "args": [],
          "env": {
            "MCP_CONFIG": "/absolute/path/to/config.toml",
            "RUST_LOG": "info"
          }
        }
      }
    }

## Manual smoke test

    {"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
    {"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
    {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"terminal_exec","arguments":{"command":"printf 'hello\\n'"}}}
    {"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"rag_store","arguments":{"text":"user prefers vim","metadata":{"kind":"pref"}}}}
    {"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"rag_search","arguments":{"query":"editor","top_k":3}}}

## Package

    zip -r ../mcp-terminal-bridge.zip . -x 'target/*' 'data/*' 'workspace/*'

## License

MIT
