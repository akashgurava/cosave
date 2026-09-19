#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# CoSave Development Helper Script
# ==============================================================================

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="${ROOT_DIR}/backend"
FRONTEND_DIR="${ROOT_DIR}/frontend"

# Centralized Port & Environment Constants
readonly COSAVE_ENV_DEV="DEV"
readonly COSAVE_ENV_PROD="PROD"
readonly COSAVE_FRONTEND_PORT=5172
readonly COSAVE_BACKEND_PORT_DEV=5171
readonly COSAVE_BACKEND_PORT_PROD=5172

# Colors for output
BOLD='\033[1m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

log_info() {
  echo -e "${BLUE}${BOLD}[cosave]${NC} $1"
}

log_success() {
  echo -e "${GREEN}${BOLD}[cosave ✓]${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}${BOLD}[cosave ⚠]${NC} $1"
}

log_error() {
  echo -e "${RED}${BOLD}[cosave ✗]${NC} $1"
}

check_port() {
  local port="$1"
  if command -v lsof &>/dev/null; then
    lsof -ti ":${port}" 2>/dev/null || true
  fi
}

# ------------------------------------------------------------------------------
# Backend Subcommands: ./dev.sh backend <lint|format|check|test|build|serve|add>
# ------------------------------------------------------------------------------
cmd_backend() {
  local subcmd="${1:-help}"
  shift || true

  if [[ "${subcmd}" == "--help" || "${subcmd}" == "-h" || "${subcmd}" == "help" ]]; then
    echo -e "${BOLD}Backend Commands:${NC}"
    echo -e "  ${GREEN}./dev.sh backend full [--no-fix]${NC} Run full pipeline: test -> check -> build -> flint (auto-fixes)"
    echo -e "  ${GREEN}./dev.sh backend fbuild${NC}         Fast build: check -> flint (auto-fixes) -> build (release)"
    echo -e "  ${GREEN}./dev.sh backend flint [--no-fix]${NC} Format (cargo fmt) and lint (clippy) (auto-fixes)"
    echo -e "  ${GREEN}./dev.sh backend lint [--fix]${NC}   Run cargo fmt and clippy (-D warnings)"
    echo -e "  ${GREEN}./dev.sh backend format${NC}        Run cargo fmt"
    echo -e "  ${GREEN}./dev.sh backend check${NC}         Run cargo check"
    echo -e "  ${GREEN}./dev.sh backend test [args]${NC}   Run cargo test"
    echo -e "  ${GREEN}./dev.sh backend build [args]${NC}  Compile backend (e.g. --release)"
    echo -e "  ${GREEN}./dev.sh backend dev [args]${NC}    Run backend dev server (-v)"
    echo -e "  ${GREEN}./dev.sh backend serve [args]${NC}  Run backend production server (release mode)"
    echo -e "  ${GREEN}./dev.sh backend add <crate>${NC}   Add crate dependency"
    return 0
  fi

  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      case "${subcmd}" in
        full)
          echo -e "${BOLD}Backend Full:${NC} Runs test -> check -> build -> flint in order."
          ;;
        fbuild)
          echo -e "${BOLD}Backend Fast Build:${NC} Runs check -> flint (auto-fixes) -> build --release without running servers or unit tests."
          ;;
        lint)
          echo -e "${BOLD}Backend Lint:${NC} Runs cargo fmt --check and clippy (-D warnings). Use --fix to auto-fix."
          ;;
        flint)
          echo -e "${BOLD}Backend Flint:${NC} Formats and lints backend code (auto-fixes by default). Use --no-fix for check-only mode."
          ;;
        format|fmt)
          echo -e "${BOLD}Backend Format:${NC} Runs cargo fmt."
          ;;
        check)
          echo -e "${BOLD}Backend Check:${NC} Runs cargo check."
          ;;
        test)
          echo -e "${BOLD}Backend Test:${NC} Runs cargo test [args]."
          ;;
        build)
          echo -e "${BOLD}Backend Build:${NC} Compiles backend (e.g. ./dev.sh backend build --release)."
          ;;
        dev|run)
          echo -e "${BOLD}Backend Dev:${NC} Runs backend dev server with verbose logging."
          ;;
        serve)
          echo -e "${BOLD}Backend Serve:${NC} Runs backend production server in release mode."
          ;;
        add)
          echo -e "${BOLD}Backend Add:${NC} Adds crate dependency via cargo."
          ;;
      esac
      return 0
    fi
  done

  case "${subcmd}" in
    full)
      local fix=true
      local extra_args=()
      for arg in "$@"; do
        if [[ "$arg" == "--no-fix" ]]; then
          fix=false
        elif [[ "$arg" != "--fix" ]]; then
          extra_args+=("$arg")
        fi
      done
      log_info "Running full backend pipeline: test -> check -> build -> flint..."
      if [[ ${#extra_args[@]} -gt 0 ]]; then
        cmd_backend test "${extra_args[@]}"
      else
        cmd_backend test
      fi
      cmd_backend check
      cmd_backend build --release
      if [[ "$fix" == true ]]; then
        cmd_backend flint --fix
      else
        cmd_backend flint --no-fix
      fi
      log_success "Full backend pipeline completed successfully!"
      ;;
    fbuild)
      log_info "Running backend fast build: check -> flint (auto-fixes) -> build..."
      cmd_backend check
      cmd_backend flint --fix
      cmd_backend build --release
      log_success "Backend fast build completed successfully!"
      ;;
    lint)
      local fix=false
      if [[ "${1:-}" == "--fix" ]]; then
        fix=true
        shift || true
      fi
      log_info "Linting backend (Rust)..."
      if [[ "$fix" == true ]]; then
        cargo fmt --manifest-path "${BACKEND_DIR}/Cargo.toml"
        cargo clippy --fix --allow-dirty --allow-staged --manifest-path "${BACKEND_DIR}/Cargo.toml" -- -D warnings "$@"
      else
        cargo fmt --manifest-path "${BACKEND_DIR}/Cargo.toml" -- --check
        cargo clippy --manifest-path "${BACKEND_DIR}/Cargo.toml" -- -D warnings "$@"
      fi
      log_success "Backend lint passed."
      ;;
    flint)
      local fix=true
      local extra_args=()
      for arg in "$@"; do
        if [[ "$arg" == "--no-fix" ]]; then
          fix=false
        elif [[ "$arg" != "--fix" ]]; then
          extra_args+=("$arg")
        fi
      done
      log_info "Running backend flint (format + lint)..."
      if [[ "$fix" == true ]]; then
        cmd_backend format
        if [[ ${#extra_args[@]} -gt 0 ]]; then
          cmd_backend lint --fix "${extra_args[@]}"
        else
          cmd_backend lint --fix
        fi
      else
        if [[ ${#extra_args[@]} -gt 0 ]]; then
          cmd_backend lint "${extra_args[@]}"
        else
          cmd_backend lint
        fi
      fi
      log_success "Backend flint passed."
      ;;
    format|fmt)
      log_info "Formatting backend..."
      cargo fmt --manifest-path "${BACKEND_DIR}/Cargo.toml" "$@"
      log_success "Backend formatted successfully!"
      ;;
    check)
      log_info "Running cargo check on backend..."
      cargo check --manifest-path "${BACKEND_DIR}/Cargo.toml" "$@"
      log_success "Backend check passed."
      ;;
    test)
      log_info "Running backend unit tests (cargo test)..."
      cargo test --manifest-path "${BACKEND_DIR}/Cargo.toml" "$@"
      log_success "Backend tests passed."
      ;;
    build)
      log_info "Compiling backend..."
      cargo build --manifest-path "${BACKEND_DIR}/Cargo.toml" "$@"
      log_success "Backend built successfully."
      ;;
    dev|run)
      local dev_port="${PORT:-${COSAVE_BACKEND_PORT_DEV}}"
      local occupying_pids
      occupying_pids=$(check_port "${dev_port}")
      if [[ -n "${occupying_pids}" ]]; then
        log_error "Port ${dev_port} is currently in use (PID: $(echo "${occupying_pids}" | tr '\n' ' ')). Please stop the occupying process before starting backend dev server."
        return 1
      fi
      log_info "Starting backend dev server (Axum on :${dev_port}, env: ${COSAVE_ENV_DEV})..."
      cargo run --manifest-path "${BACKEND_DIR}/Cargo.toml" -- api --env "${COSAVE_ENV_DEV}" --host 0.0.0.0 --port "${dev_port}" -v "$@"
      ;;
    serve)
      local serve_port="${PORT:-${COSAVE_BACKEND_PORT_PROD}}"
      local occupying_pids
      occupying_pids=$(check_port "${serve_port}")
      if [[ -n "${occupying_pids}" ]]; then
        log_error "Port ${serve_port} is currently in use (PID: $(echo "${occupying_pids}" | tr '\n' ' ')). Please stop the occupying process before starting backend production server."
        return 1
      fi
      log_info "Starting backend production server (Axum release mode on :${serve_port}, env: ${COSAVE_ENV_PROD})..."
      cargo run --release --manifest-path "${BACKEND_DIR}/Cargo.toml" -- --env "${COSAVE_ENV_PROD}" --host 0.0.0.0 --port "${serve_port}" --static-dir "${FRONTEND_DIR}/dist" "$@"
      ;;
    add)
      if [[ $# -eq 0 ]]; then
        log_error "No crate specified. Usage: ./dev.sh backend add <crate> [options]"
        exit 1
      fi
      log_info "Adding crate dependency: $*..."
      cargo add --manifest-path "${BACKEND_DIR}/Cargo.toml" "$@"
      ;;
    *)
      log_error "Unknown backend command: ${subcmd}"
      cmd_backend help
      exit 1
      ;;
  esac
}

