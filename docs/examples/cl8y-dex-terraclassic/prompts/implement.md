# Implement — GitLab issue

Implement and verify GitLab issue **#{{iid}}**: {{title}}

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
   glab issue view {{iid}} -R plasticdigits/{{project_name}} --comments
   ```

   Treat them as acceptance criteria.

2. Implement changes plus tests that satisfy acceptance criteria. Keep code clean, modular, and maintainable.

3. **Max 5 iterations:** test, verify, and retest until the implementation is clean and meets acceptance criteria.

4. Ensure invariants are documented, documentation is updated, and cross-linked with the systems you are verifying, including any `./skills/` docs for third-party agent users.

5. **If there are code/doc changes** — open an MR (**not draft**) against the default branch. MR description must include:

   - Summary of changes (what/why, tied to #{{iid}})
   - Checklist: each acceptance item → command or manual step → PASS/FAIL/SKIP
   - Leave the issue **open**. If any criteria FAIL/SKIP, list blockers.
   - Verification checklist for third parties to verify your work
   - Ideas for follow-ups only if relevant (otherwise omit)
   - MR must **not** be a draft

6. **Else — no code/doc changes needed** (all acceptance criteria already satisfied):

   - Do **not** open an MR.
   - Comment on the issue with `glab issue note` (or `--body-file`):
     - What you verified and PASS/FAIL/SKIP
     - How you verified (commands/logs)
     - If **all** criteria pass: close the issue with an explanation; do not use an MR to close
     - If not all pass: leave the issue open; list blockers
     - Follow-ups only if relevant (otherwise omit)

## GitLab / glab

- Use `$GITLAB_TOKEN` from the environment for `glab`.
- Use the **plasticdigits** account for glab — not the Cursor account.

## Cleanup

When complete, remove any `agent:implement` or `agent:verify` labels from the issue.
