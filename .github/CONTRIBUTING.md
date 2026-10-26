# Contributing to CodeAtlas MCP Server

Thank you for your interest in contributing to **CodeAtlas MCP Server**! We welcome bug fixes, documentation improvements, new features, and architectural enhancements.

---

## 🛠️ Prerequisites & Local Setup

CodeAtlas MCP Server is built as a modular Rust workspace following Clean Architecture principles.

### Requirements:
- **Rust**: 1.80.0 or later (`stable` channel recommended).
- **Cargo**: Included with standard Rust installation.
- **Components**: `rustfmt` and `clippy`.
  ```bash
  rustup component add rustfmt clippy
  ```

### Quick Setup:
```bash
# Clone repository
git clone https://github.com/giauphan/codeatlas-mcp-rust.git
cd codeatlas-mcp-rust

# Verify workspace compilation
cargo check --workspace

# Run full unit and integration test suite
cargo test --workspace
```

---

## 🏛️ Architecture & Clean Code Principles

The project strictly follows **Clean Architecture & Domain-Driven Design (DDD)**:

1. **Inward Dependency Rule**:
   - `codeatlas-domain`: Pure domain models, value objects, and port traits. Zero external framework or infrastructure dependencies.
   - `codeatlas-application`: Use cases, interactors, and business workflows. Depends only on `codeatlas-domain`.
   - `codeatlas-infrastructure`: Tree-sitter AST parsers, petgraph store, file watchers, and caching engines.
   - `codeatlas-adapters`: MCP stdio transport, CLI subcommands (`doctor`, `setup-claude`, `setup-zed`), and external HTTP clients.
   - `codeatlas-cli`: Root binary entry point stitching components together.
2. **Framework Decoupling**:
   - Never leak transport-specific types (e.g., raw JSON-RPC frames, CLI flags) into domain entities.
3. **Module Sizing**:
   - Target under 450 lines per module (hard limit of 800 lines). Break larger modules into cohesive submodules with facade re-exports (`pub(crate) use`).
4. **Boundary Validation**:
   - Validate and sanitize all external inputs at entry boundaries. Fail closed on malformed requests.

---

## 🔄 Development & Verification Workflow

Before submitting a pull request, ensure all quality gates pass locally:

```bash
# 1. Format check
cargo fmt --all -- --check

# 2. Linting (zero warnings enforced)
cargo clippy --workspace --all-targets -- -D warnings

# 3. Unit and integration tests
cargo test --workspace

# 4. Manifest and package verification
cargo package --workspace --no-verify
```

---

## 📝 Commit Conventions & Pull Requests

1. **Branching**: Create a feature branch from `main` (e.g. `fix/ast-python-decorator` or `feat/memory-cache`).
2. **Commit Messages**: Follow [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat(...)`: New user-facing feature or tool.
   - `fix(...)`: Bug fix.
   - `docs(...)`: Documentation changes.
   - `refactor(...)`: Code changes without behavior modifications.
   - `perf(...)`: Performance improvements.
   - `test(...)`: Adding or updating test suites.
   - `chore(...)`: Tooling, CI, dependencies, or metadata updates.
3. **Pull Request**:
   - Ensure all GitHub Actions checks pass (CI, Pre-Push Quality Gate, CodeQL).
   - Link any relevant issues.

---

## 🤝 Community & Support

- **Code of Conduct**: All contributors are expected to uphold the [Code of Conduct](./CODE_OF_CONDUCT.md).
- **Bug Reports & Feature Requests**: Submit an issue via [GitHub Issues](https://github.com/giauphan/codeatlas-mcp-rust/issues).
- **Security**: Report sensitive vulnerabilities privately following our [Security Policy](./SECURITY.md).
