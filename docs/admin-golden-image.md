# Golden image builder (Hetzner)

Build one **snapshot per GitLab project**. Job VMs boot from it via `gchconfig` tags (`security`, `verify`, `implement`).

Controller setup, firewall, and webhooks: [README](../README.md) and [runbook-cl8y-dex-terraclassic.md](runbook-cl8y-dex-terraclassic.md) (same `gchconfig` pattern for all projects).

## Builder VM

| Setting | Value |
|---------|-------|
| Plan | **CPX32** (4 vCPU, 8 GB RAM) |
| OS | **Ubuntu 24.04** — required; Playwright auto-detects the correct browser build (no platform override) |
| Location | **fsn1** (Falkenstein) — match where job VMs provision |
| SSH | Your admin key |

## 1. Sync GCH files into the project repo

`gch-cloud-setup.sh` runs from the **project** clone, not this repo. Copy from `docs/examples/<project>/` into the project on `main` before building:

| File | Purpose |
|------|---------|
| `gch-cloud-setup.sh` | Builder entrypoint (creates `agent`, bootstraps, runs finalize agent) |
| `gch-cloud-init.sh` | Job VM cloud-init (sources runner script) |
| `gch-cloud-init-runner.sh` | Must use `jq -r '.git_ref // empty'` |
| `gch-agent-idle-wrap.py` | Agent idle timeout on job VMs |
| `gch-golden-image-finalize.md` | Finalize agent prompt (verification + verify log) |
| `prompts/` | Agent prompt templates |

Example sources:

- [docs/examples/yieldomega/](examples/yieldomega/) — EVM / Foundry / Rabby / Anvil
- [docs/examples/cl8y-dex-terraclassic/](examples/cl8y-dex-terraclassic/) — Terra / Keplr / LocalTerra

Each project also needs its own bootstrap scripts on `main` (e.g. yieldomega `scripts/bootstrap-*.sh`, cl8y `scripts/setup-cloud-agent-localterra.sh`).

## 2. Run setup on a fresh builder VM

SSH as **root**. `/home/agent` does not exist until setup creates it — clone to a staging dir first:

```bash
git clone https://gitlab.com/plasticdigits/<project> /tmp/gch-workspace
chmod +x /tmp/gch-workspace/gch-cloud-setup.sh
export CURSOR_API_KEY='your-cursor-api-key'
# optional: export GCH_GOLDEN_IMAGE_MODEL=composer-2.5-fast
bash /tmp/gch-workspace/gch-cloud-setup.sh
```

Setup installs OS packages, creates the `agent` user, runs project bootstrap scripts, then a **finalize Cursor agent** (`gch-golden-image-finalize.md`) that writes `/home/agent/.gch/golden-image-verify.log`.

Stream output is tee'd to `/home/agent/.gch/finalize-agent.raw` and formatted by `gch-agent-stream-watch.py` (installed by setup).

## 3. Pass criteria

```bash
cat /home/agent/.gch/golden-image-verify.log   # must end with OVERALL: PASS
sudo -u agent agent about
grep -q 'GCH job secrets' /home/agent/.bashrc
grep "git_ref" /home/agent/gch-cloud-init-runner.sh   # jq -r '.git_ref // empty'
```

**yieldomega:** Rabby at `/opt/cursor/browser-extensions/rabby`; finalize agent runs `scripts/e2e-anvil.sh` (must print `Done.`). Postgres on **5433** — `bootstrap-cloud-vm-toolchain.sh` starts it once; do not run `bootstrap-cloud-postgres-native.sh` again from setup.

**cl8y-dex-terraclassic:** Playwright browsers from `frontend-dapp` lockfile under `~/.cache/ms-playwright/`; Keplr under `~/.gch/extensions/keplr`.

If `apt upgrade` installed a new kernel, **reboot** and re-check the log before cleanup.

## 4. Re-run finalize only

If bootstrap succeeded but finalize failed or the verify log is missing:

```bash
pkill -9 -f 'chrome-profile-rabby' 2>/dev/null || true
rm -f /opt/cursor/chrome-profile-rabby/SingletonLock
if ! pgrep -x Xvfb >/dev/null 2>&1; then Xvfb :99 -screen 0 1920x1080x24 & sleep 1; fi

# yieldomega: finish Playwright if wallet import was skipped during setup
sudo -u agent env DISPLAY=:99 \
  YIELDOMEGA_SKIP_RABBY_WALLET_IMPORT=1 YIELDOMEGA_SKIP_RABBY_INJECTION_VERIFY=1 \
  bash -lc 'bash /home/agent/workspace/scripts/bootstrap-cloud-agent.sh' || true

export CURSOR_API_KEY='your-cursor-api-key'
sudo -u agent env CURSOR_API_KEY="$CURSOR_API_KEY" DISPLAY=:99 bash -lc '
  cd /home/agent/workspace
  agent --print "$(cat gch-golden-image-finalize.md)" \
    --model composer-2.5 --force --trust \
    --workspace /home/agent/workspace \
    --output-format stream-json --stream-partial-output \
    2>&1 | tee /home/agent/.gch/finalize-agent.raw | python3 /home/agent/gch-agent-stream-watch.py
'
cat /home/agent/.gch/golden-image-verify.log
```

## 5. Pre-snapshot cleanup

Run as **root** after checks pass:

```bash
apt-get clean
journalctl --vacuum-time=1s
rm -rf /var/lib/cloud/instances/* /var/log/cloud-init*
history -c
truncate -s 0 /root/.bash_history /home/agent/.bash_history
```

Optional — stop stray dev servers so the snapshot is clean:

```bash
# cl8y
sudo -u agent bash -lc 'cd /home/agent/workspace && make stop' || true
# yieldomega (e2e-anvil uses pid files; prefer reading those over broad pkill)
sudo -u agent bash -lc 'for f in /tmp/yieldomega-anvil-8545.pid /tmp/yieldomega-vite-preview-4173.pid; do
  [[ -f "$f" ]] && kill "$(cat "$f")" 2>/dev/null || true
done'
```

## 6. Snapshot and register

1. Power off the builder VM
2. Hetzner **Images → Create Image** — note the numeric snapshot ID
3. On the controller:

```bash
source /opt/gitlab-cursor-webhook/scripts/gch-controller-shell.sh
SNAPSHOT=<new_id>
run_gch tag add --project <project> --name security   --snapshot "$SNAPSHOT"
run_gch tag add --project <project> --name verify     --snapshot "$SNAPSHOT"
run_gch tag add --project <project> --name implement  --snapshot "$SNAPSHOT"
```

Example: `--project yieldomega` or `--project cl8y-dex-terraclassic`.

The snapshot must exist in a location job provisioning can reach (typically **fsn1**). `GITLAB_TOKEN` on the controller is for job-time `glab` — do not bake it into the image.
