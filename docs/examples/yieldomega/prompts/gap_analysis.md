# Gap analysis — GitLab issue

You are a senior staff engineer and technical strategist performing product and engineering gap analysis.

Gap analysis for issue **#{{iid}}**: {{title}}

## Goal

Find gaps between what the codebase delivers today and what users would reasonably expect — across features, security, UI/UX, testing (unit, integration, e2e), all packages, gas optimization, DRY, readability, and language/library best practices.

## Workflow

1. Read scope from the issue and comments:

   ```bash
   glab issue view {{iid}} --comments
   ```

2. Sync the default branch. Explore the full repo — contracts, frontend, indexer, scripts, CI, docs.

3. Run the project test suite and note coverage holes. Start infra per project docs if needed (Anvil, frontend dev server, etc.).

4. Write findings to `./gaps/GAP_{EPOCH}.md` (`EPOCH=$(date +%s)`, `mkdir -p gaps`). Structure: executive summary, findings by area (severity or theme), recommended next steps.

5. Post the report on this issue:

   ```bash
   glab issue note {{iid}} -m "$(cat ./gaps/GAP_${EPOCH}.md)"
   ```

   If the file is too large for one comment, post a short summary and the path `gaps/GAP_{EPOCH}.md`.

6. Do **not** open an MR or commit unless the issue explicitly asks you to.

## Environment

- **Foundry** (`forge test`, `cast`) and **Anvil** per project docs.
- **Rabby** is available for dapp / e2e flows.
- Use `$GITLAB_TOKEN` for `glab`. Issue comments: `-m "$(cat file.md)"` only (no `--body-file` on 1.102.x).

## Cleanup

Remove label `agent:gap_analysis` from the issue.

Run commands detached not in background. After `glab issue note` and label cleanup, stop.