# ------------------------------------------------------------------------------
# UI / Frontend Subcommands: ./dev.sh ui <lint|format|check|test|build|serve|add>
# ------------------------------------------------------------------------------
cmd_ui() {
  local subcmd="${1:-help}"
  shift || true

  if [[ "${subcmd}" == "--help" || "${subcmd}" == "-h" || "${subcmd}" == "help" ]]; then
    echo -e "${BOLD}UI Commands:${NC}"
    echo -e "  ${GREEN}./dev.sh ui full [--no-fix]${NC}      Run full pipeline: test -> check -> build -> flint (auto-fixes)"
    echo -e "  ${GREEN}./dev.sh ui fbuild${NC}              Fast build: check -> flint (auto-fixes) -> build"
    echo -e "  ${GREEN}./dev.sh ui flint [--no-fix]${NC}       Run format and lint (auto-fixes by default)"
    echo -e "  ${GREEN}./dev.sh ui lint [--fix]${NC}        Run svelte-check, canonical classes, ESLint, Prettier"
    echo -e "  ${GREEN}./dev.sh ui format${NC}              Format with canonical Tailwind & Prettier"
    echo -e "  ${GREEN}./dev.sh ui check${NC}               Run svelte-check and canonical classes check"
    echo -e "  ${GREEN}./dev.sh ui test [feature <name>|args]${NC} Run vitest unit tests (or feature contract tests)"
    echo -e "  ${GREEN}./dev.sh ui build${NC}               Build SvelteKit static SPA into dist/"
    echo -e "  ${GREEN}./dev.sh ui dev [args]${NC}          Start Vite dev server (:5172)"
    echo -e "  ${GREEN}./dev.sh ui serve [args]${NC}        Preview compiled static SPA (vite preview)"
    echo -e "  ${GREEN}./dev.sh ui add <pkg>${NC}           Add dependency via pnpm"
    echo -e "  ${GREEN}./dev.sh ui shadcn <component>${NC}  Add shadcn-svelte primitive component"
    return 0
  fi

  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      case "${subcmd}" in
        full)
          echo -e "${BOLD}UI Full:${NC} Runs test -> check -> build -> flint in order. Use --fix to auto-fix formatting."
          ;;
        fbuild)
          echo -e "${BOLD}UI Fast Build:${NC} Runs check -> flint (auto-fixes) -> build without running servers or unit tests."
          ;;
        flint)
          echo -e "${BOLD}UI Flint:${NC} Formats (canonical Tailwind & Prettier) and lints frontend (auto-fixes by default). Use --no-fix for check-only mode."
          ;;
        lint)
          echo -e "${BOLD}UI Lint:${NC} Runs svelte-check, canonical classes, ESLint, and Prettier."
          ;;
        format|fmt)
          echo -e "${BOLD}UI Format:${NC} Auto-formats via canonical Tailwind and Prettier."
          ;;
        check)
          echo -e "${BOLD}UI Check:${NC} Runs svelte-check and canonical Tailwind classes check."
          ;;
        test)
          echo -e "${BOLD}UI Test:${NC} Runs vitest unit tests. Supports '${GREEN}./dev.sh ui test feature <name>${NC}' to run contract tests for a specific feature."
          ;;
        build)
          echo -e "${BOLD}UI Build:${NC} Builds SvelteKit static SPA into dist/."
          ;;
        dev|run)
          echo -e "${BOLD}UI Dev:${NC} Starts Vite dev server on :5172."
          ;;
        serve|preview)
          echo -e "${BOLD}UI Serve:${NC} Previews the compiled static SPA bundle via Vite preview."
          ;;
        add)
          echo -e "${BOLD}UI Add:${NC} Adds package dependency via pnpm."
          ;;
        shadcn|add-ui)
          echo -e "${BOLD}UI Shadcn:${NC} Adds shadcn-svelte primitive component non-interactively."
          ;;
      esac
      return 0
    fi
  done

  case "${subcmd}" in
    full)
      local fix=true
      local extra_args=()
      for arg in "$@"; do
        if [[ "$arg" == "--no-fix" ]]; then
          fix=false
        elif [[ "$arg" != "--fix" ]]; then
          extra_args+=("$arg")
        fi
      done
      log_info "Running full UI pipeline: test -> check -> build -> flint..."
      log_info "1/4 Running frontend unit tests..."
      if [[ ${#extra_args[@]} -gt 0 ]]; then
        cmd_ui test "${extra_args[@]}"
      else
        cmd_ui test
      fi
      log_info "2/4 Running frontend type & canonical checks..."
      cmd_ui check
      log_info "3/4 Building frontend static bundle..."
      cmd_ui build
      log_info "4/4 Running frontend flint (format + lint)..."
      if [[ "$fix" == true ]]; then
        cmd_ui flint --fix
      else
        cmd_ui flint --no-fix
      fi
      log_success "Full UI pipeline completed successfully!"
      ;;
    fbuild)
      log_info "Running frontend fast build: check -> flint (auto-fixes) -> build..."
      cmd_ui check
      cmd_ui flint --fix
      cmd_ui build
      log_success "Frontend fast build completed successfully!"
      ;;
    lint)
      local fix=false
      if [[ "${1:-}" == "--fix" ]]; then
        fix=true
        shift || true
      fi
      log_info "Linting frontend (SvelteKit 2 / TypeScript / Tailwind)..."
      if [[ "$fix" == true ]]; then
        (cd "${FRONTEND_DIR}" && pnpm run format && pnpm run lint:fix)
      else
        (cd "${FRONTEND_DIR}" && pnpm run check && pnpm run lint && pnpm run format:check)
      fi
      log_success "Frontend lint passed."
      ;;
    flint)
      local fix=true
      local extra_args=()
      for arg in "$@"; do
        if [[ "$arg" == "--no-fix" ]]; then
          fix=false
        elif [[ "$arg" != "--fix" ]]; then
          extra_args+=("$arg")
        fi
      done
      log_info "Running frontend flint (format + lint)..."
      if [[ "$fix" == true ]]; then
        cmd_ui format
        if [[ ${#extra_args[@]} -gt 0 ]]; then
          cmd_ui lint --fix "${extra_args[@]}"
        else
          cmd_ui lint --fix
        fi
      else
        if [[ ${#extra_args[@]} -gt 0 ]]; then
          cmd_ui lint "${extra_args[@]}"
        else
          cmd_ui lint
        fi
      fi
      log_success "Frontend flint passed."
      ;;
    format|fmt)
      log_info "Formatting frontend (canonical Tailwind & Prettier)..."
      (cd "${FRONTEND_DIR}" && pnpm run format)
      log_success "Frontend formatted successfully!"
      ;;
    check)
      log_info "Running frontend type check & Tailwind canonical check..."
      (cd "${FRONTEND_DIR}" && pnpm run check)
      log_success "Frontend checks passed."
      ;;
    test)
      local test_args=()
      if [[ "${1:-}" == "feature" ]]; then
        shift
        local feat="${1:-}"
        shift || true
        if [[ -z "${feat}" ]]; then
          log_error "No feature specified. Usage: ./dev.sh ui test feature <name> [args]"
          return 1
        fi
        local feat_target="src/lib/features/${feat}"
        if [[ -f "${FRONTEND_DIR}/${feat_target}/api.test.ts" ]]; then
          log_info "Running contract tests for feature '${feat}' (vitest)..."
          test_args+=("${feat_target}/api.test.ts" "$@")
        elif [[ -d "${FRONTEND_DIR}/${feat_target}" ]]; then
          log_info "Running tests for feature '${feat}' (vitest)..."
          test_args+=("${feat_target}" "$@")
        else
          log_error "Feature '${feat}' not found in src/lib/features/."
          return 1
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
      ;;
    build)
      log_info "Building frontend static bundle (SvelteKit)..."
      (cd "${FRONTEND_DIR}" && pnpm run build)
      log_success "Frontend build completed."
      ;;
    dev|run)
      local occupying_pids
      occupying_pids=$(check_port "${COSAVE_FRONTEND_PORT}")
      if [[ -n "${occupying_pids}" ]]; then
        log_error "Port ${COSAVE_FRONTEND_PORT} is currently in use (PID: $(echo "${occupying_pids}" | tr '\n' ' ')). Please stop the occupying process before starting frontend dev server."
        return 1
      fi
      log_info "Starting frontend Vite dev server (on :${COSAVE_FRONTEND_PORT})..."
      (cd "${FRONTEND_DIR}" && pnpm run dev "$@")
      ;;
    serve|preview)
      log_info "Previewing frontend static SPA..."
      (cd "${FRONTEND_DIR}" && pnpm run preview "$@")
      ;;
    add)
      if [[ $# -eq 0 ]]; then
        log_error "No package specified. Usage: ./dev.sh ui add <package> [options]"
        exit 1
      fi
      log_info "Adding package dependency: $*..."
      (cd "${FRONTEND_DIR}" && pnpm add "$@")
      ;;
    shadcn|add-ui)
      if [[ $# -eq 0 ]]; then
        log_error "No component specified. Usage: ./dev.sh ui shadcn <component> [options]"
        exit 1
      fi
      log_info "Adding shadcn-svelte component: $*..."
      (cd "${FRONTEND_DIR}" && pnpm dlx shadcn-svelte@latest add -y "$@")
      ;;
    *)
      log_error "Unknown UI command: ${subcmd}"
      cmd_ui help
      exit 1
      ;;
  esac
}

# ------------------------------------------------------------------------------
# Subcommand: lint
# ------------------------------------------------------------------------------
cmd_lint() {
  local target="all"
  local fix=false
  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      echo -e "${BOLD}Lint Command:${NC}"
      echo -e "  Runs formatting and linter checks on backend and frontend."
      echo
      echo -e "${BOLD}Usage:${NC} ./dev.sh lint [backend|ui|all] [--fix]"
      return 0
    elif [[ "$arg" == "--fix" ]]; then
      fix=true
    elif [[ "$arg" == "backend" || "$arg" == "api" ]]; then
      target="backend"
    elif [[ "$arg" == "ui" || "$arg" == "frontend" ]]; then
      target="ui"
    elif [[ "$arg" == "all" ]]; then
      target="all"
    fi
  done

  case "${target}" in
    backend)
      if [[ "$fix" == true ]]; then
        cmd_backend lint --fix
      else
        cmd_backend lint
      fi
      ;;
    ui)
      if [[ "$fix" == true ]]; then
        cmd_ui lint --fix
      else
        cmd_ui lint
      fi
      ;;
    all)
      if [[ "$fix" == true ]]; then
        cmd_backend lint --fix
        cmd_ui lint --fix
      else
        cmd_backend lint
        cmd_ui lint
      fi
      log_success "All linting checks completed successfully!"
      ;;
  esac
}

