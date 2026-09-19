#!/usr/bin/env bash
# ==============================================================================
# CoSave Development CLI: Whole Workspace Subcommands
# Target: all
# ==============================================================================
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/dev/common.sh
source "${SCRIPT_DIR}/common.sh"

all_help() {
  echo -e "${BOLD}Workspace Commands (${GREEN}./dev.sh all <action>${NC}):${NC}"
  echo -e "  ${GREEN}test [args...]${NC}        Run backend cargo tests and frontend vitest tests"
  echo -e "  ${GREEN}check [args...]${NC}       Run backend cargo check and frontend svelte-check"
  echo -e "  ${GREEN}lint [--fix]${NC}          Run Rust, Frontend, and Bash (ShellCheck) linters"
  echo -e "  ${GREEN}format [--check]${NC}      Format backend and frontend code"
  echo -e "  ${GREEN}flint [--no-fix]${NC}      Format and lint all code with auto-fix enabled"
  echo -e "  ${GREEN}build${NC}                 Compile backend binary and build frontend static SPA"
  echo -e "  ${GREEN}fbuild${NC}                Fast build: flint -> check -> build"
  echo -e "  ${GREEN}audit${NC}                 Full pipeline: test -> check -> build -> flint --no-fix"
  echo -e "  ${GREEN}dev${NC}                   Start backend (:5171) and frontend (:5172) dev servers"
  echo -e "  ${GREEN}serve [opts]${NC}          Run production server (release binary serving static SPA)"
  echo -e "  ${GREEN}curl <path|url> [opts]${NC} Query backend or frontend API via curl"
  echo -e "  ${GREEN}clean${NC}                 Clean build artifacts in both backend and frontend"
  echo -e "  ${GREEN}exec <cmd...>${NC}         Execute arbitrary command from repository root"
}

cmd_all_test() {
  log_info "1/2 Running backend unit tests..."
  "${SCRIPT_DIR}/backend.sh" test "$@"
  log_info "2/2 Running frontend unit tests..."
  "${SCRIPT_DIR}/ui.sh" test "$@"
  log_success "All unit and contract tests passed."
}

cmd_all_check() {
  log_info "1/2 Running backend check..."
  "${SCRIPT_DIR}/backend.sh" check "$@"
  log_info "2/2 Running frontend check..."
  "${SCRIPT_DIR}/ui.sh" check "$@"
  log_success "All type and compiler checks passed."
}

