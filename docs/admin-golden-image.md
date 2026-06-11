# Golden Image Runbook (Hetzner CPX32)

Create one golden snapshot per GitLab project (`yieldomega`, `cl8y-dex-terraclassic`). Agent VMs are cloned from these snapshots on each job.

## VM spec

| Setting | Value |
|---------|-------|
| Plan | **CPX32** (4 vCPU, 8 GB RAM, 160 GB NVMe) — shared vCPU, good price/perf for agents |
| OS | Ubuntu 24.04 |
| Location | `fsn1` (Falkenstein) — match snapshot and tag location |
| Extra volume | None |

Approximate cost: billed per hour while the VM runs; snapshots incur small storage fees.

## Steps

### 1. Create builder VM

1. Hetzner Cloud Console → **Add Server**
2. Ubuntu 24.04, CPX32, location Falkenstein (`fsn1`), your SSH key
3. SSH in as root

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

### 4. Pre-snapshot cleanup

Run as root **after** setup completes:

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

On the controller host, follow the project runbook:

- [runbook-cl8y-dex-terraclassic.md](runbook-cl8y-dex-terraclassic.md) — Terra Classic (full `gchconfig` + webhook steps)
- yieldomega — same pattern; prompts under `docs/examples/yieldomega/prompts/`

Typically one snapshot ID serves all three tags (`security`, `verify`, `implement`).

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
