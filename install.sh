#!/usr/bin/env bash
# ==============================================================================
# CodeAtlas Installer for macOS & Linux
# https://github.com/giauphan/codeatlas-mcp-rust
#
# Usage:
#   tmp=$(mktemp) && { curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/giauphan/codeatlas-mcp-rust/main/install.sh -o "$tmp" && bash "$tmp"; ret=$?; rm -f "$tmp"; (exit $ret); }
#   or locally: ./install.sh [--setup-claude] [--setup-zed]
# ==============================================================================

set -euo pipefail

# Visual formatting
BOLD="$(tput bold 2>/dev/null || true)"
GREEN="$(tput setaf 2 2>/dev/null || true)"
BLUE="$(tput setaf 4 2>/dev/null || true)"
YELLOW="$(tput setaf 3 2>/dev/null || true)"
RED="$(tput setaf 1 2>/dev/null || true)"
RESET="$(tput sgr0 2>/dev/null || true)"

log_info()    { echo "${BLUE}==>${RESET} ${BOLD}$*${RESET}"; }
log_success() { echo "${GREEN}✓${RESET} ${BOLD}$*${RESET}"; }
log_warn()    { echo "${YELLOW}!${RESET} ${BOLD}$*${RESET}"; }
log_error()   { echo "${RED}✗${RESET} ${BOLD}$*${RESET}" >&2; }

# Parse arguments
SETUP_CLAUDE=false
SETUP_ZED=false
FROM_SOURCE=false

for arg in "$@"; do
  case "$arg" in
    --from-source)
      FROM_SOURCE=true
      ;;
    --setup-claude|--setup-claude-desktop)
      SETUP_CLAUDE=true
      ;;
    --setup-zed)
      SETUP_ZED=true
      ;;
    --help|-h)
      echo "CodeAtlas Installer"
      echo ""
      echo "Usage: ./install.sh [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --from-source     Build and install directly from local repository source checkout"
      echo "  --setup-claude    Automatically configure Claude Desktop MCP settings after install"
      echo "  --setup-zed       Automatically configure Zed editor MCP settings after install"
      echo "  -h, --help        Show this help message"
      exit 0
      ;;
    *)
      log_warn "Unknown option: $arg (ignored)"
      ;;
  esac
done

echo "${BOLD}================================================================${RESET}"
echo "${BOLD}  🚀 CodeAtlas Native Rust MCP Engine Installer                 ${RESET}"
echo "${BOLD}================================================================${RESET}"
echo ""

# 1. Platform Detection
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    OS_TYPE="darwin"
    ;;
  Linux)
    OS_TYPE="linux"
    ;;
  *)
    log_error "Unsupported operating system: $OS. CodeAtlas requires macOS or Linux."
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64)
    ARCH_TYPE="x86_64"
    ;;
  arm64|aarch64)
    ARCH_TYPE="aarch64"
    ;;
  *)
    log_error "Unsupported architecture: $ARCH. CodeAtlas supports x86_64 and aarch64/arm64."
    exit 1
    ;;
esac

log_info "Detected Platform: ${OS_TYPE}-${ARCH_TYPE} (${OS} ${ARCH})"

# 2. Determine target binary directory
if [[ -n "${CARGO_HOME:-}" ]]; then
  INSTALL_DIR="${CARGO_HOME}/bin"
  mkdir -p "$INSTALL_DIR"
elif [[ -d "${HOME}/.cargo/bin" ]]; then
  INSTALL_DIR="${HOME}/.cargo/bin"
elif [[ -d "${HOME}/.local/bin" ]]; then
  INSTALL_DIR="${HOME}/.local/bin"
else
  INSTALL_DIR="${HOME}/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

BIN_TARGET="${INSTALL_DIR}/codeatlas"
log_info "Target Installation Directory: ${INSTALL_DIR}"

# 3. Installation Strategy
INSTALLED=false

if [[ "$FROM_SOURCE" = true ]]; then
  if [[ ! -f "Cargo.toml" || ! -d "crates" ]]; then
    log_error "Current working directory is not a valid CodeAtlas repository checkout."
    exit 1
  fi
  log_info "Detected explicit --from-source flag. Compiling from repository source..."
  if ! command -v cargo >/dev/null 2>&1; then
    log_error "Cargo is required for local compilation but was not found in PATH."
    exit 1
  fi
  if cargo build --release --locked --manifest-path "Cargo.toml" --target-dir "target"; then
    cp -f "target/release/codeatlas" "$BIN_TARGET"
    chmod 755 "$BIN_TARGET"
    INSTALLED=true
  else
    log_error "Local source compilation failed."
    exit 1
  fi
fi

# If not installed from local source, require Cargo and install from crates.io
if [[ "$INSTALLED" = false ]]; then
  if ! command -v cargo >/dev/null 2>&1; then
    log_error "Rust and Cargo are required to install CodeAtlas but were not found in PATH."
    echo ""
    echo "To install Rust and Cargo, run:"
    echo "  ${BOLD}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${RESET}"
    echo ""
    echo "After installing Rust, rerun this installer to build CodeAtlas."
    exit 1
  fi

  log_info "Installing codeatlas-cli from crates.io via Cargo..."
  INSTALL_ROOT="$(dirname "$INSTALL_DIR")"
  if cargo install codeatlas-cli --locked --root "$INSTALL_ROOT" --force && [[ -f "$BIN_TARGET" ]]; then
    INSTALLED=true
  else
    log_error "Failed to install codeatlas from crates.io via Cargo."
    exit 1
  fi
fi

# 4. PATH verification
echo ""
log_success "CodeAtlas binary installed to: ${BIN_TARGET}"

if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
  log_warn "${INSTALL_DIR} is not in your current PATH!"
  echo ""
  echo "Add it to your shell profile to run 'codeatlas' from anywhere:"
  CURRENT_SHELL="$(basename "${SHELL:-bash}")"
  if [[ -n "${ZSH_VERSION:-}" || "$CURRENT_SHELL" == "zsh" ]]; then
    echo "  ${BOLD}echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc && source ~/.zshrc${RESET}"
  else
    echo "  ${BOLD}echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc && source ~/.bashrc${RESET}"
  fi
  echo ""
fi

# 5. Verification
echo ""
log_info "Verifying installation..."
if ! "$BIN_TARGET" --version; then
  log_error "The installed CodeAtlas binary could not start."
  exit 1
fi
echo ""
"$BIN_TARGET" doctor || true

# 6. Optional Client Integrations
if [[ "$SETUP_CLAUDE" = true ]]; then
  echo ""
  log_info "Configuring Claude Desktop MCP integration..."
  "$BIN_TARGET" setup-claude || { log_error "Claude Desktop MCP integration setup failed."; exit 1; }
fi

if [[ "$SETUP_ZED" = true ]]; then
  echo ""
  log_info "Configuring Zed editor MCP integration..."
  "$BIN_TARGET" setup-zed || { log_error "Zed editor MCP setup failed."; exit 1; }
fi

echo ""
echo "${BOLD}================================================================${RESET}"
log_success "CodeAtlas installation complete! Run 'codeatlas --help' to get started."
echo "${BOLD}================================================================${RESET}"