# ------------------------------------------------------------------------------
# Subcommand: format
# ------------------------------------------------------------------------------
cmd_format() {
  local target="all"
  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      echo -e "${BOLD}Format Command:${NC}"
      echo -e "  Formats backend (cargo fmt) and frontend (canonical Tailwind & Prettier)."
      echo
      echo -e "${BOLD}Usage:${NC} ./dev.sh format [backend|ui|all]"
      return 0
    elif [[ "$arg" == "backend" || "$arg" == "api" ]]; then
      target="backend"
    elif [[ "$arg" == "ui" || "$arg" == "frontend" ]]; then
      target="ui"
    elif [[ "$arg" == "all" ]]; then
      target="all"
    fi
  done

  case "${target}" in
    backend)
      cmd_backend format
      ;;
    ui)
      cmd_ui format
      ;;
    all)
      cmd_backend format
      cmd_ui format
      log_success "All files formatted successfully!"
      ;;
  esac
}

# ------------------------------------------------------------------------------
# Subcommand: flint (Format + Lint)
# ------------------------------------------------------------------------------
cmd_flint() {
  local target="all"
  local fix=true
  local extra_args=()
  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      echo -e "${BOLD}Flint Command (Format + Lint):${NC}"
      echo -e "  Formats and lints backend and frontend (auto-fixes by default)."
      echo
      echo -e "${BOLD}Usage:${NC} ./dev.sh flint [backend|ui|all] [--no-fix]"
      echo -e "  ${GREEN}all${NC}       Format and lint both backend and frontend [default]"
      echo -e "  ${GREEN}backend${NC}   Format and lint backend"
      echo -e "  ${GREEN}ui${NC}        Format and lint frontend"
      echo -e "  ${YELLOW}--no-fix${NC}  Run in check mode without auto-fixing"
      return 0
    elif [[ "$arg" == "--no-fix" ]]; then
      fix=false
    elif [[ "$arg" == "--fix" ]]; then
      fix=true
    elif [[ "$arg" == "backend" || "$arg" == "api" ]]; then
      target="backend"
    elif [[ "$arg" == "ui" || "$arg" == "frontend" ]]; then
      target="ui"
    elif [[ "$arg" == "all" ]]; then
      target="all"
    else
      extra_args+=("$arg")
    fi
  done

  case "${target}" in
    backend)
      if [[ "$fix" == true ]]; then
        cmd_backend flint --fix ${extra_args[@]+"${extra_args[@]}"}
      else
        cmd_backend flint --no-fix ${extra_args[@]+"${extra_args[@]}"}
      fi
      ;;
    ui)
      if [[ "$fix" == true ]]; then
        cmd_ui flint --fix ${extra_args[@]+"${extra_args[@]}"}
      else
        cmd_ui flint --no-fix ${extra_args[@]+"${extra_args[@]}"}
      fi
      ;;
    all)
      log_info "Running flint (format + lint) on all components (auto-fixes by default)..."
      if [[ "$fix" == true ]]; then
        cmd_backend flint --fix ${extra_args[@]+"${extra_args[@]}"}
        cmd_ui flint --fix ${extra_args[@]+"${extra_args[@]}"}
      else
        cmd_backend flint --no-fix ${extra_args[@]+"${extra_args[@]}"}
        cmd_ui flint --no-fix ${extra_args[@]+"${extra_args[@]}"}
      fi
      log_success "All formatting and linting completed successfully!"
      ;;
  esac
}

