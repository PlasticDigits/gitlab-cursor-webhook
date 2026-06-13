# Golden Image Runbook (Hetzner CPX32)

Create one golden snapshot per GitLab project (`yieldomega`, `cl8y-dex-terraclassic`). Agent VMs are cloned from these snapshots on each job.

## VM spec

| Setting | Value |
|---------|-------|
| Plan | **CPX32** (4 vCPU, 8 GB RAM, 160 GB NVMe) — shared vCPU, good price/perf for agents |
| OS | **Ubuntu 24.04 only** (not 26.04 — Playwright and agent tooling are validated on 24.04) |
| Location | `fsn1` (Falkenstein) — match snapshot and tag location |
| Extra volume | None |

Approximate cost: billed per hour while the VM runs; snapshots incur small storage fees.

## Steps

### 1. Create builder VM

1. Hetzner Cloud Console → **Add Server**
2. Ubuntu **24.04** (not 26.04), CPX32, location Falkenstein (`fsn1`), your SSH key
3. SSH in as root and confirm OS:

```bash
lsb_release -ds    # expect Ubuntu 24.04.x
```

### 1b. Prerequisites (project repo)

`gch-cloud-setup.sh` is run from the **GitLab project** clone, not from this repo. Before building, ensure the project on `main` includes the files below. Copy from `gitlab-cursor-webhook/docs/examples/<project>/` if the project repo is behind.

**cl8y-dex-terraclassic (Terra / Keplr / LocalTerra)**

| File | Requirement |
|------|-------------|
| `gch-cloud-setup.sh` | `/etc/profile.d/gch-agent.sh`, agent `.bashrc` sources `/etc/gch/job.env`; after `npm ci`, `playwright install` + `install-deps` in `frontend-dapp` |
| `gch-cloud-init-runner.sh` | `jq -r '.git_ref // empty'` (not bare `empty` as filename) |
| `gch-golden-image-finalize.md` | Keplr + LocalTerra finalize tasks |
| `scripts/setup-cloud-agent-localterra.sh` | `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE` before `playwright install` |

**yieldomega (EVM / Rabby / Anvil)**

| File | Requirement |
|------|-------------|
| `gch-cloud-setup.sh` | `/etc/profile.d/gch-agent.sh`, agent `.bashrc` sources `/etc/gch/job.env`; calls `scripts/bootstrap-*.sh` |
| `gch-cloud-init.sh` | Sources `gch-cloud-init-runner.sh`; Foundry on PATH |
| `gch-cloud-init-runner.sh` | `jq -r '.git_ref // empty'` |
| `gch-golden-image-finalize.md` | Rabby (`/opt/cursor/…`) + Anvil finalize tasks |
| `gch-agent-idle-wrap.py` | Agent idle timeout wrapper |
| `scripts/bootstrap-dev.sh` | Git submodules + `frontend/` `npm ci` |
| `scripts/bootstrap-cloud-vm-toolchain.sh` | Foundry, Rabby extension, glab, Docker |
| `scripts/bootstrap-cloud-postgres-native.sh` | Indexer Postgres on port 5433 |
| `scripts/bootstrap-cloud-agent.sh` | Playwright from `frontend/` lock + Rabby wallet import |
| `scripts/install-browser-extensions.sh` | Unpacked Rabby under `/opt/cursor/browser-extensions/rabby` |

### 2. Clone project and run setup

`/home/agent` does not exist on a fresh builder VM until setup creates the `agent` user. Clone to a staging directory first, run setup from the project repo, then move it into place.

Clone over HTTPS (no GitLab SSH key needed on the builder VM). Setup requires `CURSOR_API_KEY` — it runs a finalize agent (`composer-2.5` by default) to install wallet extensions, configure LocalTerra/Foundry, and verify the toolchain.

```bash
git clone https://gitlab.com/plasticdigits/<project> /tmp/gch-workspace
chmod +x /tmp/gch-workspace/gch-cloud-setup.sh
export CURSOR_API_KEY='your-cursor-api-key'
bash /tmp/gch-workspace/gch-cloud-setup.sh
rm -rf /tmp/gch-workspace   # optional; repo is copied to /home/agent/workspace
```

Example for YieldOmega (EVM / Rabby / Anvil):

```bash
git clone https://gitlab.com/plasticdigits/yieldomega /tmp/gch-workspace
chmod +x /tmp/gch-workspace/gch-cloud-setup.sh
export CURSOR_API_KEY='your-cursor-api-key'
bash /tmp/gch-workspace/gch-cloud-setup.sh
```

