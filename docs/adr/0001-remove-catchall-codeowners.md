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

Overview (merge gate, runtime): [`architecture.md`](../architecture.md). Do
not copy that table here. **G3** there is three groups: protection GET (six
flags), merge procedure (**G3-4**), tree contracts (**G3-1**, **G3-6**,
**G3-7**).

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
Woodpecker context `ci/woodpecker/pr/woodpecker` when **G3-8** holds,
SHA-pinned `Do: merge`, no direct push, no `force_merge`.

**Land vehicle B:** product PR
[#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) (or a successor)
ships **S0+S1+S2 on one tip** — copy the two design files from **the
independently accepted SHA** (this SHA or a successor), delete `CODEOWNERS`,
add a short README pointer. Merging that PR closes `#3`. Design branch
`cac-design-issue-3` is review/transport only; it is **not** merged as a
docs-only PR and is **not** the product PR.

**Leftover issue (land gate).** Before merge of `#3`, S2 opens one leftover
issue in **`code/gitlab-cursor-webhook` only** (title, repo, and body template
under Migration). That issue owns S3. Land criterion 5 fails if the issue is
missing, empty, or opened in another repo.

**Leftover-complete** (S3: dated protection GET of the six **G3** flags +
dedicated post-merge plant-check PR `{n}` + four-path absence, **G3-1 last**)
lives on that leftover issue. S3 is not a close gate for `#3`.

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
same planted team request.

Fleet protection under #48 is described as
`block_on_official_review_requests=false` (**G3-10**) and
`required_approvals=0` (**G3-9**). This design has **no** dated protection JSON
for `code/gitlab-cursor-webhook` (unauthenticated GET is 401). Do **not** treat
the leftover request on `#3` as non-blocking from fleet values. It is
non-blocking **only if** a dated GET of **this** repo’s `main` rule shows
**G3-9** and **G3-10**. If **G3-10** is still `true` here, merge is 405; do not
land `#3` on an unverified GET. That leftover request also must not be treated
as S3 evidence. Leftover-complete still requires a repo-admin GET of **this**
repo’s `main` rule for all six flags.

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

This controller provisions Hetzner VMs and may Coolify-rebuild on `main`.
Landing #3 is still not a #297 deploy grant: the product-PR diff must not
change image, compose, Terraform, tokens, or auto-deploy.

## Non-goals

- Forgejo protection JSON / `apply_repo_policy.py` / migrate `_ensure_codeowners`
  / templates ([cl8y-forgejo#48](https://git.cl8y.com/PlasticDigits/cl8y-forgejo/issues/48)
  / [pulls/50](https://git.cl8y.com/PlasticDigits/cl8y-forgejo/pulls/50)).
  Sister-repo `_ensure_codeowners` stays out of this slice; leftover-complete
  still fails if a later apply has put the file back (Failure modes).
- PATCHing branch protection from this tree, including force-push allowlist
  fields and `apply_to_admins` (forge #48).
- CAC autoland predicates, occupying jobs, or `DrainSkip::OfficialReview`
  cleanup ([cl8y-agent-control#429](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/429),
  leftover of #388). Drain comments such as `drain skip: no occupying job…` on
  #3 are **#429**, not a #3 failure.
- Dismissing reviewers from the controller (forbidden substitute in #388).
- Path-specific CODEOWNERS, a second maintainer, or `required_approvals: 1`.
- Adding, enabling, or digest-pinning Woodpecker for this repo. Missing
  `ci/woodpecker/pr/woodpecker` statuses are pre-existing. Do not add
  `.woodpecker.yaml` / `.woodpecker/` in the #3 diff.
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
   #2). Treat them as non-blocking **only if** a dated GET of
   `code/gitlab-cursor-webhook` `main` shows **G3-9** and **G3-10**. Do not
   dismiss them from CAC. Human dismiss is optional leftover, not AC.
5. **Do not** PATCH branch protection from this repository.
6. **Split land from leftover-complete.** Product PR `#3` (or successor) is
   S0+S1+S2 (vehicle **B**). S3 lives on the leftover issue S2 opens in this
   repo before that merge (template below). Require a **dedicated** post-merge
   plant-check PR. Do not accept `#3`’s own official request, `#2`, or “the
   next natural PR.”
7. **Host CI is not an in-diff deliverable.** If a dated GET of this repo
   shows **G3-8**, merge of `#3` waits until `ci/woodpecker/pr/woodpecker`
   posts on that tip. Do not add the pipeline in the CODEOWNERS diff. Do not
   fake statuses. Do not `force_merge`. Do not say file-delete + docs unblocks
   merge while **G3-8** is true and the context is missing. No Woodpecker iid
   exists in this repo today (no `DEPS`).

## Component / state / interface changes

| Surface | Change |
| --- | --- |
| `CODEOWNERS` (root) | Remove file. |
| `docs/CODEOWNERS`, `.gitea/CODEOWNERS`, `.forgejo/CODEOWNERS` | Must remain absent (no empty file). |
| `docs/adr/0001-remove-catchall-codeowners.md`, `docs/architecture.md` | Copy from the independently accepted SHA onto the product PR so the merged tip is S0+S1+S2 (byte-identical). Standing G3 contract lands with the delete. |
| Forgejo PR review interface | After land, a **dedicated** plant-check PR against `main` must not get an official CODEOWNERS team request. |
| Branch protection API | No write from this ticket. Operator reads must still match architecture **protection GET** (six flags for the `main` rule). In-repo CI cannot perform that GET. Land of `#3` while leftover official requests remain additionally requires a dated GET of **G3-9** and **G3-10** on **this** repo. |
| `.woodpecker.yaml` / `.woodpecker/` | Must remain absent in this ticket. Do not add one to unblock merge. |
| `.gitlab-ci.yml` | Unchanged. |
| Rust crates, Terraform, Docker, `.env.example`, gitleaks | Unchanged. |
| README | Mandatory on the product PR: merge gate is **G3**, documented in `docs/architecture.md`; product map stays README. Relative links to ADR 0001 / architecture, which exist on that same tip. Keep the product overview. |
| Runbooks under `docs/` | Unchanged except adding `adr/` + `architecture.md`. |
| Leftover issue | New issue in `code/gitlab-cursor-webhook` only, opened before merge of `#3`, body quotes leftover-complete items 1–3. |
| CAC / Coolify / org team `maintainers` in org `code` | Unchanged. The team may keep existing; it simply is not planted as official review. Coolify may rebuild from a `main` push; that is existing host follow, not a new deploy grant. |

No runtime state, schema, or HTTP API.

## Affected invariants

IDs live in [`architecture.md`](../architecture.md). This ADR changes **G3-1**
(four-path absence). It does not write protection JSON. Leftover-complete
**reads** the six protection flags (**G3-2**, **G3-8**, **G3-3**, **G3-9**,
**G3-10**, **G3-5**). Land does not prove those six GET flags. Land **does**
require a dated GET of **G3-9** and **G3-10** on this repo before treating
leftover official requests as non-blocking. Merge procedure remains **G3-4**.
Coolify and CAC policy remain **G3-6** and **G3-7**. **G3-2** is
`enable_push == false` (no direct push); it does not claim force-push policy.

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
| Add `.woodpecker.yaml` in the same PR | Different change (CI enablement). Missing statuses are pre-existing. **G3** already names the required context. |
| `force_merge` or fake Woodpecker statuses to land #3 | Forbidden by **G3-4** / **G3-3**. |
| Claim file-delete + docs closes #3 while **G3-8** is true and the context is missing | Host will refuse. Wait for a named CI issue; do not restore CODEOWNERS. |
| Treat `#3`’s plant, `#2`, or the next natural PR as S3 | Merge closes `#3` before leftover-complete. A dedicated post-merge PR is the evidence. |
| Vehicle **A**: docs-only PR from `cac-design-issue-3` onto `main`, then `#3` as S1+S2 | Second merge vehicle. That branch is design transport, not a product PR. Vehicle **B** puts the two files on the deletion PR so README links resolve on one tip. Draft `022f4f5` is not that tip until S0 files and S2 are added. |
| Wait on sibling `code/*` CODEOWNERS PRs / hello#15 | Wrong repo; no product iid dependency. |
| Open the leftover issue in `PlasticDigits/*` or leave it empty | Land criterion 5 would pass a wrong-repo or vacant issue. S2 names this repo and quotes leftover-complete items 1–3. |
| Treat Coolify rebuild after merge as leftover-complete | **G3-6**. Plant-check and protection GET are leftover-complete. |
| Write the archival `PlasticDigits/gitlab-cursor-webhook` clone | CAC invariant 21. Wrong repo. |

## Complexity added / removed

**Removed:** catch-all official-review robot on every diff; operator dismiss
step; false “CODEOWNERS is the trusted-PR gate” story in this repo.

**Added:** a small standing doc (this ADR + architecture **G3**) that lands on
`main` via the product PR so later agents do not re-add `.* @code/maintainers`
as a merge requirement, and a leftover issue in this repo that survives merge
of `#3`. No new services, jobs, flags, pipelines, or test harnesses.

## Migration

1. Fleet protection is owned by forge #48. This ticket does not PATCH. Do not
   treat a GET recorded on `code/hello` as proof for this repo.
2. **Leftover issue (S2, land gate).** Before merging the product PR, open
   **one** follow-up issue in **`code/gitlab-cursor-webhook` only**. Do not
   open it in `PlasticDigits/cl8y-forgejo`, `PlasticDigits/cl8y-agent-control`,
   `PlasticDigits/gitlab-cursor-webhook`, or any other repo. Suggested title:
   `chore: leftover CODEOWNERS S3 (protection GET + plant-check)`. Body
   **must quote** leftover-complete items 1–3 from this ADR (dated `main`
   protection GET of the six **G3** flags; dedicated post-merge plant-check
   `{n}`, close without merge; four-path `test -f` **G3-1 last**). Record that
   issue’s iid on the product PR before merge. Merging
   [#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) closes that
   number; S3 must not live only there.

   Body template (quote onto the leftover issue):

   ```
   Leftover-complete for ADR 0001 after merge of product PR #3. This issue
   does not close #3. Opened in code/gitlab-cursor-webhook before that merge.

   1. Dated operator GET of this repo's `main` protection rule equals the six
      G3 protection flags (G3-2, G3-8, G3-3, G3-9, G3-10, G3-5) in
      docs/architecture.md. Attest the JSON here. Not copied from code/hello.
      A GET recorded before a later template re-copy does not count.

   2. After the delete is on `main`: dedicated plant-check PR {n} (not draft,
      title not WIP, at least one changed file, no manual reviewer request);
      GET immediately after open; if both plant signals empty, wait 30s and
      re-GET. Pass iff requested_reviewers_teams length 0 AND no review with
      official == true, state == "REQUEST_REVIEW", team.name == "maintainers"
      (optional team.id == 4). Do not require team.organization on reviews.
      Record {n} and the two JSON bodies, then close without merge.
      Not #3, not #2, not the next natural PR.

   3. G3-1 last: test -f fails on CODEOWNERS, docs/CODEOWNERS,
      .gitea/CODEOWNERS, .forgejo/CODEOWNERS. Recorded after items 1 and 2.
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
4. Open PRs created while the file existed (#3, #2) may still show an official
   team request. Non-blocking **only if** a dated GET of this repo shows
   **G3-9** and **G3-10**. No bulk dismiss required to land `#3` under that
   GET. If **G3-10** is `true`, stop; merge is 405.
5. Do not restore the file from `docs/templates/CODEOWNERS` in cl8y-forgejo;
   that template is owned by #48.
6. If host protection requires `ci/woodpecker/pr/woodpecker` (**G3-8**) and
   nothing posts it, PRs cannot merge until CI is enabled — that is a
   **separate** host/CI issue, not a reason to restore catch-all CODEOWNERS or
   `force_merge`. No such iid exists in this repo today.

## Observability

Relative reads. Do not log tokens, hosts, or protection-script inventories. Do
not add a Forgejo admin token to Woodpecker, cargo tests, or Coolify.

**Land GET (G3-9 / G3-10, this repo).** Before merging `#3` while leftover
official requests remain, repo admin of `code/gitlab-cursor-webhook` attests a
dated JSON of the `main` rule showing **G3-9** (`required_approvals == 0`) and
**G3-10** (`block_on_official_review_requests == false`). Same endpoints as
the leftover-complete GET. Fleet #48 / `code/hello` is not this GET. If
**G3-10** is `true`, do not land.

**Protection (operator, leftover-complete).** Repo admin of
`code/gitlab-cursor-webhook` attests dated JSON of the six protection flags
for the `main` rule onto the leftover issue in this repo.
`GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections` (array; pick
`rule_name == "main"`) or
`GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections/main`. Pass
iff that rule equals architecture **protection GET** for **G3-2**, **G3-8**,
**G3-3**, **G3-9**, **G3-10**, and **G3-5**. `status_check_contexts` must
**equal** `["ci/woodpecker/pr/woodpecker"]`. Fail if any of those six differ.
Do not compare the Merge API / **G3-4** row (not a protection field). Do not
treat `enable_push == false` as a force-push read. A green `cargo test`, empty
commit statuses, or a Coolify deploy does not satisfy this read. In-repo CI
cannot perform this GET.

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
`main`. GET both endpoints **immediately after open**. If either plant signal
is present, fail. If both signals are empty, wait once **30 seconds** and
re-GET both before pass; pass only if the second pair is still empty. PR GET
must have `draft == false`. Title must not contain `WIP` (case-insensitive).
Record `{n}` **and** the two JSON bodies (the pair used for the pass decision)
on the leftover issue, then **close without merge**. PR `#3`’s own official
request does not pass. `#2` does not pass. “The next natural PR” does not
pass.

**CI / deploy.** Required host context remains `ci/woodpecker/pr/woodpecker`
when **G3-8** is true. This tree does not post it today. Drain comments such as
`drain skip: no occupying job…` are **#429**, not a #3 failure. Coolify rebuild
on `main` is **G3-6** (existing host), not leftover-complete.

## Failure modes

| Mode | Handling |
| --- | --- |
| File deleted on a branch but still on `main` | New PRs keep planting official review until the product PR merges. Expected until land. |
| README links ADR/architecture but those files are not on the same tip | 404 after merge. Land fails criterion 2. Vehicle **B**: copy both files onto the product PR before merging README. |
| Copy left in `docs/`, `.gitea/`, or `.forgejo/` (including empty/comments-only) | Forgejo still loads the first existing path and may plant. Land fails **G3-1**; delete those paths too (none exist on current `main`). |
| Whole-tree `git grep` for `.* @` | Hits this ADR after a correct delete. Use the pathspec in Tests item 1. |
| cl8y-forgejo migrate/apply re-copies a template | Sister-repo race ([cl8y-forgejo#48](https://git.cl8y.com/PlasticDigits/cl8y-forgejo/issues/48) `_ensure_codeowners`). Out of this slice. If a later apply re-adds the file, leftover-complete is **not** done: delete again via PR, then one new dated leftover comment with all three leftover-complete items, **G3-1 last**. A GET or plant-check from before a re-copy does not count. Never direct-push `main`. |
| Plant-check → re-copy → stale-green GET | Void. Leftover-complete requires one dated leftover comment with (1) protection GET, (2) plant-check `{n}` + JSON, (3) four-path `test -f` fails — **G3-1 last** (or all three timestamps in one attest). |
| Official request leftover on #3 or #2 | Non-blocking **only if** dated GET of this repo shows **G3-9** / **G3-10**. If **G3-10** is `true`, merge is 405; do not land; do not `force_merge`; optional human dismiss is not a CAC substitute. Not S3 evidence. Not a rollback signal. |
| Treating merge of `#3` as leftover-complete | Merge closes `#3` before S3. Use the leftover issue in this repo. |
| Leftover issue opened in the wrong repo, or empty | Land fails criterion 5. Open in `code/gitlab-cursor-webhook` with the quoted body. |
| Plant-check matcher uses `team.name == "code/maintainers"` or requires `team.organization` on reviews GET | Misses the live plant (`team.name == "maintainers"`, `team.organization == null` on reviews). Use the Observability fail-closed pair. |
| Plant-check is draft (`draft != false` on PR GET), title contains `WIP` (case-insensitive), has no changed file, or reviewers were requested in the UI / `POST .../requested_reviewers` | False pass (CODEOWNERS skipped or would not have planted) or false fail (manual team request). Recipe fails closed; open a new probe. |
| Merging the plant-check PR | A `main` push **will** rebuild Coolify if auto-deploy is on. **Close without merge.** |
| Protection silently reverted to official-review true | Merge 405 returns. Out of this repo; re-apply via forge policy, do not `force_merge`. Not proven by scanners. |
| `enable_push` flipped true | **G3-2** regression (direct push). Refuse. Not a force-push claim. |
| Host requires Woodpecker and nothing posts | Pre-existing deadlock. Not solved by restoring CODEOWNERS. Not #3 implement. Do not fake statuses. Do not add `.woodpecker.yaml` in this diff. |
| Re-adding CODEOWNERS “for safety” in a follow-up | Violates **G3-1**. Reviewers must reject unless a new ADR allowlists path owners. |
| Rust / Terraform / Docker / token / runbook sneak into the MR | Fail review. |
| README treats `docs/architecture.md` as product architecture or stubs the product map | Fail S2. Two architecture docs must not collide. |
| Implement deploys Coolify, rotates tokens, or edits `autonomy.rs` / HMAC | Forbidden (#297). |

## Ordered implementation slices

| Slice | Work | Depends on |
| --- | --- | --- |
| **S0** | This design (ADR 0001 + architecture **G3**). Transport on `cac-design-issue-3`. Copy **the independently accepted SHA** (this SHA or a successor) onto the product tip; do not call this tip accepted until independent review says so. `cac-design-issue-3` is never the merge vehicle. | None in `code/gitlab-cursor-webhook`. |
| **S1** | Delete root `CODEOWNERS`. Confirm `test -f` fails on all four Forgejo paths. | S0 files present on the **same product-PR tip** (not “S0 accepted” alone). Draft delete already on `chore/remove-catchall-codeowners`. |
| **S2** | README pointer on that **same** tip: keep README as product map; point at `docs/architecture.md` **only** for the merge gate (**G3**); relative links to ADR 0001 / architecture (those files already on the tip); do not imply CODEOWNERS is the trusted-merge gate. Keep the product overview. No Docker/Woodpecker/Terraform/Rust edits. Open the leftover issue in **`code/gitlab-cursor-webhook` only**, before merge, with the body template under Migration. Attest dated **G3-9** / **G3-10** GET of this repo before treating leftover official requests as non-blocking. | S0 files on the same tip as S1. Same PR as S1. |
| **S3** | Leftover-complete: one dated leftover comment with (1) protection GET six flags, (2) dedicated plant-check `{n}` + JSON, (3) four-path `test -f` fails — **G3-1 last**. Does **not** close `#3`. | S0+S1+S2 merged to `main`. Tracked on the leftover issue in this repo. |

PR `#3` (or successor) ships **S0+S1+S2**. S2 depends on S0 files being on the
same tip, not only on S1.

If dated GET shows **G3-8**, merge of that PR additionally waits for
`ci/woodpecker/pr/woodpecker` on the tip. That wait is not a slice of #3
implement and is not a local `DEPS` (no Woodpecker iid exists today).

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
   still pass on the product PR. Gitleaks config is unchanged. Do not require a
   Woodpecker run that this tree cannot post. Do not treat empty commit
   statuses as leftover-complete. Do not treat `.gitlab-ci.yml` as **G3-3**.
3. **Protection (leftover-complete, operator read).** Repo admin of
   `code/gitlab-cursor-webhook` attests dated JSON:
   `GET .../branch_protections` `main` rule equals the six architecture
   **protection GET** flags. Fail if any differ. Not inferred from a green
   scanner. Not compared to the Merge API row. Not copied from `code/hello`.
   **G3-2** pass is `enable_push == false` only.
4. **No new plant (leftover-complete).** After the delete is on `main`, on the
   leftover issue in this repo run this recipe:
   1. Delete already on `main` (**G3-1** still holds; if a later apply
      re-copied the file, delete again via PR first).
   2. Open a **dedicated** plant-check PR. PR GET must have `draft == false`.
      Title must not contain `WIP` (case-insensitive). At least one changed
      file (any path matches Go `.*`). Do not use `.env.example`, Terraform,
      or Docker files as the probe if that would be tempting to merge.
   3. Do not request users or teams in the UI or via
      `POST .../requested_reviewers`.
   4. GET `.../pulls/{n}` and `.../pulls/{n}/reviews` **immediately after
      open**.
   5. Pass iff Observability’s fail-closed pair holds. If either plant signal
      is present, fail. If both signals are empty, wait once **30 seconds**
      and re-GET both; pass only if the second pair is still empty.
   6. Record `{n}` **and** the two JSON bodies (the pair used for the pass
      decision) on the leftover issue, then **close without merge**.
   Fail if `draft != false`, if the title contains `WIP` (case-insensitive),
   if the PR has no changed file, if reviewers were requested manually, or if
   either GET signal is present after the wait. Do not use `#3`. Do not use
   `#2`. Do not use “the next natural PR.” Who opens the PR: anyone who can
   create a PR on `code/gitlab-cursor-webhook`. Who GETs protection (item 3):
   repo admin; not cargo; not Coolify.
   Leftover-complete attest order is Integration: **G3-1 last**. Do not treat
   a passing GET or plant-check from before a later `_ensure_codeowners` apply
   as done.
5. **Reject still blocks (doc-level).** Do not turn off
   `block_on_rejected_reviews` to “make autoland easier.”
6. **Diff guard (land).** Product PR does not change Rust sources, Terraform,
   Docker, `.env.example`, runbooks, or add `.woodpecker.yaml`.
7. **Land GET (G3-9 / G3-10).** If leftover official requests remain on `#3`
   or `#2`, a dated GET of this repo’s `main` rule shows **G3-9** and
   **G3-10** before merge. Fail closed if missing or if **G3-10** is `true`.
8. **README collision (land, S2).** Product PR README still is the product map
   and points at `docs/architecture.md` only for the merge gate.
9. **Host context (merge of #3, if G3-8).** If dated GET of this repo shows
   **G3-8**, `GET .../statuses/{product-tip-sha}` includes context
   `ci/woodpecker/pr/woodpecker` in a success state before merge. Empty
   statuses on `022f4f5` mean that tip is not merge-ready under **G3-8**.
   Slice S2 does not satisfy this.

## Rollout

- Merge vehicle **B**: existing
  [#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) once that
  branch’s tip is S0 files + S1 delete + S2 README **or** a successor PR with
  the same three. Design-only `cac-design-issue-3` must not be opened as the
  product PR and must not be merged first as docs-only.
- Order: fleet protection already owned by #48 → copy S0 files from the
  independently accepted SHA + delete file + README via one product PR,
  leftover issue open in this repo, dated **G3-9** / **G3-10** GET of this
  repo if leftover official requests remain (land) → leftover issue remains
  open → leftover-complete as one dated comment with protection GET,
  plant-check (close without merge), then four-path `test -f` (**G3-1 last**).
- If host-required Woodpecker context is missing, wait for a separate CI
  issue; do not weaken **G3-3** / **G3-8** to land #3; do not add the pipeline
  in this diff.
- Canary role: other `code/*` catch-all deletions may copy this pattern; this
  ADR does not merge those repos. hello#15 is not a gate.
- [#297](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/297): no
  deploy, spend, custody, or CAC policy expansion. Landing #3 does not
  authorize Coolify config changes, token rotation, Hetzner spend, or
  autonomy changes. An existing Coolify `main` follow that rebuilds the image
  after merge is not a new grant.

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
5. A leftover issue exists in **`code/gitlab-cursor-webhook` only**, opened
   before this merge, whose body quotes leftover-complete items 1–3 (dated
   `main` protection GET of the six **G3** flags; dedicated post-merge
   plant-check `{n}`, close without merge; four-path `test -f` **G3-1 last**).
   An empty issue or an issue in another repo does not satisfy this criterion.
   Record its iid on the product PR.
6. If leftover official requests remain on `#3` or `#2`, a dated GET of this
   repo’s `main` rule shows **G3-9** and **G3-10**. Do not land on an
   unverified GET. If **G3-10** is `true`, merge is 405; stop.
7. If a dated GET of this repo shows **G3-8**, `ci/woodpecker/pr/woodpecker`
   has posted success on the product-PR tip. Empty statuses on `022f4f5` do
   not satisfy this. Do not fake the context. Do not add `.woodpecker.yaml` in
   this ticket to satisfy it.

Green `cargo test` on `chore/remove-catchall-codeowners` **before** merge is
useful and not sufficient for leftover-complete. Empty commit statuses on
`022f4f5` document the CI gap; they are not leftover-complete. Draft
`022f4f5` is incomplete without S0 files and S2.

### Leftover-complete (S3) — leftover issue in this repo; survives merge of `#3`

Require **one dated leftover comment** with all three items, **G3-1 last**
(or all three timestamps in one attest). A GET or plant-check from before a
later `_ensure_codeowners` apply does not count; re-delete via PR and write a
new comment.

1. Protection GET six flags: repo admin of `code/gitlab-cursor-webhook`
   attests dated JSON: `GET .../branch_protections` `main` rule equals the six
   architecture **protection GET** flags (**G3-2**, **G3-8**, **G3-3**,
   **G3-9**, **G3-10**, **G3-5**). Not the Merge API row. Not inferred from a
   green scanner. Not copied from another repo. **G3-2** is
   `enable_push == false` only.
2. Plant-check `{n}` + JSON: a **dedicated** plant-check PR following Tests
   item 4 and Observability’s fail-closed pair. Record `{n}` and the two JSON
   bodies, then close without merge. `#3`’s own official request does not
   count. `#2` does not count. “The next natural PR” does not count.
3. **G3-1 last:** `test -f` fails on `CODEOWNERS`, `docs/CODEOWNERS`,
   `.gitea/CODEOWNERS`, and `.forgejo/CODEOWNERS`. Recorded after items 1 and
   2 (same comment or later timestamp in the same attest).

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
