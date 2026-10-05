#!/usr/bin/env bash
# ==============================================================================
# CoSave Development CLI: Common Utilities & Environment
# ==============================================================================
set -Eeuo pipefail

# Error Trap Handler: prints line number and failing command stack on uncaught errors
dev_error_handler() {
  local exit_code="$1"
  local line_no="$2"
  local bash_cmd="$3"
  local script_name="${BASH_SOURCE[1]:-${BASH_SOURCE[0]}}"
  echo -e "\033[0;31m\033[1m[cosave ✗ FATAL]\033[0m Command failed with exit code ${exit_code} at ${script_name}:${line_no}" >&2
  echo -e "  \033[0;33mFailing expression:\033[0m ${bash_cmd}" >&2
  exit "${exit_code}"
}
trap 'dev_error_handler $? $LINENO "$BASH_COMMAND"' ERR

# Root and Subproject Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
export ROOT_DIR
export BACKEND_DIR="${ROOT_DIR}/backend"
export FRONTEND_DIR="${ROOT_DIR}/frontend"

# Centralized Environment Constants & Ports
export COSAVE_ENV_DEV="DEV"
export COSAVE_ENV_PROD="PROD"
export COSAVE_FRONTEND_PORT=5172
export COSAVE_BACKEND_PORT_DEV=5171
export COSAVE_BACKEND_PORT_PROD=5172

# Color Support Detection (respect NO_COLOR and dumb terminals)
if [[ -n "${NO_COLOR:-}" || "${TERM:-}" == "dumb" ]]; then
  BOLD=''
  GREEN=''
  BLUE=''
  YELLOW=''
  RED=''
  NC=''
else
  BOLD='\033[1m'
  GREEN='\033[0;32m'
  BLUE='\033[0;34m'
  YELLOW='\033[0;33m'
  RED='\033[0;31m'
  NC='\033[0m'
fi

# Standard Logging Utilities
log_info() {
  echo -e "${BLUE}${BOLD}[cosave]${NC} $1"
}

log_success() {
  echo -e "${GREEN}${BOLD}[cosave ✓]${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}${BOLD}[cosave ⚠]${NC} $1" >&2
}

log_error() {
  echo -e "${RED}${BOLD}[cosave ✗]${NC} $1" >&2
}

die() {
  local msg="$1"
  local code="${2:-1}"
  log_error "${msg}"
  exit "${code}"
}

# Port check helper: prints occupying PIDs if port is active
check_port() {
  local port="$1"
  if command -v lsof &>/dev/null; then
    lsof -ti ":${port}" 2>/dev/null || true
  fi
}

# Curl helper: auto-detects active ports, resolves convenience shortcuts and endpoints
cmd_curl() {
  if [[ $# -eq 0 ]]; then
    echo -e "${BOLD}CoSave Curl Helper${NC}"
    echo -e "Usage: ${GREEN}./dev.sh curl [options...] <path|url> [options...]${NC}"
    echo -e "       ${GREEN}./dev.sh backend curl <path|url> [options...]${NC}"
    echo
    echo -e "Target Port Resolution:"
    echo -e "  - Dev Backend:  http://localhost:${COSAVE_BACKEND_PORT_DEV} (if port 5171 active or path is /api/*)"
    echo -e "  - Frontend/SPA: http://localhost:${COSAVE_FRONTEND_PORT} (if port 5172 active or non-API path)"
    echo
    echo -e "Convenience Shortcuts:"
    echo -e "  ${GREEN}./dev.sh curl health${NC}       -> GET /api/v1/health"
    echo -e "  ${GREEN}./dev.sh curl categories${NC}   -> GET /api/v1/categories"
    echo
    echo -e "Examples:"
    echo -e "  ./dev.sh curl /api/v1/health"
    echo -e "  ./dev.sh curl -X POST /api/v1/auth/login -H 'Content-Type: application/json' -d '{\"name\":\"admin\",\"password\":\"pass\"}'"
    return 1
  fi

  local p_backend p_frontend
  p_backend=$(check_port "${COSAVE_BACKEND_PORT_DEV}")
  p_frontend=$(check_port "${COSAVE_FRONTEND_PORT}")

  local default_api_base="http://127.0.0.1:${COSAVE_BACKEND_PORT_DEV}"
  local default_ui_base="http://127.0.0.1:${COSAVE_FRONTEND_PORT}"

  # If backend port is inactive but frontend is active, Axum may be running in prod/serve mode on 5172
  if [[ -z "${p_backend}" && -n "${p_frontend}" ]]; then
    default_api_base="http://127.0.0.1:${COSAVE_FRONTEND_PORT}"
  fi

  if [[ -n "${COSAVE_BASE_URL:-}" ]]; then
    default_api_base="${COSAVE_BASE_URL}"
    default_ui_base="${COSAVE_BASE_URL}"
  elif [[ -n "${BASE_URL:-}" ]]; then
    default_api_base="${BASE_URL}"
    default_ui_base="${BASE_URL}"
  fi

  local resolved_args=()
  local url_resolved=false

  for arg in "$@"; do
    if [[ "${url_resolved}" == false ]]; then
      if [[ "${arg}" == "health" ]]; then
        resolved_args+=("${default_api_base}/api/v1/health")
        url_resolved=true
        continue
      elif [[ "${arg}" == "categories" ]]; then
        resolved_args+=("${default_api_base}/api/v1/categories")
        url_resolved=true
        continue
      elif [[ "${arg}" =~ ^https?:// ]]; then
        resolved_args+=("${arg}")
        url_resolved=true
        continue
      elif [[ "${arg}" =~ ^/api/ ]]; then
        resolved_args+=("${default_api_base}${arg}")
        url_resolved=true
        continue
      elif [[ "${arg}" =~ ^api/ ]]; then
        resolved_args+=("${default_api_base}/${arg}")
        url_resolved=true
        continue
      elif [[ "${arg}" =~ ^/ ]]; then
        resolved_args+=("${default_ui_base}${arg}")
        url_resolved=true
        continue
      fi
    fi
    resolved_args+=("${arg}")
  done

  # Default to -sS unless custom verbosity or silence is explicitly passed
  local has_silent=false
  for arg in "${resolved_args[@]}"; do
    if [[ "${arg}" == "-s" || "${arg}" == "-sS" || "${arg}" == "--silent" || "${arg}" == "-v" || "${arg}" == "--verbose" ]]; then
      has_silent=true
      break
    fi
  done

  local curl_cmd=(curl)
  if [[ "${has_silent}" == false ]]; then
    curl_cmd+=(-sS)
  fi

  "${curl_cmd[@]}" "${resolved_args[@]}"
  local exit_code=$?
  if [[ -t 1 && "${exit_code}" -eq 0 ]]; then
    echo ""
  fi
  return "${exit_code}"
}

