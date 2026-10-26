#!/usr/bin/env bash
# ==============================================================================
# CodeAtlas Installer for macOS & Linux
# https://github.com/giauphan/codeatlas-mcp-rust
#
# Usage:
#   tmp=$(mktemp) && curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/giauphan/codeatlas-mcp-rust/main/install.sh -o "$tmp" && bash "$tmp"; rm -f "$tmp"
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

for arg in "$@"; do
  case "$arg" in
    --setup-claude)
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
      echo "  --setup-claude    Automatically configure Claude Code integration after install"
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
if [[ -n "${CARGO_HOME:-}" && -d "${CARGO_HOME}/bin" ]]; then
  INSTALL_DIR="${CARGO_HOME}/bin"
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
# Check if running from local source clone
SCRIPT_DIR=""
if [[ -n "${BASH_SOURCE[0]:-}" ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || echo "")"
fi
INSTALLED=false

if [[ -n "$SCRIPT_DIR" && -f "${SCRIPT_DIR}/Cargo.toml" && -d "${SCRIPT_DIR}/crates" ]]; then
  log_info "Detected local source directory. Compiling from repository source..."
  if command -v cargo >/dev/null 2>&1; then
    if cargo build --release --locked --manifest-path "${SCRIPT_DIR}/Cargo.toml" --target-dir "${SCRIPT_DIR}/target"; then
      cp -f "${SCRIPT_DIR}/target/release/codeatlas" "$BIN_TARGET"
      chmod 755 "$BIN_TARGET"
      INSTALLED=true
    else
      log_error "Local source compilation failed."
      exit 1
    fi
  else
    log_error "Cargo is required for local compilation but was not found in PATH."
    exit 1
  fi
fi

# If not installed from local source, try cargo install from crates.io
if [[ "$INSTALLED" = false ]] && command -v cargo >/dev/null 2>&1; then
  log_info "Installing codeatlas from crates.io via Cargo..."
  INSTALL_ROOT="$(dirname "$INSTALL_DIR")"
  if cargo install codeatlas --locked --root "$INSTALL_ROOT" --force 2>/dev/null && [[ -f "$BIN_TARGET" ]]; then
    INSTALLED=true
  else
    log_warn "cargo install from crates.io not available or package in publish queue."
  fi
fi

# Fallback: Download prebuilt binary release from GitHub if available
if [[ "$INSTALLED" = false ]]; then
  RELEASE_REPO="giauphan/codeatlas-mcp-rust"
  TARGET_TRIPLE=""

  if [[ "$OS_TYPE" == "darwin" && "$ARCH_TYPE" == "aarch64" ]]; then
    TARGET_TRIPLE="aarch64-apple-darwin"
  elif [[ "$OS_TYPE" == "darwin" && "$ARCH_TYPE" == "x86_64" ]]; then
    TARGET_TRIPLE="x86_64-apple-darwin"
  elif [[ "$OS_TYPE" == "linux" && "$ARCH_TYPE" == "x86_64" ]]; then
    TARGET_TRIPLE="x86_64-unknown-linux-gnu"
  elif [[ "$OS_TYPE" == "linux" && "$ARCH_TYPE" == "aarch64" ]]; then
    TARGET_TRIPLE="aarch64-unknown-linux-gnu"
  fi

  if [[ -n "$TARGET_TRIPLE" ]]; then
    DOWNLOAD_URL="https://github.com/${RELEASE_REPO}/releases/latest/download/codeatlas-${TARGET_TRIPLE}.tar.gz"
    log_info "Attempting to download prebuilt binary: ${DOWNLOAD_URL}"

    TEMP_DIR="$(mktemp -d)"
    trap 'rm -rf "$TEMP_DIR"' EXIT

    HTTP_STATUS=""
    if command -v curl >/dev/null 2>&1; then
      HTTP_STATUS="$(curl --proto '=https' --proto-redir '=https' -sSL -w "%{http_code}" -o "${TEMP_DIR}/codeatlas.tar.gz" "$DOWNLOAD_URL" || true)"
    fi

    if [[ "$HTTP_STATUS" == "200" && -s "${TEMP_DIR}/codeatlas.tar.gz" ]]; then
      tar -xzf "${TEMP_DIR}/codeatlas.tar.gz" -C "$TEMP_DIR"
      if [[ -f "${TEMP_DIR}/codeatlas" ]]; then
        cp -f "${TEMP_DIR}/codeatlas" "$BIN_TARGET"
        chmod 755 "$BIN_TARGET"
        INSTALLED=true
      fi
    fi
  fi
fi

# If still not installed, prompt user to install Rust toolchain
if [[ "$INSTALLED" = false ]]; then
  if ! command -v cargo >/dev/null 2>&1; then
    log_warn "Rust/Cargo was not detected on this system."
    echo ""
    echo "To install Rust and Cargo, run:"
    echo "  ${BOLD}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${RESET}"
    echo ""
    echo "After installing Rust, rerun this installer to build CodeAtlas."
  fi
  log_error "Installation could not be completed automatically."
  exit 1
fi

# 4. PATH verification
echo ""
log_success "CodeAtlas binary installed to: ${BIN_TARGET}"

if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
  log_warn "${INSTALL_DIR} is not in your current PATH!"
  echo ""
  echo "Add it to your shell profile to run 'codeatlas' from anywhere:"
  if [[ -n "${ZSH_VERSION:-}" || -f "${HOME}/.zshrc" ]]; then
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
  log_info "Configuring Claude Code integration..."
  "$BIN_TARGET" setup-claude || { log_error "Claude Code integration setup failed."; exit 1; }
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
