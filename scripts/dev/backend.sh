#!/usr/bin/env bash
# ==============================================================================
# CoSave Development CLI: Backend Subcommands
# Target: backend
# ==============================================================================
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/dev/common.sh
source "${SCRIPT_DIR}/common.sh"

backend_help() {
  echo -e "${BOLD}Backend Commands (${GREEN}./dev.sh backend <action>${NC}):${NC}"
  echo -e "  ${GREEN}test [args...]${NC}        Run Rust unit & integration tests (cargo test)"
  echo -e "  ${GREEN}check [args...]${NC}       Run cargo check"
  echo -e "  ${GREEN}lint [--fix]${NC}          Run cargo fmt --check and clippy (-D warnings)"
  echo -e "  ${GREEN}format [--check]${NC}      Run cargo fmt"
  echo -e "  ${GREEN}flint [--no-fix]${NC}      Format and clippy with auto-fix enabled"
  echo -e "  ${GREEN}build [--release]${NC}     Compile backend Rust binary"
  echo -e "  ${GREEN}fbuild${NC}                Fast build: flint -> check -> build --release"
  echo -e "  ${GREEN}audit${NC}                 Full pipeline: test -> check -> build -> flint --no-fix"
  echo -e "  ${GREEN}clean${NC}                 Clean cargo target directory"
  echo -e "  ${GREEN}add <crate> [opts]${NC}    Add crate dependency via cargo add"
  echo -e "  ${GREEN}cargo <args...>${NC}       Direct cargo passthrough with backend manifest"
  echo -e "  ${GREEN}db <subcmd> [args]${NC}    SQLite helper: query <sql>, schema, reset"
  echo -e "  ${GREEN}curl <path|url> [opts]${NC} Query backend API (auto-resolves active port :5171/:5172)"
}

cmd_backend_test() {
  log_info "Running backend unit tests (without cli feature)..."
  cargo test --manifest-path "${BACKEND_DIR}/Cargo.toml" "$@"
  log_info "Running backend unit tests (with cli feature)..."
  cargo test --manifest-path "${BACKEND_DIR}/Cargo.toml" --features cli "$@"
  log_success "Backend tests passed."
}

cmd_backend_check() {
  log_info "Running cargo check on backend (without cli feature)..."
  cargo check --manifest-path "${BACKEND_DIR}/Cargo.toml" "$@"
  log_info "Running cargo check on backend (with cli feature)..."
  cargo check --manifest-path "${BACKEND_DIR}/Cargo.toml" --features cli "$@"
  log_success "Backend check passed."
}

cmd_backend_lint() {
  local fix=false
  local extra_args=()
  for arg in "$@"; do
    if [[ "${arg}" == "--fix" ]]; then
      fix=true
    else
      extra_args+=("${arg}")
    fi
  done

  log_info "Linting backend (Rust)..."
  if [[ "${fix}" == true ]]; then
    cargo fmt --manifest-path "${BACKEND_DIR}/Cargo.toml"
    cargo clippy --fix --allow-dirty --allow-staged --manifest-path "${BACKEND_DIR}/Cargo.toml" -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
    cargo clippy --fix --allow-dirty --allow-staged --manifest-path "${BACKEND_DIR}/Cargo.toml" --features cli -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
  else
    cargo fmt --manifest-path "${BACKEND_DIR}/Cargo.toml" -- --check
    cargo clippy --manifest-path "${BACKEND_DIR}/Cargo.toml" -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
    cargo clippy --manifest-path "${BACKEND_DIR}/Cargo.toml" --features cli -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
  fi
  log_success "Backend lint passed."
}

cmd_backend_format() {
  log_info "Formatting backend (cargo fmt)..."
  cargo fmt --manifest-path "${BACKEND_DIR}/Cargo.toml" "$@"
  log_success "Backend formatted successfully."
}

cmd_backend_flint() {
  local fix=true
  local extra_args=()
  for arg in "$@"; do
    if [[ "${arg}" == "--no-fix" ]]; then
      fix=false
    elif [[ "${arg}" != "--fix" ]]; then
      extra_args+=("${arg}")
    fi
  done

  log_info "Running backend flint (format + clippy)..."
  if [[ "${fix}" == true ]]; then
    cargo fmt --manifest-path "${BACKEND_DIR}/Cargo.toml"
    cargo clippy --fix --allow-dirty --allow-staged --manifest-path "${BACKEND_DIR}/Cargo.toml" -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
    cargo clippy --fix --allow-dirty --allow-staged --manifest-path "${BACKEND_DIR}/Cargo.toml" --features cli -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
  else
    cargo fmt --manifest-path "${BACKEND_DIR}/Cargo.toml" -- --check
    cargo clippy --manifest-path "${BACKEND_DIR}/Cargo.toml" -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
    cargo clippy --manifest-path "${BACKEND_DIR}/Cargo.toml" --features cli -- -D warnings ${extra_args[@]+"${extra_args[@]}"}
  fi
  log_success "Backend flint passed."
}

