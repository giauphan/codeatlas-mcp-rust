# Development Guide

## Requirements

- **Rust 1.75+** with `cargo`, `rustfmt`, and `clippy`. Install via [rustup](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **C Compiler / Build Tools**:
  - macOS: `xcode-select --install`
  - Linux (Ubuntu/Debian): `sudo apt-get install build-essential`
  - Linux (Fedora): `sudo dnf groupinstall "Development Tools"`

No external database or background service is required to build and run CodeAtlas. The MCP transport operates over standard I/O (`stdio`). Optional cloud synchronization features connect via the `codeatlas-platform` HTTP API.

---

## Building and Running

```bash
# Build debug binary
cargo build

# Run CodeAtlas diagnostics directly
cargo run -- doctor

# Build optimized release binary
cargo build --release --locked
```

---

## Workspace Commands

| Task | Command |
| :--- | :--- |
| **Check Code** | `cargo check --workspace` |
| **Format Check** | `cargo fmt --all -- --check` |
| **Apply Formatting** | `cargo fmt --all` |
| **Linter (Clippy)** | `cargo clippy --workspace --all-targets -- -D warnings` |
| **Run Tests** | `cargo test --workspace` |
| **Verify Packaging** | `cargo package --workspace --no-verify` |

---

## Pre-PR Verification

Before submitting changes or opening a pull request, run the standard quality checks locally:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
