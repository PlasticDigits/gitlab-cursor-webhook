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

```bash
git clone git@gitlab.com:plasticdigits/<project>.git /home/agent/workspace
# Copy gch-cloud-setup.sh from the project repo into workspace root, then:
chmod +x /home/agent/workspace/gch-cloud-setup.sh
bash /home/agent/workspace/gch-cloud-setup.sh
```

See [docs/examples/](examples/) for project-specific setup scripts to copy into each repo.

### 3. Verify agent user

```bash
id agent
sudo -u agent agent about   # Cursor CLI
sudo -u agent docker ps
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
- `ALLOWED_USERS`, `GITLAB_WEBHOOK_SECRET`
- Terraform CLI installed
- `GCH_DB_PATH` (default `/var/lib/gch/gch.db`)
