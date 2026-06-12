# Runbook: cl8y-dex-terraclassic on GitLab Cloud Host

Register `plasticdigits/cl8y-dex-terraclassic` on the controller with all three agents (`security`, `verify`, `implement`).

Prerequisites:

- **Recent build** of `gchconfig` and `gchcontroller` (includes per-project `set-signing-token`). If you see `unrecognized subcommand 'set-signing-token'`, rebuild and redeploy — see [§0. Build and deploy](#0-build-and-deploy).
- Controller host with `gchcontroller` running (see [admin-golden-image.md](admin-golden-image.md))
- Golden snapshot built for Terra Classic (same doc, §2–5)
- `/etc/gitlab-cursor-webhook.env` loaded by systemd (`EnvironmentFile=` in the unit)
- `gchconfig` and repo at `/opt/gitlab-cursor-webhook`

## Agent triggers

| gchconfig tag | When it runs | GitLab signal |
|---------------|--------------|---------------|
| `security` | MR opened, or MR updated with **new commits** | Merge request webhook; deduped per commit SHA |
| `fix_conflicts` | MR with label `agent:fix_conflicts` (open or label added on update) | Merge request webhook |
| `fix_security` | MR with label `agent:fix_security` (open or label added on update) | Merge request webhook |
| `fix_bugfix` | MR with label `agent:fix_bugfix` (open or label added on update) | Merge request webhook |
| `verify` | Issue open/update with label `agent:verify` | Issue webhook |
| `implement` | Issue open/update with label `agent:implement` | Issue webhook (`implement` wins if both labels) |
| `gap_analysis` | Issue with label `agent:gap_analysis` | Issue webhook |
| `security_audit` | Issue with label `agent:security_audit` | Issue webhook |
| `open_issues` | Issue with label `agent:open_issues` | Issue webhook; triages parent comments, opens child issues for approved items |
| *(other tags)* | Issue with label `agent:{tag}` | Issue webhook; `security` is reserved (MR-only) |

`security` does **not** use GitLab tags or labels — only merge request events. Enable **Merge request events** and **Issue events** on the webhook; tag push is not required.

## 0. Build and deploy

On the controller host after `git pull`:

```bash
cd /opt/gitlab-cursor-webhook
cargo build --release
sudo cp target/release/gchconfig target/release/gchcontroller /usr/local/bin/
# or wherever your systemd unit points (check: systemctl cat gitlab-cursor-webhook.service)

gchconfig project --help   # should list set-signing-token
sudo systemctl restart gitlab-cursor-webhook.service
```

## 1. GitLab webhook

In `plasticdigits/cl8y-dex-terraclassic`: **Settings → Webhooks**

| Setting | Value |
|---------|--------|
| URL | `{GCH_CONTROLLER_URL}/webhook` |
| Merge request events | on |
| Issue events | on |
| Authentication | **Generate signing token** (not Secret token) |

Copy the one-time `whsec_...` value — it is shown only once.

## 2. Register project in SQLite

On the controller host:

```bash
cd /opt/gitlab-cursor-webhook
source scripts/gch-controller-shell.sh   # defines run_gch; loads /etc/gitlab-cursor-webhook.env

REPO=/opt/gitlab-cursor-webhook

# Fill these in before running:
TERRA_SNAPSHOT=12345678            # Hetzner snapshot ID from golden image
TERRA_SIGNING_TOKEN=whsec_...      # from GitLab webhook (step 1)

run_gch project add --gitlab plasticdigits/cl8y-dex-terraclassic --workspace /home/agent/workspace
run_gch project set-signing-token --gitlab plasticdigits/cl8y-dex-terraclassic --token "$TERRA_SIGNING_TOKEN"

run_gch tag add --project cl8y-dex-terraclassic --name security  --snapshot "$TERRA_SNAPSHOT"
run_gch tag add --project cl8y-dex-terraclassic --name verify    --snapshot "$TERRA_SNAPSHOT"
run_gch tag add --project cl8y-dex-terraclassic --name implement --snapshot "$TERRA_SNAPSHOT"

run_gch prompt set --project cl8y-dex-terraclassic --tag security  --file "$REPO/docs/examples/cl8y-dex-terraclassic/prompts/security.md"
run_gch prompt set --project cl8y-dex-terraclassic --tag verify    --file "$REPO/docs/examples/cl8y-dex-terraclassic/prompts/verify.md"
run_gch prompt set --project cl8y-dex-terraclassic --tag implement --file "$REPO/docs/examples/cl8y-dex-terraclassic/prompts/implement.md"

# Optional audit flows (same snapshot):
run_gch tag add --project cl8y-dex-terraclassic --name gap_analysis    --snapshot "$TERRA_SNAPSHOT"
run_gch tag add --project cl8y-dex-terraclassic --name security_audit  --snapshot "$TERRA_SNAPSHOT"
run_gch prompt set --project cl8y-dex-terraclassic --tag gap_analysis   --file "$REPO/docs/examples/cl8y-dex-terraclassic/prompts/gap_analysis.md"
run_gch prompt set --project cl8y-dex-terraclassic --tag security_audit --file "$REPO/docs/examples/cl8y-dex-terraclassic/prompts/security_audit.md"
run_gch tag add --project cl8y-dex-terraclassic --name open_issues    --snapshot "$TERRA_SNAPSHOT"
run_gch prompt set --project cl8y-dex-terraclassic --tag open_issues    --file "$REPO/docs/examples/cl8y-dex-terraclassic/prompts/open_issues.md"
run_gch tag add --project cl8y-dex-terraclassic --name fix_conflicts --snapshot "$TERRA_SNAPSHOT"
run_gch prompt set --project cl8y-dex-terraclassic --tag fix_conflicts --file "$REPO/docs/examples/cl8y-dex-terraclassic/prompts/fix_conflicts.md"
run_gch tag add --project cl8y-dex-terraclassic --name fix_security  --snapshot "$TERRA_SNAPSHOT"
run_gch prompt set --project cl8y-dex-terraclassic --tag fix_security  --file "$REPO/docs/examples/cl8y-dex-terraclassic/prompts/fix_security.md"
run_gch tag add --project cl8y-dex-terraclassic --name fix_bugfix   --snapshot "$TERRA_SNAPSHOT"
run_gch prompt set --project cl8y-dex-terraclassic --tag fix_bugfix   --file "$REPO/docs/examples/cl8y-dex-terraclassic/prompts/fix_bugfix.md"
```

One snapshot ID is enough for all three tags (same golden image).

If `project add` fails because the project already exists, skip that line and continue.

## 3. Verify

```bash
run_gch project list
run_gch tag list
run_gch doctor
```

`doctor` should report signing token set, tags, prompts, and snapshot IDs.

Dry-run an MR open fixture (no VM provisioned):

```bash
run_gch dry-run --fixture "$REPO/crates/gchcontroller/tests/fixtures/mr_open.json"
```

Restart the controller after env or binary changes:

```bash
systemctl restart gitlab-cursor-webhook.service
journalctl -u gitlab-cursor-webhook.service -n 30 --no-pager
```

## 4. Smoke test in GitLab

1. Open an MR as a user listed in `ALLOWED_USERS` → security agent should provision a VM.
2. Push a new commit to the same MR → security agent runs again (new commit, not deduped).
3. Open an issue, add label `agent:verify` → verify agent runs.
4. Add label `agent:implement` on another issue → implement agent runs.

## Troubleshooting

| Symptom | Check |
|---------|--------|
| `gchconfig project list` empty | `GCH_DB_PATH` — use `export GCH_DB_PATH=/var/lib/gch/gch.db` or `run_gch` wrapper; confirm same path as `gchcontroller` (`/proc/$(systemctl show -p MainPID --value gitlab-cursor-webhook.service)/environ`) |
| Webhook 401 | Signing token in SQLite matches GitLab webhook; regenerate if lost |
| Webhook 200 skipped | User not in `ALLOWED_USERS`; project/tags/prompts missing; MR action filtered (e.g. approval only) |
| Duplicate skipped | Expected for the same tag/flow within its TTL: MR `DEDUP_TTL_SECS` (default 24h), issue `ISSUE_DEDUP_TTL_SECS` (default 15m). Implement and verify dedupe independently. |

## Related

- Golden image: [admin-golden-image.md](admin-golden-image.md) (Terra example in §2)
- Example scripts and prompts: [examples/cl8y-dex-terraclassic/](examples/cl8y-dex-terraclassic/)
