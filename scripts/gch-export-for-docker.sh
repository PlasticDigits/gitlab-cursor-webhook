#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Export SQLite DB, job/terraform workspaces, and env from a bare-metal controller
# for migration to Docker / Coolify.
#
# Run on the existing controller host (as root or with sudo):
#   sudo ./scripts/gch-export-for-docker.sh
#   sudo ./scripts/gch-export-for-docker.sh /tmp/gch-migration-bundle
#
# Copy the resulting tarball to your Coolify server, then follow docs/docker-deploy.md.

set -euo pipefail

GCH_DATA_DIR="${GCH_DATA_DIR:-/var/lib/gch}"
ENV_FILE="${ENV_FILE:-/etc/gitlab-cursor-webhook.env}"
OUT_DIR="${1:-./gch-migration-bundle}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
ARCHIVE="${OUT_DIR}/gch-migration-${STAMP}.tar.gz"

mkdir -p "${OUT_DIR}"

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "error: env file not found: ${ENV_FILE}" >&2
  exit 1
fi

if [[ ! -f "${GCH_DATA_DIR}/gch.db" ]]; then
  echo "error: SQLite DB not found: ${GCH_DATA_DIR}/gch.db" >&2
  exit 1
fi

echo "==> stopping controller (if running)"
if systemctl is-active --quiet gitlab-cursor-webhook.service 2>/dev/null; then
  systemctl stop gitlab-cursor-webhook.service
  RESTART_AFTER=true
else
  RESTART_AFTER=false
fi

cleanup() {
  if [[ "${RESTART_AFTER}" == true ]]; then
    echo "==> restarting controller"
    systemctl start gitlab-cursor-webhook.service || true
  fi
}
trap cleanup EXIT

echo "==> copying data from ${GCH_DATA_DIR}"
STAGING="$(mktemp -d)"
mkdir -p "${STAGING}/var/lib/gch"
cp -a "${GCH_DATA_DIR}/gch.db" "${STAGING}/var/lib/gch/"
if [[ -d "${GCH_DATA_DIR}/jobs" ]]; then
  cp -a "${GCH_DATA_DIR}/jobs" "${STAGING}/var/lib/gch/"
fi

echo "==> copying env (secrets — handle tarball carefully)"
cp -a "${ENV_FILE}" "${STAGING}/gitlab-cursor-webhook.env"

cat >"${STAGING}/README.txt" <<EOF
GCH migration bundle (${STAMP})

Contents:
  var/lib/gch/gch.db          SQLite projects, tags, prompts, settings
  var/lib/gch/jobs/           Per-job terraform workspaces (if any active)
  gitlab-cursor-webhook.env   Controller environment variables

Next: docs/docker-deploy.md on the target Coolify host.
EOF

echo "==> creating ${ARCHIVE}"
tar -C "${STAGING}" -czf "${ARCHIVE}" .
rm -rf "${STAGING}"

echo "==> done"
echo "    ${ARCHIVE}"
echo "    scp this file to Coolify, then import per docs/docker-deploy.md"
