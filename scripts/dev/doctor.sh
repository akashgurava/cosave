#!/usr/bin/env bash
# ==============================================================================
# CoSave Development CLI: Environment Doctor / Diagnostics
# Target: doctor
# ==============================================================================
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/dev/common.sh
source "${SCRIPT_DIR}/common.sh"

cmd_doctor() {
  log_info "Running CoSave environment diagnostics..."
  local all_ok=true

  # Check Rust
  if command -v rustc &>/dev/null; then
    local rust_ver
    rust_ver=$(rustc --version)
    log_success "Rust: ${rust_ver}"
  else
    log_error "Rust: not found. Please install Rust 1.80+ (https://rustup.rs)"
    all_ok=false
  fi

  # Check Cargo
  if command -v cargo &>/dev/null; then
    local cargo_ver
    cargo_ver=$(cargo --version)
    log_success "Cargo: ${cargo_ver}"
  else
    log_error "Cargo: not found."
    all_ok=false
  fi

  # Check Node
  if command -v node &>/dev/null; then
    local node_ver
    node_ver=$(node --version)
    log_success "Node.js: ${node_ver}"
  else
    log_error "Node.js: not found. Please install Node 20+"
    all_ok=false
  fi

  # Check pnpm
  if command -v pnpm &>/dev/null; then
    local pnpm_ver
    pnpm_ver=$(pnpm --version)
    log_success "pnpm: v${pnpm_ver}"
  else
    log_error "pnpm: not found. Please install pnpm ('corepack enable pnpm' or 'npm i -g pnpm')"
    all_ok=false
  fi

  # Check ShellCheck
  if command -v shellcheck &>/dev/null; then
    local sc_ver
    sc_ver=$(shellcheck --version | grep 'version:' || true)
    log_success "ShellCheck: available (${sc_ver})"
  else
    log_warn "ShellCheck: not installed (recommended for bash linting: 'brew install shellcheck')"
  fi

  # Check Docker
  if command -v docker &>/dev/null; then
    if docker info &>/dev/null; then
      log_success "Docker: available and daemon is running"
    else
      log_warn "Docker: CLI installed, but daemon is not running"
    fi
  else
    log_warn "Docker: not installed (only required for container builds/smoke tests)"
  fi

  # Report status of dev ports without modifying them
  local p_backend p_frontend
  p_backend=$(check_port "${COSAVE_BACKEND_PORT_DEV}")
  p_frontend=$(check_port "${COSAVE_FRONTEND_PORT}")
  if [[ -n "${p_backend}" ]]; then
    log_warn "Port ${COSAVE_BACKEND_PORT_DEV} (Backend Dev): currently in use by PID ${p_backend}"
  else
    log_success "Port ${COSAVE_BACKEND_PORT_DEV} (Backend Dev): free"
  fi
  if [[ -n "${p_frontend}" ]]; then
    log_warn "Port ${COSAVE_FRONTEND_PORT} (Frontend): currently in use by PID ${p_frontend}"
  else
    log_success "Port ${COSAVE_FRONTEND_PORT} (Frontend): free"
  fi

  if [[ "${all_ok}" == true ]]; then
    log_success "All core environment requirements are satisfied!"
  else
    die "Some environment requirements are missing. Please address the errors above."
  fi
}

cmd_doctor
