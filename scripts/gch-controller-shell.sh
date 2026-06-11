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

run_gch() {
  sudo -u gch env \
    GCH_DB_PATH="${GCH_DB_PATH}" \
    GCH_ADMIN_TOKEN="${GCH_ADMIN_TOKEN:-}" \
    gchconfig "$@"
}

unset _gch_env_file