Example for Terra Classic:

```bash
git clone https://gitlab.com/plasticdigits/cl8y-dex-terraclassic /tmp/gch-workspace
chmod +x /tmp/gch-workspace/gch-cloud-setup.sh
export CURSOR_API_KEY='your-cursor-api-key'
bash /tmp/gch-workspace/gch-cloud-setup.sh
```

Optional: override the finalize model with `GCH_GOLDEN_IMAGE_MODEL=composer-2.5-fast`.

After setup, review `/home/agent/.gch/golden-image-verify.log`.

**Playwright browsers (cl8y-dex-terraclassic / `frontend-dapp`):** `gch-cloud-setup.sh` runs `npm ci` in `frontend-dapp`, installs browsers from that package’s locked `@playwright/test` version, then `npx playwright install-deps` for apt libraries (GTK, GStreamer, etc.).

**Playwright + Rabby (yieldomega / `frontend/`):** `gch-cloud-setup.sh` runs `scripts/bootstrap-dev.sh` then `scripts/bootstrap-cloud-agent.sh`, which installs Playwright Chromium from `frontend/package-lock.json`. Rabby lives at `/opt/cursor/browser-extensions/rabby` (not `~/.gch/extensions`).

Browsers are cached under `~/.cache/ms-playwright/` (not `frontend/node_modules/.cache/`). Use full `playwright install` — not `install chromium` alone — so `chromium_headless_shell` matches e2e tests.

If setup was interrupted or frontend dependencies changed, re-run as `agent` before snapshot:

**Terra Classic (`frontend-dapp`):**

```bash
sudo -u agent bash -lc '
  export PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64
  export NVM_DIR="$HOME/.nvm"
  [ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"
  nvm use 24.15.0
  cd /home/agent/workspace/frontend-dapp
  npx playwright install
  sudo -E env "PATH=$PATH" npx playwright install-deps
'
```

**YieldOmega (`frontend/`):**

```bash
sudo -u agent bash -lc '
  export PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64
  cd /home/agent/workspace/frontend
  npx playwright install chromium
  sudo -E env "PATH=$PATH" npx playwright install-deps chromium
  bash /home/agent/workspace/scripts/bootstrap-cloud-agent.sh
'
```

Avoid `with-node.sh` for Playwright on Node 24.16+ — it can hang during browser zip extraction. Pin `.nvmrc` to `24.15.0` or upgrade `@playwright/test` to ≥ 1.60.0 (cl8y only).

Do not install Playwright from a separate `~/.gch/playwright` sandbox — that can pin a different browser build than the frontend lock and break e2e.

### 3. Verify agent user

```bash
id agent
agent about                            # Cursor CLI (symlinked to /usr/local/bin by setup)
sudo -u agent docker ps
cat /home/agent/.cursor/cli-config.json   # attribution off, approvalMode unrestricted
cat /home/agent/.gch/golden-image-verify.log
```

The `agent` user must have:

- No password (`passwd -l agent`)
- Passwordless sudo (`/etc/sudoers.d/agent`)
- Membership in `docker` group

**Confirm baked image** (must pass before snapshot):

```bash
grep "git_ref" /home/agent/gch-cloud-init-runner.sh
# expect: jq -r '.git_ref // empty'

test -f /etc/profile.d/gch-agent.sh
grep -q 'GCH job secrets' /home/agent/.bashrc
```

**cl8y-dex-terraclassic only:**

```bash
grep -q PLAYWRIGHT_HOST_PLATFORM_OVERRIDE /home/agent/workspace/scripts/setup-cloud-agent-localterra.sh
sudo -u agent bash -lc 'ls "$HOME/.cache/ms-playwright"/chromium-* "$HOME/.cache/ms-playwright"/chromium_headless_shell-* >/dev/null'
```

**yieldomega only:**

```bash
test -f /opt/cursor/browser-extensions/rabby/manifest.json
sudo -u agent bash -lc 'forge --version && anvil --version'
sudo -u agent bash -lc 'bash /home/agent/workspace/scripts/verify-rabby-playwright-injection.sh'
sudo -u agent bash -lc 'bash /home/agent/workspace/scripts/verify-cloud-postgres.sh'
sudo -u agent bash -lc 'ls "$HOME/.cache/ms-playwright"/chromium-* >/dev/null'
# optional strong signal:
# sudo -u agent bash -lc 'cd /home/agent/workspace && bash scripts/e2e-anvil.sh'
```

