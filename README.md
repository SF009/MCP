# MCP Terminal Bridge + RAG Memory

Rust MCP server over stdio JSON-RPC for a Podman sandbox, filesystem/Git helpers, and local RAG memory.

## Architecture

The intended deployment for this project is:

    Bionic / AI model (host)
             |
             | MCP stdio / JSON-RPC
             v
    mcp-terminal-bridge
             |
             v
    Distrobox: mcp
             |
             v
    rootless Podman
             |
             v
    ai-agent-lab
             |
             +-- /workspace
             +-- bash / git / python / rg / jq / tree
             +-- network: none
             +-- capabilities: dropped
             +-- no-new-privileges

Distrobox is the environment for the MCP process and the nested Podman engine.
The model remains on the host. The sandbox is the execution environment.

Distrobox documents rootful + `--unshare-all` as the setup for running a separate
Podman instance inside a Distrobox. The MCP process and inner Podman engine run as the normal Distrobox user; the `ai-agent-lab` sandbox itself runs as root.

## Tools

| Tool | Purpose |
|---|---|
| terminal_exec | Run a shell command inside the configured container |
| terminal_read | Bounded file read |
| fs_read / fs_write / fs_list | Container filesystem operations |
| git_status / git_diff / git_commit | Git helpers inside the container |
| rag_store / rag_search | JSONL memory with optional Ollama embeddings |
| container_info | Runtime and sandbox information |

## Build

    cargo test
    cargo build --release

Every push to `main` runs the test/build pipeline and publishes a Linux x86_64
release artifact from the successful release build. The artifact contains the
binary, configuration, container definition, scripts, README, and license,
plus a SHA-256 checksum.

GitHub Actions artifact:

    mcp-terminal-bridge-linux-x86_64

Artifacts are retained by GitHub Actions for 30 days.

## Distrobox + nested Podman setup

Install Distrobox and Podman on the host first.

Then from the repository:

    bash scripts/setup-distrobox.sh

This creates a rootful, unshared Distrobox named `mcp`, following the documented
Distrobox pattern for nested Podman.

Enter it:

    distrobox enter --root mcp

The bootstrap script performs the subordinate UID/GID setup for the normal user and configures rootless Podman. Do not run it as root.

Then bootstrap Podman and the MCP sandbox as the normal Distrobox user. Because `--root` enters the rootful Distrobox as root, use `runuser`:

    distrobox enter --root mcp -- bash -lc 'runuser -u YOUR_USER -- bash /path/to/MCP/scripts/bootstrap-distrobox.sh'

Replace `YOUR_USER` and `/path/to/MCP` with your actual host username and repository path.

Verify:

    podman info
    podman ps -a
    podman exec ai-agent-lab id
    podman exec ai-agent-lab pwd

The MCP configuration defaults to:

    container = "ai-agent-lab"
    podman = "podman"
    shell = "/bin/bash"
    workspace = "/workspace"
    auto_start = true

The bridge automatically starts `ai-agent-lab` if it exists but is stopped. Commands executed through the bridge run as UID 0 inside `ai-agent-lab`.

## Running MCP for Bionic

Build the release binary inside the Distrobox:

    cargo build --release

From the host, the MCP launcher is:

    bash scripts/run-mcp-distrobox.sh

For a client configuration, use the launcher as the command and do not start
the binary directly on the host:

    {
      "mcpServers": {
        "terminal-bridge": {
          "command": "/absolute/path/to/MCP/scripts/run-mcp-distrobox.sh",
          "args": []
        }
      }
    }

If the client cannot execute the script directly because its Git checkout has
mode 100644, use:

    {
      "mcpServers": {
        "terminal-bridge": {
          "command": "/bin/bash",
          "args": [
            "/absolute/path/to/MCP/scripts/run-mcp-distrobox.sh"
          ]
        }
      }
    }

The launcher uses `distrobox enter --root --no-tty` and then drops into the
normal Distrobox user before starting the MCP process. This keeps MCP stdio
clean: logs go to stderr and JSON-RPC responses go to stdout.

Because rootful Distrobox requires host authentication, run `sudo -v` before
starting a long-lived MCP client if your sudo policy would otherwise prompt.
Do not grant broad passwordless sudo to Distrobox unless you explicitly accept
the host-root implications.

## Manual smoke test

Inside the Distrobox:

    cd /path/to/MCP
    cargo build --release
    bash scripts/build-container.sh

Then run:

    bash scripts/run-mcp-distrobox.sh

The process waits on stdin/stdout; that is expected for an MCP stdio server.

Example JSON-RPC messages:

    {"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
    {"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
    {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"terminal_exec","arguments":{"command":"printf 'hello\\n'"}}}

## Sandbox

The supplied `ai-agent-lab` image runs as an unprivileged user. The container
has no network, drops Linux capabilities, enables no-new-privileges, and limits
PIDs/memory. Only the repository's `workspace` directory is mounted into
`/workspace`.

The MCP bridge itself has strong control over its configured Podman engine.
It is therefore a trusted local integration and must not be exposed to
untrusted clients.

Distrobox is intentionally tightly integrated with the host and is not itself
a security sandbox. The security boundary for arbitrary model-generated shell
commands is the nested `ai-agent-lab` Podman container.

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

The RAG HTTP request is made by the MCP process, not by `ai-agent-lab`.
Keep `ai-agent-lab` network-isolated.

## Configuration

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

## Package

    tar -czf ../mcp-terminal-bridge.tar.gz . --exclude=target --exclude=data --exclude=workspace

## License

MIT
