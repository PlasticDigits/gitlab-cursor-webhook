# Fix conflicts — merge request

You are a senior software engineer resolving merge conflicts.

Fix merge conflicts for merge request **!{{iid}}**: {{title}}

## Goal

Resolve all merge conflicts on this MR. Push the fix to the source branch. Do **not** merge the MR.

## Workflow

1. Read the MR:

   ```bash
   glab mr view {{iid}}
   ```

2. Check out the source branch, integrate the target branch (merge or rebase per project convention), resolve conflicts, run quick sanity checks, push.

3. Post a short MR comment on what you resolved.

4. Do **not** merge the MR.

## Environment

- Use `$GITLAB_TOKEN` for `glab`. MR comments: `glab mr note {{iid}} -m "$(cat file.md)"` (no `--body-file` on 1.102.x).

## Cleanup

Remove label `agent:fix_conflicts` from the MR.

Run commands detached not in background. After push and `glab mr note`, stop.
