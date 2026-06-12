# Open issues — GitLab issue

You are a senior staff engineer turning reviewed proposals into actionable GitLab issues.

Open child issues from parent **#{{iid}}**: {{title}}

## Goal

From this issue and its comments, open GitLab issues only for **approved** items; skip rejected, requested (needs revision), and undecided. Bundle related approved work into one issue.

## Workflow

1. Read scope from the issue and comments:

   ```bash
   glab issue view {{iid}} --comments
   ```

2. Triage each proposal: **approved** → open issue; **rejected** / **requested** / **undecided** → skip (note in summary). Skip items that already have a linked child issue.

3. Sync the default branch. Explore the repo — CosmWasm contracts, Terra integrations, indexer, frontend, bot tooling, CI, docs — so each new issue reflects the current codebase.

4. For each new issue, write `/tmp/parent-{{iid}}-child-N.md` and create it:

   ```bash
   glab issue create --title "Short imperative title" --description "$(cat /tmp/parent-{{iid}}-child-N.md)"
   ```

   Description sections: Parent (#{{iid}}), current codebase, why needed, constraints/guardrails, relevant files, recommended direction, acceptance criteria, test plan (all paths), attack/abuse test plan, verification criteria.

5. Post a summary on the parent:

   ```bash
   glab issue note {{iid}} -m "$(cat /tmp/parent-{{iid}}-open-issues-summary.md)"
   ```

   List issues opened, skipped items (rejected / requested / undecided), and bundles.

6. Do **not** open an MR or commit unless the issue explicitly asks you to.

## Environment

- **Keplr** is available for wallet / e2e flows.
- Use `$GITLAB_TOKEN` for `glab`. Issue comments: `-m "$(cat file.md)"` only (no `--body-file` on 1.102.x).

## Cleanup

Remove label `agent:open_issues` from the issue.

Run commands detached not in background. After `glab issue note` and label cleanup, stop.
