#!/usr/bin/env bash
# ==============================================================================
# CoSave Development CLI: Smoke / Container Integration Tests
# Target: smoke
# ==============================================================================
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/dev/common.sh
source "${SCRIPT_DIR}/common.sh"

smoke_help() {
  echo -e "${BOLD}Smoke Commands (${GREEN}./dev.sh smoke <action>${NC}):${NC}"
  echo -e "  ${GREEN}test${NC}     Run full Docker container integration & HTTP API smoke tests"
  echo -e "  ${GREEN}build${NC}    Build multi-stage Docker image (cosave:latest)"
  echo -e "  ${GREEN}run [opts]${NC} Run production Docker container locally"
  echo -e "  ${GREEN}clean${NC}    Remove stale test containers"
}

cmd_smoke_build() {
  log_info "Building production Docker image (cosave:latest)..."
  docker build -t cosave:latest "${ROOT_DIR}"
  log_success "Docker image cosave:latest built."
}

cmd_smoke_clean() {
  log_info "Cleaning stale smoke test containers..."
  if command -v docker &>/dev/null && docker info &>/dev/null; then
    local stale_containers
    stale_containers=$(docker ps -aq --filter "name=cosave-test-runner" 2>/dev/null || true)
    if [[ -n "${stale_containers}" ]]; then
      echo "${stale_containers}" | xargs docker rm -f 2>/dev/null || true
    fi
  fi
  log_success "Smoke test containers cleaned."
}

cmd_smoke_run() {
  local port="${PORT:-${COSAVE_BACKEND_PORT_PROD}}"
  local occupying_pids
  occupying_pids=$(check_port "${port}")
  if [[ -n "${occupying_pids}" ]]; then
    die "Port ${port} is currently in use (PID: $(echo "${occupying_pids}" | tr '\n' ' ')). Please stop the occupying process."
  fi
  log_info "Running production Docker container on http://localhost:${port}..."
  docker run --rm -it -p "${port}:${COSAVE_BACKEND_PORT_PROD}" cosave:latest "$@"
}

