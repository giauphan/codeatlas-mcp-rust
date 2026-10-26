# Installation and Deployment Guide

CodeAtlas MCP Server is a high-performance, local-first Model Context Protocol (MCP) server written in Rust.

---

## 🏛️ Deployment & Process Model

1. **Local-First Process Lifecycle**:
   - CodeAtlas is spawned directly as a child process by the host MCP client (Claude Desktop, Zed, Cursor, VS Code).
   - Communication occurs strictly over standard input and output (`stdio`) using JSON-RPC 2.0 messages.
   - The server maintains its own in-memory Petgraph dependency structure and local filesystem state under `~/.codeatlas/`.
2. **Zero Daemon or Web Server Overhead**:
   - CodeAtlas does **not** host an HTTP, SSE, or WebSocket listener.
   - External reverse proxies (such as Nginx, Traefik, or Caddy) and container orchestration are not needed for normal operation.
   - Process lifetimes begin when the AI client opens the workspace and terminate when the client closes.
3. **Zero Database Footprint**:
   - The server requires **no database servers** (such as Oracle, PostgreSQL, MySQL, or Redis).
   - All AST graph models and caller/callee indices are constructed in-memory.

---

## 📦 Installation Methods

### 1. crates.io (Recommended)
If you have Rust and Cargo installed:
```bash
cargo install codeatlas-cli --locked
```
This builds and installs the `codeatlas` binary to your `$CARGO_HOME/bin` or `$HOME/.cargo/bin`.

To update to the latest release:
```bash
cargo install codeatlas-cli --locked --force
```

### 2. Standalone Shell Installer (macOS & Linux)
Installs the latest release directly without manual configuration:
```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/giauphan/codeatlas-mcp-rust/main/install.sh | bash
```

### 3. Build from Source
```bash
git clone https://github.com/giauphan/codeatlas-mcp-rust.git
cd codeatlas-mcp-rust
cargo build --release
```
The optimized native binary is produced at `target/release/codeatlas`.

---

## 💻 Client Integration Configurations

### Claude Desktop
Add to your `claude_desktop_config.json` (`~/Library/Application Support/Claude/` on macOS or `%APPDATA%\Claude\` on Windows):
```json
{
  "mcpServers": {
    "codeatlas": {
      "command": "codeatlas",
      "args": []
    }
  }
}
```

### Zed Editor
You can run the automatic configuration tool:
```bash
codeatlas setup-zed
```
Or manually configure in `~/.config/zed/settings.json`:
```json
{
  "context_servers": {
    "codeatlas": {
      "command": "codeatlas",
      "args": []
    }
  }
}
```

### Cursor / VS Code (stdio)
In your workspace or user `.cursor/settings.json` or `.vscode/settings.json`:
```json
{
  "mcpServers": {
    "codeatlas": {
      "command": "codeatlas",
      "args": []
    }
  }
}
```

---

## 🌐 Optional Cloud & Second Brain Synchronization

Local AST parsing and graph intelligence require zero external network access. To synchronize shared dream memory or team immune genes:

1. Obtain credentials from your CodeAtlas Platform dashboard.
2. Provide them in your MCP client environment configuration:
   ```json
   {
     "mcpServers": {
       "codeatlas": {
         "command": "codeatlas",
         "args": [],
         "env": {
           "CODEATLAS_API_URL": "https://api.codeatlas.ai",
           "CODEATLAS_API_KEY": "your_secure_api_key_here"
         }
       }
     }
   }
   ```
3. Outbound requests are strictly allowlisted to official CodeAtlas domains (`*.codeatlas.ai`, `opencode.ai`) or local addresses (`127.0.0.1`, `localhost`). Non-local unencrypted HTTP traffic is rejected by default.
