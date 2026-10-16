#!/usr/bin/env bash
# ==============================================================================
# CoSave Development CLI: Leptos RS Frontend Subcommands
# Target: leptos
# ==============================================================================
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/dev/common.sh
source "${SCRIPT_DIR}/common.sh"

leptos_help() {
  echo -e "${BOLD}Leptos Commands (${GREEN}./dev.sh leptos <action>${NC}):${NC}"
  echo -e "  ${GREEN}dev [args...]${NC}         Start Leptos dev server on :${COSAVE_LEPTOS_PORT:-5173} with Trunk"
  echo -e "  ${GREEN}build [args...]${NC}       Compile Leptos WASM bundle into frontend-leptos/dist/"
  echo -e "  ${GREEN}check [args...]${NC}       Run cargo check for wasm32-unknown-unknown"
  echo -e "  ${GREEN}test [args...]${NC}        Run cargo tests for frontend-leptos"
  echo -e "  ${GREEN}clean${NC}                 Clean frontend-leptos/dist/ and target/ directories"
}

cmd_leptos_dev() {
  if has_help_flag "$@"; then
    echo -e "${BOLD}Leptos Dev Server (${GREEN}./dev.sh leptos dev${NC}):${NC}"
    echo -e "Starts the Leptos development server on port ${COSAVE_LEPTOS_PORT:-5173} via Trunk."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo -e "  ${GREEN}./dev.sh leptos dev${NC} [trunk options...]"
    return 0
  fi
  local port="${COSAVE_LEPTOS_PORT:-5173}"
  log_info "Starting Leptos development server with Trunk on port ${port} (0.0.0.0)..."
  (cd "${FRONTEND_LEPTOS_DIR}" && trunk serve --address 0.0.0.0 --port "${port}" "$@")
}

cmd_leptos_build() {
  if has_help_flag "$@"; then
    echo -e "${BOLD}Leptos Build (${GREEN}./dev.sh leptos build${NC}):${NC}"
    echo -e "Compiles the Leptos frontend bundle into frontend-leptos/dist/ via Trunk."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo -e "  ${GREEN}./dev.sh leptos build${NC} [trunk options...]"
    return 0
  fi
  log_info "Building Leptos frontend bundle (Trunk)..."
  (cd "${FRONTEND_LEPTOS_DIR}" && trunk build "$@")
  log_success "Leptos build completed."
}

cmd_leptos_check() {
  if has_help_flag "$@"; then
    echo -e "${BOLD}Leptos Check (${GREEN}./dev.sh leptos check${NC}):${NC}"
    echo -e "Runs cargo check targeting wasm32-unknown-unknown."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo -e "  ${GREEN}./dev.sh leptos check${NC} [cargo options...]"
    return 0
  fi
  log_info "Checking Leptos frontend (wasm32-unknown-unknown)..."
  cargo check --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml" --target wasm32-unknown-unknown "$@"
  log_success "Leptos check passed."
}

cmd_leptos_test() {
  if has_help_flag "$@"; then
    echo -e "${BOLD}Leptos Tests (${GREEN}./dev.sh leptos test${NC}):${NC}"
    echo -e "Runs cargo test for frontend-leptos."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo -e "  ${GREEN}./dev.sh leptos test${NC} [cargo options...]"
    return 0
  fi
  log_info "Running Leptos tests..."
  cargo test --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml" "$@"
  log_success "Leptos tests passed."
}

cmd_leptos_clean() {
  if has_help_flag "$@"; then
    echo -e "${BOLD}Clean Leptos Artifacts (${GREEN}./dev.sh leptos clean${NC}):${NC}"
    echo -e "Removes frontend-leptos/dist/ and target/ directories."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo -e "  ${GREEN}./dev.sh leptos clean${NC}"
    return 0
  fi
  log_info "Cleaning Leptos build artifacts..."
  rm -rf "${FRONTEND_LEPTOS_DIR}/dist" "${FRONTEND_LEPTOS_DIR}/target"
  log_success "Leptos build artifacts removed."
}

# Main Action Dispatcher for Leptos
ACTION="${1:-help}"
shift || true

case "${ACTION}" in
  dev)        cmd_leptos_dev "$@" ;;
  build)      cmd_leptos_build "$@" ;;
  check)      cmd_leptos_check "$@" ;;
  test)       cmd_leptos_test "$@" ;;
  clean)      cmd_leptos_clean "$@" ;;
  help|--help|-h)
    leptos_help
    ;;
  *)
    die "Unknown leptos action: '${ACTION}'. Run './dev.sh leptos help' for valid actions."
    ;;
esac
