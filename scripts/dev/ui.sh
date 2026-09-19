#!/usr/bin/env bash
# ==============================================================================
# CoSave Development CLI: UI / Frontend Subcommands
# Target: ui
# ==============================================================================
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/dev/common.sh
source "${SCRIPT_DIR}/common.sh"

ui_help() {
  echo -e "${BOLD}UI Commands (${GREEN}./dev.sh ui <action>${NC}):${NC}"
  echo -e "  ${GREEN}test [args...]${NC}        Run Vitest unit/contract tests (or 'feature <name>')"
  echo -e "  ${GREEN}check [args...]${NC}       Run svelte-check and canonical Tailwind check"
  echo -e "  ${GREEN}lint [--fix]${NC}          Run ESLint, canonical classes check, and Prettier"
  echo -e "  ${GREEN}format [--check]${NC}      Format code via canonical Tailwind & Prettier"
  echo -e "  ${GREEN}flint [--no-fix]${NC}      Format and lint frontend with auto-fix enabled"
  echo -e "  ${GREEN}build${NC}                 Compile SvelteKit static SPA into dist/"
  echo -e "  ${GREEN}fbuild${NC}                Fast build: flint -> check -> build"
  echo -e "  ${GREEN}audit${NC}                 Full pipeline: test -> check -> build -> flint --no-fix"
  echo -e "  ${GREEN}clean${NC}                 Clean frontend dist/ and .svelte-kit directories"
  echo -e "  ${GREEN}add <pkg> [opts]${NC}      Add npm package dependency via pnpm"
  echo -e "  ${GREEN}shadcn <comp> [opts]${NC}  Add shadcn-svelte component (auto-injects -y and -o/--overwrite)"
  echo -e "  ${GREEN}node <args...>${NC}        Run Node.js inside frontend/ directory"
  echo -e "  ${GREEN}exec <cmd...>${NC}         Execute command in frontend/ with node_modules/.bin in PATH"
  echo -e "  ${GREEN}pnpm <args...>${NC}        Direct pnpm passthrough inside frontend/ directory"
  echo -e "  ${GREEN}capture [url] [dir]${NC}   Capture desktop and mobile screenshots for ui-review"
}

