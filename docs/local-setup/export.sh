#!/usr/bin/env bash
# Load local AWS / datastore env.
#   ./docs/local-setup/export.sh          # new shell with env
#   source docs/local-setup/export.sh     # current shell

_local_setup_is_sourced() {
  if [ -n "${ZSH_VERSION:-}" ]; then
    case "${ZSH_EVAL_CONTEXT:-}" in
      *:file*) return 0 ;;
    esac
    return 1
  fi
  [ -n "${BASH_SOURCE[0]:-}" ] && [ "${BASH_SOURCE[0]}" != "${0}" ]
}

if [ -n "${ZSH_VERSION:-}" ]; then
  _THIS="${(%):-%x}"
elif [ -n "${BASH_SOURCE[0]:-}" ]; then
  _THIS="${BASH_SOURCE[0]}"
else
  _THIS="$0"
fi

_LOCAL_SETUP_DIR="$(cd "$(dirname "${_THIS}")" && pwd)"
_REPO_ROOT="$(cd "${_LOCAL_SETUP_DIR}/../.." && pwd)"

if [ -f "${_REPO_ROOT}/.env" ]; then
  _ENV_FILE="${_REPO_ROOT}/.env"
else
  _ENV_FILE="${_LOCAL_SETUP_DIR}/.env.template"
fi

set -a
# shellcheck disable=SC1090
source "${_ENV_FILE}"
set +a

# File logs: logs/<binary>.log under the repo (override with LOG_DIR / LOG_NAME).
if [ -z "${LOG_DIR:-}" ]; then
  export LOG_DIR="${_REPO_ROOT}/logs"
fi

case "${AWS_CONFIG_FILE:-}" in
  /*) ;;
  *) AWS_CONFIG_FILE="${_REPO_ROOT}/${AWS_CONFIG_FILE}" ;;
esac
case "${AWS_SHARED_CREDENTIALS_FILE:-}" in
  /*) ;;
  *) AWS_SHARED_CREDENTIALS_FILE="${_REPO_ROOT}/${AWS_SHARED_CREDENTIALS_FILE}" ;;
esac
export AWS_CONFIG_FILE AWS_SHARED_CREDENTIALS_FILE AWS_PROFILE

echo "loaded ${_ENV_FILE}"
echo "  AWS_PROFILE=${AWS_PROFILE}"
echo "  AWS_ENDPOINT_URL=${AWS_ENDPOINT_URL}"

_SOURCED=0
_local_setup_is_sourced && _SOURCED=1
unset _THIS _LOCAL_SETUP_DIR _REPO_ROOT _ENV_FILE
unset -f _local_setup_is_sourced

if [ "${_SOURCED}" -eq 1 ]; then
  unset _SOURCED
  return 0
fi

unset _SOURCED
echo "starting a shell with this env (type exit to leave)"
cd "${PWD}" || true
exec "${SHELL:-/bin/zsh}" -i
