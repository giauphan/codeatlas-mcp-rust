# CodeAtlas Installation Guide

CodeAtlas is a high-performance native Rust Model Context Protocol (MCP) server providing deep AST analysis, architectural insight, semantic second-brain memory, and A2A orchestration.

This guide covers installation options for **macOS**, **Linux**, and **WSL** across `x86_64` (Intel/AMD) and `aarch64` / `arm64` (Apple Silicon & ARM) architectures.

---

## 1. System Requirements & Platform Support

| Operating System | Architecture | Supported | Tier |
| :--- | :--- | :---: | :---: |
| **macOS 12+** | Apple Silicon (`arm64` / M1/M2/M3/M4) | ✅ | Tier 1 |
| **macOS 11+** | Intel (`x86_64`) | ✅ | Tier 1 |
| **Linux (glibc 2.31+)** | `x86_64` (Ubuntu, Debian, Fedora, Arch) | ✅ | Tier 1 |
| **Linux (glibc 2.31+)** | `aarch64` / `arm64` | ✅ | Tier 1 |
| **Windows (WSL2)** | `x86_64` / `aarch64` via Ubuntu/Debian | ✅ | Tier 1 |

---

## 2. Quick Install via Shell Script (Recommended)

The easiest way to install CodeAtlas on macOS and Linux is using our official installer script:

```bash
tmp=$(mktemp) && curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/giauphan/codeatlas-mcp-rust/main/install.sh -o "$tmp" && bash "$tmp"; rm -f "$tmp"
```

### Automated Client Configuration Flags

You can pass post-install integration flags directly:

```bash
# Install and automatically configure Claude Code hooks
tmp=$(mktemp) && curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/giauphan/codeatlas-mcp-rust/main/install.sh -o "$tmp" && bash "$tmp" --setup-claude; rm -f "$tmp"

# Install and automatically configure Zed Editor MCP settings
tmp=$(mktemp) && curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/giauphan/codeatlas-mcp-rust/main/install.sh -o "$tmp" && bash "$tmp" --setup-zed; rm -f "$tmp"

# Configure both Claude and Zed
tmp=$(mktemp) && curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/giauphan/codeatlas-mcp-rust/main/install.sh -o "$tmp" && bash "$tmp" --setup-claude --setup-zed; rm -f "$tmp"
```

---

## 3. Installation via Cargo (crates.io)

If you already have Rust and Cargo installed, you can install CodeAtlas directly from [crates.io](https://crates.io/crates/codeatlas):

```bash
cargo install codeatlas --locked
```

To update to the latest release:

```bash
cargo install codeatlas --locked --force
```

> **Note**: Make sure `~/.cargo/bin` is in your `PATH`:
> ```bash
> echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc   # For Bash
> echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc    # For Zsh
> ```

---

## 4. Building from Source

For developers contributing to CodeAtlas or building directly from source:

### Prerequisites

- **Rust 1.75+**: Install via [rustup](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Build Tools**:
  - macOS: `xcode-select --install`
  - Ubuntu/Debian: `sudo apt-get install build-essential`
  - Fedora: `sudo dnf groupinstall "Development Tools"`

### Build Steps

```bash
# Clone the repository
git clone https://github.com/giauphan/codeatlas-mcp-rust.git
cd codeatlas-mcp-rust

# Build release binary
cargo build --release --locked

# Run the local installer
./install.sh
```

---

## 5. Verifying Your Installation

Confirm that the `codeatlas` binary is functional:

```bash
# Check version
codeatlas --version

# Run system diagnostics
codeatlas doctor
```

Example diagnostics output:
```
CodeAtlas System Diagnostics:
  ✓ Rust Native Engine: active
  ✓ ADR Directory: exists (/home/username/.codeatlas/adr)
  ✓ CodeAtlas Cloud API Key: configured

Diagnostics complete: System is healthy.
```

---

## 6. Client & Editor Integrations

### Claude Code

CodeAtlas integrates with Claude Code to provide automatic AST analysis, task routing, and second-brain memory checkpoints:

```bash
codeatlas setup-claude
```

This registers the native hooks into `~/.claude/settings.json`.

### Zed Editor

To connect CodeAtlas to Zed's built-in assistant as an MCP server:

```bash
codeatlas setup-zed
```

This adds the CodeAtlas MCP server definition to `~/.config/zed/settings.json`.

### Cursor & Claude Desktop

Add CodeAtlas to your MCP configuration file (`~/.config/Claude/claude_desktop_config.json` on Linux or `~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

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

## 7. Configuration & Environment Variables

| Variable | Description | Default |
| :--- | :--- | :--- |
| `CODEATLAS_API_KEY` | Optional API key for Second Brain cloud synchronization | `""` |
| `CODEATLAS_API_URL` | Optional custom endpoint for CodeAtlas Cloud | `https://api.codeatlas.dev` |
| `CODEATLAS_LOG` | Tracing log level (`error`, `warn`, `info`, `debug`, `trace`) | `info` |

---

## 8. Uninstallation

To remove CodeAtlas:

```bash
# If installed via cargo install from crates.io:
cargo uninstall codeatlas

# If installed via install.sh (or copied directly):
# Remove the binary from your installation directory (e.g. ~/.cargo/bin or ~/.local/bin):
rm -f ~/.cargo/bin/codeatlas ~/.local/bin/codeatlas
```
