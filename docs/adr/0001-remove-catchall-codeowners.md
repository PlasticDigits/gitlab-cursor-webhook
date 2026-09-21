# ADR 0001: Remove catch-all CODEOWNERS

## Status

Proposed ([#3](https://git.cl8y.com/code/gitlab-cursor-webhook/issues/3)). Not
accepted by this design-author pass. Do not call this tip accepted until
independent review says so. Keywords in the issue body are not architecture
approval. There is no separate standing issue `#3`; the issues URL and the
product PR share one number (`pull_request` present).

After independent ACCEPT, pin **that accepted hex** on leftover `{iid}`
(`iid != 3`) and on
[`pulls/3`](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3). Land
criterion: `git diff <accepted> -- docs/adr/0001-remove-catchall-codeowners.md docs/architecture.md`
is empty on the product tip. Do not treat `62ffd58` as copyable.

Overview (merge gate, runtime, six-row **this-repo host target**, one
Woodpecker land rule that classifies **(a)** / **(b)** / **(c)**; merge of
`#3` only under **(c)**):
[`architecture.md`](../architecture.md). Do not copy that table here. **G3**
there is three groups: protection GET **host target** (six flags; this
repo’s `main` target, not forge INVARIANTS **2–7** — item 2 is “No
force-push,” omitted and assigned to #48, not the `#3` body), merge
procedure (**G3-4**), tree contracts (**G3-1**, **G3-6**, **G3-7**). Issue
`#3` maps only to **G3-2** (`enable_push == false`; `#3` does **not** own
force-push — named #48), **G3-8**/**G3-3** (standing remaining CI gate),
**G3-4**.

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
requests on every change. Merge to `main` stays: pull request, no direct
push, Woodpecker context `ci/woodpecker/pr/woodpecker` (issue `#3` body;
standing **G3-8** ∧ **G3-3**), SHA-pinned `Do: merge` by `@PlasticDigits`,
no `force_merge`. Decision 7 classifies observed protection **(a)** /
**(b)** / **(c)** from `@PlasticDigits`’s dated GET; it does not drop
Woodpecker from that remaining gate. Merge of `#3` **only** under **(c)**.
**(a)** / **(b)** are distinct **stop** reasons (named **G3-8** DEPS vs
named **G3-3** DEPS); do not `Do: merge` on either. **S2 never merges `#3` or `{w}`.**

**Land vehicle B:** product PR
[#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3)
ships **S0+S1+S2 on one tip** — copy the two design files from **the
independently accepted hex**, delete `CODEOWNERS`, add a short README pointer.
Merging `pulls/3` closes `#3`. A successor PR is allowed only if it
`Fixes #3` **and** closes or supersedes `pulls/3` (merge of the successor
then closes `#3`). “Or a successor” without those trailers does **not**
close `#3`. Design branch `cac-design-issue-3` is review/transport only;
it is **not** merged as a docs-only PR and is **not** the product PR. PR
`#3` must not use `Fixes` / `Closes` of leftover `{iid}` or of any
Woodpecker / unstick iid. A successor may `Fixes #3`; it still must not
`Fixes` / `Closes` leftover or Woodpecker iids.

**Leftover issue (land gate).** Before merge of `#3`, S2 opens one leftover
Forgejo **issue** in **`code/gitlab-cursor-webhook` only** (owners and body
template under Migration). Prove it with
`GET /api/v1/repos/code/gitlab-cursor-webhook/issues/{iid}`:
`pull_request` absent, `iid != 3`, `repository.full_name ==
"code/gitlab-cursor-webhook"`. That issue owns S3. Never call `#3` the
leftover. Closing `#3` does not assign S3. Land criterion 5 fails if that GET
does not match, if the body lacks two Forgejo `@login`s, or if a placeholder
remains.

**Land vs leftover-complete.** Land fail-closes on **G3-9** / **G3-10** and
on `enable_push != false` (**G3-2**) via **repo admin** attest (not S2 GET).
If **G3-9 ≠ 0** or **G3-10** is `true` or `enable_push != false`, that
mismatch is a **named** local/host DEPS. Merge of `#3` is Decision 7 **(c)**
only (architecture executable **(c)** sequence: `{w}` merged under statuses
success → S2 rebase → statuses GET success → **G3-4**). Leftover-complete (S3) **records** dated observed
JSON plus a written vs-target diff (does not “prove” flags) **after (c)**
land; drift after land is `#48`, not an S3 fail. Plus dedicated post-merge
plant-check PR `{n}` closed unmerged, plus four-path absence (**G3-1 last**).
S3 is not a close gate for `#3`.

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

Fleet protection under #48 is **this repo’s** architecture six-row **host
target**, including `block_on_official_review_requests=false` (**G3-10**) and
`required_approvals=0` (**G3-9**). Force-push (forge INVARIANTS item 2) is
out of `#3` and stays named #48. Issue `#3` did not publish that table.
Unauthenticated HTML of `#48` / `INVARIANTS.md` **404s**; the contract is
readable with a repo token. Do not treat fleet / `code/hello` as this repo’s
GET. This design has **no** dated protection JSON pasted for
`code/gitlab-cursor-webhook` (unauthenticated GET is 401). Do **not** paste a
dated JSON from an unauthenticated session. Classify **(a)** / **(b)** /
**(c)** from `@PlasticDigits`’s dated GET of **this** repo’s `main` rule on
leftover `{iid}` **and** on `pulls/3`. If that GET is **(c)**, take the
architecture executable sequence. If it is **(a)** or **(b)**, **stop**. Do
**not** treat the leftover request on `#3` as non-blocking from fleet
values. It is non-blocking **only after** `@PlasticDigits` **comments** that
dated GET JSON showing **G3-9**, **G3-10**, and **G3-2**. If **G3-9 ≠ 0** or
**G3-10** is still `true` here, or `enable_push != false`, or that JSON is
missing, do not land `#3`; record a named local/host DEPS. That leftover
request also must not be treated as S3 evidence.

Incomplete product tip (not this design commit; PR `#3` is `draft: false`):
`022f4f510113c6e8fcfd973a0869753a6fb375be` on `chore/remove-catchall-codeowners`
deletes the six-line file and does not touch README or copy `docs/`. Incomplete
without S0 files and S2 on that PR (or a successor that `Fixes #3` and closes
or supersedes `pulls/3`). That commit has empty
commit statuses (`[]`). Design SHA `9f8dec8` also had empty statuses (`[]`).
Those empty statuses are the **CI gap** only, not protection flags.

`origin/main` already has `docs/` runbooks (`admin-golden-image.md`,
`docker-deploy.md`, examples) and **no** `docs/architecture.md` or
`docs/adr/`. Relative README links to ADR 0001 / architecture 404 unless those
two files land on the **same merged tip** as the README pointer. Do not rewrite
those runbooks.

Issue body points at forgejo `docs/INVARIANTS.md`. Unauthenticated HTML
404s. The six-row table is this repo’s host target (readable with a repo
token). Forge INVARIANTS item 2 (“No force-push”) is out of `#3`. Do not fork
INVARIANTS into this repo.

Open PRs in this repo when #3 was filed:
[#2](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/2) (Renovate
onboarding) and #3. Renovate benefits from fewer planted reviews; it is not a
sequencing dep. Closed [#1](https://git.cl8y.com/code/gitlab-cursor-webhook/issues/1)
is unrelated.

This controller provisions Hetzner VMs. This Coolify app **may** rebuild if
the existing host is git-follow on `main` (not verified in-tree:
`docker-compose.yml` and `docs/docker-deploy.md` only record Dockerfile deploy
plus `/var/lib/gch`; they do **not** record git-follow, auto-deploy,
rebuild-on-`main`, or Coolify builds on `pull_request` / non-`main`). Landing
#3 is still not a #297 deploy grant: the product-PR diff must not change image,
compose, Terraform, tokens, or auto-deploy. Predecessor `{w}` must not mint a
Coolify / Hetzner deploy either (Decision 7).

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
  #3 are **#429**, not a #3 failure. CAC #429 does not merge `#3` **or**
  plant-check `{n}`.
- Dismissing reviewers from the controller (forbidden substitute in #388).
- Path-specific CODEOWNERS, a second maintainer, or `required_approvals: 1`.
- Adding, enabling, or digest-pinning Woodpecker **in the #3 diff**. Missing
  `ci/woodpecker/pr/woodpecker` statuses are pre-existing. Do not add
  `.woodpecker.yaml` / `.woodpecker/` in the #3 diff. Observed **(c)** uses
  the architecture executable sequence (predecessor `{w}` authored and
  merged by `@PlasticDigits`, then a **new push** of `#3`); not this diff.
  `{w}` adds **only** the pinned root `.woodpecker.yaml` in architecture
  executable **(c)** step 1 (`when` is **only** `pull_request`; required
  gitleaks against existing `.gitleaks.toml`; optional `cargo test` /
  `clippy` only on `rust:1.88-bookworm`). `{w}` must not add Terraform,
  Coolify, Hetzner, compose, tokens, auto-deploy, `echo`-only commands, or
  Coolify-on-`push`/`main`. ACCEPT of `#3` must not mint a deploy via `{w}`.
  Do not copy `code/hello`’s pipeline. Do not clear **G3-8** / **G3-3** to
  land `#3`. Land of `{w}` fails review if the yaml is not that shape.
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
   On the product PR (or a successor that `Fixes #3` and closes or
   supersedes `pulls/3`), copy
   `docs/adr/0001-remove-catchall-codeowners.md` and `docs/architecture.md`
   from **the independently accepted hex** onto that **same tip**, then add a
   short README pointer. README remains the product overview (not a stub).
   Point at `docs/architecture.md` **only** for the merge gate (**G3**), and at
   this ADR for the delete decision. Do not imply CODEOWNERS is what makes
   merge trusted. Do not merge a README that points at those paths until they
   exist on that tip. The two `docs/` files on the product tip must satisfy
   `git diff <accepted> --` those two paths empty. Do not rewrite runbooks.
4. **Leave** already-planted official requests on open PRs (including #3,
   #2, and predecessor `{w}` once opened). `{w}` is opened after I6
   (**G3-10**=false) while `CODEOWNERS` is still on `main`, so it will be
   planted and host-mergeable once Woodpecker is green. Treat `#3` / `#2` /
   `{w}` leftover official requests as non-blocking **only after** repo
   admin **comments** dated GET JSON of `code/gitlab-cursor-webhook` `main`
   showing **G3-9**, **G3-10**, and **G3-2** (`enable_push == false`) on
   leftover `{iid}` **and** on `pulls/3` (same I6 attest; no CAC dismiss).
   Human dismiss is optional leftover, not AC. If **G3-9 ≠ 0** or
   **G3-10** is `true` or `enable_push != false` or the GET is missing,
   stop; record a named local/host DEPS. Do not `force_merge`. **S2 never
   merges `{w}`.** CAC #429 does not merge `{w}`.
5. **Do not** PATCH branch protection from this repository.
6. **Split land from leftover-complete.** Product PR `#3` (or a successor
   that `Fixes #3` and closes or supersedes `pulls/3`) is
   S0+S1+S2 (vehicle **B**). S3 lives on leftover `{iid}` S2 opens in this repo
   before that merge (template below), with two Forgejo `@login`s recorded
   before merge. Closing `#3` does not assign S3. Require a **dedicated**
   post-merge plant-check PR. Do not accept `#3`’s own official request, `#2`,
   or “the next natural PR.”
7. **One Woodpecker land rule** (classify **(a)** / **(b)** / **(c)**; do
   not collapse **(a)** and **(b)**). Full text:
   [`architecture.md`](../architecture.md) “One Woodpecker land rule”. Same
   procedure as Tests item 9, land criterion 7, the **land-of-`#3`**
   diagram, and the `pulls/3` body. Standing remaining CI gate stays
   Woodpecker `ci/woodpecker/pr/woodpecker` (issue `#3` body). Repo admin
   dated GET JSON of this repo’s `main` rule (S2 does not GET):
   - **(a)** `enable_status_check != true` or field absent → **G3-8** drift.
     Record a **named** **G3-8** DEPS. **Stop. Do not `Do: merge`.** **This
     S2 session is complete** (wait-table item 1). Do not wait for `{w}`.
     Distinct from **(b)**. Not a rewrite of the issue body.
   - **(b)** `true` and `status_check_contexts` **not** equal to
     `["ci/woodpecker/pr/woodpecker"]` → **G3-3** drift. Record a **named**
     **G3-3** DEPS. **Stop. Do not `Do: merge`.** **This S2 session is
     complete** (wait-table item 1). Do not wait for `{w}`. Distinct from
     **(a)**. Do not merge after the DEPS iid alone, and do not merge after
     observed-context success either.
   - **(c)** `true` **and** contexts equal that array. Host requires
     `ci/woodpecker/pr/woodpecker` on the product-tip SHA. Merge of `#3`
     **only** under **(c)**. Executable sequence (architecture steps 1–4,
     then **G3-4**): `@PlasticDigits` activates this repo in Woodpecker
     (Allow pull requests on, agent online; retrigger if `{w}` opened
     first) → predecessor `{w}` that **adds** only the pinned root
     `.woodpecker.yaml` (architecture step 1: `when` is **only**
     `pull_request`; required gitleaks against existing `.gitleaks.toml`;
     optional `cargo test` / `clippy` only on `rust:1.88-bookworm`; no
     `echo`-only, `coolify`, `deploy`, Terraform, Hetzner, compose, tokens,
     auto-deploy, or Coolify-on-`push`/`main`; land of `{w}` fails review
     if not that shape; not in the `#3` diff; do not copy `code/hello`) →
     `@PlasticDigits` records `{w}` on `pulls/3` when opening it →
     Woodpecker posts onto `{w}` as exactly `ci/woodpecker/pr/woodpecker` →
     `@PlasticDigits` merges `{w}` under statuses GET success (S2 never
     merges `{w}`; CAC #429 does not merge `{w}`; `{w}`’s planted official
     request is non-blocking only under the same I6 attest) →
     `@PlasticDigits` comments that `{w}` is on `main` so S2 can resume
     (wait-table item 2; **(c)** only) → S2 **new push** of `#3` (rebase
     onto `main`; Woodpecker reads the **tree** of the new SHA, not the
     diff vs `main`; do not merge `022f4f5`; empty-commit of `022f4f5`
     will not post) →
     `GET .../statuses/{new-product-tip-sha}` success → `@PlasticDigits`
     SHA-pinned `Do: merge`. Do not clear **G3-8** / **G3-3**. Do not add
     `.woodpecker.yaml` in the `#3` diff. Do not fake statuses. Do not
     `force_merge`. A leftover “enable/post” issue is **not** sufficient
     DEPS. Activation deadlock: architecture step 2 — comment **deadlock**
     on `pulls/3`; that S2 session is complete. No **may**.

   **S2 wait table** (same as architecture **One Woodpecker land rule**).
   After leftover `{iid}` + this Decision 7 comment, S2 waits for **one**
   of:

   1. Dated GET classified **(a)** or **(b)** and a named **G3-8** /
      **G3-3** DEPS iid on `pulls/3` → **this S2 session is complete**. Do
      not wait for `{w}`. Do not merge `#3`.
   2. `{w}` is on `main` → resume rebase (**(c)** only).

   Deadlock comment on `pulls/3` completes this S2 session the same way as
   item 1. **(a)** / **(b)** / deadlock is a terminal stop for **this**
   land attempt, not abandon-`#3`, and not a hang. After that DEPS is
   resolved: `@PlasticDigits` **re-GETs** this repo’s `main` rule onto
   leftover `{iid}` and `pulls/3`. If still **(a)** / **(b)**, stop again.
   If **(c)**, run steps 1–4, then comment `{w}` is on `main`. New S2
   session only then.

   S2 comments this same procedure onto `pulls/3` (issue ≡ PR body).
   **S2 never merges `#3` or `{w}`.** The **land-of-`#3`** diagram is this
   same procedure (I6 protection GET fail-close; **(c)** is `{w}` statuses
   success → rebase → statuses GET → **G3-4**).

## Actors

Split so S2 cannot skip the land GET. “Implementer” is S2. **S2 never
merges `#3` or `{w}`.**

- **S2:** copy S0 files, delete `CODEOWNERS`, README pointer; open leftover
  `{iid}` (`iid != 3`, `pull_request` absent) with two `@login`s and pasted
  protection endpoints; comment Decision 7 onto `pulls/3`; **do not GET**
  protection; **never merge `#3` or `{w}`**. After leftover `{iid}` exists
  and S2 has commented Decision 7, **stop**. **S2 wait table** (architecture
  **One Woodpecker land rule**): wait for **one** of (1) dated GET
  classified **(a)** or **(b)** and a named **G3-8** / **G3-3** DEPS iid on
  `pulls/3` → **this S2 session is complete** (do not wait for `{w}`; do
  not merge `#3`); (2) `{w}` is on `main` → resume rebase (**(c)** only).
  Deadlock comment on `pulls/3` completes this session like item 1.
  `@PlasticDigits` does SHA-pinned `Do: merge` of `#3`.
- **`@PlasticDigits`** (repo admin of `code/gitlab-cursor-webhook`): **comment**
  dated GET JSON of the `main` rule for **G3-9**, **G3-10**, and **G3-2**
  (same JSON answers observed **G3-8** / **G3-3**) on leftover `{iid}`
  **and** on `pulls/3`. Unauthenticated GET is 401; in-repo CI cannot do
  this. Activates this repo in Woodpecker (Allow pull requests on, agent
  online; retrigger if `{w}` opened first). Opens predecessor `{w}`,
  records `{w}` on `pulls/3` when opening it, and merges `{w}` under
  Decision 7 **(c)** (architecture executable sequence; **S2 never merges
  `{w}`**; CAC #429 does not merge `{w}`). `{w}`’s planted official request
  is non-blocking only under the same I6 attest. Comments on `pulls/3` when
  `{w}` is on `main` so S2 can resume (wait-table item 2; **(c)** only).
  After **(a)** / **(b)** / deadlock DEPS is resolved, **re-GET** this
  repo’s `main` rule onto leftover `{iid}` and `pulls/3`; if still **(a)**
  / **(b)**, stop again; if **(c)**, run steps 1–4 then comment `{w}` is on
  `main`. After Integration items 5, 6, and 7 (item 7 **is** Decision 7
  **(c)**: `{w}` statuses success → rebase → statuses GET),
  `@PlasticDigits` performs SHA-pinned `Do: merge` of `#3` (**G3-4**;
  architecture step 5). CAC #429 does not merge `#3`, `{w}`,
  or plant-check `{n}`.
- **S3 owners** (two Forgejo `@login`s on leftover `{iid}` **before** merge of
  `#3`; closing `#3` does not assign S3):
  1. `@PlasticDigits` — protection GET comment.
  2. `@lifejkskla` — plant-check `{n}` + **G3-1 last**. Owner 2 **must be
     able to open a PR** in this repo. S2 **must** replace this line with
     another real `@login` that can open a PR here **before merge** if
     `@lifejkskla` cannot. A post-merge replacement comment on leftover
     `{iid}` is allowed if the login later cannot open `{n}`. Land of `#3`
     still does not wait on S3. Land fails if either line still contains
     `repo admin of`, `GCH implementer`, `<Forgejo`, or lacks a `@login`.

## Component / state / interface changes

| Surface | Change |
| --- | --- |
| `CODEOWNERS` (root) | Remove file. |
| `docs/CODEOWNERS`, `.gitea/CODEOWNERS`, `.forgejo/CODEOWNERS` | Must remain absent (no empty file). |
| `docs/adr/0001-remove-catchall-codeowners.md`, `docs/architecture.md` | Copy from the independently accepted hex onto the product PR so the merged tip is S0+S1+S2 (`git diff <accepted> --` empty). Standing G3 contract lands with the delete. |
| Forgejo PR review interface | After land, a **dedicated** plant-check PR against `main` must not get an official CODEOWNERS team request. |
| Branch protection API | No write from this ticket. The six-row table is **this repo’s host target**, not forge INVARIANTS **2–7**, not a measured GET pasted here, not the `#3` body. `#3` does **not** own force-push (named #48). S2 does not GET. `@PlasticDigits` **comments** dated GET JSON: land fail-closes on **G3-9** / **G3-10** and on `enable_push != false` (**G3-2**); leftover-complete records observed JSON plus vs-target diff plus plant-check after **(c)** land. |
| `.woodpecker.yaml` / `.woodpecker/` | Must remain absent in the #3 diff. `{w}` adds **only** the pinned root `.woodpecker.yaml` (architecture executable **(c)** step 1) on a separate tree. Observed **(c)** uses that sequence, not this diff. |
| `.gitlab-ci.yml` | Unchanged. |
| Rust crates, Terraform, Docker, `.env.example`, gitleaks | Unchanged in the `#3` diff. `{w}` required step is gitleaks against existing `.gitleaks.toml`; optional `cargo test` / `clippy` only on `rust:1.88-bookworm`. `{w}` must not change Terraform / Docker / tokens. |
| README | Mandatory on the product PR: merge gate is **G3**, documented in `docs/architecture.md`; product map stays README. Relative links to ADR 0001 / architecture, which exist on that same tip. Keep the product overview. |
| Runbooks under `docs/` | Unchanged except adding `adr/` + `architecture.md`. Plant-check may add a throwaway non-runbook path under `docs/` (Tests item 4). |
| Leftover `{iid}` | New Forgejo issue (`pull_request` absent, `iid != 3`) in `code/gitlab-cursor-webhook` only, opened before merge of `#3`, two `@login`s, body quotes leftover-complete items 1–3. Never call `#3` the leftover. |
| Named Woodpecker / drift DEPS | Predecessor `{w}` under Decision 7 **(c)** (authored, recorded on `pulls/3`, and merged by `@PlasticDigits`; S2 never merges `{w}`; operator Woodpecker activation is a precondition, not leftover `{iid}`); named **G3-8** DEPS if **(a)**; named **G3-3** DEPS if **(b)**; named infra DEPS if Woodpecker ACL/server is outside this repo (deadlock); **G3-9**/**G3-10**/**G3-2** mismatch. Not leftover `{iid}`. Not a ticket that clears **G3-8** / **G3-3**. |
| `pulls/3` body | S2 comments the Decision 7 procedure (same three cases + wait table). `@PlasticDigits` records `{w}` when opening it, comments **deadlock** if statuses stay `[]`/`pending` after activate, and comments when `{w}` is on `main` (**(c)** only). |
| CAC / Coolify / org team `maintainers` in org `code` | Unchanged. The team may keep existing; it simply is not planted as official review. Coolify **may** rebuild if the host is git-follow on `main` (not verified in-tree; `pull_request` / non-`main` also unverified); that is not a new deploy grant. `{w}` must not add a Coolify deploy step. CAC #429 does not merge `#3`, `{w}`, or plant-check `{n}`. |

No runtime state, schema, or HTTP API.

## Affected invariants

IDs live in [`architecture.md`](../architecture.md). This ADR changes **G3-1**
(four-path absence). It does not write protection JSON. The six-row table is
**this repo’s host target**, not forge INVARIANTS **2–7**, not the `#3` body.
Force-push stays #48. Land fail-closes on **G3-9** / **G3-10** and on
`enable_push != false` (**G3-2**) via repo-admin attest (not S2 GET).
Leftover-complete records observed JSON plus a written vs-target diff
**after (c)** land, plus plant-check, plus **G3-1 last**. Merge procedure
remains **G3-4** (`@PlasticDigits`; S2 never merges `#3`). Standing **G3-4**
after land does not keep Integration 5–7 as the forever contract. Coolify
**may** rebuild if git-follow on `main` (**G3-6**, not verified in-tree;
`pull_request` / non-`main` also unverified). CAC policy remains **G3-7**.
**G3-2** target is `enable_push == false` (issue bullet “no direct `main`”);
`#3` does **not** own force-push (INVARIANTS item 2 / named #48).
Leftover-complete does not require a **G3-2** match for S3 pass
(record-not-fail on post-land drift).

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
| Add `.woodpecker.yaml` in the same PR | Different change (CI enablement). Missing statuses are pre-existing. Observed **(c)** unsticks via predecessor `{w}` authored and merged by `@PlasticDigits` (pinned architecture step-1 yaml), then a new push of `#3`, not this diff. |
| Copy `code/hello` `.woodpecker.yaml` onto `{w}` | Fails here (no `.opengrep.yml`) and, after `{w}` merges, a `main` push can hit Coolify (#297). `{w}` is the pinned architecture step-1 yaml only (`when` is **only** `pull_request`; required gitleaks; no Terraform / Coolify / Hetzner / compose / tokens / auto-deploy / `echo`-only). |
| Noop `echo` pipeline on `{w}` | Makes the standing Woodpecker gate a paper check after CODEOWNERS is gone. |
| `force_merge` or fake Woodpecker statuses to land #3 | Forbidden by **G3-4** / **G3-3**. |
| Dispatch land on “**G3-8** is `false` or status checks unset” | Collapses **(a)** and **(b)**. Classify only **(a)** / **(b)** / **(c)**. |
| Named host ticket that clears **G3-8** / **G3-3** to land `#3` | Policy expansion / #297 / contradicts Rollout. Forbidden. Unstick is Woodpecker success on the product-tip SHA, or stop. Clearing the check is forge `#48` / founder. |
| Treat a leftover “enable/post” issue as Woodpecker DEPS | Not sufficient under **(c)**. Unstick is predecessor `{w}` that actually posts, then `GET .../statuses/{sha}` success. Yaml-on-the-PR is not enough; activate Woodpecker / Allow PRs / agent first. |
| Merge under **(a)** or **(b)** | **Stop.** Record the named DEPS. Do not `Do: merge`. Land of `#3` is **(c)** only. |
| Merge under **(b)** after recording a DEPS iid only | Host **405** if observed `status_check_contexts` are not success, and land of `#3` still must not take **(b)**. |
| Rewrite the standing gate as “Woodpecker only under **(c)**” | Issue `#3` keeps Woodpecker as the remaining gate. **(a)** / **(b)** are distinct **stop** reasons for land of `#3`, not alternate merge paths. |
| Let S2 merge `#3` after leftover + admin comment + Decision 7 | **S2 never merges `#3`**. After leftover `{iid}` + Decision 7 comment, wait table: **(a)** / **(b)** + named DEPS iid → this S2 session is complete (do not wait for `{w}`); `{w}` on `main` → resume rebase (**(c)** only). `@PlasticDigits` does SHA-pinned `Do: merge`. |
| Let S2 or CAC #429 merge `{w}` | **S2 never merges `{w}`.** CAC #429 does not merge `{w}`. Merger is `@PlasticDigits` after statuses GET success on `{w}`-tip. `{w}`’s planted official request is non-blocking only under the same I6 attest. |
| Treat empty draft plant-check GET as leftover-complete; open `{n}` ready-first; or undraft and wait 30s | Forgejo skips CODEOWNERS while `pr.IsWorkInProgress()`; API `draft` follows the title WIP prefix. Empty draft signals are the skip, not leftover-complete. `#3`’s known-plant sample is `"draft": false`. **Mandatory sandwich:** open **WIP/draft** → strip WIP / mark ready → **GET immediately** (pass GET `draft == false`, no WIP prefix) → **re-apply WIP immediately** → close unmerged in the same session. Do **not** open ready-first. Record the **ready** GET pair, not the re-WIP GET. Residual race ready → re-WIP is still open to a poller; `do-not-merge` is not a host-block; CAC #429 prose is not a mutex. If `{n}` merges, leftover-complete **fails**; revert via PR. No 30s wait. |
| Treat `#3`’s plant, `#2`, or the next natural PR as S3 | Merge closes `#3` before leftover-complete. A dedicated post-merge PR is the evidence. |
| Vehicle **A**: docs-only PR from `cac-design-issue-3` onto `main`, then `#3` as S1+S2 | Second merge vehicle. That branch is design transport, not a product PR. Vehicle **B** puts the two files on the deletion PR so README links resolve on one tip. Incomplete product tip `022f4f5` is not that tip until S0 files and S2 are added. |
| Wait on sibling `code/*` CODEOWNERS PRs / hello#15 | Wrong repo; no product iid dependency. |
| Open leftover `{iid}` in `PlasticDigits/*` or leave it without two `@login`s | Land criterion 5 would pass a wrong-repo or vacant issue. S2 names this repo and two `@login`s. |
| Treat Coolify rebuild after merge as leftover-complete | **G3-6**. Plant-check and observed-vs-target record are leftover-complete. A rebuild is not a #297 grant. |
| Merge plant-check `{n}` and call leftover-complete a pass | Leftover-complete **fails** (possible host rebuild / #297 incident). Close without merge in the same session; recovery under Tests item 4. |
| Let S2 GET protection or skip admin attest | S2 cannot GET (401). Skipping land fail-close on **G3-9** / **G3-10** / **G3-2** can merge into 405 or leave direct `main`. |
| Write the archival `PlasticDigits/gitlab-cursor-webhook` clone | CAC invariant 21. Wrong repo. |
| Require dated protection JSON in S0 | Freezes design behind a token this pass does not have. That GET is leftover/admin work. |
| Call leftover-complete a “prove” of **G3-2** / **G3-8** / **G3-3** / **G3-5** | Drift must not fail S3. Record JSON + vs-target diff instead. |
| `Fixes` / `Closes` leftover or Woodpecker iids on PR `#3` | Merge would auto-close S3 / unstick tickets. Ban those trailers. A successor of `pulls/3` may `Fixes #3` **and** must close or supersede `pulls/3`; without that, a successor does not close `#3`. |

## Complexity added / removed

**Removed:** catch-all official-review robot on every diff; operator dismiss
step; false “CODEOWNERS is the trusted-PR gate” story in this repo.

**Added:** a small standing doc (this ADR + architecture **G3**) that lands on
`main` via the product PR so later agents do not re-add `.* @code/maintainers`
as a merge requirement, and a leftover Forgejo issue in this repo that
survives merge of `#3`. No new services, jobs, flags, or test harnesses. No
new pipelines **in the `#3` diff**. Predecessor `{w}` adds the pinned
architecture step-1 Woodpecker file on `main`. Named predecessor
`{w}` / **G3-8** / **G3-3** / **G3-9**/**G3-10**/**G3-2** DEPS only when
observed JSON requires them. Do not add a ticket that clears **G3-8** /
**G3-3**.

## Migration

1. Fleet protection is owned by forge #48. This ticket does not PATCH. Do not
   treat a GET recorded on `code/hello` as proof for this repo. The six-row
   table is **this repo’s host target**; leftover-complete records **observed**
   JSON plus a vs-target diff.
2. **Leftover `{iid}` (S2, land gate).** Before merging the product PR, open
   **one** follow-up Forgejo issue in **`code/gitlab-cursor-webhook` only**.
   Do not open it in `PlasticDigits/cl8y-forgejo`,
   `PlasticDigits/cl8y-agent-control`,
   `PlasticDigits/gitlab-cursor-webhook`, or any other repo. Suggested title:
   `chore: leftover CODEOWNERS S3`. Title substring `Forgejo issue, not PR`
   is **not** a land gate. Body **must** name two `@login`s (below) and
   **quote** leftover-complete items 1–3 from this ADR. Record `{iid}` on
   `pulls/3` before merge. Prove with
   `GET /api/v1/repos/code/gitlab-cursor-webhook/issues/{iid}`:
   `pull_request` absent, `iid != 3`, repo `code/gitlab-cursor-webhook`.
   Merging [#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3)
   closes that number; closing `#3` does not assign S3. Never call `#3` the
   leftover. Do not put `Fixes` / `Closes` `{iid}` on PR `#3`.

   Body template (quote onto leftover `{iid}`):

   ```
   Leftover-complete for ADR 0001 after merge of product PR #3.
   This issue does not close #3. Closing #3 does not assign S3.
   Opened in code/gitlab-cursor-webhook before that merge.
   GET .../issues/{iid} must show pull_request absent, iid != 3.

   Owners (required before merge of #3; two Forgejo @logins; land fails if a
   placeholder remains):
   1. Protection GET comment: @PlasticDigits
   2. Plant-check {n} + G3-1 last: @lifejkskla
      Owner 2 must be able to open a PR in this repo. S2 must replace this
      login before merge if it cannot. A post-merge replacement comment on
      this issue is allowed. Land of #3 does not wait on S3.

   S2 pasted these endpoints; S2 does not GET; S2 never merges #3:
   - GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections
   - GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections/main

   Repo admin comments dated GET JSON for G3-9, G3-10, and G3-2
   (enable_push == false) (same JSON answers observed G3-8 and G3-3 for
   Decision 7) on this issue AND on pulls/3 before merge. Do not POST
   protection. If G3-9 != 0 or G3-10 is true or enable_push != false,
   that is a named local/host DEPS (not "wait on #48" with no ticket).

   1. Dated operator GET of this repo's `main` protection rule: comment
      observed JSON here. Not copied from code/hello. Write a vs-target
      diff against the six-row host target in docs/architecture.md.
      Leftover-complete records that JSON + diff. Drift is a #48 leftover,
      not a GCH PATCH, not a reason to restore CODEOWNERS, and not a
      leftover-complete fail. A GET recorded before a later template
      re-copy does not count.

   2. After the delete is on `main`: dedicated plant-check PR {n}
      (throwaway path under docs/ that is not a runbook, example
      docs/_plant-check-adr0001.md; no manual reviewer request). Forbid
      .env.example, Terraform, and Docker as the probe. Mandatory sandwich:
      open WIP/draft → strip WIP / mark ready → GET immediately (pass GET
      draft==false, no WIP prefix) → re-apply WIP immediately → close
      unmerged in the same session. Do not open ready-first. Comment
      `do-not-merge`. Record the ready GET pair (pulls/{n} and
      pulls/{n}/reviews), not the re-WIP GET. Pass iff
      requested_reviewers_teams length 0 AND no review with official == true,
      state == "REQUEST_REVIEW", team.name == "maintainers" (optional
      team.id == 4) on that ready pair. Empty draft GET is not a pass.
      Present on draft still fails. Do not require team.organization on
      reviews. Residual race ready → re-WIP is still open to a poller;
      do-not-merge is not a host-block; CAC #429 prose is not a mutex. If
      {n} is merged, leftover-complete fails (possible host rebuild / #297
      incident); revert the throwaway path via PR (not direct main). It is
      not a pass with a Coolify rebuild. Not #3, not #2, not the next
      natural PR.

   3. G3-1 last: test -f fails on CODEOWNERS, docs/CODEOWNERS,
      .gitea/CODEOWNERS, .forgejo/CODEOWNERS. Recorded after items 1 and 2.
      Owner: @login on line 2.
   ```

3. **Vehicle B.** Land
   [#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) whose tip
   is S0+S1+S2: copy
   `docs/adr/0001-remove-catchall-codeowners.md` and `docs/architecture.md`
   from **the independently accepted hex**, delete root `CODEOWNERS`, add the
   README pointer with relative links to those two paths. A successor PR is
   allowed only if it `Fixes #3` **and** closes or supersedes `pulls/3`.
   `cac-design-issue-3`
   stays the review/transport branch; do not open it as the product PR; do not
   merge it as docs-only first. Do not merge a README that points at those
   paths until they exist on that tip. After ACCEPT, pin that hex on leftover
   `{iid}` and on `pulls/3`; `git diff <accepted> --` the two `docs/` files
   must be empty on the product tip. Incomplete product tip `022f4f5` is not
   that tip until S0 files and S2 are added.
4. Open PRs created while the file existed (#3, #2, and `{w}` once opened)
   still show an official team request. Non-blocking **only after** the
   admin I6 comment in Actors (same attest for `{w}`; no CAC dismiss).
   No bulk dismiss required to land `#3` under that attest. If **G3-9 ≠ 0** or
   **G3-10** is `true` or `enable_push != false` or the GET is missing, stop;
   record named DEPS; merge is 405, still review-gated, or would allow
   direct `main`. **S2 never merges `{w}`.**
5. Do not restore the file from `docs/templates/CODEOWNERS` in cl8y-forgejo;
   that template is owned by #48.
6. Woodpecker: Decision 7 only **(a)**/**(b)**/**(c)**. Classify; do not
   collapse **(a)** and **(b)**. Merge of `#3` only under **(c)**. Do not
   restore catch-all CODEOWNERS. Do not `force_merge`. Do not add
   `.woodpecker.yaml` in the #3 diff. Do not clear **G3-8** / **G3-3**. S2
   comments the same procedure on `pulls/3`. S2 never merges `#3` or `{w}`.
   S2 wait table after leftover `{iid}` + Decision 7.

## Observability

Relative reads. Do not log tokens, hosts, or protection-script inventories. Do
not add a Forgejo admin token to Woodpecker, cargo tests, or Coolify.

**Land GET (G3-9 / G3-10 / G3-2, admin attest).** Always applies. Before
merging `#3`, **repo admin** of `code/gitlab-cursor-webhook` **comments**
dated GET JSON of the `main` rule showing **G3-9** (`required_approvals == 0`),
**G3-10** (`block_on_official_review_requests == false`), and **G3-2**
(`enable_push == false`) on leftover `{iid}` **and** on `pulls/3`. Same
endpoints as leftover-complete. S2 pastes the endpoints and does not GET.
Fleet #48 / `code/hello` is not this GET. Fail closed if the JSON is missing,
if **G3-9 ≠ 0**, if **G3-10** is `true`, or if `enable_push != false`; that
mismatch is a named local/host DEPS (this-repo issue or cl8y-forgejo iid —
not “wait on #48” with no ticket). Do not `force_merge`. The same JSON
answers observed **G3-8** / **G3-3** for Decision 7.

**Protection (operator, leftover-complete).** Repo admin of
`code/gitlab-cursor-webhook` **comments** dated **observed** JSON of the
`main` rule onto leftover `{iid}` in this repo.
`GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections` (array; pick
`rule_name == "main"`) or
`GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections/main`.
Leftover-complete **records** that JSON and a written vs-target diff of the
six flags. Drift vs target is a #48 leftover, not a GCH PATCH, not a
leftover-complete fail, not a reason to restore `CODEOWNERS`. Do not compare
the Merge API / **G3-4** row (not a protection field). Do not treat
`enable_push == false` as a leftover-complete pass predicate. A green
`cargo test`, empty commit statuses, or a Coolify deploy does not satisfy
this read. In-repo CI cannot perform this GET. S2 cannot perform this GET.

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
this predicate; not S3 evidence). S3 must show a **new** PR of that shape
(`draft == false`, title with no WIP prefix) does **not** get `maintainers`
/ `official == true` `REQUEST_REVIEW`. Empty draft plant-signals are the
skip, not leftover-complete.

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
`main` by leftover owner 2. Recipe: Tests item 4. Probe a throwaway path
under `docs/` that is not a runbook (example:
`docs/_plant-check-adr0001.md`). Forbid `.env.example`, Terraform, and
Docker as the probe. After land of `#3`, **G3-9** is proven `0` and
**G3-10** proven `false`, so a ready PR against `main` can autoland;
`do-not-merge` is only a comment, not a host-block. CAC #429 prose is not
a mutex. **Mandatory sandwich:** open **WIP/draft** → strip WIP / mark
ready → **GET immediately** (pass GET `draft == false`, no WIP prefix) →
**re-apply WIP immediately** → close unmerged in the same session. Do
**not** open ready-first. Comment `do-not-merge`. Pass iff the fail-closed
pair holds on that **ready** pair. Record the **ready** GET pair, not the
re-WIP GET. Empty draft GET is **not** a pass. Present on draft still
**fails** leftover-complete. Residual race ready → re-WIP is still open
to a poller; if `{n}` is merged, leftover-complete **fails** (possible
host rebuild / #297 incident). Recovery: revert the throwaway path via a
new PR (not direct `main`); a Coolify rebuild from `{n}` is a #297
incident, not a GCH grant. In-tree compose / docker-deploy do not record
Coolify builds on `pull_request` / non-`main` (**G3-6**). It is not a pass
with a Coolify rebuild. PR `#3`’s own official request does not pass. `#2`
does not pass. “The next natural PR” does not pass. CAC #429 does not
merge `{n}`. No 30s wait.

**CI / deploy.** Land of `#3` uses the architecture **land-of-`#3`** diagram
only. Standing after land uses the **standing-after-land** diagram only.
Standing remaining CI gate is Woodpecker `ci/woodpecker/pr/woodpecker`. This
tree does not post Woodpecker today. Drain comments such as `drain skip: no
occupying job…` are **#429**, not a #3 failure. Coolify **may** rebuild if
the host is git-follow on `main` (**G3-6**, not verified in-tree;
`pull_request` / non-`main` also unverified); that is not leftover-complete.
`{w}` must not add a Coolify deploy step.

## Failure modes

| Mode | Handling |
| --- | --- |
| File deleted on a branch but still on `main` | New PRs keep planting official review until the product PR merges. Expected until land. |
| README links ADR/architecture but those files are not on the same tip | 404 after merge. Land fails criterion 2. Vehicle **B**: copy both files onto the product PR before merging README. |
| Copy left in `docs/`, `.gitea/`, or `.forgejo/` (including empty/comments-only) | Forgejo still loads the first existing path and may plant. Land fails **G3-1**; delete those paths too (none exist on current `main`). |
| Whole-tree `git grep` for `.* @` | Hits this ADR after a correct delete. Use the pathspec in Tests item 1. |
| cl8y-forgejo migrate/apply re-copies a template | Sister-repo race ([cl8y-forgejo#48](https://git.cl8y.com/PlasticDigits/cl8y-forgejo/issues/48) `_ensure_codeowners`). Out of this slice. If a later apply re-adds the file, leftover-complete is **not** done: delete again via PR, then one new dated leftover comment with all three leftover-complete items, **G3-1 last**. A GET or plant-check from before a re-copy does not count. Never direct-push `main`. |
| Plant-check → re-copy → stale-green GET | Void. Leftover-complete requires one dated leftover comment with (1) observed JSON + vs-target diff, (2) plant-check `{n}` + JSON, (3) four-path `test -f` fails — **G3-1 last** (or all three timestamps in one attest). |
| Official request leftover on #3, #2, or `{w}` | Non-blocking **only after** admin comments **G3-9** / **G3-10** / **G3-2** on leftover `{iid}` and `pulls/3` (same I6 attest for `{w}`; no CAC dismiss). If **G3-9 ≠ 0** or **G3-10** is `true` or `enable_push != false` or GET missing, do not land; named local/host DEPS; do not `force_merge`; optional human dismiss is not a CAC substitute. Not S3 evidence. Not a rollback signal. **S2 never merges `{w}`.** CAC #429 does not merge `{w}`. |
| Treating merge of `#3` as leftover-complete | Merge closes `#3` before S3. Closing `#3` does not assign S3. Use leftover `{iid}` in this repo. Never call `#3` the leftover. |
| Leftover `{iid}` opened in the wrong repo, is a PR, is iid 3, or lacks two `@login`s | Land fails criterion 5. `GET .../issues/{iid}` must show `pull_request` absent, `iid != 3`, repo `code/gitlab-cursor-webhook`, two `@login`s. |
| Plant-check matcher uses `team.name == "code/maintainers"` or requires `team.organization` on reviews GET | Misses the live plant (`team.name == "maintainers"`, `team.organization == null` on reviews). Use the Observability fail-closed pair. |
| Plant-check pass GET is still `draft == true` or title still contains `WIP`; leftover-complete claimed from an empty draft GET; `{n}` opened ready-first; probe is `.env.example` / Terraform / Docker / a runbook; no changed file; reviewers requested in the UI / `POST .../requested_reviewers`; or no `do-not-merge` comment | False pass (CODEOWNERS skipped while `pr.IsWorkInProgress()`) or false fail (manual team request). **Mandatory sandwich** (do not open ready-first). Residual race ready → re-WIP is still open to a poller; CAC #429 prose is not a mutex. Recipe fails closed; open a new probe. |
| Leaving plant-check `{n}` open ready | `do-not-merge` is not a host-block (**G3-9**=0, **G3-10**=false). Sandwich re-applies WIP then closes unmerged in the same session. |
| Merging the plant-check PR `{n}` | Leftover-complete **fails** (possible host rebuild / #297 incident). Recovery: revert throwaway path via PR; Coolify rebuild from `{n}` is #297. Close without merge in the same session. |
| Protection silently reverted to official-review true | Merge 405 returns. Out of this repo; re-apply via forge policy, do not `force_merge`. Not proven by scanners. |
| `enable_push` flipped true | **G3-2** regression (direct push). Land of `#3` fail-closes if the dated GET shows `enable_push != false`. After land, refuse on later PRs. Not a leftover-complete fail (record-not-fail on post-land drift). Not a force-push claim. |
| Observed **(c)** and nothing posts Woodpecker | Owner of activation remains `@PlasticDigits`. If after activate + Allow PRs + agent online + retrigger, `GET .../statuses/{w-tip-sha}` is still `[]` or `pending`: comment **deadlock** on `pulls/3`. That S2 session is **complete** (same wait table as **(a)** / **(b)**: do not wait for `{w}`; do not merge `#3`). If Woodpecker ACL/server is outside this repo, record a **named** infra DEPS (not leftover `{iid}`, not a ticket that clears **G3-8** / **G3-3**). Land stays blocked until statuses success. `{w}` yaml is the pinned architecture step-1 file. A leftover “enable/post” issue is not sufficient DEPS. Not solved by restoring CODEOWNERS. Do not fake statuses. Do not add `.woodpecker.yaml` in the #3 diff. Do not clear **G3-8** / **G3-3**. |
| Merge `#3` under **(a)** | Violates Decision 7. Record named **G3-8** DEPS. **Stop. Do not `Do: merge`.** |
| Merge `#3` under **(b)** | Violates Decision 7. Record named **G3-3** DEPS. **Stop. Do not `Do: merge`.** |
| Re-adding CODEOWNERS “for safety” in a follow-up | Violates **G3-1**. Reviewers must reject unless a new ADR allowlists path owners. |
| Observed flags differ from the six-row host target | #48 leftover. Not a GCH PATCH. Not a reason to restore `CODEOWNERS`. Not a leftover-complete fail. Land blocker for **G3-9** / **G3-10** / **G3-2**. |
| Rust / Terraform / Docker / token / runbook sneak into the MR | Fail review. |
| `{w}` copies `code/hello` deploy steps, attaches `push`/`main`, or a noop `echo` | `#297` (Coolify on `main` push) or a paper standing gate. `{w}` is the pinned architecture step-1 yaml only. Land of `{w}` fails review. |
| README treats `docs/architecture.md` as product architecture or stubs the product map | Fail S2. Two architecture docs must not collide. |
| Implement deploys Coolify, rotates tokens, or edits `autonomy.rs` / HMAC | Forbidden (#297). |
| S2 performs the protection GET or merges `#3` or `{w}` | Forbidden. S2 pastes endpoints and **never merges `#3` or `{w}`**. After leftover `{iid}` + Decision 7 comment, wait table: **(a)** / **(b)** / deadlock → this S2 session is complete; `{w}` on `main` → resume rebase (**(c)** only). |
| `Fixes` / `Closes` leftover `{iid}` or Woodpecker/unstick iids on PR `#3` | Ban. Merge must not auto-close S3. A successor may `Fixes #3` and must close or supersede `pulls/3`. |
| Successor PR without `Fixes #3` claimed to close `#3` | A successor iid does not close `#3` unless it `Fixes #3` or `pulls/3` is superseded. Drop the closes claim or require those trailers. |

## Ordered implementation slices

| Slice | Work | Depends on |
| --- | --- | --- |
| **S0** | This design (ADR 0001 + architecture **G3**). Transport on `cac-design-issue-3`. Copy **the independently accepted hex** onto the product tip; do not call this tip accepted until independent review says so. `cac-design-issue-3` is never the merge vehicle. Do not GET protection in S0. | None in `code/gitlab-cursor-webhook`. |
| **S1** | Delete root `CODEOWNERS`. Confirm `test -f` fails on all four Forgejo paths. | S0 files present on the **same product-PR tip** (not “S0 accepted” alone). Incomplete product tip already on `chore/remove-catchall-codeowners`. |
| **S2** | README pointer on that **same** tip: keep README as product map; point at `docs/architecture.md` **only** for the merge gate (**G3**); relative links to ADR 0001 / architecture (those files already on the tip); do not imply CODEOWNERS is the trusted-merge gate. Keep the product overview. No Docker/Woodpecker/Terraform/Rust edits. Open leftover `{iid}` in **`code/gitlab-cursor-webhook` only**, before merge, with two `@login`s and the body template under Migration (paste the two protection endpoints). Owner 2 **must** be able to open a PR here; replace before merge if not. Comment Decision 7 onto `pulls/3`. Do not GET protection. **Never merge `#3` or `{w}`**. After leftover `{iid}` + Decision 7 comment, **stop**. **S2 wait table:** (1) dated GET **(a)** / **(b)** + named DEPS iid on `pulls/3` → this session complete (do not wait for `{w}`); (2) `{w}` on `main` → resume rebase (**(c)** only). Deadlock comment completes like (1). Do not `Fixes`/`Closes` leftover or unstick iids. | S0 files on the same tip as S1. Same PR as S1. Admin land attest is **not** an S2 implement step. |
| **S3** | Leftover-complete on leftover `{iid}`: one dated leftover comment with (1) observed JSON + vs-target diff, (2) dedicated plant-check `{n}` + JSON closed unmerged, (3) four-path `test -f` fails — **G3-1 last**. Owners: `@PlasticDigits` (1), leftover owner 2 (2+3). Does **not** close `#3`. Closing `#3` does not assign S3. | S0+S1+S2 merged to `main`. Tracked on leftover `{iid}` in this repo. |

PR `#3` ships **S0+S1+S2**. A successor is allowed only if it `Fixes #3`
**and** closes or supersedes `pulls/3`. S2 depends on S0 files being on the
same tip, not only on S1.

**`@PlasticDigits` (not a slice of implement):** **comment** dated GET JSON
for **G3-9**, **G3-10**, and **G3-2** on leftover `{iid}` **and** on
`pulls/3` before merge. That JSON also answers Decision 7 (**G3-8** /
**G3-3**). Activate Woodpecker; open `{w}` and record it on `pulls/3`;
merge `{w}` under **(c)** (**S2 never merges `{w}`**; CAC #429 does not
merge `{w}`); comment **deadlock** if statuses stay `[]`/`pending` after
activate; comment `{w}` is on `main` only under **(c)** (wait-table item
2). After **(a)** / **(b)** / deadlock DEPS is resolved, **re-GET**. Then
SHA-pinned `Do: merge` of `#3` after items 5, 6, and 7. **S2 never merges
`#3`.** CAC #429 does not merge `#3`, `{w}`, or plant-check `{n}`.

Merge of `#3` only under Decision 7 **(c)** (`{w}` statuses success → S2
rebase → statuses GET success → **G3-4**). **(a)** / **(b)**: wait-table
item 1, this S2 session complete, do not `Do: merge`. Do not add
`.woodpecker.yaml` in the #3 diff. Do not clear **G3-8** / **G3-3**.

Sister repos (not slices of #3, not local `DEPS` unless a named iid is
recorded under Decision 7 or **G3-9**/**G3-10**/**G3-2** mismatch): forge #48
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
   `code/gitlab-cursor-webhook` comments dated **observed** JSON of the `main`
   rule and writes a vs-target diff against the six-row **host target**.
   Leftover-complete records that JSON + diff. Drift is a #48 leftover, not a
   leftover-complete fail, not a GCH PATCH. Not inferred from a green scanner.
   Not compared to the Merge API row. Not copied from `code/hello`. Do not
   treat **G3-2** match (`enable_push == false`) as an S3 pass predicate.
   **G3-9** / **G3-10** / **G3-2** were already land-checked by admin attest.
4. **No new plant (leftover-complete).** After the delete is on `main`, leftover
   owner 2 runs this recipe on leftover `{iid}`:
   1. Delete already on `main` (**G3-1** still holds; if a later apply
      re-copied the file, delete again via PR first).
   2. Open a **dedicated** plant-check PR as **WIP/draft**. Do **not** open
      ready-first. Probe a throwaway path under `docs/` that is not a
      runbook (example: `docs/_plant-check-adr0001.md`). Forbid
      `.env.example`, Terraform, and Docker as the probe. Do not use a
      runbook as the probe.
   3. Do not request users or teams in the UI or via
      `POST .../requested_reviewers`.
   4. Comment `do-not-merge` (on open). **Mandatory sandwich:** strip WIP /
      mark ready → **GET immediately** `.../pulls/{n}` and
      `.../pulls/{n}/reviews` (pass GET `draft == false`, no WIP prefix) →
      **re-apply WIP immediately** → close unmerged in the same session.
   5. Pass GET must show `draft == false` and a title with no WIP prefix
      (case-insensitive). Pass iff Observability’s fail-closed pair holds
      on that **ready** pair. Record the **ready** GET pair, not the re-WIP
      GET. Empty draft GET is **not** a pass. Either plant signal
      **present on draft** still **fails** leftover-complete (file still
      plants). Residual race ready → re-WIP is still open to a poller;
      `do-not-merge` is not a host-block; CAC #429 prose is not a mutex.
   6. Record `{n}` **and** the two JSON bodies (the **ready** pair used for
      the pass decision) on leftover `{iid}`, then **close without merge in
      the same session** (WIP re-applied first). Do not leave `{n}` open
      ready.
   Fail if `{n}` was opened ready-first, if the pass GET is still
   `draft == true` or the title still contains `WIP` (case-insensitive), if
   leftover-complete is claimed from an empty draft GET or from the re-WIP
   GET, if the PR has no changed file, if the probe is `.env.example` /
   Terraform / Docker / a runbook, if reviewers were requested manually,
   if there is no `do-not-merge` comment, or if either GET signal is present
   on the ready pair (or present on draft). Do **not** wait 30s.
   If `{n}` is merged, leftover-complete **fails** (possible host rebuild /
   #297 incident). Recovery: revert the throwaway path via PR (not direct
   `main`); a Coolify rebuild from `{n}` is a #297 incident. In-tree compose
   / docker-deploy do not record Coolify builds on `pull_request` / non-`main`
   (**G3-6**). It is not a pass with a Coolify rebuild. Do not use `#3`.
   Do not use `#2`. Do not use “the next natural PR.” Who opens `{n}`: leftover
   owner 2 (must be able to open a PR here; S2 replaced before merge, or a
   post-merge replacement comment on leftover `{iid}`). Who GETs protection
   (item 3): repo admin; not S2; not cargo; not Coolify.
   Leftover-complete attest order is Integration: **G3-1 last**. Do not treat
   a passing GET or plant-check from before a later `_ensure_codeowners` apply
   as done.
5. **Reject still blocks (doc-level).** Do not turn off
   `block_on_rejected_reviews` to “make autoland easier.”
6. **Diff guard (land).** Product PR does not change Rust sources, Terraform,
   Docker, `.env.example`, runbooks, or add `.woodpecker.yaml`. Product PR
   body / commits must not `Fixes` / `Closes` leftover `{iid}` or Woodpecker /
   unstick iids. A successor of `pulls/3` may `Fixes #3` and must close or
   supersede `pulls/3`.
7. **Land GET (G3-9 / G3-10 / G3-2, always).** Repo admin **comments** dated
   GET JSON of this repo’s `main` rule showing **G3-9**, **G3-10**, and
   **G3-2** (`enable_push == false`) on leftover `{iid}` **and** on
   `pulls/3` before merge. S2 does not GET. Fail closed if missing, if
   **G3-9 ≠ 0**, if **G3-10** is `true`, or if `enable_push != false`. That
   mismatch is a named local/host DEPS. Do not `force_merge`.
8. **README collision (land, S2).** Product PR README still is the product map
   and points at `docs/architecture.md` only for the merge gate.
9. **Host context (Decision 7).** Same **(a)**/**(b)**/**(c)** as architecture
   “One Woodpecker land rule”. Classify only; do not collapse **(a)** and
   **(b)**. **S2 wait table** after leftover `{iid}` + Decision 7. **(a)**:
   named **G3-8** DEPS; **this S2 session is complete; do not wait for `{w}`;
   do not `Do: merge`.** **(b)**: named **G3-3** DEPS; **this S2 session is
   complete; do not wait for `{w}`; do not `Do: merge`.** **(c)**:
   architecture steps 1–4 — `@PlasticDigits` activates Woodpecker (Allow PRs
   on, agent online; owner of activation remains `@PlasticDigits`),
   predecessor `{w}` is the pinned root `.woodpecker.yaml` (`when` is
   **only** `pull_request`; required gitleaks; optional cargo only on
   `rust:1.88-bookworm`), `{w}` recorded on `pulls/3` when opened, `{w}`
   merged under statuses GET success (**S2 never merges `{w}`**; CAC #429
   does not merge `{w}`), S2 new-pushes `#3` after wait-table item 2
   (Woodpecker reads the **tree** of the new SHA),
   `GET .../statuses/{new-product-tip-sha}` includes
   `ci/woodpecker/pr/woodpecker` in a success state before merge. Empty
   statuses on `022f4f5` / `9f8dec8` are the CI gap, not a classification.
   A leftover “enable/post” issue is not sufficient DEPS. If after activate
   + Allow PRs + agent online + retrigger, `GET .../statuses/{w-tip-sha}` is
   still `[]` or `pending`: comment **deadlock** on `pulls/3`; that S2
   session is **complete** (same wait table as **(a)** / **(b)**). If
   Woodpecker ACL/server is outside this repo, record a **named** infra
   DEPS (not leftover `{iid}`, not a ticket that clears **G3-8** /
   **G3-3**). Land stays blocked until statuses success. Do not clear
   **G3-8** / **G3-3**. Slice S2 does not add `.woodpecker.yaml`.

## Rollout

- Merge vehicle **B**: existing
  [#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) once that
  branch’s tip is S0 files + S1 delete + S2 README. A successor PR with the
  same three is allowed only if it `Fixes #3` **and** closes or supersedes
  `pulls/3`. Design-only `cac-design-issue-3` must not be opened as the
  product PR and must not be merged first as docs-only.
- Order: fleet protection already owned by #48 → copy S0 files from the
  independently accepted hex + delete file + README via one product PR → S2
  opens leftover `{iid}` with two `@login`s (owner 2 can open a PR here)
  and pasted endpoints, comments Decision 7 on `pulls/3` → first S2 session
  **stops** (wait table) → `@PlasticDigits` comments **G3-9** / **G3-10** /
  **G3-2** (and observed **G3-8** / **G3-3**) on leftover `{iid}` and
  `pulls/3` → I6 fail-close if **G3-9** / **G3-10** / **G3-2** mismatch →
  if that GET is **(a)** or **(b)**, **this S2 session is complete** (named
  DEPS; do not wait for `{w}`; do not merge `#3`) → if **(c)**, architecture
  executable sequence (`{w}` merged under statuses success → S2 rebase →
  statuses GET success) → `@PlasticDigits` SHA-pinned `Do: merge` of `#3`
  after items 5, 6, and 7 (**S2 never merges `#3` or `{w}`**) → leftover
  `{iid}` remains open → leftover-complete **after (c)** as one dated
  comment with observed JSON + vs-target diff, plant-check (mandatory
  sandwich; record ready GET pair; close unmerged in the same session), then
  four-path `test -f` (**G3-1 last**). After **(a)** / **(b)** / deadlock
  DEPS is resolved, `@PlasticDigits` **re-GETs**; new S2 session only then.
- Woodpecker: Decision 7 only. Merge of `#3` only under **(c)** when the
  dated GET classifies **(c)**. Do not weaken **G3-3** / **G3-8** **host
  target** rows to land #3; do not add the pipeline in the `#3` diff; do not
  clear those rows; do not merge under **(a)** or **(b)**. `{w}` yaml is the
  pinned architecture step-1 file.
- Canary role: other `code/*` catch-all deletions may copy this pattern; this
  ADR does not merge those repos. hello#15 is not a gate.
- [#297](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/297): no
  deploy, spend, custody, or CAC policy expansion. Landing #3 does not
  authorize Coolify config changes, token rotation, Hetzner spend, or
  autonomy changes. `{w}` must not add Coolify / Terraform / Hetzner /
  compose / tokens / auto-deploy. This Coolify app **may** rebuild if the
  existing host is git-follow on `main` (not verified in-tree; `pull_request`
  / non-`main` also unverified); that is not a new grant and is not
  leftover-complete.

## Rollback

Restore the previous `CODEOWNERS` **via PR**, not direct `main`, from
`72133f5` (six-line catch-all). That re-plants official requests. It does
**not** by itself re-enable merge-block
(`block_on_official_review_requests`); restoring the 405 gate is a
forge-policy revert, founder-scoped, and is not a GCH rollback step.

Woodpecker files are untouched **by `#3`**. `{w}` is a separate tree on
`main`; rollback of `{w}` is a separate PR if needed. Coolify / Terraform
rollback is unused for the `#3` diff.

If S0 docs need revert, revert via PR together with README so relative links
do not 404.

## Integration completion criteria

### Land (S0+S1+S2) — merge of PR `#3` (successor only with `Fixes #3`)

All must be true on the merged tip. This is what merging `#3` completes. It
does **not** wait for S3. Merger: `@PlasticDigits` (**G3-4**) after items
5, 6, and 7. Item 7 **(c)** is `{w}` merged under statuses success → S2
rebase → statuses GET success. Step 5 **is** the merge. **S2 never merges
`#3` or `{w}`.** Standing **G3-4** after land does not keep items 5–7 as
the forever contract.

1. `main` has no CODEOWNERS file at the four Forgejo paths: `test -f` fails on
   `CODEOWNERS`, `docs/CODEOWNERS`, `.gitea/CODEOWNERS`, `.forgejo/CODEOWNERS`
   (**G3-1**).
2. Merged tip contains `docs/adr/0001-remove-catchall-codeowners.md` and
   `docs/architecture.md` that satisfy
   `git diff <accepted> -- docs/adr/0001-remove-catchall-codeowners.md docs/architecture.md`
   empty, where `<accepted>` is the independently accepted hex pinned on
   leftover `{iid}` and on `pulls/3` after ACCEPT. README relative links to
   those paths resolve. Do not merge a README that points at those paths until
   they exist on that tip. Do not treat `62ffd58` as that hex.
3. README states the **G3** gate via `docs/architecture.md`, keeps the product
   map, and does not imply CODEOWNERS is what makes merge trusted. Product
   overview remains. No stub escape.
4. Diff does not change Rust, Terraform, Docker, `.env.example`, runbooks, or
   add `.woodpecker.yaml`. No `force_merge`, no direct `main`, no CAC
   dismiss-as-merge, no Coolify/HMAC/`autonomy.rs` edits in the product-PR
   diff. No `Fixes` / `Closes` of leftover `{iid}` or Woodpecker / unstick
   iids. A successor may `Fixes #3` and must close or supersede `pulls/3`.
5. Leftover `{iid}` exists: `GET /api/v1/repos/code/gitlab-cursor-webhook/issues/{iid}`
   shows `pull_request` absent, `iid != 3`,
   `repository.full_name == "code/gitlab-cursor-webhook"`, body names two
   Forgejo `@login`s (`@PlasticDigits` and leftover owner 2) and quotes
   leftover-complete items 1–3, and states that closing `#3` does not assign
   S3. Owner 2 must be able to open a PR here; S2 **must** replace owner 2
   before merge if that login cannot. A post-merge replacement comment on
   leftover `{iid}` is allowed. Land of `#3` still does not wait on S3.
   Fail if that GET does not match, if a placeholder (`repo admin of`,
   `GCH implementer`, `<Forgejo`) remains, or if either owner line lacks a
   `@login`. Title substring `Forgejo issue, not PR` is **not** a land gate.
   Record `{iid}` on `pulls/3`. Never call `#3` the leftover.
6. **Always.** `@PlasticDigits` has **commented** dated GET JSON of this
   repo’s `main` rule showing **G3-9**, **G3-10**, and **G3-2**
   (`enable_push == false`) on leftover `{iid}` **and** on `pulls/3`. S2
   did not perform that GET. Do not land if the JSON is missing. If
   **G3-9 ≠ 0** or **G3-10** is `true` or `enable_push != false`, stop;
   named local/host DEPS; do not `force_merge`.
7. Decision 7 **(a)**/**(b)**/**(c)** as in architecture “One Woodpecker land
   rule”, Tests item 9, the **land-of-`#3`** diagram, and the `pulls/3` body.
   Classify only; do not collapse **(a)** and **(b)**. **(a)**: named
   **G3-8** DEPS recorded; **this S2 session is complete; do not wait for
   `{w}`; do not `Do: merge`.** **(b)**: named **G3-3** DEPS recorded;
   **this S2 session is complete; do not wait for `{w}`; do not `Do: merge`.**
   **(c)**: `{w}` merged under statuses GET success → S2 rebase →
   `GET .../statuses/{new-product-tip-sha}` shows
   `ci/woodpecker/pr/woodpecker` success → **G3-4**. Empty statuses on
   `022f4f5` / `9f8dec8` do not satisfy **(c)** (CI gap only). Do not fake
   the context. Do not add `.woodpecker.yaml` in this ticket. Do not clear
   **G3-8** / **G3-3**. A leftover “enable/post” issue is not sufficient
   DEPS. Deadlock comment on `pulls/3` completes this S2 session like
   **(a)** / **(b)**.

Green `cargo test` on `chore/remove-catchall-codeowners` **before** merge is
useful and not sufficient for leftover-complete. Empty commit statuses on
`022f4f5` document the CI gap; they are not leftover-complete. Incomplete
product tip `022f4f5` is incomplete without S0 files and S2.

### Leftover-complete (S3) — leftover `{iid}` in this repo; survives merge of `#3`

Require **one dated leftover comment** with all three items, **G3-1 last**
(or all three timestamps in one attest). A GET or plant-check from before a
later `_ensure_codeowners` apply does not count; re-delete via PR and write a
new comment. Closing `#3` does not assign S3. Pass is dated observed JSON +
written vs-target diff + plant-check `{n}` closed unmerged + **G3-1 last**.
Recording vs-target drift is leftover-complete **after (c)** land of `#3`
(the only merge path; same “after (c)” as Rollout). Do not “prove” flags
whose drift must not fail S3. Drift after that is `#48`, not an S3 fail.

1. Observed JSON + vs-target diff: `@PlasticDigits` comments dated JSON of
   the `main` rule and writes the diff of **G3-2**, **G3-8**, **G3-3**,
   **G3-9**, **G3-10**, **G3-5** against the architecture **host target**.
   Not the Merge API row. Not inferred from a green scanner. Not copied from
   another repo. Drift vs target is a #48 leftover, not a GCH PATCH, not a
   leftover-complete fail, not a reason to restore `CODEOWNERS`.
2. Plant-check `{n}` + JSON: leftover owner 2 runs a **dedicated** plant-check
   PR following Tests item 4 and Observability’s fail-closed pair. Throwaway
   `docs/` path that is not a runbook; **mandatory sandwich** (do not open
   ready-first): open **WIP/draft** → strip WIP / mark ready → **GET
   immediately** (pass GET `draft == false`, no WIP prefix) → **re-apply WIP
   immediately** → close unmerged in the same session. Record the **ready**
   GET pair, not the re-WIP GET. Empty draft GET is not a pass; present on
   draft still fails. Residual race ready → re-WIP is still open to a
   poller; `do-not-merge` is not a host-block; CAC #429 prose is not a
   mutex. If `{n}` is merged, leftover-complete **fails**; revert via PR.
   `#3`’s own official request does not count. `#2` does not count. “The
   next natural PR” does not count. CAC #429 does not merge `{n}`.
3. **G3-1 last:** `test -f` fails on `CODEOWNERS`, `docs/CODEOWNERS`,
   `.gitea/CODEOWNERS`, and `.forgejo/CODEOWNERS`. Recorded after items 1 and
   2 (same comment or later timestamp in the same attest). Owner: leftover
   owner 2.

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
