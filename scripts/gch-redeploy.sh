#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Build and install gch binaries. Restarts gchcontroller only when safe.
#
# Usage:
#   ./scripts/gch-redeploy.sh              # build + install; restart if no active jobs
#   ./scripts/gch-redeploy.sh --wait       # poll until jobs finish, then restart
#   ./scripts/gch-redeploy.sh --no-restart # build + install only (prompt SQL updates need no restart)
#   ./scripts/gch-redeploy.sh --force      # restart even with active jobs (orphans in-flight VMs)

export SYSTEMD_PAGER=cat
export PAGER=cat

set -euo pipefail

REPO=/opt/gitlab-cursor-webhook
ENV_FILE=/etc/gitlab-cursor-webhook.env
SERVICE=gitlab-cursor-webhook.service
BINDIR=/usr/local/bin

WAIT=false
NO_RESTART=false
FORCE=false
POLL_SECS=30

usage() {
  sed -n '3,10p' "$0" | sed 's/^# \{0,1\}//'
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --wait) WAIT=true; shift ;;
    --no-restart) NO_RESTART=true; shift ;;
    --force) FORCE=true; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

cd "${REPO}"

echo "==> git pull"
git pull

echo "==> cargo build --release"
cargo build --release

echo "==> install binaries to ${BINDIR}"
install -m 755 "${REPO}/target/release/gchconfig" "${BINDIR}/gchconfig"
install -m 755 "${REPO}/target/release/gchcontroller" "${BINDIR}/gchcontroller"

echo "==> reload env"
set -a
# shellcheck source=/dev/null
source "${ENV_FILE}"
set +a
export GCH_DB_PATH="${GCH_DB_PATH:-/var/lib/gch/gch.db}"

count_active_jobs() {
  local out n
  if ! out="$(gchconfig jobs list --active 2>&1)"; then
    echo "warn: could not list active jobs (set GCH_ADMIN_TOKEN?); assuming unknown" >&2
    echo -1
    return
  fi
  if grep -q 'no jobs in controller memory' <<<"${out}"; then
    echo 0
    return
  fi
  n="$(grep -cE 'status=(queued|provisioning|running)' <<<"${out}" || true)"
  echo "${n}"
}

if [[ "${NO_RESTART}" == true ]]; then
  echo "==> skip restart (--no-restart)"
  echo "    gchconfig / prompt changes are live; restart later for new gchcontroller routing."
  exit 0
fi

while true; do
  active="$(count_active_jobs)"
  if [[ "${active}" -eq 0 ]]; then
    break
  fi
  if [[ "${active}" -lt 0 ]]; then
    if [[ "${FORCE}" == true ]]; then
      echo "==> active jobs unknown; continuing (--force)"
      break
    fi
    echo "==> cannot confirm active jobs; use --force or --wait" >&2
    exit 1
  fi
  if [[ "${FORCE}" == true ]]; then
    echo "==> ${active} active job(s); restarting anyway (--force)"
    echo "    in-flight VMs will be orphaned until manual Hetzner / terraform cleanup"
    break
  fi
  if [[ "${WAIT}" != true ]]; then
    echo "==> ${active} active job(s); refusing to restart"
    gchconfig jobs list --active || true
    echo "    binaries installed. retry with: $0 --wait   or after jobs finish: systemctl restart ${SERVICE}"
    exit 1
  fi
  echo "==> ${active} active job(s); waiting ${POLL_SECS}s (--wait)"
  gchconfig jobs list --active || true
  sleep "${POLL_SECS}"
done

echo "==> restart ${SERVICE}"
systemctl restart "${SERVICE}"

echo "==> verify binaries"
echo "root: $(which gchconfig)"
echo "gch:  $(sudo -u gch which gchconfig)"
systemctl show "${SERVICE}" -p ExecStart --value

echo "==> status"
systemctl --no-pager status "${SERVICE}"

echo "==> recent logs"
journalctl -u "${SERVICE}" -n 20 --no-pager