cmd_backend_build() {
  log_info "Compiling backend (without cli feature)..."
  cargo build --manifest-path "${BACKEND_DIR}/Cargo.toml" "$@"
  log_info "Compiling backend (with cli feature)..."
  cargo build --manifest-path "${BACKEND_DIR}/Cargo.toml" --features cli "$@"
  log_success "Backend built successfully."
}

cmd_backend_fbuild() {
  log_info "Running backend fast build: flint -> check -> build --release..."
  cmd_backend_flint --fix
  cmd_backend_check
  cmd_backend_build --release
  log_success "Backend fast build completed successfully."
}

cmd_backend_audit() {
  log_info "Running full backend audit pipeline: test -> check -> build -> flint..."
  cmd_backend_test "$@"
  cmd_backend_check
  cmd_backend_build --release
  cmd_backend_flint --no-fix
  log_success "Backend audit passed with zero errors/warnings."
}

cmd_backend_clean() {
  log_info "Cleaning backend target directory..."
  cargo clean --manifest-path "${BACKEND_DIR}/Cargo.toml"
  log_success "Backend target cleaned."
}

cmd_backend_add() {
  if [[ $# -eq 0 ]]; then
    die "No crate specified. Usage: ./dev.sh backend add <crate> [options]"
  fi
  log_info "Adding crate dependency: $*..."
  cargo add --manifest-path "${BACKEND_DIR}/Cargo.toml" "$@"
}

cmd_backend_cargo() {
  local subcmd="${1:-}"
  shift || true
  cargo "${subcmd}" --manifest-path "${BACKEND_DIR}/Cargo.toml" "$@"
}

cmd_backend_db() {
  local db_subcmd="${1:-help}"
  shift || true

  local db_file="${ROOT_DIR}/data/cosave.db"
  case "${db_subcmd}" in
    query)
      if [[ $# -eq 0 ]]; then
        die "No SQL query provided. Usage: ./dev.sh backend db query \"<SQL>\""
      fi
      local sql="$1"
      if [[ ! -f "${db_file}" ]]; then
        die "Database file not found at ${db_file}. Run './dev.sh all dev' once to initialize database."
      fi
      if command -v sqlite3 &>/dev/null; then
        sqlite3 -header -column "${db_file}" "${sql}"
      elif command -v python3 &>/dev/null; then
        python3 -c "
import sqlite3, sys
conn = sqlite3.connect('${db_file}')
cursor = conn.cursor()
for row in cursor.execute(sys.argv[1]):
    print(row)
conn.close()
" "${sql}"
      else
        die "Neither sqlite3 nor python3 found to run SQLite query."
      fi
      ;;
    schema)
      if [[ ! -f "${db_file}" ]]; then
        die "Database file not found at ${db_file}."
      fi
      if command -v sqlite3 &>/dev/null; then
        sqlite3 "${db_file}" ".schema"
      else
        python3 -c "
import sqlite3
conn = sqlite3.connect('${db_file}')
for row in conn.cursor().execute(\"SELECT sql FROM sqlite_master WHERE sql IS NOT NULL;\"):
    print(row[0] + ';')
conn.close()
"
      fi
      ;;
    reset)
      log_warn "Resetting SQLite database at ${db_file}..."
      if [[ -f "${db_file}" ]]; then
        cp "${db_file}" "${db_file}.bak.$(date +%s)"
        rm -f "${db_file}" "${db_file}-wal" "${db_file}-shm"
        log_success "Existing database backed up and deleted. Next backend boot will reinitialize schema."
      else
        log_info "No database file exists at ${db_file} to reset."
      fi
      ;;
    help|--help|-h)
      echo -e "${BOLD}Backend DB Helper Commands:${NC}"
      echo -e "  ${GREEN}query \"<sql>\"${NC}   Execute SQL query against local SQLite DB"
      echo -e "  ${GREEN}schema${NC}          Dump database table schema"
      echo -e "  ${GREEN}reset${NC}           Backup and remove data/cosave.db"
      ;;
    *)
      die "Unknown backend db subcommand: ${db_subcmd}. Run './dev.sh backend db help'."
      ;;
  esac
}

cmd_backend_curl() {
  cmd_curl "$@"
}

# Main Action Dispatcher for Backend
ACTION="${1:-help}"
shift || true

case "${ACTION}" in
  test)       cmd_backend_test "$@" ;;
  check)      cmd_backend_check "$@" ;;
  lint)       cmd_backend_lint "$@" ;;
  format|fmt) cmd_backend_format "$@" ;;
  flint)      cmd_backend_flint "$@" ;;
  build)      cmd_backend_build "$@" ;;
  fbuild)     cmd_backend_fbuild ;;
  audit)      cmd_backend_audit "$@" ;;
  clean)      cmd_backend_clean ;;
  add)        cmd_backend_add "$@" ;;
  cargo)      cmd_backend_cargo "$@" ;;
  db)         cmd_backend_db "$@" ;;
  curl)       cmd_backend_curl "$@" ;;
  help|--help|-h)
    backend_help
    ;;
  *)
    die "Unknown backend action: '${ACTION}'. Run './dev.sh backend help' for valid actions."
    ;;
esac
