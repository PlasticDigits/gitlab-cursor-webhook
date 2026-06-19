# Docker / Coolify deployment

Run `gchcontroller` in a container with persistent storage for the SQLite database and per-job Terraform workspaces.

## What persists

| Path in container | Contents |
|-------------------|----------|
| `/var/lib/gch/gch.db` | Projects, tags, prompts, signing tokens, settings |
| `/var/lib/gch/jobs/` | Per-job `terraform.tfstate` and manifests |

Everything else (binaries, Terraform module, cloud-init template) ships in the image.

## Migrate from bare-metal (systemd)

### 1. Pre-flight

- Wait for active jobs to finish, or accept orphaned Hetzner VMs if you cut over mid-job:
  ```bash
  gchconfig jobs list --active
  ```
- Note your public URL (`GCH_CONTROLLER_URL`). If Coolify uses a new hostname, update GitLab webhook URLs and `gchconfig setting controller_url` after cutover.

### 2. Export from the old controller

On the existing host:

```bash
cd /opt/gitlab-cursor-webhook
sudo ./scripts/gch-export-for-docker.sh /tmp/gch-migration
```

This stops `gitlab-cursor-webhook.service` briefly, copies `gch.db`, `jobs/`, and `/etc/gitlab-cursor-webhook.env`, then restarts the service.

Copy the tarball to your Coolify server:

```bash
scp /tmp/gch-migration/gch-migration-*.tar.gz coolify:/tmp/
```

### 3. Create the Coolify service

1. **New Resource** → **Application** → your Git repo (or Dockerfile build).
2. **Build pack**: Dockerfile at repo root.
3. **Port**: `8080` (container listens on `0.0.0.0:8080`).
4. **Health check**: `GET /health` on port `8080`.
5. **Persistent storage**: mount a volume at `/var/lib/gch` (required).
6. **Environment variables**: paste from `gitlab-cursor-webhook.env` (see below).

Disable the old reverse proxy / systemd host only after the new deployment is verified.

### 4. Import data on Coolify

SSH to the Coolify server and find the volume path for your service (Coolify UI → Storage, or under `/data/coolify/...`).

```bash
# Example — replace with your actual volume mount path
VOL=/data/coolify/applications/<uuid>/volumes/gch-data
sudo mkdir -p "$VOL"
cd "$VOL"
sudo tar -xzf /tmp/gch-migration-*.tar.gz -C /tmp/gch-import
sudo cp -a /tmp/gch-import/var/lib/gch/* .
sudo chown -R 9999:9999 .   # adjust if your container runs as a specific UID
```

If the container is already running, restart it after copying files.

### 5. Environment variables (Coolify)

Copy every key from `/etc/gitlab-cursor-webhook.env`. Required keys:

| Variable | Notes |
|----------|-------|
| `ALLOWED_USERS` | Comma-separated GitLab usernames |
| `HCLOUD_TOKEN` | Hetzner API token |
| `GCH_FIREWALL_ID` | Or set via SQLite `gchconfig setting firewall_id` |
| `GCH_CONTROLLER_URL` | Public HTTPS URL (Coolify domain) |
| `CURSOR_API_KEY` | Injected into agent VMs |
| `GITLAB_TOKEN` | Injected into agent VMs |
| `GCH_SSH_KEY_IDS` | Hetzner SSH keys for agent VMs |

**Do not set** `GCH_TERRAFORM_MODULE` or `GCH_CLOUD_INIT_TEMPLATE` unless overriding — the image defaults to `/app/terraform/...` and `/app/templates/...`.

**Change for Docker**:

| Variable | Bare-metal | Docker / Coolify |
|----------|------------|------------------|
| `LISTEN_ADDR` | `127.0.0.1:8080` | `0.0.0.0:8080` (Coolify proxy reaches the container) |
| `GCH_DB_PATH` | `/var/lib/gch/gch.db` | `/var/lib/gch/gch.db` (same; volume-backed) |
| `GCH_JOBS_DIR` | `/var/lib/gch/jobs` | `/var/lib/gch/jobs` (same; volume-backed) |

Optional: `GCH_ADMIN_TOKEN` for admin API and `gchconfig jobs`.

### 6. Verify

```bash
curl -s https://your-coolify-domain/health
# {"status":"ok"}
```

From your laptop (with env pointing at the volume or via `docker exec`):

```bash
docker exec -it <container> gchconfig doctor
docker exec -it <container> gchconfig project list
```

Trigger a test webhook or `gchconfig dry-run`.

### 7. Cutover

1. Point DNS / Coolify domain to the new deployment.
2. Update `GCH_CONTROLLER_URL` in Coolify env if the URL changed.
3. Update GitLab project webhook URLs to the new domain.
4. Stop and disable the old host:
   ```bash
   sudo systemctl disable --now gitlab-cursor-webhook.service
   ```

## Local smoke test

```bash
cp .env.example .env
# Edit .env with real secrets

docker compose up --build
curl -s http://127.0.0.1:8080/health
```

To import an existing DB locally:

```bash
docker compose down
tar -xzf gch-migration-*.tar.gz -C /tmp/gch-import
sudo cp -a /tmp/gch-import/var/lib/gch/* ./gch-data/   # or docker volume path
docker compose up -d
```

## `gchconfig` after migration

Run inside the container:

```bash
docker exec -it <container> gchconfig project list
docker exec -it <container> gchconfig prompt set --project myproject --tag security --file /path/in/container
```

Or install `gchconfig` on your workstation and point `GCH_DB_PATH` at a copy of `gch.db` (read-only recommended).

## Troubleshooting

| Symptom | Check |
|---------|-------|
| `configuration error: missing ...` | Required env vars in Coolify UI |
| Webhook 401 | Signing tokens in SQLite; `gchconfig project list` |
| Terraform failures | `docker exec <c> terraform version`; jobs dir writable on volume |
| Empty project list | Volume not mounted at `/var/lib/gch` or wrong `GCH_DB_PATH` |
| Active jobs lost on restart | In-memory job state is not in SQLite; wait for jobs to finish before migration |