cmd_all_lint() {
  local fix=false
  for arg in "$@"; do
    if [[ "${arg}" == "--fix" ]]; then
      fix=true
    fi
  done

  log_info "1/3 Linting backend..."
  if [[ "${fix}" == true ]]; then
    "${SCRIPT_DIR}/backend.sh" lint --fix
  else
    "${SCRIPT_DIR}/backend.sh" lint
  fi

  log_info "2/3 Linting frontend..."
  if [[ "${fix}" == true ]]; then
    "${SCRIPT_DIR}/ui.sh" lint --fix
  else
    "${SCRIPT_DIR}/ui.sh" lint
  fi

  log_info "3/3 Linting bash scripts (ShellCheck)..."
  if command -v shellcheck &>/dev/null; then
    shellcheck "${ROOT_DIR}/dev.sh" "${SCRIPT_DIR}"/*.sh
    log_success "ShellCheck passed on all bash scripts."
  else
    log_warn "ShellCheck not found; skipping bash linting."
  fi

  log_success "All linting checks passed."
}

cmd_all_format() {
  log_info "1/2 Formatting backend..."
  "${SCRIPT_DIR}/backend.sh" format "$@"
  log_info "2/2 Formatting frontend..."
  "${SCRIPT_DIR}/ui.sh" format
  log_success "All files formatted successfully."
}

cmd_all_flint() {
  local fix=true
  for arg in "$@"; do
    if [[ "${arg}" == "--no-fix" ]]; then
      fix=false
    elif [[ "${arg}" == "--fix" ]]; then
      fix=true
    fi
  done

  log_info "1/3 Running backend flint..."
  if [[ "${fix}" == true ]]; then
    "${SCRIPT_DIR}/backend.sh" flint --fix
  else
    "${SCRIPT_DIR}/backend.sh" flint --no-fix
  fi

  log_info "2/3 Running frontend flint..."
  if [[ "${fix}" == true ]]; then
    "${SCRIPT_DIR}/ui.sh" flint --fix
  else
    "${SCRIPT_DIR}/ui.sh" flint --no-fix
  fi

  log_info "3/3 Checking bash scripts with ShellCheck..."
  if command -v shellcheck &>/dev/null; then
    shellcheck "${ROOT_DIR}/dev.sh" "${SCRIPT_DIR}"/*.sh
    log_success "ShellCheck passed on all bash scripts."
  else
    log_warn "ShellCheck not found; skipping bash linting."
  fi

  log_success "Workspace flint completed."
}

cmd_all_build() {
  log_info "1/2 Compiling backend (release mode)..."
  "${SCRIPT_DIR}/backend.sh" build --release
  log_info "2/2 Building frontend static bundle..."
  "${SCRIPT_DIR}/ui.sh" build
  log_success "All build targets compiled successfully."
}

cmd_all_fbuild() {
  log_info "Running workspace fast build: flint -> check -> build..."
  cmd_all_flint --fix
  cmd_all_check
  cmd_all_build
  log_success "Workspace fast build completed successfully."
}

cmd_all_audit() {
  log_info "Running full verification audit pipeline: test -> check -> build -> flint..."
  cmd_all_test "$@"
  cmd_all_check
  cmd_all_build
  cmd_all_flint --no-fix
  log_success "Full workspace audit passed with zero errors/warnings."
}

cmd_all_clean() {
  log_info "Cleaning workspace artifacts..."
  "${SCRIPT_DIR}/backend.sh" clean
  "${SCRIPT_DIR}/ui.sh" clean
  log_success "Workspace cleaned."
}

cmd_all_dev() {
  # Port conflict detection
  local backend_occupied frontend_occupied
  backend_occupied=$(check_port "${COSAVE_BACKEND_PORT_DEV}")
  frontend_occupied=$(check_port "${COSAVE_FRONTEND_PORT}")

  if [[ -n "${backend_occupied}" || -n "${frontend_occupied}" ]]; then
    log_warn "Port conflict detected for development servers:"
    if [[ -n "${backend_occupied}" ]]; then
      log_warn "  - Port ${COSAVE_BACKEND_PORT_DEV} (Backend Dev) is in use by PID(s): $(echo "${backend_occupied}" | tr '\n' ' ')"
    fi
    if [[ -n "${frontend_occupied}" ]]; then
      log_warn "  - Port ${COSAVE_FRONTEND_PORT} (Frontend Dev) is in use by PID(s): $(echo "${frontend_occupied}" | tr '\n' ' ')"
    fi
    log_info "Server appears to be running outside (e.g. host terminal or IDE)."
    log_info "Skipping server startup. Existing instance at http://localhost:${COSAVE_FRONTEND_PORT} (UI) and http://localhost:${COSAVE_BACKEND_PORT_DEV} (API) is active."
    log_info "You can interact directly via './dev.sh curl <endpoint>' or browser."
    return 0
  fi

  log_info "Starting CoSave development servers..."
  log_info "  - Backend:  http://localhost:${COSAVE_BACKEND_PORT_DEV} (Axum API mode with debug logging)"
  log_info "  - Frontend: http://localhost:${COSAVE_FRONTEND_PORT} (Vite dev server with /api proxy)"

  cleanup_dev() {
    log_warn "Stopping development servers..."
    trap - EXIT INT TERM
    if [[ -n "${BACKEND_PID:-}" ]]; then
      kill "${BACKEND_PID}" 2>/dev/null || true
    fi
    if [[ -n "${FRONTEND_PID:-}" ]]; then
      kill "${FRONTEND_PID}" 2>/dev/null || true
    fi
    local pids
    pids=$(jobs -p 2>/dev/null || true)
    if [[ -n "${pids}" ]]; then
      kill "${pids}" 2>/dev/null || true
    fi
  }
  trap cleanup_dev EXIT INT TERM

  # Start backend in background
  cargo run --manifest-path "${BACKEND_DIR}/Cargo.toml" -- api --env "${COSAVE_ENV_DEV}" --host 0.0.0.0 --port "${COSAVE_BACKEND_PORT_DEV}" -v &
  BACKEND_PID=$!

  # Start frontend Vite in background
  (cd "${FRONTEND_DIR}" && pnpm run dev) &
  FRONTEND_PID=$!

  wait "${BACKEND_PID}" "${FRONTEND_PID}"
}

cmd_all_serve() {
  local serve_port="${PORT:-${COSAVE_BACKEND_PORT_PROD}}"
  local occupying_pids
  occupying_pids=$(check_port "${serve_port}")
  if [[ -n "${occupying_pids}" ]]; then
    log_warn "Port ${serve_port} is already in use by PID(s): $(echo "${occupying_pids}" | tr '\n' ' ')"
    log_info "Production server appears to be running outside."
    log_info "Skipping server startup. Existing instance at http://localhost:${serve_port} is active."
    log_info "You can interact directly via './dev.sh curl <endpoint>' or browser."
    return 0
  fi

  local build_ui=true
  local server_args=()
  for arg in "$@"; do
    if [[ "${arg}" == "--no-build" || "${arg}" == "-n" ]]; then
      build_ui=false
    elif [[ "${arg}" == "--build" || "${arg}" == "-b" ]]; then
      build_ui=true
    else
      server_args+=("${arg}")
    fi
  done

  if [[ "${build_ui}" == true || ! -d "${FRONTEND_DIR}/dist" ]]; then
    log_info "Building frontend static SPA bundle..."
    "${SCRIPT_DIR}/ui.sh" build
  fi

  log_info "Starting CoSave production server (Axum release mode on :${serve_port})..."
  log_info "  - Serving static SPA from: ${FRONTEND_DIR}/dist"
  cargo run --release --manifest-path "${BACKEND_DIR}/Cargo.toml" -- --env "${COSAVE_ENV_PROD}" --host 0.0.0.0 --port "${serve_port}" --static-dir "${FRONTEND_DIR}/dist" ${server_args[@]+"${server_args[@]}"}
}

cmd_all_exec() {
  if [[ $# -eq 0 ]]; then
    die "No command specified. Usage: ./dev.sh all exec <command> [args...]"
  fi
  (cd "${ROOT_DIR}" && "$@")
}

cmd_all_curl() {
  cmd_curl "$@"
}

# Main Action Dispatcher for All
ACTION="${1:-help}"
shift || true

case "${ACTION}" in
  test)       cmd_all_test "$@" ;;
  check)      cmd_all_check "$@" ;;
  lint)       cmd_all_lint "$@" ;;
  format|fmt) cmd_all_format "$@" ;;
  flint)      cmd_all_flint "$@" ;;
  build)      cmd_all_build ;;
  fbuild)     cmd_all_fbuild ;;
  audit)      cmd_all_audit "$@" ;;
  dev)        cmd_all_dev ;;
  serve)      cmd_all_serve "$@" ;;
  curl)       cmd_all_curl "$@" ;;
  clean)      cmd_all_clean ;;
  exec)       cmd_all_exec "$@" ;;
  help|--help|-h)
    all_help
    ;;
  *)
    die "Unknown all action: '${ACTION}'. Run './dev.sh all help' for valid actions."
    ;;
esac
