# gitlab-cursor-webhook

Filter GitLab merge request webhooks and forward only actionable events to [Cursor Automations](https://cursor.com/docs/automations), with user and fork guards.

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
| `CURSOR_WEBHOOK_URL` | **Yes** | Full webhook URL from the Cursor Automations dashboard. |
| `CURSOR_TOKEN` | **Yes** | Cursor API key (`crsr_…` only — do **not** include `Bearer`). |
| `GITLAB_WEBHOOK_SECRET` | No | GitLab project webhook secret token. When set, requests must include matching `X-Gitlab-Token`. When unset, this check is skipped. |
| `ALLOWED_USERS` | **Yes** | Comma-separated GitLab usernames (e.g. `plasticdigits,brouie`). The service **refuses to start** if unset or empty. |
| `RUST_LOG` | No | e.g. `gitlab_cursor_webhook=info` |

Copy [`.env.example`](.env.example) to `.env` for local development. Never commit `.env`.

## Local development

```bash
cp .env.example .env
# Edit .env with your Cursor URL and token

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

Sample webhook (should **forward** when `CURSOR_WEBHOOK_URL` is valid — open action):

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
4. **Trigger:** Merge request events only.
5. **Secret token:** same value as `GITLAB_WEBHOOK_SECRET` on Render (recommended).
6. **Do not** add a custom `Authorization` header on the GitLab webhook — this service adds `Bearer` when calling Cursor.
7. Enable SSL verification.

## Cursor setup

1. Create a Cursor Automation with an **Incoming webhook** trigger.
2. Copy the webhook URL → set as `CURSOR_WEBHOOK_URL` on Render.
3. Copy the API key (`crsr_…`) → set as `CURSOR_TOKEN` on Render (without `Bearer`).
4. In the automation prompt, use the forwarded JSON fields: `event_type`, `username`, `project_name`, `web_url`, `description`, `iid`, `merge_commit_sha`, `title`, `last_commit`.

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
- Fork MR: `source_project_id != target_project_id` (when both are present)
- `user.username` is not in `ALLOWED_USERS`

Otherwise it forwards to Cursor and returns Cursor's HTTP status and body.

## Render deployment

1. Create a new **Web Service** named `gitlab-cursor-webhook`.
2. Connect this repository; enable auto-deploy on `main`.
3. **Environment:** Native Rust (or Docker if you prefer).
   - **Build command:** `cargo build --release`
   - **Start command:** `./target/release/gitlab-cursor-webhook`
4. **Health check path:** `/health`
5. Set environment variables in the Render dashboard (never commit secrets):
   - `CURSOR_WEBHOOK_URL`
   - `CURSOR_TOKEN`
   - `ALLOWED_USERS` (e.g. `plasticdigits,brouie`)
   - `GITLAB_WEBHOOK_SECRET` (recommended)
   - `RUST_LOG=gitlab_cursor_webhook=info` (optional)
6. Public webhook URL for GitLab: `https://<service>.onrender.com/webhook`

## Security

- The Cursor `Bearer` token protects the Cursor webhook endpoint from arbitrary internet callers. It does **not** authenticate GitLab.
- The real trust boundary is **GitLab MR policy**: private repository, no fork MRs (filtered), and `ALLOWED_USERS`.
- For public or fork-friendly projects, always set `GITLAB_WEBHOOK_SECRET`, keep the fork filter enabled, and maintain a strict allowlist.
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
