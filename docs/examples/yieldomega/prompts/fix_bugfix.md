# Fix bugfix — merge request

You are a senior software engineer fixing bugs raised in merge request review.

Fix review bugs for merge request **!{{iid}}**: {{title}}

## Goal

Fix bugs called out in MR notes and unresolved review threads. Push fixes to the source branch. **Resolve every thread** your changes address. Do **not** merge the MR.

## Workflow

1. Read the MR, comments, and open threads:

   ```bash
   glab mr view {{iid}} --comments
   glab mr note list {{iid}} --state unresolved
   ```

2. For each bug in top-level notes or unresolved threads, apply a minimal correct fix. Re-run relevant tests.

3. Push to the source branch.

4. Resolve each fixed thread (by note or discussion id):

   ```bash
   glab mr note {{iid}} --resolve <note-id>
   # or: glab mr note resolve {{iid}} <discussion-id>
   ```

5. Post a short MR comment mapping each item → fix (or why no change was needed). Do **not** merge the MR.

## Environment

- Use `$GITLAB_TOKEN` for `glab`. MR comments: `glab mr note {{iid}} -m "$(cat file.md)"` (no `--body-file` on 1.102.x).

## Cleanup

Remove label `agent:fix_bugfix` from the MR.

Run commands detached not in background. After push, thread resolution, and `glab mr note`, stop.
