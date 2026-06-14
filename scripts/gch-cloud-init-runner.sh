#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Shared agent runner — sourced or called from project gch-cloud-init.sh
set -euo pipefail

: "${GCH_JOB_ID:?}"
: "${GCH_CONTROLLER_URL:?}"
: "${JOB_RUNTIME_TOKEN:?}"
: "${CURSOR_API_KEY:?}"

HEARTBEAT_PID=""

cleanup() {
  if [[ -n "${HEARTBEAT_PID}" ]]; then
    kill "${HEARTBEAT_PID}" 2>/dev/null || true
  fi
}
trap cleanup EXIT

start_heartbeat() {
  (
    while true; do
      sleep 60
      curl -sf -X POST \
        -H "Authorization: Bearer ${JOB_RUNTIME_TOKEN}" \
        "${GCH_CONTROLLER_URL}/api/jobs/${GCH_JOB_ID}/heartbeat" \
        >/dev/null || true
    done
  ) &
  HEARTBEAT_PID=$!
}

fetch_job() {
  curl -sf \
    -H "Authorization: Bearer ${JOB_RUNTIME_TOKEN}" \
    "${GCH_CONTROLLER_URL}/api/jobs/${GCH_JOB_ID}"
}

post_status() {
  local phase="$1"
  local message="${2:-}"
  curl -sf -X POST \
    -H "Authorization: Bearer ${JOB_RUNTIME_TOKEN}" \
    -H "Content-Type: application/json" \
    -d "{\"phase\":\"${phase}\",\"message\":\"${message}\"}" \
    "${GCH_CONTROLLER_URL}/api/jobs/${GCH_JOB_ID}/status" \
    >/dev/null || true
}

post_complete() {
  local status="$1"
  local exit_code="${2:-0}"
  local meta_file="${GCH_COMPLETE_META_FILE:-/tmp/gch-agent-complete-meta.json}"
  local payload

  if [[ -f "${meta_file}" ]]; then
    payload="$(jq -n \
      --arg status "${status}" \
      --argjson exit_code "${exit_code}" \
      --slurpfile meta "${meta_file}" \
      '${meta[0]} + {status: $status, exit_code: $exit_code}')"
    rm -f "${meta_file}"
  else
    payload="$(jq -n \
      --arg status "${status}" \
      --argjson exit_code "${exit_code}" \
      '{status: $status, exit_code: $exit_code, reason: "agent_exit"}')"
  fi

  curl -sf -X POST \
    -H "Authorization: Bearer ${JOB_RUNTIME_TOKEN}" \
    -H "Content-Type: application/json" \
    -d "${payload}" \
    "${GCH_CONTROLLER_URL}/api/jobs/${GCH_JOB_ID}/complete" \
    >/dev/null || true
}

gch_git() {
  local -a git_cfg=()
  if [[ -n "${GITLAB_TOKEN:-}" ]]; then
    git_cfg+=(-c "url.https://oauth2:${GITLAB_TOKEN}@gitlab.com/.insteadOf=https://gitlab.com/")
  fi
  git "${git_cfg[@]}" "$@"
}