cmd_smoke_test() {
  if ! command -v docker &>/dev/null || ! docker info &>/dev/null; then
    die "Docker is not installed or Docker daemon is not running."
  fi

  cmd_smoke_build

  log_info "Spinning up smoke test container to verify HTTP API..."
  local test_container="cosave-test-runner-$$"
  local test_port="3099"
  local cookie_jar
  cookie_jar=$(mktemp "${TMPDIR:-/tmp}/cosave_smoke_cookies_XXXXXX.txt")

  cleanup_container() {
    rm -f "${cookie_jar:-}"
    docker rm -f "${test_container}" 2>/dev/null || true
  }
  trap cleanup_container EXIT INT TERM

  cleanup_container
  docker run -d --name "${test_container}" -p "${test_port}:5172" cosave:latest >/dev/null

  log_info "Waiting for container service at http://localhost:${test_port}..."
  local max_retries=10
  local count=0
  local up=false
  while [[ $count -lt $max_retries ]]; do
    if curl -s "http://localhost:${test_port}/api/v1/health" >/dev/null; then
      up=true
      break
    fi
    sleep 0.5
    count=$((count + 1))
  done

  if [[ "${up}" != true ]]; then
    docker logs "${test_container}"
    die "Server in container did not become ready in time."
  fi

  log_info "Verifying GET /api/v1/health response payload..."
  local health_resp
  health_resp=$(curl -s "http://localhost:${test_port}/api/v1/health")
  if [[ "${health_resp}" != *'"code":0'* ]] || [[ "${health_resp}" != *'"status":"HEALTHY"'* ]]; then
    die "Health response assertion failed! Unexpected payload: ${health_resp}"
  fi
  log_success "Health check assertion passed."

  log_info "Verifying GET / serves static SvelteKit SPA..."
  local html_resp
  html_resp=$(curl -s "http://localhost:${test_port}/")
  if [[ "${html_resp}" != *'<title>CoSave'* ]] || [[ "${html_resp}" != *'__sveltekit'* ]]; then
    die "Root HTML assertion failed! Unexpected output."
  fi
  log_success "Root route static UI serving assertion passed."

  log_info "Verifying GET /settings client-side fallback route..."
  local settings_resp
  settings_resp=$(curl -s "http://localhost:${test_port}/settings")
  if [[ "${settings_resp}" != *'<title>CoSave'* ]] || [[ "${settings_resp}" != *'__sveltekit'* ]]; then
    die "Settings HTML assertion failed! Unexpected output."
  fi
  log_success "Multi-route client-side fallback assertion passed."

  log_info "Verifying POST /api/v1/auth/register and cookie session inside container..."
  local auth_resp
  auth_resp=$(curl -s -c "${cookie_jar}" -X POST \
    -H "Content-Type: application/json" \
    -d '{"name":"smoke_admin","password":"password123"}' \
    "http://localhost:${test_port}/api/v1/auth/register")
  if [[ "${auth_resp}" != *'"code":0'* ]] || [[ "${auth_resp}" != *'"role":"admin"'* ]]; then
    die "Auth register assertion failed! Payload: ${auth_resp}"
  fi
  log_success "Registration and admin assignment assertion passed."

  log_info "Verifying GET /api/v1/auth/me using authenticated session cookie..."
  local me_resp
  me_resp=$(curl -s -b "${cookie_jar}" "http://localhost:${test_port}/api/v1/auth/me")
  if [[ "${me_resp}" != *'"name":"smoke_admin"'* ]] || [[ "${me_resp}" != *'"role":"admin"'* ]]; then
    die "Auth me assertion failed! Payload: ${me_resp}"
  fi
  log_success "Session cookie verification assertion passed."

  log_info "Verifying GET /api/v1/categories hierarchy endpoint..."
  local categories_resp
  categories_resp=$(curl -s "http://localhost:${test_port}/api/v1/categories")
  if [[ "${categories_resp}" != *'"code":0'* ]] || [[ "${categories_resp}" != *'"name":"Income"'* ]] || [[ "${categories_resp}" != *'"name":"Expense"'* ]]; then
    die "Categories hierarchy assertion failed! Payload: ${categories_resp}"
  fi
  log_success "Categories hierarchy API assertion passed."

  log_info "Verifying POST /api/v1/auth/login with invalid credentials..."
  local invalid_login_resp
  invalid_login_resp=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"name":"smoke_admin","password":"wrongpassword"}' \
    "http://localhost:${test_port}/api/v1/auth/login")
  if [[ "${invalid_login_resp}" != *'"code":401'* ]] || [[ "${invalid_login_resp}" != *'"status":"INVALID_CREDENTIALS"'* ]]; then
    die "Invalid login assertion failed! Payload: ${invalid_login_resp}"
  fi
  log_success "Invalid login contract assertion passed."

  log_info "Verifying POST /api/v1/auth/logout..."
  local logout_resp
  logout_resp=$(curl -s -b "${cookie_jar}" -c "${cookie_jar}" -X POST "http://localhost:${test_port}/api/v1/auth/logout")
  if [[ "${logout_resp}" != *'"code":0'* ]]; then
    die "Logout assertion failed! Payload: ${logout_resp}"
  fi
  log_success "Auth logout assertion passed."

  cleanup_container
  log_success "All smoke container integration assertions passed successfully!"
}

# Main Action Dispatcher for Smoke
ACTION="${1:-help}"
shift || true

case "${ACTION}" in
  test)  cmd_smoke_test ;;
  build) cmd_smoke_build ;;
  run)   cmd_smoke_run "$@" ;;
  clean) cmd_smoke_clean ;;
  help|--help|-h)
    smoke_help
    ;;
  *)
    die "Unknown smoke action: '${ACTION}'. Run './dev.sh smoke help' for valid actions."
    ;;
esac
