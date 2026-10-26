# Security Policy

## Supported Versions

The following table lists the security support status for versions of CodeAtlas MCP Server:

| Version | Supported          | Status |
| ------- | ------------------ | ------ |
| 0.3.x   | :white_check_mark: | Active security updates and bug fixes |
| < 0.3   | :x:                | End of life; upgrade to 0.3.x recommended |

## Reporting a Vulnerability

We take the security of CodeAtlas seriously. If you believe you have found a security vulnerability, please do **not** report it through public GitHub issues or discussions.

Instead, please report vulnerabilities through a [private GitHub security advisory](https://github.com/giauphan/codeatlas-mcp-rust/security/advisories/new).

You can also contact the repository maintainer directly at **giauphan012@gmail.com**, the public email listed on the repository owner's GitHub profile.

### What to include in your report:
- Affected component or crate (`codeatlas-cli`, `codeatlas-adapters`, `codeatlas-infrastructure`, `codeatlas-application`, `codeatlas-domain`).
- Affected version or commit SHA.
- Step-by-step reproduction instructions or proof-of-concept (PoC).
- Potential impact and threat model.

### Response timeline:
- **Initial Acknowledgment**: Within 48 hours.
- **Triage & Assessment**: Within 5 business days.
- **Fix & Advisory Release**: Coordinated disclosure with CVE assignment if applicable.

Please keep reports confidential until a patch has been published.

## Security Architecture & Local-First Isolation

CodeAtlas MCP Server is designed with local-first security principles:
1. **Local-First Stdio Execution**: Runs as an MCP child process communicating strictly over `stdin`/`stdout`. It opens no external HTTP/SSE listening ports by default.
2. **Credential Hygiene**: The API key (`CODEATLAS_API_KEY`) is read from process environment variables or validated local client configs. Sensitive keys are masked and redacted from all debug logs and traces.
3. **Outbound Destination Whitelisting**: Outbound cloud requests (such as dream memory sync) strictly validate target endpoints against allowed official domains (`*.codeatlas.ai`, `opencode.ai`) or local addresses (`127.0.0.1`, `localhost`). Untrusted endpoints require explicit operator opt-in via `CODEATLAS_ALLOW_CUSTOM_URL=true`.
4. **Symlink Boundary Checks**: File access routines enforce strict boundary checks and avoid traversal beyond authorized workspace roots.