# ------------------------------------------------------------------------------
# Subcommand: check
# ------------------------------------------------------------------------------
cmd_check() {
  local target="all"
  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      echo -e "${BOLD}Check Command:${NC}"
      echo -e "  Runs cargo check on backend and svelte-check & canonical classes on frontend."
      echo
      echo -e "${BOLD}Usage:${NC} ./dev.sh check [backend|ui|all]"
      return 0
    elif [[ "$arg" == "backend" || "$arg" == "api" ]]; then
      target="backend"
    elif [[ "$arg" == "ui" || "$arg" == "frontend" ]]; then
      target="ui"
    elif [[ "$arg" == "all" ]]; then
      target="all"
    fi
  done

  case "${target}" in
    backend)
      cmd_backend check
      ;;
    ui)
      cmd_ui check
      ;;
    all)
      cmd_backend check
      cmd_ui check
      log_success "All type and status checks passed!"
      ;;
  esac
}

# ------------------------------------------------------------------------------
# Subcommand: build
# ------------------------------------------------------------------------------
cmd_build() {
  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      echo -e "${BOLD}Build Command:${NC}"
      echo -e "  Compiles project build targets."
      echo
      echo -e "${BOLD}Usage:${NC} ./dev.sh build [target]"
      echo -e "  ${GREEN}all${NC}       Build backend (release), frontend (dist), and Docker image [default]"
      echo -e "  ${GREEN}backend${NC}   Build backend release binary"
      echo -e "  ${GREEN}ui${NC}        Build frontend static SPA into dist/"
      echo -e "  ${GREEN}docker${NC}    Build multi-stage Docker image (cosave:latest)"
      return 0
    fi
  done

  local target="${1:-all}"
  case "${target}" in
    backend|api)
      log_info "Building backend binary (release mode)..."
      cargo build --release --manifest-path "${BACKEND_DIR}/Cargo.toml"
      log_success "Backend compiled."
      ;;
    ui|frontend)
      log_info "Building frontend static bundle..."
      (cd "${FRONTEND_DIR}" && pnpm run build)
      log_success "Frontend compiled."
      ;;
    docker)
      log_info "Building multi-stage Docker image (cosave:latest)..."
      docker build -t cosave:latest "${ROOT_DIR}"
      log_success "Docker image cosave:latest built."
      ;;
    all)
      log_info "1/3 Building backend binary (release mode)..."
      cargo build --release --manifest-path "${BACKEND_DIR}/Cargo.toml"
      log_success "Backend compiled."

      log_info "2/3 Building frontend static bundle..."
      (cd "${FRONTEND_DIR}" && pnpm run build)
      log_success "Frontend compiled."

      log_info "3/3 Building multi-stage Docker image (cosave:latest)..."
      if command -v docker &>/dev/null && docker info &>/dev/null; then
        docker build -t cosave:latest "${ROOT_DIR}"
        log_success "Docker image cosave:latest built."
      else
        log_warn "Docker daemon is not running or unavailable; skipped container build."
      fi

      log_success "All build targets completed!"
      ;;
    *)
      log_error "Unknown build target: ${target}. Expected: backend, ui, docker, or all."
      exit 1
      ;;
  esac
}

