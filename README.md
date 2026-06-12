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
| `POST` | `/webhook` | GitLab webhook ingress (Standard Webhooks HMAC signature) |
| `GET` | `/api/jobs/{job_id}` | Agent VM fetches job (Bearer `JOB_RUNTIME_TOKEN`) |
| `POST` | `/api/jobs/{job_id}/heartbeat` | Agent heartbeat (every 60s) |
| `POST` | `/api/jobs/{job_id}/status` | Optional progress callback |
| `POST` | `/api/jobs/{job_id}/complete` | Agent finished — triggers VM destroy |
| `GET` | `/api/admin/jobs` | List in-memory jobs (Bearer `GCH_ADMIN_TOKEN`) |
| `GET` | `/api/admin/jobs/{job_id}` | Job details (Bearer `GCH_ADMIN_TOKEN`) |

## Flow

1. GitLab sends MR or issue webhook → filter/validate/dedup (unchanged rules)
2. `gchcontroller` resolves project + tag from SQLite (`security` for MRs; issue labels `agent:{tag}`)
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
gchconfig project set-signing-token --gitlab plasticdigits/yieldomega --token whsec_...
gchconfig tag add --project yieldomega --name security --snapshot 12345678
gchconfig prompt set --project yieldomega --tag security --file prompts/security.md
gchconfig setting controller_url https://gch.example.com
gchconfig doctor
gchconfig jobs list              # requires GCH_ADMIN_TOKEN + controller_url
gchconfig jobs list --active     # provisioning or running only
gchconfig jobs show <job-uuid>
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

### Merge requests

- **`security`** (default): forwards on `open`, or `update` with new commits (`oldrev` set); user in `ALLOWED_USERS`; project configured.
- **Other tags** (e.g. `fix_conflicts`, `fix_security`): forwards when an `agent:{tag}` label is on the MR at open, or newly added on `update` — same label rules as issues. Deduped per `project_id:iid:tag` (issue dedup TTL).

### Issues → any configured tag via `agent:{tag}` label

Forwards when an `agent:{tag}` label is present on open, or newly added on update (e.g. `agent:verify`, `agent:gap_analysis`). The `{tag}` must match a `gchconfig` tag for that project. **Reserved:** `security` is MR-only — `agent:security` on issues is ignored. When multiple agent labels fire, priority is `implement` > `verify` > others alphabetically.

Dedup: MR by `project_id:iid:commit_sha:tag`; issues by `project_id:iid:tag` (each tag/flow dedupes independently).

## Golden images

See [docs/admin-golden-image.md](docs/admin-golden-image.md).

Project runbooks (controller + webhook + `gchconfig`):

- [docs/runbook-cl8y-dex-terraclassic.md](docs/runbook-cl8y-dex-terraclassic.md)

Example project scripts:

- [docs/examples/yieldomega/](docs/examples/yieldomega/) — EVM / Foundry / Rabby
- [docs/examples/cl8y-dex-terraclassic/](docs/examples/cl8y-dex-terraclassic/) — Terra / Keplr

Copy `gch-cloud-setup.sh`, `gch-cloud-init.sh`, `gch-cloud-init-runner.sh`, `gch-golden-image-finalize.md`, and `prompts/` into each project repo before building snapshots.

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