cmd_ui_test() {
  local test_args=()
  if [[ "${1:-}" == "feature" ]]; then
    shift
    local feat="${1:-}"
    shift || true
    if [[ -z "${feat}" ]]; then
      die "No feature specified. Usage: ./dev.sh ui test feature <name> [args]"
    fi
    local feat_target="src/lib/features/${feat}"
    if [[ -f "${FRONTEND_DIR}/${feat_target}/api.test.ts" ]]; then
      log_info "Running contract tests for feature '${feat}' (vitest)..."
      test_args+=("${feat_target}/api.test.ts" "$@")
    elif [[ -d "${FRONTEND_DIR}/${feat_target}" ]]; then
      log_info "Running tests for feature '${feat}' (vitest)..."
      test_args+=("${feat_target}" "$@")
    else
      die "Feature '${feat}' not found in src/lib/features/."
    fi
  elif [[ -n "${1:-}" && -d "${FRONTEND_DIR}/src/lib/features/${1}" && "${1}" != -* ]]; then
    local feat="$1"
    shift || true
    local feat_target="src/lib/features/${feat}"
    if [[ -f "${FRONTEND_DIR}/${feat_target}/api.test.ts" ]]; then
      log_info "Running contract tests for feature '${feat}' (vitest)..."
      test_args+=("${feat_target}/api.test.ts" "$@")
    else
      log_info "Running tests for feature '${feat}' (vitest)..."
      test_args+=("${feat_target}" "$@")
    fi
  else
    log_info "Running frontend unit tests (vitest)..."
    if [[ $# -gt 0 ]]; then
      test_args=("$@")
    fi
  fi

  if [[ ${#test_args[@]} -gt 0 ]]; then
    (cd "${FRONTEND_DIR}" && pnpm run test "${test_args[@]}")
  else
    (cd "${FRONTEND_DIR}" && pnpm run test)
  fi
  log_success "Frontend tests passed."
}

cmd_ui_check() {
  log_info "Running frontend type check & Tailwind canonical check..."
  (cd "${FRONTEND_DIR}" && pnpm run check "$@")
  log_success "Frontend checks passed."
}

cmd_ui_lint() {
  local fix=false
  for arg in "$@"; do
    if [[ "${arg}" == "--fix" ]]; then
      fix=true
    fi
  done

  log_info "Linting frontend (SvelteKit 2 / TypeScript / Tailwind)..."
  if [[ "${fix}" == true ]]; then
    (cd "${FRONTEND_DIR}" && pnpm run format && pnpm run lint:fix)
  else
    (cd "${FRONTEND_DIR}" && pnpm run check && pnpm run lint && pnpm run format:check)
  fi
  log_success "Frontend lint passed."
}

cmd_ui_format() {
  log_info "Formatting frontend (canonical Tailwind & Prettier)..."
  (cd "${FRONTEND_DIR}" && pnpm run format)
  log_success "Frontend formatted successfully."
}

cmd_ui_flint() {
  local fix=true
  for arg in "$@"; do
    if [[ "${arg}" == "--no-fix" ]]; then
      fix=false
    elif [[ "${arg}" == "--fix" ]]; then
      fix=true
    fi
  done

  log_info "Running frontend flint (format + lint)..."
  if [[ "${fix}" == true ]]; then
    cmd_ui_format
    (cd "${FRONTEND_DIR}" && pnpm run lint:fix)
  else
    (cd "${FRONTEND_DIR}" && pnpm run lint && pnpm run format:check)
  fi
  log_success "Frontend flint passed."
}

cmd_ui_build() {
  log_info "Building frontend static bundle (SvelteKit)..."
  (cd "${FRONTEND_DIR}" && pnpm run build "$@")
  log_success "Frontend build completed."
}

cmd_ui_fbuild() {
  log_info "Running frontend fast build: flint -> check -> build..."
  cmd_ui_flint --fix
  cmd_ui_check
  cmd_ui_build
  log_success "Frontend fast build completed successfully."
}

cmd_ui_audit() {
  log_info "Running full frontend audit pipeline: test -> check -> build -> flint..."
  cmd_ui_test "$@"
  cmd_ui_check
  cmd_ui_build
  cmd_ui_flint --no-fix
  log_success "Frontend audit passed with zero errors/warnings."
}

cmd_ui_clean() {
  log_info "Cleaning frontend build artifacts..."
  rm -rf "${FRONTEND_DIR}/dist" "${FRONTEND_DIR}/.svelte-kit"
  log_success "Frontend build artifacts removed."
}

cmd_ui_add() {
  if [[ $# -eq 0 ]]; then
    die "No package specified. Usage: ./dev.sh ui add <package> [options]"
  fi
  log_info "Adding package dependency: $*..."
  (cd "${FRONTEND_DIR}" && pnpm add "$@")
}

cmd_ui_shadcn() {
  if [[ $# -eq 0 ]]; then
    die "No component specified. Usage: ./dev.sh ui shadcn <component> [options]"
  fi

  # Ensure -y (non-interactive) and -o / --overwrite are passed so AI agents never hang
  local has_yes=false
  local has_overwrite=false
  local forwarded_args=()

  for arg in "$@"; do
    if [[ "${arg}" == "-y" || "${arg}" == "--yes" ]]; then
      has_yes=true
    fi
    if [[ "${arg}" == "-o" || "${arg}" == "--overwrite" ]]; then
      has_overwrite=true
    fi
    forwarded_args+=("${arg}")
  done

  local auto_flags=()
  if [[ "${has_yes}" == false ]]; then
    auto_flags+=("-y")
  fi
  if [[ "${has_overwrite}" == false ]]; then
    auto_flags+=("-o")
  fi

  log_info "Adding shadcn-svelte component with auto-overwrite: ${forwarded_args[*]}..."
  (cd "${FRONTEND_DIR}" && pnpm dlx shadcn-svelte@latest add "${auto_flags[@]}" "${forwarded_args[@]}")
  log_success "Component installation completed."
}

cmd_ui_node() {
  log_info "Running Node.js in frontend context..."
  (cd "${FRONTEND_DIR}" && node "$@")
}

cmd_ui_exec() {
  if [[ $# -eq 0 ]]; then
    die "No command specified. Usage: ./dev.sh ui exec <command> [args...]"
  fi
  (cd "${FRONTEND_DIR}" && PATH="${FRONTEND_DIR}/node_modules/.bin:${PATH}" "$@")
}

cmd_ui_pnpm() {
  (cd "${FRONTEND_DIR}" && pnpm "$@")
}

cmd_ui_capture() {
  if [[ "${1:-}" == "--help" || "${1:-}" == "-h" || "${1:-}" == "help" ]]; then
    echo -e "${BOLD}UI Screenshot Capture Helper${NC}"
    echo -e "Usage: ${GREEN}./dev.sh ui capture [target-url|path] [output-dir]${NC}"
    echo -e "Defaults: url=http://localhost:${COSAVE_FRONTEND_PORT}, output=.scratch/ui-review"
    echo -e "Examples:"
    echo -e "  ./dev.sh ui capture"
    echo -e "  ./dev.sh ui capture /configuration/family"
    echo -e "  ./dev.sh ui capture http://localhost:5172/settings"
    return 0
  fi
  local target_url="${1:-http://localhost:${COSAVE_FRONTEND_PORT}}"
  if [[ "${target_url}" =~ ^/ ]]; then
    target_url="http://localhost:${COSAVE_FRONTEND_PORT}${target_url}"
  fi
  local output_dir="${2:-${ROOT_DIR}/.scratch/ui-review}"
  log_info "Capturing screenshots for ${target_url} into ${output_dir}..."
  node "${ROOT_DIR}/.agents/skills/ui-review/scripts/capture.js" "${target_url}" "${output_dir}"
}

# Main Action Dispatcher for UI
ACTION="${1:-help}"
shift || true

case "${ACTION}" in
  test)       cmd_ui_test "$@" ;;
  check)      cmd_ui_check "$@" ;;
  lint)       cmd_ui_lint "$@" ;;
  format|fmt) cmd_ui_format ;;
  flint)      cmd_ui_flint "$@" ;;
  build)      cmd_ui_build "$@" ;;
  fbuild)     cmd_ui_fbuild ;;
  audit)      cmd_ui_audit "$@" ;;
  clean)      cmd_ui_clean ;;
  add)        cmd_ui_add "$@" ;;
  shadcn)     cmd_ui_shadcn "$@" ;;
  node)       cmd_ui_node "$@" ;;
  exec)       cmd_ui_exec "$@" ;;
  pnpm)       cmd_ui_pnpm "$@" ;;
  capture)    cmd_ui_capture "$@" ;;
  help|--help|-h)
    ui_help
    ;;
  *)
    die "Unknown ui action: '${ACTION}'. Run './dev.sh ui help' for valid actions."
    ;;
esac
