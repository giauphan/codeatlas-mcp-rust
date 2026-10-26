# CodeAtlas MCP Server

Local-first, high-performance native Rust Model Context Protocol (MCP) server for AI-powered codebase intelligence — AST analysis, dependency graphs, and semantic code intelligence. Your source code never leaves your machine.

[![CI](https://github.com/giauphan/codeatlas-mcp-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/giauphan/codeatlas-mcp-rust/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/codeatlas-cli.svg)](https://crates.io/crates/codeatlas-cli)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust 2021](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org)
[![Security Policy](https://img.shields.io/badge/security-policy-brightgreen.svg)](./SECURITY.md)

---

## 📌 What is CodeAtlas MCP Server?

A high-performance **Model Context Protocol (MCP)** server built in Rust that provides:
- **Tree-sitter AST Analysis**: Deep parsing for TypeScript, JavaScript, Python, PHP, Rust, and more.
- **Dependency & Call Graphs**: Fast petgraph-backed module relationships, caller/callee tracing, and cycle detection.
- **Semantic Code Search**: Instant search and indexing across multi-repository workspaces.
- **Local-First & Private**: Code execution stays entirely on your local machine; zero telemetry by default.
- **Cloud Optional**: Seamless integration with [codeatlas-platform](https://github.com/giauphan/codeatlas-platform) for persistent dream memory and immune genome verification.

### Why Use It?
| Use Case | Benefit |
|---|---|
| **AI IDE Integration** | Claude Desktop, Cursor, Zed, VSCode — instant context-aware code intelligence. |
| **Codebase Audits** | Automated call graph analysis, architectural smell detection, and impact tracing. |
| **Semantic Code Search** | Locate functions, types, and logic patterns across complex polyglot repos. |
| **Native Performance** | Instant cold starts, minimal RAM footprint, and jemalloc-optimized concurrency. |

---

## 🏗 Architecture

```
AI Client (Claude / Zed / Cursor)
      │ (JSON-RPC over stdin/stdout)
      ▼
MCP Transport Adapter (crates/codeatlas-adapters)
      │
Application Use Cases (crates/codeatlas-application)
      │
Pure Domain Entities & Ports (crates/codeatlas-domain)
      │
Infrastructure Engines (crates/codeatlas-infrastructure)
  ├── Tree-sitter Parser
  ├── Petgraph Store
  └── Local Cache & Watchers
```

---

## 🚀 Quick Start

### 1. Install via Cargo (crates.io)
```bash
cargo install codeatlas-cli --locked
```

### 2. Or Install via One-Line Shell Script (macOS / Linux)
```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/giauphan/codeatlas-mcp-rust/main/install.sh | bash
```

### 3. Or Build from Source
```bash
git clone https://github.com/giauphan/codeatlas-mcp-rust.git
cd codeatlas-mcp-rust
cargo build --release
```
The compiled binary will be located at `target/release/codeatlas`.

---

## 📡 MCP Tools

CodeAtlas provides a comprehensive suite of MCP tools across multiple categories:

| Category | Tools |
|---|---|
| **Code Analysis** | `analyze`, `list_projects`, `get_project_structure`, `get_insights`, `code_search`, `search_entities`, `get_file_entities`, `get_code_snippet`, `index_coverage`, `project_context` |
| **Dependencies & Graphs** | `get_dependencies`, `get_callers`, `get_callees`, `impact_analysis`, `generate_system_flow`, `generate_feature_flow_diagram`, `trace_feature_flow` |
| **Security & Quality** | `scan_enterprise_vulnerabilities`, `detect_architectural_smells` |
| **Cloud Memory (Optional)** | `save_dream_memory`, `query_dream_memories`, `sync_dreams` |
| **Second Brain & Genome** | `brain_context`, `memory_bootstrap`, `sync_system_memory`, `get_system_memory`, `search_genome`, `get_gene`, `scan_immune_genes`, `save_immune_gene`, `setup_second_brain`, `check_second_brain_status` |
| **Workspace & Utilities** | `run_script`, `git_changes`, `manage_adr`, `detect_code_similarities`, `export_team_artifact`, `sync_skills_inventory`, `route_task` |

---

## 💻 Client Configuration Examples

### Claude Desktop
Add to your `claude_desktop_config.json`:
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
Run the automatic setup command:
```bash
codeatlas setup-zed
```
Or manually add to `~/.config/zed/settings.json`:
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

### Cursor / VSCode
Configure in your Cursor or VSCode MCP settings:
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

## 🔒 Security & Environment Variables

CodeAtlas MCP Server operates locally by default over standard I/O with strict boundary protections:
- **Environment Isolation**: API tokens are read directly from process environment variables and masked from diagnostic outputs.
- **Whitelisted Cloud Destinations**: Outbound cloud requests are strictly restricted to verified endpoints (`*.codeatlas.ai`, `opencode.ai`, `127.0.0.1`, `localhost`).
- **Symlink Protection**: Filesystem queries enforce boundary validation to prevent unauthorized directory traversal.

| Variable | Description | Default |
|---|---|---|
| `CODEATLAS_API_KEY` | Optional key to connect to CodeAtlas Cloud services for shared memory. | (none) |
| `CODEATLAS_API_URL` | Optional base URL for CodeAtlas Cloud backend. | (none) |
| `CODEATLAS_ALLOW_CUSTOM_URL` | Set to `true` to allow custom unlisted backend URLs. | `false` |

---

## ⚠️ Operational Considerations & Boundaries

CodeAtlas MCP Server is designed as a local-first, low-overhead native binary. Keep the following operational boundaries in mind:

- **Local-First & Zero Database Footprint**:
  - The server operates entirely standalone and requires **no database servers** (such as Oracle, PostgreSQL, or Redis).
  - All AST dependency graphs, call hierarchies, and indexing structures are maintained in-memory using Petgraph and backed by local filesystem caches (`~/.codeatlas/`).
  - Optional cloud features (dream memory and immune genome synchronization) connect to upstream CodeAtlas Platform services via HTTPS only when `CODEATLAS_API_KEY` and `CODEATLAS_API_URL` are explicitly configured.
- **Single-Tenant Process Isolation**:
  - CodeAtlas executes as a dedicated child process attached to your AI client (Claude Desktop, Zed, Cursor, VS Code) via standard input/output (`stdio`).
  - Process execution boundaries, file system permissions, and workspace access align directly with the user account running the IDE. Multi-user isolation is enforced by the host operating system.
- **Network Boundaries & No Open Ports**:
  - CodeAtlas does not listen on any network ports or expose an HTTP/SSE server. Reverse proxies (such as Nginx) and external firewall port forwarding are unnecessary.
  - Outbound cloud communication is strictly allowlisted to official CodeAtlas domains (`*.codeatlas.ai`, `opencode.ai`) or local loopback (`127.0.0.1`, `localhost`). Non-local unencrypted HTTP traffic is rejected.
- **Static Analysis Scope**:
  - Code analysis is performed statically using Tree-sitter AST parsers. It does not execute untrusted user code or evaluate runtime bytecode.
  - Repository graph size is bound by available host RAM. Concurrency and allocations are optimized with Tokio and jemalloc.

---

## 📚 Resources & Community

- 📖 [Development Guide](./docs/DEVELOPMENT.md) — Local development workflow and commands.
- 🚀 [Pipeline Runbook](./docs/PIPELINE.md) — CI quality gates and crates.io CD automation.
- 🛡️ [Security Policy](./SECURITY.md) — Vulnerability reporting and supported versions.
- 🤝 [Contributing Guidelines](./CONTRIBUTING.md) — How to contribute, commit conventions, and code standards.
- 📜 [Code of Conduct](./CODE_OF_CONDUCT.md) — Community standards and pledge.
- 📄 [MIT License](./LICENSE) — Open-source license terms.

---

## 🐛 Issues & Support

- Report bugs or request features: [GitHub Issues](https://github.com/giauphan/codeatlas-mcp-rust/issues)
- Community discussions: [GitHub Discussions](https://github.com/giauphan/codeatlas-mcp-rust/discussions)
