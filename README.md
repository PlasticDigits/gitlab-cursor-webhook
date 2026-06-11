# gitlab-cursor-webhook (GitLab Cloud Host)

Filter GitLab webhooks and provision ephemeral **Hetzner agent VMs** from golden snapshots. Each VM runs the **Cursor CLI** headless agent against the project workspace.

## Components

| Binary | Purpose |
|--------|---------|
| `gchcontroller` | HTTP server: GitLab webhooks, job API, VM provisioning via Terraform |
| `gchconfig` | SQLite CLI: projects, tags, prompts, doctor, dry-run |

| Crate | Purpose |
|-------|---------|
| `gch-core` | Shared filter logic, dedup, database, prompt rendering |

## Routes

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Liveness — `{"status":"ok"}` |
| `POST` | `/webhook` | GitLab webhook ingress |
| `GET` | `/api/jobs/{job_id}` | Agent VM fetches job (Bearer `JOB_RUNTIME_TOKEN`) |
| `POST` | `/api/jobs/{job_id}/heartbeat` | Agent heartbeat (every 60s) |
| `POST` | `/api/jobs/{job_id}/status` | Optional progress callback |
| `POST` | `/api/jobs/{job_id}/complete` | Agent finished — triggers VM destroy |

## Flow

1. GitLab sends MR or issue webhook → filter/validate/dedup (unchanged rules)
2. `gchcontroller` resolves project + tag (`security`, `verify`, `implement`) from SQLite
3. Renders prompt template, creates job, runs **isolated Terraform apply** (one state file per job)
4. VM boots from golden snapshot, cloud-init injects secrets and runs project `gch-cloud-init.sh`
5. VM calls `GET /api/jobs/{id}`, runs `agent -p ... --force --trust`, sends heartbeats
6. On complete, timeout (3h), or stale heartbeat (5m), controller destroys the VM

## Build

```bash
cargo build --release
# Binaries: target/release/gchcontroller, target/release/gchconfig
```

## Configuration

### Controller environment

See [`.env.example`](.env.example). Required:

- `ALLOWED_USERS` — comma-separated GitLab usernames (fail-closed if empty)
- `HCLOUD_TOKEN`, `GCH_FIREWALL_ID`, `GCH_CONTROLLER_URL`
- `CURSOR_API_KEY`, `GITLAB_TOKEN`

### SQLite (`gchconfig`)

```bash
gchconfig project add --gitlab plasticdigits/yieldomega --workspace /home/agent/workspace
gchconfig tag add --project yieldomega --name security --snapshot 12345678
gchconfig prompt set --project yieldomega --tag security --file prompts/security.md
gchconfig setting controller_url https://gch.example.com
gchconfig doctor
```

Migrate legacy env-based project paths:

```bash
gchconfig import-env   # reads PROJECT_WEBHOOKS_* for project paths only
```

### Dry-run (no VM)

```bash
gchconfig dry-run --fixture tests/fixtures/mr_open.json
```

## Filter behavior

### Merge requests → tag `security`

Forwards when: `open`, or `update` with new commits (`oldrev` set); user in `ALLOWED_USERS`; project configured.

### Issues → tags `verify` / `implement`

Forwards when label `agent:verify` or `agent:implement` is present on open, or newly added on update. **Implement** wins when both would fire.

Dedup: MR by `project_id:iid:commit_sha`; issues by `project_id:iid` (shared across verify/implement).

## Golden images

See [docs/admin-golden-image.md](docs/admin-golden-image.md).

Example project scripts:

- [docs/examples/yieldomega/](docs/examples/yieldomega/) — EVM / Foundry / Rabby
- [docs/examples/cl8y-dex-terraclassic/](docs/examples/cl8y-dex-terraclassic/) — Terra / Keplr

Copy `gch-cloud-setup.sh` and `gch-cloud-init.sh` into each project repo before building snapshots.

## Terraform

- **Firewall** (one-time): [terraform/firewall/](terraform/firewall/)
- **Agent VM module**: [terraform/modules/agent-vm/](terraform/modules/agent-vm/)

Each job gets its own directory under `GCH_JOBS_DIR` with a separate `terraform.tfstate`.

## Development

```bash
cp .env.example .env
# Configure GCH_DB_PATH, disable terraform in tests via GCH_PROVISION_ENABLED=false

cargo test
cargo clippy -- -D warnings
```

## License

AGPL-3.0-or-later — see [LICENSE](LICENSE).