# ------------------------------------------------------------------------------
# Subcommand: dev (Start Development Servers)
# ------------------------------------------------------------------------------
cmd_dev() {
  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      echo -e "${BOLD}Dev Command:${NC}"
      echo -e "  Starts development servers with hot-reloading."
      echo
      echo -e "${BOLD}Usage:${NC} ./dev.sh dev [target] [options]"
      echo -e "  ${GREEN}all${NC}        Concurrently start backend (Axum :5171) and frontend (Vite :5172) [default]"
      echo -e "  ${GREEN}backend${NC}    Start backend Axum server with debug logging (-v)"
      echo -e "  ${GREEN}ui${NC}         Start frontend Vite dev server on :5172"
      return 0
    fi
  done

  local target="${1:-all}"
  shift || true

  if [[ "${target}" == "backend" || "${target}" == "api" ]]; then
    cmd_backend dev "$@"
    return
  elif [[ "${target}" == "ui" || "${target}" == "frontend" ]]; then
    cmd_ui dev "$@"
    return
  fi

  # Check if dev ports are already in use
  local port_conflict=false
  for port in "${COSAVE_BACKEND_PORT_DEV}" "${COSAVE_FRONTEND_PORT}"; do
    local occupying_pids
    occupying_pids=$(check_port "${port}")
    if [[ -n "${occupying_pids}" ]]; then
      log_error "Port ${port} is currently in use (PID: $(echo "${occupying_pids}" | tr '\n' ' ')). Please stop the occupying process before starting dev servers."
      port_conflict=true
    fi
  done
  if [[ "$port_conflict" == true ]]; then
    return 1
  fi

  log_info "Starting CoSave development servers..."
  log_info "  - Backend:  http://localhost:${COSAVE_BACKEND_PORT_DEV} (Axum API mode with debug logging)"
  log_info "  - Frontend: http://localhost:${COSAVE_FRONTEND_PORT} (Vite dev server with /api proxy)"

  # Kill child jobs upon exit safely without unbound errors
  cleanup() {
    log_warn "Stopping dev servers..."
    trap - EXIT INT TERM
    if [[ -n "${BACKEND_PID:-}" ]]; then
      pkill -P "${BACKEND_PID}" 2>/dev/null || true
      kill "${BACKEND_PID}" 2>/dev/null || true
    fi
    if [[ -n "${FRONTEND_PID:-}" ]]; then
      pkill -P "${FRONTEND_PID}" 2>/dev/null || true
      kill "${FRONTEND_PID}" 2>/dev/null || true
    fi
    local pids
    pids=$(jobs -p 2>/dev/null || true)
    if [[ -n "${pids}" ]]; then
      kill "${pids}" 2>/dev/null || true
    fi
  }
  trap cleanup EXIT INT TERM

  # Start backend
  cargo run --manifest-path "${BACKEND_DIR}/Cargo.toml" -- api --env "${COSAVE_ENV_DEV}" --host 0.0.0.0 --port "${COSAVE_BACKEND_PORT_DEV}" -v &
  BACKEND_PID=$!

  # Start frontend Vite
  (cd "${FRONTEND_DIR}" && pnpm run dev) &
  FRONTEND_PID=$!

  # Wait for both
  wait "${BACKEND_PID}" "${FRONTEND_PID}"
}

