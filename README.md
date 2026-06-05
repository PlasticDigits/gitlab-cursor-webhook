# gitlab-cursor-webhook

Filter GitLab webhooks and forward only actionable events to [Cursor Automations](https://cursor.com/docs/automations), with user allowlist guards.

- **Merge requests** → security review automation (`PROJECT_WEBHOOKS_SECURITY`)
- **Issues** with `agent:verify` or `agent:implement` labels → separate verify/implement automations

Moves filtering out of your automation prompts so spurious activity (MR approvals, title edits, unrelated issue updates, etc.) never reaches Cursor.

## Routes

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Render health check — `{"status":"ok"}` |
| `POST` | `/webhook` | GitLab webhook ingress |

## Environment variables

| Variable | Required | Description |
|----------|----------|-------------|
| `PORT` | No (Render sets) | Listen port; default `8080`. Binds `0.0.0.0:PORT`. |
| `PROJECT_WEBHOOKS_SECURITY` | **Yes** | Comma-separated `path-or-id=url|crsr_token` pairs for MR security review. Key by `path_with_namespace` (e.g. `plasticdigits/yieldomega`) or numeric GitLab project `id`. Token is the `crsr_…` value only — do **not** include `Bearer`. |
| `PROJECT_WEBHOOKS_VERIFY` | No | Same format. Routes issues with label `agent:verify` to the verify automation. |
| `PROJECT_WEBHOOKS_IMPLEMENT` | No | Same format. Routes issues with label `agent:implement` to the implement automation. |
| `GITLAB_WEBHOOK_SECRET` | No | GitLab project webhook secret token. When set, requests must include matching `X-Gitlab-Token`. When unset, this check is skipped. |
| `ALLOWED_USERS` | **Yes** | Comma-separated GitLab usernames (e.g. `plasticdigits,brouie`). The service **refuses to start** if unset or empty. |
| `DEDUP_TTL_SECS` | No | How long to remember forwarded MR `project_id` + `iid` + `last_commit.id` triples (default `86400`). Bounds memory; GitLab duplicate deliveries are usually immediate. Issue webhooks are not deduplicated. |
| `RUST_LOG` | No | e.g. `gitlab_cursor_webhook=info` |

Copy [`.env.example`](.env.example) to `.env` for local development. Never commit `.env`.

## Local development

```bash
cp .env.example .env
# Edit .env with PROJECT_WEBHOOKS_* (url + token per project per agent)

cargo run
```

Health check:

```bash
curl -s "http://127.0.0.1:${PORT:-8080}/health"
# {"status":"ok"}
```

Sample MR webhook (should **skip** — approval action):

```bash
curl -s -X POST "http://127.0.0.1:${PORT:-8080}/webhook" \
  -H 'Content-Type: application/json' \
  -d @tests/fixtures/mr_approval.json
# {"status":"skipped"}
```

Sample MR webhook (should **forward** when the project is listed in `PROJECT_WEBHOOKS_SECURITY` — open action):

```bash
curl -s -X POST "http://127.0.0.1:${PORT:-8080}/webhook" \
  -H 'Content-Type: application/json' \
  -d @tests/fixtures/mr_open.json
```

Sample issue webhook (should **forward** to verify automation when `agent:verify` is present):

```bash
curl -s -X POST "http://127.0.0.1:${PORT:-8080}/webhook" \
  -H 'Content-Type: application/json' \
  -d @tests/fixtures/issue_open_with_verify_label.json
```

With `GITLAB_WEBHOOK_SECRET` set locally, add:

```bash
  -H "X-Gitlab-Token: your-secret"
```

## GitLab webhook setup

1. In your GitLab project: **Settings → Webhooks**.
2. **URL:** `https://<your-render-service>.onrender.com/webhook`
3. **Name:** `Cursor automation (filtered)`
4. **Trigger:** Merge request events **and** Issue events. (Note, push, and other hooks are acknowledged with `200` and skipped.)
5. **Secret token:** same value as `GITLAB_WEBHOOK_SECRET` on Render (recommended).
6. **Do not** add a custom `Authorization` header on the GitLab webhook — this service adds `Bearer` when calling Cursor.
7. Enable SSL verification.

All projects share the same service URL. The service routes each event to the correct Cursor automation based on event type, label, and the `PROJECT_WEBHOOKS_*` maps.

Example for two projects with three automations each:

```bash
PROJECT_WEBHOOKS_SECURITY=plasticdigits/yieldomega=https://cursor.com/webhooks/yieldomega-security|crsr_SEC_YIELDOMEGA,plasticdigits/cl8y-dex-terraclassic=https://cursor.com/webhooks/cl8y-security|crsr_SEC_CL8Y
PROJECT_WEBHOOKS_VERIFY=plasticdigits/yieldomega=https://cursor.com/webhooks/yieldomega-verify|crsr_VERIFY_YIELDOMEGA,plasticdigits/cl8y-dex-terraclassic=https://cursor.com/webhooks/cl8y-verify|crsr_VERIFY_CL8Y
PROJECT_WEBHOOKS_IMPLEMENT=plasticdigits/yieldomega=https://cursor.com/webhooks/yieldomega-implement|crsr_IMPL_YIELDOMEGA,plasticdigits/cl8y-dex-terraclassic=https://cursor.com/webhooks/cl8y-implement|crsr_IMPL_CL8Y
```

Find each project's `path_with_namespace` under **Settings → General** in GitLab. You can also key by numeric project id: `12345=https://...`.

## Cursor setup

Create three Cursor Automations per GitLab project (security, verify, implement), each with an **Incoming webhook** trigger. Copy each webhook URL and API key (`crsr_…`) into the matching `PROJECT_WEBHOOKS_*` entry for that project's GitLab path.

### Forwarded MR payload shape

```json
{
  "event_type": "open",
  "username": "plasticdigits",
  "project_name": "example-project",
  "web_url": "https://gitlab.example/.../merge_requests/7",
  "description": "...",
  "iid": 7,
  "merge_commit_sha": null,
  "title": "Add feature X",
  "last_commit": { "id": "...", "message": "...", "timestamp": "...", "url": "..." }
}
```

`event_type` is the MR action (`open` or `update` with new commits), not GitLab's top-level `"merge_request"`.

### Forwarded issue payload shape

```json
{
  "event_type": "open",
  "agent": "verify",
  "username": "plasticdigits",
  "project_name": "example-project",
  "web_url": "https://gitlab.example/.../issues/3",
  "description": "...",
  "iid": 3,
  "title": "Verify this feature",
  "labels": [{ "title": "agent:verify" }]
}
```

`agent` is `"verify"` or `"implement"`. `event_type` is `open` or `update`.

## Filter behavior

### Merge requests (`PROJECT_WEBHOOKS_SECURITY`)

The service responds `200` with `{"status":"skipped"}` (and does **not** call Cursor) when:

- `object_kind` is not `merge_request`
- `object_attributes.action` is not `open`, or `update` without a non-empty `oldrev`
- `user.username` is not in `ALLOWED_USERS`
- The GitLab project is not listed in `PROJECT_WEBHOOKS_SECURITY`
- The same MR `project_id` + `iid` + `last_commit.id` was already forwarded within `DEDUP_TTL_SECS`

### Issues (`PROJECT_WEBHOOKS_VERIFY` / `PROJECT_WEBHOOKS_IMPLEMENT`)

Forwards to Cursor when **all** of the following hold:

- `object_kind` is `issue`
- `user.username` is in `ALLOWED_USERS`
- The project is configured in the matching `PROJECT_WEBHOOKS_*` map
- **Either:**
  - `action` is `open` and the top-level `labels` array includes `agent:verify` or `agent:implement`, **or**
  - `action` is `update` and `changes.labels` shows the label was **just added** (present in `current`, absent in `previous`)

Skipped (no Cursor call) when:

- Issue opened or updated without a triggering label
- Label was already present (`previous` already contains it)
- Label was removed
- Only non-label fields changed (title, description, milestone, etc.)
- `action` is `close`, `reopen`, or other non-open/update actions

When both `agent:verify` and `agent:implement` trigger in the same event, the service forwards to **both** automations.

#### GitLab `changes.labels` quirk

GitLab has had bugs where `changes.labels.previous` was empty on later issue/MR updates even when labels had not changed in that event ([gitlab#28832](https://gitlab.com/gitlab-org/gitlab/-/issues/28832)). This filter assumes GitLab sends correct `previous`/`current` arrays for **work item (issue) events**. If you see false positives (label appears "newly added" when it was already there), check the raw webhook payload before changing filter logic.

### Response to GitLab

GitLab always receives `200` so delivery is not disabled when Cursor returns an error. The JSON body reports the outcome:

- `{"status":"forwarded","cursor_status":202}` — Cursor accepted the payload
- `{"status":"forward_failed","cursor_status":400}` — Cursor returned a client/server error (check automation URL, token, and payload)
- `{"status":"forward_failed","error":"failed to reach cursor webhook"}` — network or DNS failure reaching Cursor

Duplicate MR skips log `skipped_reason=duplicate`. A new push on the same MR has a different `last_commit.id` and is forwarded. Entries expire after the TTL so the in-memory cache does not grow without bound.

## Render deployment

1. Create a new **Web Service** named `gitlab-cursor-webhook`.
2. Connect this repository; enable auto-deploy on `main`.
3. **Environment:** Native Rust (or Docker if you prefer).
   - **Build command:** `cargo build --release`
   - **Start command:** `./target/release/gitlab-cursor-webhook`
4. **Health check path:** `/health`
5. Set environment variables in the Render dashboard (never commit secrets):
   - `PROJECT_WEBHOOKS_SECURITY`, `PROJECT_WEBHOOKS_VERIFY`, `PROJECT_WEBHOOKS_IMPLEMENT`
   - `ALLOWED_USERS` (e.g. `plasticdigits,brouie`)
   - `GITLAB_WEBHOOK_SECRET` (recommended)
   - `RUST_LOG=gitlab_cursor_webhook=info` (optional)
6. Public webhook URL for GitLab: `https://<service>.onrender.com/webhook`

## Security

- The Cursor `Bearer` token protects the Cursor webhook endpoint from arbitrary internet callers. It does **not** authenticate GitLab.
- The real trust boundary is **GitLab webhook policy** plus `ALLOWED_USERS` (fork MRs are forwarded when they pass the action and user filters).
- For public or fork-friendly projects, always set `GITLAB_WEBHOOK_SECRET` and maintain a strict allowlist.
- Do **not** put the Cursor Bearer token in GitLab webhook custom headers.

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

[gitleaks](https://github.com/gitleaks/gitleaks) is configured via [`.gitleaks.toml`](.gitleaks.toml). Run before pushing:

```bash
gitleaks detect --source . --verbose
```

## Out of scope (v1)

- GitLab `webhook-signature` HMAC verification
- Rate limiting
- GitHub support
- GitLab API membership checks

## License

AGPL-3.0-or-later — see [LICENSE](LICENSE).
