# ADR 0001: Remove catch-all CODEOWNERS

## Status

Proposed ([#3](https://git.cl8y.com/code/gitlab-cursor-webhook/issues/3)). Not
accepted by this design-author pass. Do not call this tip accepted until
independent review says so. Keywords in the issue body are not architecture
approval. There is no separate standing issue `#3`; the issues URL and the
product PR share one number.

Copy **the independently accepted SHA** (this SHA or a successor) onto the
product tip. `cac-design-issue-3` is never the merge vehicle. Land criterion:
the two `docs/` files on the product tip are byte-identical to that SHA.

Overview (merge gate, runtime, six-row **target**, one Woodpecker land rule):
[`architecture.md`](../architecture.md). Do not copy that table here. **G3**
there is three groups: protection GET **target** (six flags), merge procedure
(**G3-4**), tree contracts (**G3-1**, **G3-6**, **G3-7**).

Sister CAC autoland work is
[#429](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/429).
CAC [#388](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/388)
historically forbade deleting CODEOWNERS;
[cl8y-forgejo#48](https://git.cl8y.com/PlasticDigits/cl8y-forgejo/issues/48)
reversed that. Do not revive the skip. Deploy / spend / custody / policy
expansion:
[agent-control #297](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/297)
— **out of scope**.

Sibling product PRs with the same chore title (hello canary
[#15](https://git.cl8y.com/code/hello/pulls/15), CL8Y-web, DEX, …) are **other
repos**. Do not edit them from this worktree. hello#15 is not a GCH iid.

## Outcome

Delete the catch-all `CODEOWNERS` so Forgejo does not plant **official** review
requests on every change. Merge to `main` stays: pull request, host-required
Woodpecker context `ci/woodpecker/pr/woodpecker` when **observed G3-8** holds
(Decision 7 / architecture mermaid), SHA-pinned `Do: merge`, no direct push,
no `force_merge`.

**Land vehicle B:** product PR
[#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) (or a successor)
ships **S0+S1+S2 on one tip** — copy the two design files from **the
independently accepted SHA** (this SHA or a successor), delete `CODEOWNERS`,
add a short README pointer. Merging that PR closes `#3`. Design branch
`cac-design-issue-3` is review/transport only; it is **not** merged as a
docs-only PR and is **not** the product PR.

**Leftover issue (land gate).** Before merge of `#3`, S2 opens one leftover
**Forgejo issue, not PR** in **`code/gitlab-cursor-webhook` only** (title,
owners, and body template under Migration). That issue owns S3. Closing `#3`
does not assign S3. Land criterion 5 fails if the issue is missing, empty,
opened in another repo, or lacks named S3 owners.

**Land vs leftover-complete.** Land proves **G3-9** / **G3-10** via **repo
admin** attest (not S2 GET). Leftover-complete (S3) proves the other four
observed flags (**G3-2**, **G3-8**, **G3-3**, **G3-5**) vs the architecture
**target**, plus dedicated post-merge plant-check PR `{n}`, plus four-path
absence (**G3-1 last**). S3 is not a close gate for `#3`.

This repo is a product tree, not the forge #48 canary (`code/hello`). #3 is the
file delete plus in-repo docs/README pointer. It does not re-roll protection,
does not implement CAC autoland, and does not deploy Coolify.

## Context

`72133f5` added root `CODEOWNERS`:

```
# Request review from trusted maintainers on every change.
# Forgejo CODEOWNERS uses Go regular expressions, not GitHub glob syntax.
# Place this file at the repository root, or in docs/ or .forgejo/.
# Product repos live in org `code` (git.cl8y.com/code/{repo}).

.* @code/maintainers
```

Forgejo uses Go regular expressions, not GitHub globs. Combined with historical
`block_on_official_review_requests`, every PR requested team **maintainers** in
org **code**. That team’s only member is the usual PR author, so self-approve
is 422 and merge is 405. CAC `RECOMMEND: ACCEPT` is not a Forgejo `APPROVED`
review.
[cl8y-agent-control#388](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/388)
skipped the deadlock; it did not remove the file or the protection.

Live proof that the file still plants requests: PR
[#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) itself has
`requested_reviewers_teams` containing team `name: "maintainers"` (org `code`,
`id: 4`) and an `official: true` `REQUEST_REVIEW` whose `team.name` is
`"maintainers"` (review `id: 209`). Forgejo `Team.name` is `"maintainers"`,
not `"code/maintainers"`. On the reviews GET, `team.organization` is `null`.
Renovate [#2](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/2) has the
same planted team request. Land criterion 6 **always** applies.

Fleet protection under #48 is described as the architecture six-row
**target**, including `block_on_official_review_requests=false` (**G3-10**)
and `required_approvals=0` (**G3-9**). This design has **no** dated protection
JSON for `code/gitlab-cursor-webhook` (unauthenticated GET is 401). Do **not**
paste a dated JSON from an unauthenticated session. Do **not** treat the
leftover request on `#3` as non-blocking from fleet values. It is
non-blocking **only after** repo admin POSTs dated JSON of **this** repo’s
`main` rule showing **G3-9** and **G3-10** on the leftover issue **and** on
`#3`. If **G3-10** is still `true` here, or that JSON is missing, merge is
405; do not land `#3`. That leftover request also must not be treated as S3
evidence.

Draft implementation (not this design commit):
`022f4f510113c6e8fcfd973a0869753a6fb375be` on `chore/remove-catchall-codeowners`
deletes the six-line file and does not touch README or copy `docs/`. Incomplete
without S0 files and S2 on that PR (or a successor). That commit has empty
commit statuses (`[]`).

`origin/main` already has `docs/` runbooks (`admin-golden-image.md`,
`docker-deploy.md`, examples) and **no** `docs/architecture.md` or
`docs/adr/`. Relative README links to ADR 0001 / architecture 404 unless those
two files land on the **same merged tip** as the README pointer. Do not rewrite
those runbooks.

Issue body points at forgejo `docs/INVARIANTS.md`. Treat #48/#50 as source.
Do not fork INVARIANTS into this repo.

Open PRs in this repo when #3 was filed:
[#2](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/2) (Renovate
onboarding) and #3. Renovate benefits from fewer planted reviews; it is not a
sequencing dep. Closed [#1](https://git.cl8y.com/code/gitlab-cursor-webhook/issues/1)
is unrelated.

This controller provisions Hetzner VMs. This Coolify app **may** rebuild if
the existing host is git-follow on `main` (not verified in-tree:
`docker-compose.yml` and `docs/docker-deploy.md` only record Dockerfile deploy
plus `/var/lib/gch`). Landing #3 is still not a #297 deploy grant: the
product-PR diff must not change image, compose, Terraform, tokens, or
auto-deploy.

## Non-goals

- Forgejo protection JSON / `apply_repo_policy.py` / migrate `_ensure_codeowners`
  / templates ([cl8y-forgejo#48](https://git.cl8y.com/PlasticDigits/cl8y-forgejo/issues/48)
  / [pulls/50](https://git.cl8y.com/PlasticDigits/cl8y-forgejo/pulls/50)).
  Sister-repo `_ensure_codeowners` stays out of this slice; leftover-complete
  still fails if a later apply has put the file back (Failure modes).
- PATCHing branch protection from this tree, including force-push allowlist
  fields and `apply_to_admins` (forge #48). Observed-vs-target drift is a
  #48 leftover, not a GCH PATCH.
- CAC autoland predicates, occupying jobs, or `DrainSkip::OfficialReview`
  cleanup ([cl8y-agent-control#429](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/429),
  leftover of #388). Drain comments such as `drain skip: no occupying job…` on
  #3 are **#429**, not a #3 failure.
- Dismissing reviewers from the controller (forbidden substitute in #388).
- Path-specific CODEOWNERS, a second maintainer, or `required_approvals: 1`.
- Adding, enabling, or digest-pinning Woodpecker **in the #3 diff**. Missing
  `ci/woodpecker/pr/woodpecker` statuses are pre-existing. Do not add
  `.woodpecker.yaml` / `.woodpecker/` in the #3 diff. If observed **G3-8**,
  enablement is a **named local issue** (Decision 7), not this diff.
- Deleting or rewriting [`.gitlab-ci.yml`](../../.gitlab-ci.yml) (GitLab
  leftover; not the Forgejo merge context).
- Changing Rust sources, `cargo` manifests, Terraform, `Dockerfile`,
  `docker-compose.yml`, entrypoint, `.env.example`, webhook HMAC, job tokens,
  `HCLOUD_TOKEN`, `CURSOR_API_KEY`, golden-image runbooks, or example prompts.
- Coolify deploy, UUID/token changes, flipping auto-deploy, or treating a
  `main` image rebuild as leftover-complete
  ([agent-control #297](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/297)).
- Weakening `cargo test` / `cargo clippy` / `.gitleaks.toml`, adding
  `force_merge`, enabling direct `main`, or posting fake commit statuses.
- Renovate onboarding ([#2](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/2)).
  Do not close or retarget it.
- Waiting on [hello#15](https://git.cl8y.com/code/hello/pulls/15) or other
  sibling file-delete PRs. Canary is informational, not a GCH dep.
- Writing `PlasticDigits/gitlab-cursor-webhook` (archival; CAC invariant 21).
- Editing `autonomy.rs` / HMAC, self-approval, or a founder card for this
  ordinary design.
- A docs-only PR from `cac-design-issue-3` (vehicle **A**). That branch
  transports design between VMs; it is not a merge vehicle.

## Decision

1. **Delete** root `CODEOWNERS`. Do not leave an empty or comments-only file
   (Forgejo still parses it).
2. **Do not add** `docs/CODEOWNERS`, `.gitea/CODEOWNERS`, or
   `.forgejo/CODEOWNERS`. After land, `test -f` fails on all four paths. None of
   those paths may contain a reviewer rule for any pattern (not only `.*`).
3. **Keep** the merge gate in [`architecture.md`](../architecture.md) **G3**.
   On the product PR (or successor), copy
   `docs/adr/0001-remove-catchall-codeowners.md` and `docs/architecture.md`
   from **the independently accepted SHA** (this SHA or a successor; not this
   tip until independent review) onto that **same tip**, then add a short
   README pointer. README remains the product overview (not a stub). Point at
   `docs/architecture.md` **only** for the merge gate (**G3**), and at this ADR
   for the delete decision. Do not imply CODEOWNERS is what makes merge
   trusted. Do not merge a README that points at those paths until they exist
   on that tip. The two `docs/` files on the product tip must be
   byte-identical to that SHA. Do not rewrite runbooks.
4. **Leave** already-planted official requests on open PRs (including #3 and
   #2). Treat them as non-blocking **only after** repo admin POSTs dated JSON
   of `code/gitlab-cursor-webhook` `main` showing **G3-9** and **G3-10** on
   the leftover issue **and** on `#3`. Do not dismiss them from CAC. Human
   dismiss is optional leftover, not AC. If **G3-10** is `true` or the GET
   is missing, stop. Do not `force_merge`.
5. **Do not** PATCH branch protection from this repository.
6. **Split land from leftover-complete.** Product PR `#3` (or successor) is
   S0+S1+S2 (vehicle **B**). S3 lives on the leftover **Forgejo issue, not
   PR** S2 opens in this repo before that merge (template below), with named
   owners recorded before merge. Closing `#3` does not assign S3. Require a
   **dedicated** post-merge plant-check PR. Do not accept `#3`’s own official
   request, `#2`, or “the next natural PR.”
7. **One Woodpecker land rule** (observed GET, not the target row). Full text:
   [`architecture.md`](../architecture.md) “One Woodpecker land rule”. Repo
   admin dated JSON of this repo’s `main` rule (S2 does not GET):
   - If **G3-8** is `true`: S2 opens a named Forgejo **issue** (not a PR) in
     `code/gitlab-cursor-webhook` whose only job is enable/post
     `ci/woodpecker/pr/woodpecker`; record that iid on `#3`; it **is** a
     local DEPS; do not add `.woodpecker.yaml` in the `#3` diff; do not merge
     `#3` until that context is green on the product tip. Do not fake
     statuses. Do not `force_merge`.
   - If **G3-8** is `false` or status checks unset: land of `#3` does **not**
     wait on Woodpecker. Leftover-complete records observed flags vs the
     six-row target. Do not PATCH protection from this tree.

## Actors

Split so S2 cannot skip the land GET.

- **S2 (implement):** copy S0 files, delete `CODEOWNERS`, README pointer;
  open the leftover Forgejo issue (not PR) with named owners and pasted
  protection endpoints; if admin JSON shows **G3-8**, open the named
  Woodpecker issue and record that iid as local DEPS; **do not GET**
  protection; **do not merge**.
- **Repo admin of `code/gitlab-cursor-webhook`:** POST dated JSON of the
  `main` rule for **G3-9** and **G3-10** (same JSON answers observed
  **G3-8**) on the leftover issue **and** on `#3`. Unauthenticated GET is
  401; in-repo CI cannot do this.
- **Implementer** may treat leftover official requests as non-blocking
  **only after that admin comment**. If **G3-10** is `true` or the GET is
  missing, stop. Do not `force_merge`.
- **S3 owners** (named on the leftover issue **before** merge of `#3`;
  closing `#3` does not assign S3):
  1. **Repo admin** of `code/gitlab-cursor-webhook` — protection GET JSON.
  2. **Named S3 implementer** (Forgejo login filled on the leftover issue
     during S2; default role: the GCH implementer who opened that leftover
     issue) — plant-check `{n}` + **G3-1 last**.

## Component / state / interface changes

| Surface | Change |
| --- | --- |
| `CODEOWNERS` (root) | Remove file. |
| `docs/CODEOWNERS`, `.gitea/CODEOWNERS`, `.forgejo/CODEOWNERS` | Must remain absent (no empty file). |
| `docs/adr/0001-remove-catchall-codeowners.md`, `docs/architecture.md` | Copy from the independently accepted SHA onto the product PR so the merged tip is S0+S1+S2 (byte-identical). Standing G3 contract lands with the delete. |
| Forgejo PR review interface | After land, a **dedicated** plant-check PR against `main` must not get an official CODEOWNERS team request. |
| Branch protection API | No write from this ticket. The six-row table is the #48 **target**, not a measured GET. S2 does not GET. Repo admin POSTs dated JSON: land proves **G3-9** / **G3-10**; leftover-complete proves the other four vs target plus plant-check. |
| `.woodpecker.yaml` / `.woodpecker/` | Must remain absent in the #3 diff. If observed **G3-8**, enablement is a named local issue (Decision 7), not this diff. |
| `.gitlab-ci.yml` | Unchanged. |
| Rust crates, Terraform, Docker, `.env.example`, gitleaks | Unchanged. |
| README | Mandatory on the product PR: merge gate is **G3**, documented in `docs/architecture.md`; product map stays README. Relative links to ADR 0001 / architecture, which exist on that same tip. Keep the product overview. |
| Runbooks under `docs/` | Unchanged except adding `adr/` + `architecture.md`. Plant-check may add a throwaway non-runbook path under `docs/` (Tests item 4). |
| Leftover issue | New **Forgejo issue, not PR** in `code/gitlab-cursor-webhook` only, opened before merge of `#3`, named S3 owners, body quotes leftover-complete items 1–3. |
| Optional Woodpecker issue | Opened by S2 only if observed **G3-8**; local DEPS for land; not the leftover S3 issue. |
| CAC / Coolify / org team `maintainers` in org `code` | Unchanged. The team may keep existing; it simply is not planted as official review. Coolify **may** rebuild if the host is git-follow on `main` (not verified in-tree); that is not a new deploy grant. |

No runtime state, schema, or HTTP API.

## Affected invariants

IDs live in [`architecture.md`](../architecture.md). This ADR changes **G3-1**
(four-path absence). It does not write protection JSON. The six-row table is
the #48 **target**. Land proves **G3-9** / **G3-10** via repo-admin attest
(not S2 GET). Leftover-complete proves the other four observed flags
(**G3-2**, **G3-8**, **G3-3**, **G3-5**) vs that target, plus plant-check,
plus **G3-1 last**. Merge procedure remains **G3-4**. Coolify **may** rebuild
if git-follow on `main` (**G3-6**, not verified in-tree). CAC policy remains
**G3-7**. **G3-2** is `enable_push == false` (no direct push); it does not
claim force-push policy.

Product filter/provision invariants in README (HMAC fail-closed,
`ALLOWED_USERS`, isolated Terraform state, no PR Coolify secrets) are
**unchanged**.

## Alternatives

| Option | Why not |
| --- | --- |
| Keep file, rely on `block_on_official_review_requests=false` | Requests still plant on every PR; drain noise; Renovate/agent PRs look like they need a human stamp; templates can re-teach the old gate. |
| Replace `.*` with path owners | No second reviewer exists; same 405/422 if official-review is ever turned on; out of scope. |
| Add a second maintainer | Founder ops, not this implement. |
| Dismiss official requests from CAC | Forbidden by #388 as a substitute for policy reversal. |
| Direct-push the delete to `main` | Violates **G3-2**. File deletes go through a PR (already [#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3)). |
| Empty or comments-only CODEOWNERS | Forgejo still parses it. Absence is the contract. |
| Add `.woodpecker.yaml` in the same PR | Different change (CI enablement). Missing statuses are pre-existing. If observed **G3-8**, that is a named local issue, not this diff. |
| `force_merge` or fake Woodpecker statuses to land #3 | Forbidden by **G3-4** / **G3-3**. |
| Wait on Woodpecker with no ticket, or skip Woodpecker while observed **G3-8** | Decision 7 is one rule: named local DEPS and green context if **G3-8**; no wait if **G3-8** false / unset. |
| Treat `#3`’s plant, `#2`, or the next natural PR as S3 | Merge closes `#3` before leftover-complete. A dedicated post-merge PR is the evidence. |
| Vehicle **A**: docs-only PR from `cac-design-issue-3` onto `main`, then `#3` as S1+S2 | Second merge vehicle. That branch is design transport, not a product PR. Vehicle **B** puts the two files on the deletion PR so README links resolve on one tip. Draft `022f4f5` is not that tip until S0 files and S2 are added. |
| Wait on sibling `code/*` CODEOWNERS PRs / hello#15 | Wrong repo; no product iid dependency. |
| Open the leftover issue in `PlasticDigits/*` or leave it empty / unnamed | Land criterion 5 would pass a wrong-repo or vacant issue. S2 names this repo, owners, and quotes leftover-complete items 1–3. |
| Treat Coolify rebuild after merge as leftover-complete | **G3-6**. Plant-check and observed-vs-target GET are leftover-complete. A rebuild is not a #297 grant. |
| Merge plant-check `{n}` and call leftover-complete a pass | Leftover-complete **fails** (possible host rebuild / #297 incident). Close without merge in the same session. |
| Let S2 GET protection or skip admin attest | S2 cannot GET (401). Skipping land-proves **G3-9** / **G3-10** can merge into 405. |
| Write the archival `PlasticDigits/gitlab-cursor-webhook` clone | CAC invariant 21. Wrong repo. |
| Require dated protection JSON in S0 | Freezes design behind a token this pass does not have. That GET is leftover/admin work. |

## Complexity added / removed

**Removed:** catch-all official-review robot on every diff; operator dismiss
step; false “CODEOWNERS is the trusted-PR gate” story in this repo.

**Added:** a small standing doc (this ADR + architecture **G3**) that lands on
`main` via the product PR so later agents do not re-add `.* @code/maintainers`
as a merge requirement, and a leftover Forgejo issue in this repo that
survives merge of `#3`. No new services, jobs, flags, pipelines, or test
harnesses. Optional named Woodpecker issue only if observed **G3-8**.

## Migration

1. Fleet protection is owned by forge #48. This ticket does not PATCH. Do not
   treat a GET recorded on `code/hello` as proof for this repo. The six-row
   table is the **target**; leftover-complete attests **observed** JSON.
2. **Leftover issue (S2, land gate).** Before merging the product PR, open
   **one** follow-up **Forgejo issue, not PR** in **`code/gitlab-cursor-webhook`
   only**. Do not open it in `PlasticDigits/cl8y-forgejo`,
   `PlasticDigits/cl8y-agent-control`,
   `PlasticDigits/gitlab-cursor-webhook`, or any other repo. Title **must**
   include `Forgejo issue, not PR`. Suggested title:
   `chore: leftover CODEOWNERS S3 (Forgejo issue, not PR)`. Body **must**
   name S3 owners (below) and **quote** leftover-complete items 1–3 from this
   ADR. Record that issue’s iid on the product PR before merge. Merging
   [#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) closes that
   number; closing `#3` does not assign S3.

   Body template (quote onto the leftover issue):

   ```
   Leftover-complete for ADR 0001 after merge of product PR #3.
   This is a Forgejo issue, not a PR.
   This issue does not close #3. Closing #3 does not assign S3.
   Opened in code/gitlab-cursor-webhook before that merge.

   Owners (required before merge of #3):
   1. Protection GET JSON: repo admin of code/gitlab-cursor-webhook
   2. Plant-check {n} + G3-1 last: <Forgejo login; fill during S2; default
      role: GCH implementer who opened this leftover issue>

   S2 (implement) pasted these endpoints; S2 does not GET; S2 does not merge:
   - GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections
   - GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections/main

   Repo admin POSTs dated JSON for G3-9 and G3-10 (same JSON answers observed
   G3-8 for the Woodpecker land rule) on this issue AND on #3 before merge.

   1. Dated operator GET of this repo's `main` protection rule: attest
      observed JSON here. Not copied from code/hello. Compare to the six-row
      target in docs/architecture.md. Leftover-complete proves observed
      G3-2, G3-8, G3-3, G3-5 vs that target. Drift is a #48 leftover, not a
      GCH PATCH, not a reason to restore CODEOWNERS, and not a
      leftover-complete fail. A GET recorded before a later template re-copy
      does not count.

   2. After the delete is on `main`: dedicated plant-check PR {n} (not draft,
      title not WIP, throwaway path under docs/ that is not a runbook,
      example docs/_plant-check-adr0001.md; no manual reviewer request).
      Forbid .env.example, Terraform, and Docker as the probe. On open,
      comment `do-not-merge`. GET immediately after open; if both plant
      signals empty, wait 30s and re-GET. Pass iff requested_reviewers_teams
      length 0 AND no review with official == true, state == "REQUEST_REVIEW",
      team.name == "maintainers" (optional team.id == 4). Do not require
      team.organization on reviews. Record {n} and the two JSON bodies, then
      close without merge in the same session after the 30s re-GET.
      If {n} is merged, leftover-complete fails (possible host rebuild /
      #297 incident). It is not a pass with a Coolify rebuild.
      Not #3, not #2, not the next natural PR.

   3. G3-1 last: test -f fails on CODEOWNERS, docs/CODEOWNERS,
      .gitea/CODEOWNERS, .forgejo/CODEOWNERS. Recorded after items 1 and 2.
      Owner: named S3 implementer (owner 2).
   ```

3. **Vehicle B.** Land
   [#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) (or a
   successor) whose tip is S0+S1+S2: copy
   `docs/adr/0001-remove-catchall-codeowners.md` and `docs/architecture.md`
   from **the independently accepted SHA** (this SHA or a successor; not this
   tip until independent review), delete root `CODEOWNERS`, add the README
   pointer with relative links to those two paths. `cac-design-issue-3` stays
   the review/transport branch; do not open it as the product PR; do not merge
   it as docs-only first. Do not merge a README that points at those paths
   until they exist on that tip. The two `docs/` files on the product tip must
   be byte-identical to that SHA. Draft `022f4f5` is not that tip until S0
   files and S2 are added.
4. Open PRs created while the file existed (#3, #2) still show an official
   team request. Non-blocking **only after** the admin comment in Actors.
   No bulk dismiss required to land `#3` under that attest. If **G3-10** is
   `true` or the GET is missing, stop; merge is 405.
5. Do not restore the file from `docs/templates/CODEOWNERS` in cl8y-forgejo;
   that template is owned by #48.
6. Woodpecker: Decision 7 only. If observed **G3-8**, S2 opens the named CI
   issue (local DEPS) and land waits for green context. If **G3-8** false /
   unset, land does not wait. Do not restore catch-all CODEOWNERS. Do not
   `force_merge`. Do not add `.woodpecker.yaml` in the #3 diff.

## Observability

Relative reads. Do not log tokens, hosts, or protection-script inventories. Do
not add a Forgejo admin token to Woodpecker, cargo tests, or Coolify.

**Land GET (G3-9 / G3-10, admin attest).** Always applies. Before merging
`#3`, **repo admin** of `code/gitlab-cursor-webhook` POSTs dated JSON of the
`main` rule showing **G3-9** (`required_approvals == 0`) and **G3-10**
(`block_on_official_review_requests == false`) on the leftover issue **and**
on `#3`. Same endpoints as leftover-complete. S2 pastes the endpoints and
does not GET. Fleet #48 / `code/hello` is not this GET. If **G3-10** is
`true` or the JSON is missing, do not land. Do not `force_merge`. The same
JSON answers observed **G3-8** for Decision 7.

**Protection (operator, leftover-complete).** Repo admin of
`code/gitlab-cursor-webhook` attests dated **observed** JSON of the `main`
rule onto the leftover issue in this repo.
`GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections` (array; pick
`rule_name == "main"`) or
`GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections/main`.
Leftover-complete **proves** observed **G3-2**, **G3-8**, **G3-3**, and
**G3-5** compared to the architecture **target**. Drift vs target is a #48
leftover, not a GCH PATCH, not a leftover-complete fail, not a reason to
restore `CODEOWNERS`. Do not compare the Merge API / **G3-4** row (not a
protection field). Do not treat `enable_push == false` as a force-push read.
A green `cargo test`, empty commit statuses, or a Coolify deploy does not
satisfy this read. In-repo CI cannot perform this GET. S2 cannot perform this
GET.

**Plant-check (dedicated post-merge PR, leftover-complete).** Recipe is Tests
item 4. Fail-closed pair (jq on the two GETs):

- Pass iff `(requested_reviewers_teams // []) | length == 0`
- **and** no review with `official == true && state == "REQUEST_REVIEW" && team.name == "maintainers"`
  (optional extra pin: `team.id == 4`). Do **not** require `team.organization`
  on the reviews GET. Do **not** match `team.name == "code/maintainers"` (that
  string is the CODEOWNERS target, not the live JSON).

`official` alone means assigned/write-access, not “planted by CODEOWNERS”; the
conjunction is the plant signal. `REQUEST_REVIEW` is `state`, not a sibling
key.

Known-plant sample: PR
[#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) (must **fail**
this predicate; not S3 evidence).

`GET /api/v1/repos/code/gitlab-cursor-webhook/pulls/3` fragment:

```json
{
  "draft": false,
  "requested_reviewers_teams": [
    {
      "id": 4,
      "name": "maintainers",
      "organization": { "id": 4, "name": "code" }
    }
  ]
}
```

`GET /api/v1/repos/code/gitlab-cursor-webhook/pulls/3/reviews` fragment:

```json
[
  {
    "id": 209,
    "official": true,
    "state": "REQUEST_REVIEW",
    "team": { "id": 4, "name": "maintainers", "organization": null }
  }
]
```

On `#3`, `requested_reviewers_teams[0].name == "maintainers"` and
`.organization.name == "code"`. On **reviews**, `team.name == "maintainers"`,
`team.id == 4`, and `team.organization == null`. An operator who checks
`team.name == "code/maintainers"` misses the plant. An operator who requires
`team.organization.name == "code"` on reviews also misses it.

`{n}` is the dedicated plant-check PR opened **after** the delete is on
`main` by the **named S3 implementer**. Probe a throwaway path under `docs/`
that is not a runbook (example: `docs/_plant-check-adr0001.md`). Forbid
`.env.example`, Terraform, and Docker as the probe. PR GET must have
`draft == false`. Title must not contain `WIP` (case-insensitive). On open,
comment `do-not-merge`. GET both endpoints **immediately after open**. If
either plant signal is present, fail. If both signals are empty, wait once
**30 seconds** and re-GET both before pass; pass only if the second pair is
still empty. Record `{n}` **and** the two JSON bodies (the pair used for the
pass decision) on the leftover issue, then **close without merge in the same
session** after the 30s re-GET. If `{n}` is merged, leftover-complete
**fails** (possible host rebuild / #297 incident). It is not a pass with a
Coolify rebuild. PR `#3`’s own official request does not pass. `#2` does not
pass. “The next natural PR” does not pass.

**CI / deploy.** Required host context follows Decision 7 / the architecture
mermaid (observed **G3-8**, not the target row). This tree does not post
Woodpecker today. Drain comments such as `drain skip: no occupying job…` are
**#429**, not a #3 failure. Coolify **may** rebuild if the host is git-follow
on `main` (**G3-6**, not verified in-tree); that is not leftover-complete.

## Failure modes

| Mode | Handling |
| --- | --- |
| File deleted on a branch but still on `main` | New PRs keep planting official review until the product PR merges. Expected until land. |
| README links ADR/architecture but those files are not on the same tip | 404 after merge. Land fails criterion 2. Vehicle **B**: copy both files onto the product PR before merging README. |
| Copy left in `docs/`, `.gitea/`, or `.forgejo/` (including empty/comments-only) | Forgejo still loads the first existing path and may plant. Land fails **G3-1**; delete those paths too (none exist on current `main`). |
| Whole-tree `git grep` for `.* @` | Hits this ADR after a correct delete. Use the pathspec in Tests item 1. |
| cl8y-forgejo migrate/apply re-copies a template | Sister-repo race ([cl8y-forgejo#48](https://git.cl8y.com/PlasticDigits/cl8y-forgejo/issues/48) `_ensure_codeowners`). Out of this slice. If a later apply re-adds the file, leftover-complete is **not** done: delete again via PR, then one new dated leftover comment with all three leftover-complete items, **G3-1 last**. A GET or plant-check from before a re-copy does not count. Never direct-push `main`. |
| Plant-check → re-copy → stale-green GET | Void. Leftover-complete requires one dated leftover comment with (1) observed-vs-target GET, (2) plant-check `{n}` + JSON, (3) four-path `test -f` fails — **G3-1 last** (or all three timestamps in one attest). |
| Official request leftover on #3 or #2 | Non-blocking **only after** admin POSTs **G3-9** / **G3-10** on leftover and `#3`. If **G3-10** is `true` or GET missing, merge is 405; do not land; do not `force_merge`; optional human dismiss is not a CAC substitute. Not S3 evidence. Not a rollback signal. |
| Treating merge of `#3` as leftover-complete | Merge closes `#3` before S3. Closing `#3` does not assign S3. Use the leftover Forgejo issue in this repo. |
| Leftover issue opened in the wrong repo, empty, or without named owners | Land fails criterion 5. Open in `code/gitlab-cursor-webhook` with title `Forgejo issue, not PR`, owners, and quoted body. |
| Plant-check matcher uses `team.name == "code/maintainers"` or requires `team.organization` on reviews GET | Misses the live plant (`team.name == "maintainers"`, `team.organization == null` on reviews). Use the Observability fail-closed pair. |
| Plant-check is draft (`draft != false` on PR GET), title contains `WIP` (case-insensitive), uses `.env.example` / Terraform / Docker / a runbook as the probe, has no changed file, reviewers were requested in the UI / `POST .../requested_reviewers`, or no `do-not-merge` comment | False pass (CODEOWNERS skipped or would not have planted) or false fail (manual team request). Recipe fails closed; open a new probe. |
| Merging the plant-check PR `{n}` | Leftover-complete **fails** (possible host rebuild / #297 incident). It is not a pass with a Coolify rebuild. Close without merge in the same session. |
| Protection silently reverted to official-review true | Merge 405 returns. Out of this repo; re-apply via forge policy, do not `force_merge`. Not proven by scanners. |
| `enable_push` flipped true | **G3-2** regression (direct push). Refuse. Not a force-push claim. |
| Observed **G3-8** and nothing posts Woodpecker | Open the named local CI issue (Decision 7); it is a DEPS; wait for green. Not solved by restoring CODEOWNERS. Do not fake statuses. Do not add `.woodpecker.yaml` in the #3 diff. |
| Observed **G3-8** false / unset and land waits on Woodpecker anyway | Violates Decision 7. Do not wait. Record observed vs target on leftover-complete. |
| Re-adding CODEOWNERS “for safety” in a follow-up | Violates **G3-1**. Reviewers must reject unless a new ADR allowlists path owners. |
| Observed flags differ from the six-row target | #48 leftover. Not a GCH PATCH. Not a reason to restore `CODEOWNERS`. Not a leftover-complete fail. Land blocker only for **G3-9** / **G3-10**. |
| Rust / Terraform / Docker / token / runbook sneak into the MR | Fail review. |
| README treats `docs/architecture.md` as product architecture or stubs the product map | Fail S2. Two architecture docs must not collide. |
| Implement deploys Coolify, rotates tokens, or edits `autonomy.rs` / HMAC | Forbidden (#297). |
| S2 performs the protection GET or merges without the admin comment | Forbidden. S2 pastes endpoints and stops. |

## Ordered implementation slices

| Slice | Work | Depends on |
| --- | --- | --- |
| **S0** | This design (ADR 0001 + architecture **G3**). Transport on `cac-design-issue-3`. Copy **the independently accepted SHA** (this SHA or a successor) onto the product tip; do not call this tip accepted until independent review says so. `cac-design-issue-3` is never the merge vehicle. Do not GET protection in S0. | None in `code/gitlab-cursor-webhook`. |
| **S1** | Delete root `CODEOWNERS`. Confirm `test -f` fails on all four Forgejo paths. | S0 files present on the **same product-PR tip** (not “S0 accepted” alone). Draft delete already on `chore/remove-catchall-codeowners`. |
| **S2** | README pointer on that **same** tip: keep README as product map; point at `docs/architecture.md` **only** for the merge gate (**G3**); relative links to ADR 0001 / architecture (those files already on the tip); do not imply CODEOWNERS is the trusted-merge gate. Keep the product overview. No Docker/Woodpecker/Terraform/Rust edits. Open the leftover **Forgejo issue, not PR** in **`code/gitlab-cursor-webhook` only**, before merge, with named S3 owners and the body template under Migration (paste the two protection endpoints). Do not GET protection. Do not merge. If admin JSON shows **G3-8**, open the named Woodpecker issue and record that iid as local DEPS. | S0 files on the same tip as S1. Same PR as S1. Admin land attest is **not** an S2 implement step. |
| **S3** | Leftover-complete on the leftover issue: one dated leftover comment with (1) observed-vs-target GET of the other four flags, (2) dedicated plant-check `{n}` + JSON, (3) four-path `test -f` fails — **G3-1 last**. Owners: repo admin (1), named S3 implementer (2+3). Does **not** close `#3`. Closing `#3` does not assign S3. | S0+S1+S2 merged to `main`. Tracked on the leftover issue in this repo. |

PR `#3` (or successor) ships **S0+S1+S2**. S2 depends on S0 files being on the
same tip, not only on S1.

**Repo admin (not a slice of implement):** POST dated JSON for **G3-9** and
**G3-10** on the leftover issue **and** on `#3` before merge. That JSON also
answers Decision 7 (**G3-8**).

If that JSON shows **G3-8**, merge of `#3` waits for
`ci/woodpecker/pr/woodpecker` on the tip **and** for the named Woodpecker
issue S2 opened (local DEPS). Do not add `.woodpecker.yaml` in the #3 diff.
If **G3-8** is false or status checks unset, land does not wait on
Woodpecker.

Sister repos (not slices of #3, not local `DEPS`): forge #48
protection+templates; CAC #429 autoland occupying job; hello#15 canary.

## Tests

In-repo CI cannot GET branch protection. Do not add a `cargo test` solely for
four-path absence (filter/provision tests are product; a whole-tree basename
check would also be the wrong contract). `cargo test` / `cargo clippy -- -D
warnings` still pass on the product PR.

1. **Absence (land, G3-1).** After S1, `test -f CODEOWNERS`,
   `test -f docs/CODEOWNERS`, `test -f .gitea/CODEOWNERS`, and
   `test -f .forgejo/CODEOWNERS` all fail. Optional content check, **pathspec
   those four paths only**:
   `git grep -nE '^\.\* @' -- CODEOWNERS docs/CODEOWNERS .gitea/CODEOWNERS .forgejo/CODEOWNERS`
   — no match is pass. Do **not** whole-tree `git grep` (the escaped form
   `git grep -n '^\\.\\* @'` matches nothing even while the catch-all file
   exists; the unescaped `git grep -nE '^\.\* @'` hits this ADR after a correct
   delete).
2. **Existing scanners (land).** `cargo test` and `cargo clippy -- -D warnings`
   still pass on the product PR. Gitleaks config is unchanged. Do not treat
   empty commit statuses as leftover-complete. Do not treat `.gitlab-ci.yml`
   as **G3-3**. Host Woodpecker is Tests item 9 / Decision 7, not this item.
3. **Protection (leftover-complete, operator read).** Repo admin of
   `code/gitlab-cursor-webhook` attests dated **observed** JSON of the `main`
   rule and compares it to the six-row **target**. Leftover-complete proves
   observed **G3-2**, **G3-8**, **G3-3**, **G3-5** vs that target. Drift is a
   #48 leftover, not a leftover-complete fail, not a GCH PATCH. Not inferred
   from a green scanner. Not compared to the Merge API row. Not copied from
   `code/hello`. **G3-2** pass is `enable_push == false` only. **G3-9** /
   **G3-10** were already land-proven by admin attest.
4. **No new plant (leftover-complete).** After the delete is on `main`, the
   **named S3 implementer** runs this recipe on the leftover issue:
   1. Delete already on `main` (**G3-1** still holds; if a later apply
      re-copied the file, delete again via PR first).
   2. Open a **dedicated** plant-check PR. PR GET must have `draft == false`.
      Title must not contain `WIP` (case-insensitive). Probe a throwaway path
      under `docs/` that is not a runbook (example:
      `docs/_plant-check-adr0001.md`). Forbid `.env.example`, Terraform, and
      Docker as the probe. Do not use a runbook as the probe.
   3. Do not request users or teams in the UI or via
      `POST .../requested_reviewers`.
   4. On open, comment `do-not-merge`. GET `.../pulls/{n}` and
      `.../pulls/{n}/reviews` **immediately after open**.
   5. Pass iff Observability’s fail-closed pair holds. If either plant signal
      is present, fail. If both signals are empty, wait once **30 seconds**
      and re-GET both; pass only if the second pair is still empty.
   6. Record `{n}` **and** the two JSON bodies (the pair used for the pass
      decision) on the leftover issue, then **close without merge in the same
      session** after the 30s re-GET.
   Fail if `draft != false`, if the title contains `WIP` (case-insensitive),
   if the PR has no changed file, if the probe is `.env.example` / Terraform /
   Docker / a runbook, if reviewers were requested manually, if there is no
   `do-not-merge` comment, or if either GET signal is present after the wait.
   If `{n}` is merged, leftover-complete **fails** (possible host rebuild /
   #297 incident). It is not a pass with a Coolify rebuild. Do not use `#3`.
   Do not use `#2`. Do not use “the next natural PR.” Who opens `{n}`: the
   named S3 implementer on the leftover issue. Who GETs protection (item 3):
   repo admin; not S2; not cargo; not Coolify.
   Leftover-complete attest order is Integration: **G3-1 last**. Do not treat
   a passing GET or plant-check from before a later `_ensure_codeowners` apply
   as done.
5. **Reject still blocks (doc-level).** Do not turn off
   `block_on_rejected_reviews` to “make autoland easier.”
6. **Diff guard (land).** Product PR does not change Rust sources, Terraform,
   Docker, `.env.example`, runbooks, or add `.woodpecker.yaml`.
7. **Land GET (G3-9 / G3-10, always).** Repo admin POSTs dated JSON of this
   repo’s `main` rule showing **G3-9** and **G3-10** on the leftover issue
   **and** on `#3` before merge. S2 does not GET. Fail closed if missing or
   if **G3-10** is `true`. Do not `force_merge`.
8. **README collision (land, S2).** Product PR README still is the product map
   and points at `docs/architecture.md` only for the merge gate.
9. **Host context (Decision 7).** If dated admin JSON of this repo shows
   **G3-8**, `GET .../statuses/{product-tip-sha}` includes context
   `ci/woodpecker/pr/woodpecker` in a success state before merge, and the
   named Woodpecker issue S2 opened is recorded as local DEPS. Empty statuses
   on `022f4f5` mean that tip is not merge-ready under observed **G3-8**. If
   **G3-8** is false or status checks unset, land does **not** wait on
   Woodpecker. Slice S2 does not add `.woodpecker.yaml`.

## Rollout

- Merge vehicle **B**: existing
  [#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) once that
  branch’s tip is S0 files + S1 delete + S2 README **or** a successor PR with
  the same three. Design-only `cac-design-issue-3` must not be opened as the
  product PR and must not be merged first as docs-only.
- Order: fleet protection already owned by #48 → copy S0 files from the
  independently accepted SHA + delete file + README via one product PR → S2
  opens leftover Forgejo issue with named owners and pasted endpoints → repo
  admin POSTs **G3-9** / **G3-10** (and observed **G3-8**) on leftover and
  `#3` → if **G3-8**, S2 opens named Woodpecker issue (local DEPS) and land
  waits for green context → leftover issue remains open → leftover-complete
  as one dated comment with observed-vs-target GET of the other four flags,
  plant-check (close without merge in the same session), then four-path
  `test -f` (**G3-1 last**).
- Woodpecker: Decision 7 only. Do not weaken **G3-3** / **G3-8** **target**
  rows to land #3; do not add the pipeline in this diff; do not wait with no
  ticket.
- Canary role: other `code/*` catch-all deletions may copy this pattern; this
  ADR does not merge those repos. hello#15 is not a gate.
- [#297](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/297): no
  deploy, spend, custody, or CAC policy expansion. Landing #3 does not
  authorize Coolify config changes, token rotation, Hetzner spend, or
  autonomy changes. This Coolify app **may** rebuild if the existing host is
  git-follow on `main` (not verified in-tree); that is not a new grant and is
  not leftover-complete.

## Rollback

Restore the previous `CODEOWNERS` **via PR**, not direct `main`, from
`72133f5` (six-line catch-all). That re-plants official requests. It does
**not** by itself re-enable merge-block
(`block_on_official_review_requests`); restoring the 405 gate is a
forge-policy revert, founder-scoped, and is not a GCH rollback step.

Woodpecker / Coolify / Terraform rollback is unused: those files are
untouched.

If S0 docs need revert, revert via PR together with README so relative links
do not 404.

## Integration completion criteria

### Land (S0+S1+S2) — merge of PR `#3` or successor

All must be true on the merged tip. This is what merging `#3` completes. It
does **not** wait for S3.

1. `main` has no CODEOWNERS file at the four Forgejo paths: `test -f` fails on
   `CODEOWNERS`, `docs/CODEOWNERS`, `.gitea/CODEOWNERS`, `.forgejo/CODEOWNERS`
   (**G3-1**).
2. Merged tip contains `docs/adr/0001-remove-catchall-codeowners.md` and
   `docs/architecture.md` **byte-identical** to the independently accepted SHA
   (this SHA or a successor; not this tip until independent review). README
   relative links to those paths resolve. Do not merge a README that points at
   those paths until they exist on that tip.
3. README states the **G3** gate via `docs/architecture.md`, keeps the product
   map, and does not imply CODEOWNERS is what makes merge trusted. Product
   overview remains. No stub escape.
4. Diff does not change Rust, Terraform, Docker, `.env.example`, runbooks, or
   add `.woodpecker.yaml`. No `force_merge`, no direct `main`, no CAC
   dismiss-as-merge, no Coolify/HMAC/`autonomy.rs` edits in the product-PR
   diff.
5. A leftover **Forgejo issue, not PR** exists in
   **`code/gitlab-cursor-webhook` only**, opened before this merge, whose
   title includes `Forgejo issue, not PR`, whose body names S3 owners (repo
   admin = protection GET; named S3 implementer = plant-check `{n}` +
   **G3-1 last**) and quotes leftover-complete items 1–3, and which states
   that closing `#3` does not assign S3. An empty issue, an issue in another
   repo, or an issue without named owners does not satisfy this criterion.
   Record its iid on the product PR.
6. **Always.** Repo admin of `code/gitlab-cursor-webhook` has POSTed dated
   JSON of this repo’s `main` rule showing **G3-9** and **G3-10** on the
   leftover issue **and** on `#3`. S2 did not perform that GET. Do not land
   if the JSON is missing. If **G3-10** is `true`, merge is 405; stop. Do not
   `force_merge`. Land proves **G3-9** / **G3-10** (admin attest).
7. Decision 7: if that same dated JSON shows **G3-8**, S2 opened a named
   Forgejo issue in this repo whose only job is enable/post
   `ci/woodpecker/pr/woodpecker`, recorded that iid on `#3` as local DEPS,
   and `ci/woodpecker/pr/woodpecker` has posted success on the product-PR
   tip. Empty statuses on `022f4f5` do not satisfy this. Do not fake the
   context. Do not add `.woodpecker.yaml` in this ticket. If **G3-8** is
   false or status checks unset, land does **not** wait on Woodpecker.

Green `cargo test` on `chore/remove-catchall-codeowners` **before** merge is
useful and not sufficient for leftover-complete. Empty commit statuses on
`022f4f5` document the CI gap; they are not leftover-complete. Draft
`022f4f5` is incomplete without S0 files and S2.

### Leftover-complete (S3) — leftover issue in this repo; survives merge of `#3`

Require **one dated leftover comment** with all three items, **G3-1 last**
(or all three timestamps in one attest). A GET or plant-check from before a
later `_ensure_codeowners` apply does not count; re-delete via PR and write a
new comment. Closing `#3` does not assign S3.

1. Observed-vs-target GET of the other four flags: repo admin of
   `code/gitlab-cursor-webhook` attests dated JSON of the `main` rule and
   compares **G3-2**, **G3-8**, **G3-3**, **G3-5** to the architecture
   **target**. Not the Merge API row. Not inferred from a green scanner. Not
   copied from another repo. **G3-2** is `enable_push == false` only. Drift
   vs target is a #48 leftover, not a GCH PATCH, not a leftover-complete
   fail, not a reason to restore `CODEOWNERS`. **G3-9** / **G3-10** were
   land-proven by admin attest.
2. Plant-check `{n}` + JSON: the **named S3 implementer** runs a **dedicated**
   plant-check PR following Tests item 4 and Observability’s fail-closed pair.
   Throwaway `docs/` path that is not a runbook; `do-not-merge` on open; close
   without merge in the same session after the 30s re-GET. Record `{n}` and
   the two JSON bodies. If `{n}` is merged, leftover-complete **fails**.
   `#3`’s own official request does not count. `#2` does not count. “The next
   natural PR” does not count.
3. **G3-1 last:** `test -f` fails on `CODEOWNERS`, `docs/CODEOWNERS`,
   `.gitea/CODEOWNERS`, and `.forgejo/CODEOWNERS`. Recorded after items 1 and
   2 (same comment or later timestamp in the same attest). Owner: named S3
   implementer.

Forgejo#50 and CAC#429 may stay open; they are not land or leftover gates for
this tree.

## Authority

[cl8y-agent-control#297](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/297):
this design does not grant deploy, spend, custody, or agent-permission
expansion. Relaxing official-review as a **forge merge gate** is fleet policy
under [cl8y-forgejo#48](https://git.cl8y.com/PlasticDigits/cl8y-forgejo/issues/48);
this ADR only removes the in-tree file that plants requests. Independent
review of this proposal is a later gate. Design author must not write
`DESIGN: APPROVE`.
