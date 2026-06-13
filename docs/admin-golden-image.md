# Golden image builder (Hetzner)

One snapshot per GitLab project. Job VMs boot from it via `gchconfig` tags. Controller setup: [README](../README.md), [runbook-cl8y-dex-terraclassic.md](runbook-cl8y-dex-terraclassic.md).

## Builder VM

- **Disk:** 80 GB
- **OS:** Ubuntu 24.04
- **SSH:** your admin key

## 1. Sync GCH files into the project repo

Copy from `docs/examples/<project>/` into the project on `main`: `gch-cloud-setup.sh`, `gch-cloud-init.sh`, `gch-cloud-init-runner.sh` (must use `jq -r '.git_ref // empty'`), `gch-agent-idle-wrap.py`, `gch-golden-image-finalize.md`, `prompts/`.

Examples: [yieldomega](examples/yieldomega/), [cl8y-dex-terraclassic](examples/cl8y-dex-terraclassic/).

## 2. Run setup

SSH as **root**. `gch-cloud-setup.sh` runs from a project clone — not from `/home/agent`. Re-clone each time you build or update a snapshot.

```bash
rm -rf /tmp/gch-workspace
git clone https://gitlab.com/plasticdigits/<project> /tmp/gch-workspace
chmod +x /tmp/gch-workspace/gch-cloud-setup.sh
export CURSOR_API_KEY='your-cursor-api-key'
bash /tmp/gch-workspace/gch-cloud-setup.sh
```

Setup creates `/home/agent`, rsyncs into `/home/agent/workspace`, runs bootstrap, then a finalize agent that writes `/home/agent/.gch/golden-image-verify.log`.

## 3. Pass criteria

```bash
cat /home/agent/.gch/golden-image-verify.log   # OVERALL: PASS
sudo -u agent agent about
grep -q 'GCH job secrets' /home/agent/.bashrc
grep "git_ref" /home/agent/gch-cloud-init-runner.sh
```

Reboot if `apt upgrade` installed a new kernel, then re-check.

## 4. Re-run finalize only

```bash
if ! pgrep -x Xvfb >/dev/null 2>&1; then Xvfb :99 -screen 0 1920x1080x24 & sleep 1; fi
export CURSOR_API_KEY='your-cursor-api-key'
```

**cl8y-dex-terraclassic:**

```bash
sudo -u agent env CURSOR_API_KEY="$CURSOR_API_KEY" DISPLAY=:99 bash -lc '
  cd /home/agent/workspace
  agent -p "$(cat gch-golden-image-finalize.md)" --model composer-2.5 --force --trust \
    --workspace /home/agent/workspace --output-format text
'
```

**yieldomega:**

```bash
pkill -9 -f 'chrome-profile-rabby' 2>/dev/null || true
rm -f /opt/cursor/chrome-profile-rabby/SingletonLock
sudo -u agent env DISPLAY=:99 \
  YIELDOMEGA_SKIP_RABBY_WALLET_IMPORT=1 YIELDOMEGA_SKIP_RABBY_INJECTION_VERIFY=1 \
  bash -lc 'bash /home/agent/workspace/scripts/bootstrap-cloud-agent.sh' || true
sudo -u agent env CURSOR_API_KEY="$CURSOR_API_KEY" DISPLAY=:99 bash -lc '
  cd /home/agent/workspace
  agent --print "$(cat gch-golden-image-finalize.md)" --model composer-2.5 --force --trust \
    --workspace /home/agent/workspace --output-format stream-json --stream-partial-output \
    2>&1 | tee /home/agent/.gch/finalize-agent.raw | python3 /home/agent/gch-agent-stream-watch.py
'
```

## 5. Pre-snapshot cleanup

```bash
apt-get clean
journalctl --vacuum-time=1s
rm -rf /var/lib/cloud/instances/* /var/log/cloud-init* /tmp/gch-workspace
history -c
truncate -s 0 /root/.bash_history /home/agent/.bash_history
sudo -u agent bash -lc 'cd /home/agent/workspace && make stop' 2>/dev/null || true   # cl8y
```

## 6. Snapshot and register

1. Power off the builder VM
2. Hetzner **Images → Create Image** — note the snapshot ID
3. On the controller:

```bash
source /opt/gitlab-cursor-webhook/scripts/gch-controller-shell.sh
SNAPSHOT=<new_id>
run_gch tag add --project <project> --name security   --snapshot "$SNAPSHOT"
run_gch tag add --project <project> --name verify     --snapshot "$SNAPSHOT"
run_gch tag add --project <project> --name implement  --snapshot "$SNAPSHOT"
```

Do not bake `GITLAB_TOKEN` into the image — the controller injects it at job time.
