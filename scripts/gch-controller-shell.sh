# SPDX-License-Identifier: AGPL-3.0-or-later
# Source on the controller host for gchconfig helpers (add to root ~/.bashrc).
# Usage: source /opt/gitlab-cursor-webhook/scripts/gch-controller-shell.sh

_gch_env_file="${GCH_ENV_FILE:-/etc/gitlab-cursor-webhook.env}"

if [[ -r "${_gch_env_file}" ]]; then
  set -a
  # shellcheck source=/dev/null
  source "${_gch_env_file}"
  set +a
fi

export GCH_DB_PATH="${GCH_DB_PATH:-/var/lib/gch/gch.db}"

# sudo -u gch drops the caller environment; forward vars gchconfig reads.
_gch_forward_env=(
  GCH_DB_PATH
  GCH_ADMIN_TOKEN
  HCLOUD_TOKEN
  GCH_FIREWALL_ID
  GCH_SSH_KEY_IDS
  GCH_SSH_KEY_NAME
  GCH_JOBS_DIR
  GCH_CONTROLLER_URL
  ALLOWED_USERS
  PROJECT_WEBHOOKS_SECURITY
  PROJECT_WEBHOOKS_VERIFY
  PROJECT_WEBHOOKS_IMPLEMENT
)

run_gch() {
  local -a env_args=()
  local var
  for var in "${_gch_forward_env[@]}"; do
    env_args+=("$var=${!var-}")
  done
  sudo -u gch env "${env_args[@]}" gchconfig "$@"
}

unset _gch_env_file _gch_forward_env
