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
  echo -e "  ${GREEN}lint [--fix]${NC}          Run cargo fmt --check and clippy (-D warnings)"
  echo -e "  ${GREEN}format [--check]${NC}      Format Rust Leptos code using cargo fmt"
  echo -e "  ${GREEN}flint [--no-fix]${NC}      Format and lint Leptos code with auto-fix enabled"
  echo -e "  ${GREEN}test [args...]${NC}        Run cargo tests for frontend-leptos"
  echo -e "  ${GREEN}add <crate>${NC}           Add cargo dependency to frontend-leptos"
  echo -e "  ${GREEN}cargo <args...>${NC}       Passthrough directly to cargo with frontend-leptos manifest"
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

cmd_leptos_add() {
  if [[ $# -eq 0 || $(has_help_flag "$@" && echo true) == "true" ]]; then
    echo -e "${BOLD}Add Leptos Crate Dependency (${GREEN}./dev.sh leptos add <crate>${NC}):${NC}"
    echo -e "Adds crate dependency to frontend-leptos/Cargo.toml via cargo add."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo -e "  ${GREEN}./dev.sh leptos add <crate> [options]${NC}"
    return 0
  fi
  log_info "Adding crate dependency to Leptos frontend: $*..."
  cargo add --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml" "$@"
}

cmd_leptos_cargo() {
  local subcmd="${1:-}"
  shift || true
  cargo "${subcmd}" --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml" "$@"
}

cmd_leptos_lint() {
  if has_help_flag "$@"; then
    echo -e "${BOLD}Leptos Lint (${GREEN}./dev.sh leptos lint${NC}):${NC}"
    echo -e "Runs cargo fmt --check and clippy (-D warnings) for wasm32-unknown-unknown."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo -e "  ${GREEN}./dev.sh leptos lint${NC} [--fix]"
    return 0
  fi
  local fix=false
  local extra_args=()
  for arg in "$@"; do
    if [[ "${arg}" == "--fix" ]]; then
      fix=true
    else
      extra_args+=("${arg}")
    fi
  done

  log_info "Linting Leptos frontend (Rust)..."
  if [[ "${fix}" == true ]]; then
    cargo fmt --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml"
    cargo clippy --fix --allow-dirty --allow-staged --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml" --target wasm32-unknown-unknown -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
  else
    cargo fmt --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml" -- --check
    cargo clippy --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml" --target wasm32-unknown-unknown -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
  fi
  log_success "Leptos lint passed."
}

cmd_leptos_format() {
  if has_help_flag "$@"; then
    echo -e "${BOLD}Leptos Format (${GREEN}./dev.sh leptos format${NC}):${NC}"
    echo -e "Formats Rust Leptos code using cargo fmt."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo -e "  ${GREEN}./dev.sh leptos format${NC} [--check]"
    return 0
  fi
  log_info "Formatting Leptos frontend (cargo fmt)..."
  cargo fmt --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml" "$@"
  log_success "Leptos formatted successfully."
}

cmd_leptos_flint() {
  if has_help_flag "$@"; then
    echo -e "${BOLD}Leptos Fast Lint (${GREEN}./dev.sh leptos flint${NC}):${NC}"
    echo -e "Formats and runs clippy with auto-fix enabled."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo -e "  ${GREEN}./dev.sh leptos flint${NC} [--no-fix]"
    return 0
  fi
  local fix=true
  local extra_args=()
  for arg in "$@"; do
    if [[ "${arg}" == "--no-fix" ]]; then
      fix=false
    elif [[ "${arg}" != "--fix" ]]; then
      extra_args+=("${arg}")
    fi
  done

  log_info "Running Leptos flint (format + clippy)..."
  if [[ "${fix}" == true ]]; then
    cargo fmt --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml"
    cargo clippy --fix --allow-dirty --allow-staged --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml" --target wasm32-unknown-unknown -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
  else
    cargo fmt --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml" -- --check
    cargo clippy --manifest-path "${FRONTEND_LEPTOS_DIR}/Cargo.toml" --target wasm32-unknown-unknown -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
  fi
  log_success "Leptos flint passed."
}

# Main Action Dispatcher for Leptos
ACTION="${1:-help}"
shift || true

case "${ACTION}" in
  dev)        cmd_leptos_dev "$@" ;;
  build)      cmd_leptos_build "$@" ;;
  check)      cmd_leptos_check "$@" ;;
  lint)       cmd_leptos_lint "$@" ;;
  format|fmt) cmd_leptos_format "$@" ;;
  flint)      cmd_leptos_flint "$@" ;;
  test)       cmd_leptos_test "$@" ;;
  clean)      cmd_leptos_clean "$@" ;;
  add)        cmd_leptos_add "$@" ;;
  cargo)      cmd_leptos_cargo "$@" ;;
  help|--help|-h)
    leptos_help
    ;;
  *)
    die "Unknown leptos action: '${ACTION}'. Run './dev.sh leptos help' for valid actions."
    ;;
esac