# ------------------------------------------------------------------------------
# Subcommand: serve (Start Production Server)
# ------------------------------------------------------------------------------
cmd_serve() {
  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      echo -e "${BOLD}Serve Command (Production):${NC}"
      echo -e "  Runs the production CoSave server serving the compiled frontend SPA."
      echo
      echo -e "${BOLD}Usage:${NC} ./dev.sh serve [target] [options]"
      echo -e "  ${GREEN}local${NC}       Run native Rust production server in release mode (out of Docker) [default]"
      echo -e "  ${GREEN}docker${NC}      Run prebuilt production Docker container (:${COSAVE_BACKEND_PORT_PROD})"
      echo
      echo -e "${BOLD}Options (for local target):${NC}"
      echo -e "  ${GREEN}--no-build, -n${NC}       Skip rebuilding frontend static bundle before serving"
      echo -e "  ${GREEN}--build, -b${NC}          Rebuild frontend static bundle (default: true)"
      echo -e "  ${GREEN}--port <port>${NC}        Port to listen on (default: ${COSAVE_BACKEND_PORT_PROD}, or PORT env)"
      echo -e "  ${GREEN}--static-dir <dir>${NC}   Directory of static files (default: frontend/dist)"
      echo -e "  ${GREEN}-v, --verbose${NC}       Enable debug logging"
      return 0
    fi
  done

  local target="local"
  if [[ "${1:-}" == "docker" ]]; then
    target="docker"
    shift || true
  elif [[ "${1:-}" == "local" || "${1:-}" == "native" ]]; then
    target="local"
    shift || true
  fi

  local serve_port="${PORT:-${COSAVE_BACKEND_PORT_PROD}}"
  local occupying_pids
  occupying_pids=$(check_port "${serve_port}")
  if [[ -n "${occupying_pids}" ]]; then
    log_error "Port ${serve_port} is currently in use (PID: $(echo "${occupying_pids}" | tr '\n' ' ')). Please stop the occupying process before starting production server."
    return 1
  fi

  if [[ "${target}" == "docker" ]]; then
    log_info "Running production Docker container (cosave:latest on http://localhost:${serve_port})..."
    docker run --rm -it -p "${serve_port}:${COSAVE_BACKEND_PORT_PROD}" cosave:latest "$@"
    return
  fi

  local build_ui=true
  local server_args=()
  for arg in "$@"; do
    if [[ "$arg" == "--no-build" || "$arg" == "-n" ]]; then
      build_ui=false
    elif [[ "$arg" == "--build" || "$arg" == "-b" ]]; then
      build_ui=true
    else
      server_args+=("$arg")
    fi
  done

  # Native / out of Docker env production server: always build UI by default to serve freshest code
  if [[ "$build_ui" == true || ! -d "${FRONTEND_DIR}/dist" ]]; then
    log_info "Building frontend static SPA bundle to ensure newest version is served..."
    cmd_ui build
  fi

  log_info "Starting CoSave production server (Rust release mode, out of Docker)..."
  log_info "  - Serving static SPA from: ${FRONTEND_DIR}/dist"
  if [[ ${#server_args[@]} -gt 0 ]]; then
    cargo run --release --manifest-path "${BACKEND_DIR}/Cargo.toml" -- --env "${COSAVE_ENV_PROD}" --host 0.0.0.0 --port "${serve_port}" --static-dir "${FRONTEND_DIR}/dist" "${server_args[@]}"
  else
    cargo run --release --manifest-path "${BACKEND_DIR}/Cargo.toml" -- --env "${COSAVE_ENV_PROD}" --host 0.0.0.0 --port "${serve_port}" --static-dir "${FRONTEND_DIR}/dist"
  fi
}

# ------------------------------------------------------------------------------
# Subcommand: test
# ------------------------------------------------------------------------------
cmd_test() {
  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      echo -e "${BOLD}Test Command:${NC}"
      echo -e "  Runs tests for backend, frontend, docker container, or all."
      echo
      echo -e "${BOLD}Usage:${NC} ./dev.sh test [backend|ui|docker|all] [options]"
      echo -e "  ${GREEN}--no-docker, --skip-docker${NC}  Skip container smoke tests when testing all"
      return 0
    fi
  done

  local run_backend=true
  local run_ui=true
  local run_docker=true
  local explicit_docker=false

  for arg in "$@"; do
    if [[ "$arg" == "--no-docker" || "$arg" == "--skip-docker" ]]; then
      run_docker=false
    elif [[ "$arg" == "backend" || "$arg" == "api" ]]; then
      shift
      cmd_backend test "$@"
      return
    elif [[ "$arg" == "ui" || "$arg" == "frontend" ]]; then
      shift
      cmd_ui test "$@"
      return
    elif [[ "$arg" == "docker" || "$arg" == "container" ]]; then
      explicit_docker=true
      run_backend=false
      run_ui=false
      run_docker=true
    fi
  done

  if [[ "$run_backend" == true ]]; then
    log_info "Running backend unit tests (cargo test)..."
    cargo test --manifest-path "${BACKEND_DIR}/Cargo.toml"
    log_success "Backend unit tests passed."
  fi

  if [[ "$run_ui" == true ]]; then
    log_info "Running frontend unit tests (vitest)..."
    (cd "${FRONTEND_DIR}" && pnpm run test)
    log_success "Frontend unit tests passed."

    log_info "Running frontend type check & Tailwind canonical check..."
    (cd "${FRONTEND_DIR}" && pnpm run check)
    log_success "Frontend checks passed."
  fi

  if [[ "$run_docker" == true ]]; then
    if ! command -v docker &>/dev/null || ! docker info &>/dev/null; then
      if [[ "$explicit_docker" == true ]]; then
        log_error "Docker is not installed or Docker daemon is not running."
        exit 1
      else
        log_warn "Docker daemon is not running or unavailable; skipping container smoke tests."
        run_docker=false
      fi
    fi
  fi

  if [[ "$run_docker" == true ]]; then
    log_info "Building production container image..."
    docker build -t cosave:latest "${ROOT_DIR}"
    log_success "Docker image built successfully."

    log_info "Spinning up test container to verify HTTP API..."
    TEST_CONTAINER="cosave-test-runner-$$"
    TEST_PORT="3099"

    cleanup_test_container() {
      docker rm -f "${TEST_CONTAINER}" 2>/dev/null || true
    }
    trap cleanup_test_container EXIT INT TERM

    cleanup_test_container
    docker run -d --name "${TEST_CONTAINER}" -p "${TEST_PORT}:5172" cosave:latest >/dev/null

    log_info "Waiting for container service at http://localhost:${TEST_PORT}..."
    MAX_RETRIES=10
    COUNT=0
    UP=false
    while [[ $COUNT -lt $MAX_RETRIES ]]; do
      if curl -s "http://localhost:${TEST_PORT}/api/v1/health" >/dev/null; then
        UP=true
        break
      fi
      sleep 0.5
      COUNT=$((COUNT + 1))
    done

    if [[ "$UP" != true ]]; then
      log_error "Server in container did not become ready in time."
      docker logs "${TEST_CONTAINER}"
      exit 1
    fi

    log_info "Verifying GET /api/v1/health response payload..."
    HEALTH_RESP=$(curl -s "http://localhost:${TEST_PORT}/api/v1/health")
    echo "  Response: ${HEALTH_RESP}"

    # Assert status code 0 and status HEALTHY
    if [[ "${HEALTH_RESP}" != *'"code":0'* ]] || [[ "${HEALTH_RESP}" != *'"status":"HEALTHY"'* ]]; then
      log_error "Health response assertion failed! Unexpected payload: ${HEALTH_RESP}"
      exit 1
    fi
    log_success "Health check assertion passed."

    log_info "Verifying GET / serves static SvelteKit SPA..."
    HTML_RESP=$(curl -s "http://localhost:${TEST_PORT}/")
    if [[ "${HTML_RESP}" != *'<title>CoSave'* ]] || [[ "${HTML_RESP}" != *'__sveltekit'* ]]; then
      log_error "Root HTML assertion failed! Unexpected output."
      exit 1
    fi
    log_success "Root route static UI serving assertion passed."

    log_info "Verifying GET /settings route fallback serving..."
    SETTINGS_RESP=$(curl -s "http://localhost:${TEST_PORT}/settings")
    if [[ "${SETTINGS_RESP}" != *'<title>CoSave'* ]] || [[ "${SETTINGS_RESP}" != *'__sveltekit'* ]]; then
      log_error "Settings HTML assertion failed! Unexpected output."
      exit 1
    fi
    log_success "Multi-route client-side fallback assertion passed."

    log_info "Verifying POST /api/v1/auth/register and cookie session inside container..."
    COOKIE_JAR=$(mktemp "${TMPDIR:-/tmp}/cosave_smoke_cookies_XXXXXX.txt")
    trap 'rm -f "${COOKIE_JAR:-}"; cleanup_test_container' EXIT INT TERM

    AUTH_RESP=$(curl -s -c "${COOKIE_JAR}" -X POST \
      -H "Content-Type: application/json" \
      -d '{"name":"smoke_admin","password":"password123"}' \
      "http://localhost:${TEST_PORT}/api/v1/auth/register")

    if [[ "${AUTH_RESP}" != *'"code":0'* ]] || [[ "${AUTH_RESP}" != *'"role":"admin"'* ]]; then
      log_error "Auth register assertion failed! Payload: ${AUTH_RESP}"
      exit 1
    fi
    log_success "Registration and admin assignment assertion passed."

    log_info "Verifying GET /api/v1/auth/me using authenticated session cookie..."
    ME_RESP=$(curl -s -b "${COOKIE_JAR}" "http://localhost:${TEST_PORT}/api/v1/auth/me")
    if [[ "${ME_RESP}" != *'"name":"smoke_admin"'* ]] || [[ "${ME_RESP}" != *'"role":"admin"'* ]]; then
      log_error "Auth me assertion failed! Payload: ${ME_RESP}"
      exit 1
    fi
    log_success "Session cookie verification assertion passed."

    log_info "Verifying GET /api/v1/categories hierarchy endpoint..."
    CATEGORIES_RESP=$(curl -s "http://localhost:${TEST_PORT}/api/v1/categories")
    if [[ "${CATEGORIES_RESP}" != *'"code":0'* ]] || [[ "${CATEGORIES_RESP}" != *'"name":"Income"'* ]] || [[ "${CATEGORIES_RESP}" != *'"name":"Expense"'* ]]; then
      log_error "Categories hierarchy assertion failed! Payload: ${CATEGORIES_RESP}"
      exit 1
    fi
    log_success "Categories hierarchy API assertion passed."

    log_info "Verifying POST /api/v1/auth/login with invalid credentials..."
    INVALID_LOGIN_RESP=$(curl -s -X POST \
      -H "Content-Type: application/json" \
      -d '{"name":"smoke_admin","password":"wrongpassword"}' \
      "http://localhost:${TEST_PORT}/api/v1/auth/login")
    if [[ "${INVALID_LOGIN_RESP}" != *'"code":401'* ]] || [[ "${INVALID_LOGIN_RESP}" != *'"status":"INVALID_CREDENTIALS"'* ]]; then
      log_error "Invalid login assertion failed! Payload: ${INVALID_LOGIN_RESP}"
      exit 1
    fi
    log_success "Invalid login contract assertion passed."

    log_info "Verifying POST /api/v1/auth/logout..."
    LOGOUT_RESP=$(curl -s -b "${COOKIE_JAR}" -c "${COOKIE_JAR}" -X POST "http://localhost:${TEST_PORT}/api/v1/auth/logout")
    if [[ "${LOGOUT_RESP}" != *'"code":0'* ]]; then
      log_error "Logout assertion failed! Payload: ${LOGOUT_RESP}"
      exit 1
    fi
    log_success "Auth logout assertion passed."

    rm -f "${COOKIE_JAR}"

    cleanup_test_container
    log_success "All container integration assertions passed successfully!"
  fi

  log_success "All automated tests completed successfully!"
}

# ------------------------------------------------------------------------------
# Subcommand: full (Runs full verification pipeline)
# ------------------------------------------------------------------------------
cmd_full() {
  local target="all"
  local fix=true
  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      echo -e "${BOLD}Full Pipeline Command:${NC}"
      echo -e "  Runs test -> check -> build -> flint in sequential order and fails fast (auto-fixes by default)."
      echo
      echo -e "${BOLD}Usage:${NC} ./dev.sh full [backend|ui|all] [--no-fix]"
      echo -e "  ${GREEN}all${NC}       Run full pipeline for both backend and frontend [default]"
      echo -e "  ${GREEN}backend${NC}   Run full pipeline for backend"
      echo -e "  ${GREEN}ui${NC}        Run full pipeline for frontend"
      echo -e "  ${YELLOW}--no-fix${NC}  Run flint in check mode without auto-fixing"
      return 0
    elif [[ "$arg" == "--no-fix" ]]; then
      fix=false
    elif [[ "$arg" == "--fix" ]]; then
      fix=true
    elif [[ "$arg" == "backend" || "$arg" == "api" ]]; then
      target="backend"
    elif [[ "$arg" == "ui" || "$arg" == "frontend" ]]; then
      target="ui"
    elif [[ "$arg" == "all" ]]; then
      target="all"
    fi
  done

  case "${target}" in
    backend)
      if [[ "$fix" == true ]]; then
        cmd_backend full --fix
      else
        cmd_backend full --no-fix
      fi
      ;;
    ui)
      if [[ "$fix" == true ]]; then
        cmd_ui full --fix
      else
        cmd_ui full --no-fix
      fi
      ;;
    all)
      log_info "Running full verification pipeline on all components (auto-fixes by default)..."
      if [[ "$fix" == true ]]; then
        cmd_backend full --fix
        cmd_ui full --fix
      else
        cmd_backend full --no-fix
        cmd_ui full --no-fix
      fi
      log_success "Full pipeline for all components completed successfully!"
      ;;
  esac
}

