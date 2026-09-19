#!/usr/bin/env bash
# ==============================================================================
# CoSave Development CLI Orchestrator
# Enforces Strict Target-First Syntax: ./dev.sh <target> <action> [options...]
# ==============================================================================
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts/dev"

# Ensure sub-scripts are executable
chmod +x "${SCRIPTS_DIR}"/*.sh 2>/dev/null || true

# shellcheck source=scripts/dev/common.sh
source "${SCRIPTS_DIR}/common.sh"

root_help() {
  echo -e "${BOLD}CoSave Development CLI Orchestrator${NC}"
  echo -e "Strict target-first syntax: ${GREEN}./dev.sh <target> <action> [options...]${NC}"
  echo
  echo -e "${BOLD}Targets:${NC}"
  echo -e "  ${GREEN}all${NC}       Whole workspace (backend + frontend) operations"
  echo -e "  ${GREEN}backend${NC}   Rust Axum backend operations"
  echo -e "  ${GREEN}ui${NC}        SvelteKit frontend operations"
  echo -e "  ${GREEN}smoke${NC}     Docker container and HTTP API smoke tests"
  echo -e "  ${GREEN}doctor${NC}    Check local environment and dependency prerequisites"
  echo
  echo -e "${BOLD}Top-Level Commands:${NC}"
  echo -e "  ${GREEN}dev${NC}       Start backend (:5171) & frontend (:5172) development servers"
  echo -e "  ${GREEN}serve${NC}     Run production Axum server serving compiled static SPA bundle"
  echo -e "  ${GREEN}curl${NC}      Query running backend (:5171) or frontend (:5172) via curl"
  echo -e "  ${GREEN}doctor${NC}    Verify local environment prerequisites (Rust, Node, Docker, ports)"
  echo
  echo -e "${BOLD}Standard Actions across Targets (all, backend, ui):${NC}"
  echo -e "  ${GREEN}test${NC}      Run unit / contract test suite"
  echo -e "  ${GREEN}check${NC}     Run compiler and static type checks"
  echo -e "  ${GREEN}lint${NC}      Run linters (Clippy, ESLint, ShellCheck) [--fix]"
  echo -e "  ${GREEN}format${NC}    Run code formatters (cargo fmt, Prettier) [--check]"
  echo -e "  ${GREEN}flint${NC}     Format and lint with auto-fix enabled (fast format+lint)"
  echo -e "  ${GREEN}build${NC}     Compile release artifacts"
  echo -e "  ${GREEN}fbuild${NC}    Fast build gate: flint -> check -> build"
  echo -e "  ${GREEN}audit${NC}     Full verification pipeline: test -> check -> build -> flint --no-fix"
  echo -e "  ${GREEN}clean${NC}     Remove compiled artifacts"
  echo
  echo -e "${BOLD}Target-Specific Actions:${NC}"
  echo -e "  ${GREEN}all dev${NC}                Concurrently start backend (:5171) & frontend (:5172) servers"
  echo -e "  ${GREEN}all serve [opts]${NC}       Run production Axum server serving static SPA bundle"
  echo -e "  ${GREEN}all exec <cmd...>${NC}      Run command from repository root"
  echo -e "  ${GREEN}all curl <path> [opts]${NC} Query backend or frontend API via curl"
  echo -e "  ${GREEN}backend add <crate>${NC}    Add cargo dependency"
  echo -e "  ${GREEN}backend cargo <args...>${NC} Passthrough directly to cargo with backend manifest"
  echo -e "  ${GREEN}backend db <subcmd>${NC}    SQLite helper (query \"<SQL>\", schema, reset)"
  echo -e "  ${GREEN}backend curl <path>${NC}    Query backend API (auto-resolves active port :5171/:5172)"
  echo -e "  ${GREEN}ui add <pkg>${NC}           Add pnpm dependency"
  echo -e "  ${GREEN}ui shadcn <comp>${NC}       Install shadcn-svelte component (auto-passes -y -o)"
  echo -e "  ${GREEN}ui capture [url]${NC}       Capture desktop and mobile screenshots for /ui-review"
  echo -e "  ${GREEN}ui node <args...>${NC}      Run Node.js inside frontend context"
  echo -e "  ${GREEN}ui exec <cmd...>${NC}       Run command inside frontend with local .bin in PATH"
  echo -e "  ${GREEN}ui pnpm <args...>${NC}      Passthrough directly to pnpm inside frontend"
  echo -e "  ${GREEN}smoke test${NC}             Build and run container HTTP API smoke test suite"
  echo
}

TARGET="${1:-help}"
shift || true

case "${TARGET}" in
  all)
    exec "${SCRIPTS_DIR}/all.sh" "$@"
    ;;
  backend)
    exec "${SCRIPTS_DIR}/backend.sh" "$@"
    ;;
  ui)
    exec "${SCRIPTS_DIR}/ui.sh" "$@"
    ;;
  smoke)
    exec "${SCRIPTS_DIR}/smoke.sh" "$@"
    ;;
  dev)
    exec "${SCRIPTS_DIR}/all.sh" dev "$@"
    ;;
  serve)
    exec "${SCRIPTS_DIR}/all.sh" serve "$@"
    ;;
  curl)
    exec "${SCRIPTS_DIR}/backend.sh" curl "$@"
    ;;
  doctor)
    exec "${SCRIPTS_DIR}/doctor.sh" "$@"
    ;;
  help|--help|-h)
    root_help
    ;;
  # Catch verb-first attempts and fail fast with instructive guidance
  test|check|lint|format|fmt|flint|build|fbuild|audit|clean|exec)
    log_error "Top-level action '${TARGET}' is forbidden. CoSave enforces strict target-first syntax."
    echo -e "  \033[1mDid you mean:\033[0m" >&2
    echo -e "    ${GREEN}./dev.sh all ${TARGET}${NC}     (to run on whole workspace)" >&2
    echo -e "    ${GREEN}./dev.sh backend ${TARGET}${NC} (to run on backend only)" >&2
    echo -e "    ${GREEN}./dev.sh ui ${TARGET}${NC}      (to run on frontend only)" >&2
    exit 1
    ;;
  *)
    log_error "Unknown target: '${TARGET}'."
    echo
    root_help
    exit 1
    ;;
esac