If `apt upgrade` installed a new kernel during setup, **reboot** and re-run the smoke checks above before cleanup:

```bash
reboot
# after SSH back:
uname -r
sudo -u agent agent about
```

Optional (Terra Classic): stop job-local docker so the snapshot is clean:

```bash
sudo -u agent bash -lc 'cd /home/agent/workspace && make stop' || true
sudo -u agent docker ps   # expect empty
```

Optional (yieldomega): stop any running Anvil or preview servers:

```bash
sudo -u agent pkill -f 'anvil.*8545' 2>/dev/null || true
sudo -u agent pkill -f 'vite preview' 2>/dev/null || true
```

### 4. Pre-snapshot cleanup

Run as root **after** setup completes and checks pass:

```bash
apt-get clean
journalctl --vacuum-time=1s
rm -rf /var/lib/cloud/instances/* /var/log/cloud-init*
history -c
truncate -s 0 /root/.bash_history /home/agent/.bash_history
```

### 5. Create snapshot

1. Power off the server
2. **Images → Create Image** (snapshot)
3. Note the numeric **snapshot ID**

### 6. Register in gchconfig

On the controller host, point all three tags at the new snapshot ID:

```bash
source /opt/gitlab-cursor-webhook/scripts/gch-controller-shell.sh
SNAPSHOT=<new_id>
run_gch tag add --project <project> --name security   --snapshot "$SNAPSHOT"
run_gch tag add --project <project> --name verify     --snapshot "$SNAPSHOT"
run_gch tag add --project <project> --name implement  --snapshot "$SNAPSHOT"
```

Example: `--project yieldomega` or `--project cl8y-dex-terraclassic`.

Full webhook and signing-token steps: [runbook-cl8y-dex-terraclassic.md](runbook-cl8y-dex-terraclassic.md) (same `gchconfig` pattern for yieldomega).

Ensure controller `GITLAB_TOKEN` in `/etc/gitlab-cursor-webhook.env` is valid for `glab` (not baked into the snapshot).

### VM placement fallbacks

`gchcontroller` tries these Hetzner type/location pairs in order until one succeeds:

1. `cx33` — `nbg1`, `fsn1`, `hel1`
2. `cpx32` — `nbg1`, `fsn1`, `hel1`
3. `cpx41` — `hil` (Hillsboro, US last resort)

Tag `server_type` / `location` in `gchconfig` are defaults for display; provisioning uses the fallback chain above. The snapshot must exist in the location that succeeds (rebuild golden image in `fsn1` if needed).

## Firewall (one-time)

```bash
cd terraform/firewall
export HCLOUD_TOKEN=...
terraform init && terraform apply
gchconfig setting firewall_id <output_id>
```

## Controller host

Run `gchcontroller` on a small always-on VM (e.g. CX23) with:

- `HCLOUD_TOKEN`, `GCH_FIREWALL_ID`, `GCH_CONTROLLER_URL`
- `GCH_SSH_KEY_IDS` — Hetzner SSH key name, fingerprint, or numeric id (e.g. `admin-ceramic`)
- `CURSOR_API_KEY`, `GITLAB_TOKEN`
- `ALLOWED_USERS` (per-project webhook signing tokens are stored in SQLite via `gchconfig`)
- Terraform CLI installed
- `GCH_DB_PATH` (default `/var/lib/gch/gch.db`)
- `GCH_ADMIN_TOKEN` (optional; enables `gchconfig jobs` and `/api/admin/jobs`)

### Shell helper (controller)

Persist `run_gch` across SSH sessions:

```bash
grep -q gch-controller-shell.sh ~/.bashrc 2>/dev/null || \
  echo 'source /opt/gitlab-cursor-webhook/scripts/gch-controller-shell.sh' >> ~/.bashrc
source ~/.bashrc
run_gch jobs list --active
```

### GitLab webhook

In each project or group: **Settings → Webhooks**

- **URL:** `{GCH_CONTROLLER_URL}/webhook`
- **Trigger:** Merge request events, Issue events
- **Authentication:** **Generate signing token** (not Secret token). Copy the one-time `whsec_...` value into SQLite:

```bash
gchconfig project set-signing-token --gitlab group/project --token whsec_...
```