# ------------------------------------------------------------------------------
# Subcommand: fbuild (Fast Build: check -> flint (auto-fixes) -> build without server)
# ------------------------------------------------------------------------------
cmd_fbuild() {
  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      echo -e "${BOLD}Fast Build Command:${NC}"
      echo -e "  Runs checks, auto-fixes flint (formatting & linter), and compiles builds without running servers or container tests."
      echo
      echo -e "${BOLD}Usage:${NC} ./dev.sh fbuild [target]"
      echo -e "  ${GREEN}all${NC}       Fast build both backend and frontend [default]"
      echo -e "  ${GREEN}backend${NC}   Check, flint (auto-fixes), and compile backend release binary"
      echo -e "  ${GREEN}ui${NC}        Check, flint (auto-fixes), and build frontend static SPA into dist/"
      return 0
    fi
  done

  local target="all"
  if [[ "${1:-}" == "backend" || "${1:-}" == "api" ]]; then
    target="backend"
    shift || true
  elif [[ "${1:-}" == "ui" || "${1:-}" == "frontend" ]]; then
    target="ui"
    shift || true
  elif [[ "${1:-}" == "all" ]]; then
    shift || true
  fi

  case "${target}" in
    backend)
      cmd_backend fbuild
      ;;
    ui)
      cmd_ui fbuild
      ;;
    all)
      log_info "Running full fast build: backend + frontend (check -> flint (auto-fixes) -> build)..."
      cmd_backend fbuild
      cmd_ui fbuild
      log_success "Full fast build completed successfully!"
      ;;
  esac
}

