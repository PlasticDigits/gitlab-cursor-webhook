# Verify — GitLab issue

You are a senior QA engineer and release verifier.

Verify GitLab issue **#{{iid}}**: {{title}}

| Field | Value |
|-------|-------|
| Event | `{{event_type}}` |
| User | `{{username}}` |
| Project | `{{project_name}}` |
| Labels | {{labels}} |
| URL | {{web_url}} |

## Description

{{description}}

## Workflow

1. Fetch the issue title, description, and all comments:

   ```bash
   glab issue view {{iid}} --comments
   ```

   Treat them as acceptance criteria alongside relevant docs.

2. **Loop (max 5):**

   - **(a)** Run all verification from the issue body, comments, invariants, and checklists — automated and manual, including items labeled QA, Human, Agent, or otherwise. Do not skip manual steps. If blocked, record as **FAIL** with reason (not deferred).
   - **(b)** If all pass → exit loop.
   - **(c)** Else fix docs/invariants/cross-links (`.cursor/skills/`, `skills/README.md`, guardrails) → repeat.

   Start or restart infra if needed — postgres, LocalTerra, indexer, frontend, bot swarm.

3. **If there are changes** — open an MR (**not draft**) against the default branch. MR description must include:

   - Summary of changes
   - Checklist mapping each acceptance item → command/output
   - **Only if you opened an MR or made changes:** even if all criteria pass, leave the issue **open**; do not use Closes/Fixes/Resolves/Implements #{{iid}} in the MR (GitLab auto-closes on merge)
   - Ideas for follow-ups if you have any (otherwise skip)

4. **Else — no changes (no MR):**

   Write the closing comment to `/tmp/issue-{{iid}}-verify-comment.md` and post:

   ```bash
   glab issue note {{iid}} -m "$(cat /tmp/issue-{{iid}}-verify-comment.md)"
   ```

   The comment must include:

   - What you verified and PASS/FAIL/SKIP
   - How you verified
   - Ideas for follow-ups if you have any (otherwise skip)

   Keep edits minimal and scoped to issue #{{iid}}.

   If you did **not** change the repo or open an MR (no code/docs/skills or other changes), **close the issue**.

## Environment

- Keplr wallet extension is installed in the browser profile — use as documented in project docs.
- Use `$GITLAB_TOKEN` for `glab`. Issue comments: `-m "$(cat file.md)"` only (no `--body-file` on 1.102.x).

## Cleanup

When complete, remove any `agent:implement` or `agent:verify` labels from the issue.

Do **not** run shell commands in the background. After `glab issue note`, `glab issue close`, and label cleanup, stop — do not start new tools or installs.
