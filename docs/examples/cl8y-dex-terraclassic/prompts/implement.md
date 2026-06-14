# Implement — GitLab issue

You are a senior software engineer who ships production-ready features end to end.

Implement and verify GitLab issue **#{{iid}}**: {{title}}

## Workflow

1. Fetch the issue title, description, and all comments:

   ```bash
   glab issue view {{iid}} --comments
   ```

   Treat them as acceptance criteria.

2. Implement changes plus tests that satisfy acceptance criteria. Keep code clean, modular, and maintainable.

3. **Max 5 iterations:** test, verify, and retest until the implementation is clean and meets acceptance criteria. Start or restart infra if needed — postgres, LocalTerra, indexer, frontend (per project docs).

4. Ensure invariants are documented, documentation is updated, and cross-linked with the systems you are verifying, including any `./skills/` docs for third-party agent users.

5. **If there are code/doc changes** — Pull latest for your MR & Fix conflicts for your MR, then open an MR (**not draft**) against the default branch. MR description must include:

   - Summary of changes (what/why, tied to #{{iid}})
   - Checklist: each acceptance item → command or manual step → PASS/FAIL/SKIP
   - Leave the issue **open**; do not use Closes/Fixes/Resolves/Implements #{{iid}} in the MR (GitLab auto-closes on merge). If any criteria FAIL/SKIP, list blockers.
   - Verification checklist for third parties to verify your work
   - Ideas for follow-ups only if relevant (otherwise omit)
   - MR must **not** be a draft

6. **Else — no code/doc changes needed** (all acceptance criteria already satisfied):

   - Do **not** open an MR.
   - Write the comment to `/tmp/issue-{{iid}}-implement-comment.md` and post:

     ```bash
     glab issue note {{iid}} -m "$(cat /tmp/issue-{{iid}}-implement-comment.md)"
     ```

     The comment must include:
     - What you verified and PASS/FAIL/SKIP
     - How you verified (commands/logs)
     - If **all** criteria pass: close the issue with an explanation; do not use an MR to close
     - If not all pass: leave the issue open; list blockers
     - Follow-ups only if relevant (otherwise omit)

## Environment

- Keplr wallet extension is in the browser profile — use per project docs.
- Use `$GITLAB_TOKEN` for `glab`. Issue comments: `-m "$(cat file.md)"` only (no `--body-file` on 1.102.x).

## Cleanup

When complete, remove any `agent:implement` or `agent:verify` labels from the issue.

Run commands detached not in background. After `glab issue note`, `glab issue close` (if applicable), and label cleanup, stop.
