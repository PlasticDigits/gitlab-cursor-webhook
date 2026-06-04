# gitlab-cursor-webhook

Filter GitLab merge request webhooks and forward only actionable events to [Cursor Automations](https://cursor.com/docs/automations), with user allowlist guards.

Moves the `action` / `oldrev` filter out of your automation prompt so spurious MR activity (approvals, title edits, etc.) never reaches Cursor.

## Routes

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Render health check — `{"status":"ok"}` |
| `POST` | `/webhook` | GitLab MR webhook ingress |

## Environment variables

| Variable | Required | Description |
|----------|----------|-------------|
| `PORT` | No (Render sets) | Listen port; default `8080`. Binds `0.0.0.0:PORT`. |
| `PROJECT_WEBHOOKS` | **Yes** | Comma-separated `path-or-id=url|crsr_token` pairs. Maps each GitLab project to its Cursor webhook URL and API key. Key by `path_with_namespace` (e.g. `plasticdigits/yieldomega`) or numeric GitLab project `id`. Token is the `crsr_…` value only — do **not** include `Bearer`. |
| `GITLAB_WEBHOOK_SECRET` | No | GitLab project webhook secret token. When set, requests must include matching `X-Gitlab-Token`. When unset, this check is skipped. |
| `ALLOWED_USERS` | **Yes** | Comma-separated GitLab usernames (e.g. `plasticdigits,brouie`). The service **refuses to start** if unset or empty. |
| `DEDUP_TTL_SECS` | No | How long to remember forwarded `project_id` + `iid` + `last_commit.id` triples (default `86400`). Bounds memory; GitLab duplicate deliveries are usually immediate. |
| `RUST_LOG` | No | e.g. `gitlab_cursor_webhook=info` |

Copy [`.env.example`](.env.example) to `.env` for local development. Never commit `.env`.

## Local development

```bash
cp .env.example .env
# Edit .env with PROJECT_WEBHOOKS (url + token per project)

cargo run
```

Health check:

```bash
curl -s "http://127.0.0.1:${PORT:-8080}/health"
# {"status":"ok"}
```

Sample webhook (should **skip** — approval action):

```bash
curl -s -X POST "http://127.0.0.1:${PORT:-8080}/webhook" \
  -H 'Content-Type: application/json' \
  -d @tests/fixtures/mr_approval.json
# {"status":"skipped"}
```

Sample webhook (should **forward** when the project is listed in `PROJECT_WEBHOOKS` — open action):

```bash
curl -s -X POST "http://127.0.0.1:${PORT:-8080}/webhook" \
  -H 'Content-Type: application/json' \
  -d @tests/fixtures/mr_open.json
```

With `GITLAB_WEBHOOK_SECRET` set locally, add:

```bash
  -H "X-Gitlab-Token: your-secret"
```

## GitLab webhook setup

1. In your GitLab project: **Settings → Webhooks**.
2. **URL:** `https://<your-render-service>.onrender.com/webhook`
3. **Name:** `Cursor MR automation (filtered)`
4. **Trigger:** Merge request events only. (Issue, note, and other hooks are acknowledged with `200` and skipped, but extra triggers add noise.)
5. **Secret token:** same value as `GITLAB_WEBHOOK_SECRET` on Render (recommended).
6. **Do not** add a custom `Authorization` header on the GitLab webhook — this service adds `Bearer` when calling Cursor.
7. Enable SSL verification.

Both GitLab projects use the same service URL (`https://<service>.onrender.com/webhook`). The service routes each MR to the correct Cursor automation based on `PROJECT_WEBHOOKS`.

Example `PROJECT_WEBHOOKS` for two projects:

```bash
PROJECT_WEBHOOKS=plasticdigits/yieldomega=https://cursor.com/webhooks/yieldomega-id|crsr_YIELDOMEGA_TOKEN,plasticdigits/cl8y-dex-terraclassic=https://cursor.com/webhooks/cl8y-id|crsr_CL8Y_TOKEN
```

Find each project's `path_with_namespace` under **Settings → General** in GitLab. You can also key by numeric project id: `12345=https://...`.

## Cursor setup

1. Create a Cursor Automation per GitLab project, each with an **Incoming webhook** trigger.
2. Copy each webhook URL and API key (`crsr_…`) into `PROJECT_WEBHOOKS` for that project's GitLab path.
3. In each automation prompt, use the forwarded JSON fields: `event_type`, `username`, `project_name`, `web_url`, `description`, `iid`, `merge_commit_sha`, `title`, `last_commit`.

### Forwarded payload shape

Only these fields are sent to Cursor:

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

## Filter behavior

The service responds `200` with `{"status":"skipped"}` (and does **not** call Cursor) when:

- `object_kind` is not `merge_request`
- `object_attributes.action` is not `open`, or `update` without a non-empty `oldrev`
- `user.username` is not in `ALLOWED_USERS`
- The GitLab project is not listed in `PROJECT_WEBHOOKS`
- The same MR `project_id` + `iid` + `last_commit.id` was already forwarded within `DEDUP_TTL_SECS`

Otherwise it forwards to Cursor. GitLab always receives `200` so delivery is not disabled when Cursor returns an error; the JSON body reports the outcome:

- `{"status":"forwarded","cursor_status":202}` — Cursor accepted the payload
- `{"status":"forward_failed","cursor_status":400}` — Cursor returned a client/server error (check automation URL, token, and payload)
- `{"status":"forward_failed","error":"failed to reach cursor webhook"}` — network or DNS failure reaching Cursor

Duplicate skips log `skipped_reason=duplicate`. A new push on the same MR has a different `last_commit.id` and is forwarded. Entries expire after the TTL so the in-memory cache does not grow without bound.

## Render deployment

1. Create a new **Web Service** named `gitlab-cursor-webhook`.
2. Connect this repository; enable auto-deploy on `main`.
3. **Environment:** Native Rust (or Docker if you prefer).
   - **Build command:** `cargo build --release`
   - **Start command:** `./target/release/gitlab-cursor-webhook`
4. **Health check path:** `/health`
5. Set environment variables in the Render dashboard (never commit secrets):
   - `PROJECT_WEBHOOKS` (e.g. `plasticdigits/yieldomega=https://...|crsr_...,plasticdigits/cl8y-dex-terraclassic=https://...|crsr_...`)
   - `ALLOWED_USERS` (e.g. `plasticdigits,brouie`)
   - `GITLAB_WEBHOOK_SECRET` (recommended)
   - `RUST_LOG=gitlab_cursor_webhook=info` (optional)
6. Public webhook URL for GitLab: `https://<service>.onrender.com/webhook`

## Security

- The Cursor `Bearer` token protects the Cursor webhook endpoint from arbitrary internet callers. It does **not** authenticate GitLab.
- The real trust boundary is **GitLab MR policy** plus `ALLOWED_USERS` (fork MRs are forwarded when they pass the action and user filters).
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