# ------------------------------------------------------------------------------
# Subcommand: clean (Clean build artifacts & test containers)
# ------------------------------------------------------------------------------
cmd_clean() {
  for arg in "$@"; do
    if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
      echo -e "${BOLD}Clean Command:${NC}"
      echo -e "  Cleans build artifacts and test containers."
      echo
      echo -e "${BOLD}Usage:${NC} ./dev.sh clean [build|docker|all]"
      echo -e "  ${GREEN}all${NC}       Clean build artifacts and Docker test containers [default]"
      echo -e "  ${GREEN}build${NC}     Clean backend target/ and frontend dist/, .svelte-kit"
      echo -e "  ${GREEN}docker${NC}    Clean stale Docker test containers"
      return 0
    fi
  done

  local target="${1:-all}"
  case "${target}" in
    build|artifacts)
      log_info "Cleaning build artifacts..."
      (cd "${BACKEND_DIR}" && cargo clean)
      rm -rf "${FRONTEND_DIR}/dist" "${FRONTEND_DIR}/.svelte-kit"
      log_success "Build artifacts removed."
      ;;
    docker|containers)
      log_info "Cleaning CoSave test containers..."
      if command -v docker &>/dev/null && docker info &>/dev/null; then
        local stale_containers
        stale_containers=$(docker ps -aq --filter "name=cosave-test-runner" 2>/dev/null || true)
        if [[ -n "${stale_containers}" ]]; then
          echo "${stale_containers}" | xargs docker rm -f 2>/dev/null || true
        fi
      fi
      log_success "Docker test containers cleaned."
      ;;
    all)
      log_info "Running cleanup (build artifacts & test containers)..."
      if command -v docker &>/dev/null && docker info &>/dev/null; then
        local stale_containers
        stale_containers=$(docker ps -aq --filter "name=cosave-test-runner" 2>/dev/null || true)
        if [[ -n "${stale_containers}" ]]; then
          echo "${stale_containers}" | xargs docker rm -f 2>/dev/null || true
        fi
      fi
      (cd "${BACKEND_DIR}" && cargo clean)
      rm -rf "${FRONTEND_DIR}/dist" "${FRONTEND_DIR}/.svelte-kit"
      log_success "Cleanup completed successfully!"
      ;;
    *)
      log_error "Unknown clean target: ${target}. Expected: build, docker, or all."
      return 1
      ;;
  esac
}

# ------------------------------------------------------------------------------
# Subcommand: doctor (Check environment prerequisites)
# ------------------------------------------------------------------------------
cmd_doctor() {
  log_info "Running CoSave environment diagnostics..."
  local all_ok=true

  # Check Rust
  if command -v rustc &>/dev/null; then
    local rust_ver
    rust_ver=$(rustc --version)
    log_success "Rust: ${rust_ver}"
  else
    log_error "Rust: not found. Please install Rust 1.98+ (https://rustup.rs)"
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
  if command -v lsof &>/dev/null; then
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
  fi

  if [[ "$all_ok" == true ]]; then
    log_success "All core environment requirements are satisfied!"
  else
    log_error "Some environment requirements are missing. Please address the errors above."
    return 1
  fi
}

# ------------------------------------------------------------------------------
# Help / Usage
# ------------------------------------------------------------------------------
cmd_help() {
  echo -e "${BOLD}CoSave Development CLI Helper${NC}"
  echo
  echo -e "${BOLD}Usage:${NC} ./dev.sh <command> [subcommand/target] [options]"
  echo
  echo -e "${BOLD}Component-Scoped Commands:${NC}"
  echo -e "  ${GREEN}backend${NC} <cmd>   Backend actions: full [--no-fix], fbuild, flint [--no-fix], lint [--fix], format, check, test, build, dev, serve, add"
  echo -e "  ${GREEN}ui${NC} <cmd>        Frontend actions: full [--no-fix], fbuild, flint [--no-fix], lint [--fix], format, check, test, build, dev, serve, add, shadcn"
  echo
  echo -e "${BOLD}Global Commands:${NC}"
  echo -e "  ${GREEN}dev${NC} [target]          Start development server with live reload (backend :5171, Vite :5172, or all)"
  echo -e "  ${GREEN}serve${NC} [local|docker]  Run production server (builds frontend SPA by default; supports --no-build)"
  echo -e "  ${GREEN}full${NC} [target] [--no-fix] Run full pipeline (test -> check -> build -> flint, auto-fixes)"
  echo -e "  ${GREEN}fbuild${NC} [target]       Fast build (check -> flint (auto-fixes) -> build) without running servers"
  echo -e "  ${GREEN}flint${NC} [target] [--no-fix] Format and lint backend, ui, or all (auto-fixes by default)"
  echo -e "  ${GREEN}lint${NC} [target] [--fix]   Lint backend, ui, or all (Clippy, fmt, svelte-check, Prettier)"
  echo -e "  ${GREEN}format${NC} [target]         Format backend, ui, or all"
  echo -e "  ${GREEN}check${NC} [target]          Type check backend (cargo check), ui (svelte-check), or all"
  echo -e "  ${GREEN}test${NC} [target]           Test backend, ui, or all + Docker container smoke tests (supports --no-docker)"
  echo -e "  ${GREEN}build${NC} [target]          Build backend, ui, docker, or all (default: all)"
  echo -e "  ${GREEN}clean${NC} [target]          Clean build artifacts (build), Docker test containers (docker), or all"
  echo -e "  ${GREEN}doctor${NC}                  Verify local environment dependencies (Rust, Cargo, Node, pnpm, Docker)"
  echo -e "  ${GREEN}help${NC}                    Show this help message"
  echo
}

# ------------------------------------------------------------------------------
# Entrypoint Dispatcher
# ------------------------------------------------------------------------------
MAIN_CMD="${1:-help}"
shift || true

case "${MAIN_CMD}" in
  backend|api|server)
    cmd_backend "$@"
    ;;
  ui|frontend|client)
    cmd_ui "$@"
    ;;
  dev|run)
    cmd_dev "$@"
    ;;
  serve)
    cmd_serve "$@"
    ;;
  full)
    cmd_full "$@"
    ;;
  fbuild)
    cmd_fbuild "$@"
    ;;
  flint)
    cmd_flint "$@"
    ;;
  lint)
    cmd_lint "$@"
    ;;
  format|fmt)
    cmd_format "$@"
    ;;
  check)
    cmd_check "$@"
    ;;
  build)
    cmd_build "$@"
    ;;
  test)
    cmd_test "$@"
    ;;
  clean)
    cmd_clean "$@"
    ;;
  doctor)
    cmd_doctor "$@"
    ;;
  help|--help|-h)
    cmd_help
    ;;
  *)
    log_error "Unknown command: ${MAIN_CMD}"
    echo
    cmd_help
    exit 1
    ;;
esac
