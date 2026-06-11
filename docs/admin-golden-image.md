# Golden Image Runbook (Hetzner CX33)

Create one golden snapshot per GitLab project (`yieldomega`, `cl8y-dex-terraclassic`). Agent VMs are cloned from these snapshots on each job.

## VM spec

| Setting | Value |
|---------|-------|
| Plan | **CX33** (4 vCPU, 8 GB RAM, 80 GB NVMe) |
| OS | Ubuntu 24.04 |
| Location | `nbg1` (or your preferred Hetzner location) |
| Extra volume | None |

Approximate cost: **€5.49/mo** cap if left running; snapshots incur small storage fees.

## Steps

### 1. Create builder VM

1. Hetzner Cloud Console → **Add Server**
2. Ubuntu 24.04, CX33, your SSH key
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

On the controller host:

```bash
gchconfig project add --gitlab plasticdigits/yieldomega --workspace /home/agent/workspace
gchconfig tag add --project yieldomega --name security --snapshot <SNAPSHOT_ID>
gchconfig tag add --project yieldomega --name verify --snapshot <SNAPSHOT_ID>
gchconfig tag add --project yieldomega --name implement --snapshot <SNAPSHOT_ID>
gchconfig prompt set --project yieldomega --tag security --file prompts/security.md
gchconfig doctor
```

Repeat tags if you use separate snapshots per tag; typically one snapshot serves all three tags.

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
- `ALLOWED_USERS`, `GITLAB_WEBHOOK_SIGNING_TOKEN` (from GitLab webhook **Generate signing token**)
- Terraform CLI installed
- `GCH_DB_PATH` (default `/var/lib/gch/gch.db`)

### GitLab webhook

In each project or group: **Settings → Webhooks**

- **URL:** `{GCH_CONTROLLER_URL}/webhook`
- **Trigger:** Merge request events, Issue events
- **Authentication:** **Generate signing token** (not Secret token). Copy the one-time `whsec_...` value into `GITLAB_WEBHOOK_SIGNING_TOKEN` on the controller host.