gch_sync_workspace() {
  local workspace="$1"
  local git_ref="${2:-}"

  if [[ -f /etc/gch/job.env ]]; then
    set -a
    # shellcheck source=/dev/null
    source /etc/gch/job.env
    set +a
  fi
  export GLAB_TOKEN="${GITLAB_TOKEN:-}"

  if [[ ! -d "${workspace}/.git" ]]; then
    post_status "git" "skipped: ${workspace} is not a git repository"
    return 0
  fi

  cd "${workspace}"

  post_status "git" "fetching latest from origin"
  local attempt
  for attempt in 1 2 3; do
    if gch_git fetch --prune origin; then
      break
    fi
    if [[ "${attempt}" -eq 3 ]]; then
      post_status "git" "warning: git fetch failed after 3 attempts; using snapshot checkout"
      return 1
    fi
    post_status "git" "fetch failed (attempt ${attempt}/3), retrying..."
    sleep 5
  done

  if [[ -n "${git_ref}" && "${git_ref}" != "null" ]]; then
    post_status "git" "checking out ${git_ref}"
    if ! gch_git checkout "${git_ref}" || ! gch_git reset --hard "${git_ref}"; then
      post_status "git" "warning: checkout ${git_ref} failed; using snapshot checkout"
      return 1
    fi
  else
    local default_branch
    default_branch="$(gch_git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null | sed 's|^origin/||' || true)"
    default_branch="${default_branch:-main}"
    post_status "git" "updating to origin/${default_branch}"
    if ! gch_git checkout "${default_branch}" 2>/dev/null; then
      if ! gch_git checkout -B "${default_branch}" "origin/${default_branch}"; then
        post_status "git" "warning: could not update to origin/${default_branch}"
        return 1
      fi
    fi
    if ! gch_git reset --hard "origin/${default_branch}"; then
      post_status "git" "warning: reset to origin/${default_branch} failed"
      return 1
    fi
  fi

  if [[ -f .gitmodules ]]; then
    post_status "git" "updating submodules"
    gch_git submodule update --init --recursive || true
  fi

  local head
  head="$(gch_git rev-parse --short HEAD)"
  post_status "git" "synced at ${head}"
  return 0
}

run_cursor_agent() {
  local workspace="$1"
  local prompt="$2"
  local model="$3"

  # Agent shell tools must inherit GitLab credentials (job.env is not automatic).
  if [[ -f /etc/gch/job.env ]]; then
    set -a
    # shellcheck source=/dev/null
    source /etc/gch/job.env
    set +a
  fi
  export GLAB_TOKEN="${GITLAB_TOKEN:-}"

  export CURSOR_API_KEY
  export DISPLAY="${DISPLAY:-:99}"

  if ! pgrep -x Xvfb >/dev/null 2>&1; then
    Xvfb :99 -screen 0 1920x1080x24 &
    sleep 1
  fi

  cd "${workspace}"

  # Long idle: no stream-json while a shell command runs (playwright, npm, forge, etc.).
  # Short idle: after thinking/tool_call completed and no tools in flight — CLI hang.
  local idle_secs="${GCH_AGENT_IDLE_TIMEOUT_SECS:-1200}"
  local idle_after_thinking_secs="${GCH_AGENT_IDLE_AFTER_THINKING_SECS:-300}"
  local max_secs="${GCH_AGENT_MAX_TIMEOUT_SECS:-10800}"
  local agent_cmd=(
    agent -p "${prompt}"
    --model "${model}"
    --force
    --trust
    --workspace "${workspace}"
    --output-format stream-json
  )

  # Cursor CLI `agent -p` is supposed to exit when done but often hangs (worker /
  # background shell tasks). Stop after idle_secs with no stdout so the runner
  # can POST /complete and the controller can destroy the VM.
  local runner_dir wrap_py
  runner_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  wrap_py="${runner_dir}/gch-agent-idle-wrap.py"
  if [[ -x "${wrap_py}" ]] || [[ -f "${wrap_py}" ]]; then
    python3 "${wrap_py}" "${idle_secs}" "${idle_after_thinking_secs}" "${max_secs}" -- "${agent_cmd[@]}"
  else
    agent -p "${prompt}" \
      --model "${model}" \
      --force \
      --trust \
      --workspace "${workspace}" \
      --output-format stream-json
  fi
}

gch_run_job() {
  local job_json
  job_json="$(fetch_job)"

  local workspace prompt model git_ref
  workspace="$(echo "${job_json}" | jq -r .workspace)"
  prompt="$(echo "${job_json}" | jq -r .prompt)"
  model="$(echo "${job_json}" | jq -r .model)"
  git_ref="$(echo "${job_json}" | jq -r '.git_ref // empty')"

  post_status "boot" "cloud-init runner started"
  start_heartbeat

  gch_sync_workspace "${workspace}" "${git_ref}" || true

  post_status "agent" "starting cursor agent"
  set +e
  run_cursor_agent "${workspace}" "${prompt}" "${model}"
  local exit_code=$?
  set -e

  if [[ ${exit_code} -eq 0 ]]; then
    post_complete "success" "${exit_code}"
  else
    post_complete "failed" "${exit_code}"
  fi

  return "${exit_code}"
}
