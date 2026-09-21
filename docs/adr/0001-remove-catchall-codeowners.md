# ADR 0001: Remove catch-all CODEOWNERS

## Status

Proposed (originating
[#3](https://git.cl8y.com/code/gitlab-cursor-webhook/issues/3) is
**closed** / merged incomplete). Not accepted by this design-author pass.
Do not call this tip accepted until independent review says so. Keywords
in the issue body are not architecture approval.

**S1 is done on `main`.** [`pulls/3`](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3)
is `merged: true` (`merged_at: 2026-09-21T13:10:05Z`, merger
`@PlasticDigits`, merge commit `19bb806`, head `022f4f5`). Issue `#3` is
closed (`pull_request.merged: true`). Official `REQUEST_REVIEW` `id: 209`
is still present and not dismissed. Open issues in this repo: **none**.
Open PR: Renovate `#2` only. Vehicle **B** (S0+S1+S2 on one open
`pulls/3`) is **dead**. Do **not** restore `CODEOWNERS` to retry it. Do
**not** use closed `#3` / merged `pulls/3` as the land vehicle or leftover
`{iid}`.

After independent ACCEPT, pin **that accepted hex** on leftover `{iid}`
(`iid != 3`, `pull_request` absent) and on new product PR `{p}`. Land
criterion: `git diff <accepted> -- docs/adr/0001-remove-catchall-codeowners.md docs/architecture.md`
is empty on `{p}`. Do not treat `62ffd58` or `6e0852c` as copyable. Drop
`Fixes #3` as a close-gate.

Overview (merge gate, runtime, six-row **this-repo host target**, one
Woodpecker land rule that classifies **(a)** / **(b)** / **(c)**; merge of
`{p}` only under **(c)**):
[`architecture.md`](../architecture.md). Do not copy that table here.
**G3** there is three groups: protection GET **host target** (six flags;
this repo’s `main` target, not forge INVARIANTS **2–7** — item 2 is “No
force-push,” omitted and assigned to #48, not the `#3` body), merge
procedure (**G3-4**), tree contracts (**G3-1**, **G3-6**, **G3-7**). Issue
`#3` mapped only to **G3-2** (`enable_push == false`; `#3` does **not**
own force-push — named #48), **G3-8**/**G3-3** (standing remaining CI
gate), **G3-4**. “Maps only” **cannot** mean skip I6: leftover official
requests remain, so land of `{p}` still requires the I6 attest of
unpublished **G3-9** / **G3-10** (and **G3-2**) even though those IDs are
not in the `#3` body map.

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

Catch-all `CODEOWNERS` is **already deleted** on `main` (`022f4f5` via
`19bb806`) so Forgejo does not plant **official** review requests on every
change. Do **not** restore it. Merge to `main` stays: pull request, no
direct push, Woodpecker context `ci/woodpecker/pr/woodpecker` (issue `#3`
body; standing **G3-8** ∧ **G3-3**), SHA-pinned `Do: merge` by
`@PlasticDigits`, no `force_merge`. Decision 7 classifies observed
protection **(a)** / **(b)** / **(c)** from `@PlasticDigits`’s dated GET;
it does not drop Woodpecker from that remaining gate. Merge of `{p}`
**only** under **(c)**. **(a)** / **(b)** are distinct **stop** reasons
(named **G3-8** DEPS vs named **G3-3** DEPS); do not `Do: merge` on
either. **S2 never merges `#3`, `{w}`, or `{p}`.**

**Land vehicle `{p}` (post-`19bb806`).** New product PR `{p}` ships **S0
docs + README G3 pointer only**. S1 is a **no-op** on `{p}` (file already
gone; confirm four-path absence). `{p}` must **not** add
`.woodpecker.yaml`. `{p}` must **not** `Fixes #3` / `Closes #3` (`#3` is
already closed). Design branch `cac-design-issue-3` is review/transport
only; it is **not** merged as a docs-only PR and is **not** `{p}`. `{p}`
must not use `Fixes` / `Closes` of leftover `{iid}` or of any Woodpecker /
unstick iid. Copy the two design files from **the independently accepted
hex**. `git diff origin/main -- CODEOWNERS` on `{p}` must be empty (no
restore). The `022f4f5` merge skipped merge-ready items 2–7; repair is
`{p}` + leftover + I6 + `{w}`, **not** a second delete.

**Leftover issue (land gate).** Before merge of `{p}`, **first-session**
S2 opens one leftover Forgejo **issue** in **`code/gitlab-cursor-webhook`
only** (owners and body template under Migration). Restart S2 does **not**
open another. Prove it with
`GET /api/v1/repos/code/gitlab-cursor-webhook/issues/{iid}`:
`pull_request` absent, `iid != 3`, `repository.full_name ==
"code/gitlab-cursor-webhook"`. That issue owns S3. Never call `#3` the
leftover. Closed `#3` is not leftover-complete. Land criterion 5 fails if
that GET does not match, if the body lacks two Forgejo `@login`s, if a
placeholder remains, or if a second leftover was opened after a stop.

**Land vs leftover-complete.** Land fail-closes on **G3-9** / **G3-10** and
on `enable_push != false` (**G3-2**) via **repo admin** attest (not S2 GET).
I6 fail: an **I6 comment posted** that lacks dated GET JSON (or fails parse
/ omits **G3-9** / **G3-10** / **G3-2**) **or** that JSON shows **G3-9 ≠ 0**
/ **G3-10** `true` / `enable_push != false` → **STOPI6**; do not wait for
`{w}`; do not `Do: merge`; do not classify through to **(c)**. **No I6
comment yet → operator waits.** Merge of `{p}` is Decision 7 **(c)** only
(architecture executable **(c)** sequence: `{w}` merged under statuses
success → **S2-restart** rebase → statuses GET success → **G3-4**). After
leftover + land-procedure comment, first-session S2 is **complete** (not a
waiter). After `{w}` is on `main`, `@PlasticDigits` **always** posts
`S2-restart`. After a stop, use the architecture **post-STOP sequence**
(option B): re-GET → STOPI6/STOPA/STOPB **or** open/reuse `{w}` → comment
`{w}` on `main` → `S2-restart`. Leftover-complete (S3) **records** dated
observed JSON plus a written vs-target diff (does not “prove” flags)
**after (c)** land; drift after land is `#48`, not an S3 fail. Plus
dedicated post-merge plant-check PR `{n}` closed unmerged, plus four-path
absence (**G3-1 last**). S3 is not a close gate for `#3` or `{p}`.

This repo is a product tree, not the forge #48 canary (`code/hello`).
Remaining work is in-repo docs/README pointer plus `{w}` plus leftover.
It does not re-roll protection, does not implement CAC autoland, and does
not deploy Coolify.

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

Live proof that the file **did** plant requests: merged PR
[#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) still has
`requested_reviewers_teams` containing team `name: "maintainers"` (org `code`,
`id: 4`) and an `official: true` `REQUEST_REVIEW` whose `team.name` is
`"maintainers"` (review `id: 209`, not dismissed). Forgejo `Team.name` is
`"maintainers"`, not `"code/maintainers"`. On the reviews GET,
`team.organization` is `null`. Renovate
[#2](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/2) has the same
planted team request. Closed `#3` / `#2` are **not** S3 evidence. Land
criterion 6 **always** applies.

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
leftover `{iid}` **and** on `{p}`. If an **I6 comment is posted** that
lacks dated GET JSON (or fails parse / omits **G3-9** / **G3-10** /
**G3-2**) **or** that JSON shows **G3-9 ≠ 0** / **G3-10** `true` /
`enable_push != false`, **stop** (**STOPI6**; do not classify through to
**(c)**). **No I6 comment yet → operator waits.** If I6 attests and that
GET is **(c)**, take the architecture executable sequence, then **always**
`S2-restart`. If it is **(a)** or **(b)**, **stop**. Do **not** treat a
leftover request on `#2` / `{w}` / `{p}` as non-blocking from fleet values.
It is non-blocking **only after** `@PlasticDigits` **comments** that dated
GET JSON showing **G3-9**, **G3-10**, and **G3-2**. If that posted I6
comment fails, do not land `{p}`; record a named local/host DEPS. That
leftover request also must not be treated as S3 evidence.

**Post-`19bb806` `main`:** no `CODEOWNERS`, no `docs/architecture.md`, no
`docs/adr/`, no `.woodpecker.yaml`, README has no G3 pointer. That is
exactly the land the prior Vehicle **B** ADR forbade as an incomplete tip,
except the delete already merged. Incomplete product tip `022f4f5` deleted
the six-line file and did not touch README or copy `docs/`. Commit statuses
on `19bb806` are `[]`. Design SHA `9f8dec8` also had `[]`. Those empty
statuses are the **CI gap** only, not protection flags, not a
classification. Standing **G3-8** ∧ **G3-3** still has no poster.

`origin/main` already has `docs/` runbooks (`admin-golden-image.md`,
`docker-deploy.md`, examples) and **no** `docs/architecture.md` or
`docs/adr/`. Relative README links to ADR 0001 / architecture 404 unless those
two files land on the **same merged tip** as the README pointer (`{p}`). Do
not rewrite those runbooks.

Issue body points at forgejo `docs/INVARIANTS.md`. Unauthenticated HTML
404s. The six-row table is this repo’s host target (readable with a repo
token). Forge INVARIANTS item 2 (“No force-push”) is out of `#3`. Do not fork
INVARIANTS into this repo.

Open PRs in this repo now: Renovate
[#2](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/2) only. `#3` is
closed. Renovate benefits from fewer planted reviews; it is not a
sequencing dep. Closed [#1](https://git.cl8y.com/code/gitlab-cursor-webhook/issues/1)
is unrelated.

Issue `#3` has `drain skip: no occupying job for rebase/fix-pr/CI-wait`;
that is **#429**, not a `#3` / `{p}` failure. Drain-skip / first-session
exit ≠ skip rebase, ≠ add yaml to `{p}`, ≠ `Do: merge`.

This controller provisions Hetzner VMs. This Coolify app **may** rebuild if
the existing host is git-follow on `main` (not verified in-tree:
`docker-compose.yml` and `docs/docker-deploy.md` only record Dockerfile deploy
plus `/var/lib/gch`; they do **not** record git-follow, auto-deploy,
rebuild-on-`main`, or Coolify builds on `pull_request` / non-`main`). Landing
`{p}` is still not a #297 deploy grant: the product-PR diff must not change
image, compose, Terraform, tokens, or auto-deploy. Predecessor `{w}` must not
mint a Coolify / Hetzner deploy either (Decision 7).

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
  leftover of #388). Drain comments such as `drain skip: no occupying job…`
  on `#3` are **#429**, not a `#3` / `{p}` failure. CAC #429 does not merge
  `{p}` **or** plant-check `{n}`. Drain-skip ≠ skip rebase.
- Dismissing reviewers from the controller (forbidden substitute in #388).
- Path-specific CODEOWNERS, a second maintainer, or `required_approvals: 1`.
- Adding, enabling, or digest-pinning Woodpecker **in the `{p}` diff**.
  Missing `ci/woodpecker/pr/woodpecker` statuses are pre-existing. Do not add
  `.woodpecker.yaml` / `.woodpecker/` in the `{p}` diff. Observed **(c)** uses
  the architecture executable sequence (predecessor `{w}` authored and
  merged by `@PlasticDigits`, then **S2-restart** new push of `{p}`); not
  this diff. `{w}` adds **only** the pinned root `.woodpecker.yaml` in
  architecture executable **(c)** step 1 (`when` is **only** `pull_request`;
  required gitleaks against existing `.gitleaks.toml`; optional `cargo test`
  / `clippy` only on `rust:1.88-bookworm`). `{w}` must not add Terraform,
  Coolify, Hetzner, compose, tokens, auto-deploy, `echo`-only commands, or
  Coolify-on-`push`/`main`. ACCEPT of this design must not mint a deploy via
  `{w}`. Do not copy `code/hello`’s pipeline. Do not clear **G3-8** / **G3-3**
  to land `{p}`. Land of `{w}` fails review if the yaml is not that shape.
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
- Retrying Vehicle **B** on closed `#3` / merged `pulls/3`, restoring
  `CODEOWNERS` to redo the delete, or treating `Fixes #3` as a close-gate.
- Using closed `#3` as leftover `{iid}` or as leftover `{iid}`’s close
  trailer.

## Decision

1. **Keep** root `CODEOWNERS` absent. Do not restore it. Do not leave an
   empty or comments-only file (Forgejo still parses it). S1 is already
   merged; `{p}` does not re-delete unless a later apply has put the file
   back (**G3-1** repair, not Vehicle **B**).
2. **Do not add** `docs/CODEOWNERS`, `.gitea/CODEOWNERS`, or
   `.forgejo/CODEOWNERS`. After land, `test -f` fails on all four paths. None of
   those paths may contain a reviewer rule for any pattern (not only `.*`).
3. **Keep** the merge gate in [`architecture.md`](../architecture.md) **G3**.
   On `{p}`, copy `docs/adr/0001-remove-catchall-codeowners.md` and
   `docs/architecture.md` from **the independently accepted hex** onto that
   **same tip**, then add a short README pointer. README remains the product
   overview (not a stub). Point at `docs/architecture.md` **only** for the
   merge gate (**G3**), and at this ADR for the delete decision. Do not imply
   CODEOWNERS is what makes merge trusted. Do not merge a README that points
   at those paths until they exist on that tip. The two `docs/` files on
   `{p}` must satisfy `git diff <accepted> --` those two paths empty. Do not
   rewrite runbooks.
4. **Leave** already-planted official requests on open PRs (including `#2`,
   and `{w}` / `{p}` once opened **if** planted). S1 already deleted the
   file, so new PRs against current `main` may not be planted; I6 still
   attests. Treat leftover official requests as non-blocking **only after**
   repo admin **comments** dated GET JSON of `code/gitlab-cursor-webhook`
   `main` showing **G3-9**, **G3-10**, and **G3-2** (`enable_push == false`)
   on leftover `{iid}` **and** on `{p}` (same I6 attest; no CAC dismiss).
   Human dismiss is optional leftover, not AC. If an **I6 comment is
   posted** that lacks dated GET JSON (or fails parse / omits **G3-9** /
   **G3-10** / **G3-2**) **or** that JSON shows **G3-9 ≠ 0** / **G3-10**
   `true` / `enable_push != false`, stop; record a named local/host DEPS;
   do not wait for `{w}`; do not `Do: merge`; do not classify through to
   **(c)**. **No I6 comment yet → operator waits.** `{w}` is opened only
   after I6 attests. Do not `force_merge`. **S2 never merges `{w}`.** CAC
   #429 does not merge `{w}`.
5. **Do not** PATCH branch protection from this repository.
6. **Split land from leftover-complete.** Product PR `{p}` is S0 docs +
   README (S1 no-op). S3 lives on leftover `{iid}` **first-session** S2
   opens in this repo before that merge (template below), with two Forgejo
   `@login`s recorded before merge. Closing `#3` did not assign S3. Closing
   `{p}` does not assign S3. Require a **dedicated** post-merge plant-check
   PR. Do not accept `#3`’s own official request, `#2`, or “the next natural
   PR.”
7. **One Woodpecker land rule.** Canonical text:
   [`architecture.md`](../architecture.md) “One Woodpecker land rule
   (canonical land procedure)”. Same procedure as Tests item 9, land
   criterion 7, the **land-of-`{p}`** diagram, and the `{p}` body. **Do not
   paste a second wait table here.** Standing remaining CI gate stays
   Woodpecker `ci/woodpecker/pr/woodpecker` (issue `#3` body). Summaries
   that must not drift from that procedure:

   - Classify **(a)** / **(b)** / **(c)** only after I6 attests. XOR is
     I6-fail ⊕ classify(a|b|c) only. Deadlock nests under **(c)** /
     statuses GET. Mixed **(c)**-on-G3-8/G3-3 + I6-fail stays **STOPI6**
     only.
   - **(a)** / **(b)**: named **G3-8** / **G3-3** DEPS (not leftover
     `{iid}`); **stop**; do not `Do: merge`.
   - **(c)** only: `{w}` (pinned root `.woodpecker.yaml`; `when` only
     `pull_request`; required gitleaks vs existing `.gitleaks.toml`;
     optional cargo only on `rust:1.88-bookworm`; not in the `{p}` diff;
     do not copy `code/hello`) merged under statuses GET success →
     `@PlasticDigits` comments `{w}` on `main` → **always** `S2-restart`
     (happy path and post-STOP) → statuses GET success → **G3-4**.
   - First-session S2 is **complete** after leftover `{iid}` + this
     procedure commented on `{p}`. It is **not** a waiter. There is **no
     wait-table item 2**. **Only S2-restart rebases**; never both.
   - Drain-skip / first-session exit ≠ skip rebase, ≠ add yaml to `{p}`,
     ≠ `Do: merge`.
   - **S2 never merges `{p}`, `#3`, or `{w}`.** Do not clear **G3-8** /
     **G3-3**. Do not merge closed `#3`.

## Actors

Split so S2 cannot skip the land GET. “Implementer” is S2. **S2 never
merges `{p}`, `#3`, or `{w}`.** Canonical sequence:
[`architecture.md`](../architecture.md) “One Woodpecker land rule”.

- **S2 (first session):** open `{p}` with S0 files + README G3 pointer
  (S1 no-op; do not restore `CODEOWNERS`; do not add `.woodpecker.yaml`);
  open leftover `{iid}` (`iid != 3`, `pull_request` absent) with two
  `@login`s and pasted protection endpoints; comment the architecture land
  procedure onto `{p}`; **do not GET** protection; **never merge `{p}` or
  `{w}`**. After leftover `{iid}` exists and S2 has commented that
  procedure, **this session is complete**. It is **not** a waiter. Do not
  rebase. Do not wait for I6 or `{w}`. Drain-skip / exit ≠ skip rebase, ≠
  add yaml to `{p}`, ≠ `Do: merge`. `@PlasticDigits` does SHA-pinned
  `Do: merge` of `{p}` after **S2-restart** and merge-ready **(c)**.
- **S2-restart:** leftover `{iid}` already recorded on `{p}` — do not open
  another; do not re-comment the land procedure as a new wait; do not
  re-enter first-session S2. **Only** this session rebases. Rebase
  immediately (`{w}` already on `main`) with the two yaml gates; stop for
  **G3-4**. Never merge `{p}` or `{w}`. Started **always** after
  `@PlasticDigits` comments `{w}` is on `main` (happy path **and** option
  B). `@PlasticDigits` is **not** the rebaser.
- **`@PlasticDigits`** (repo admin of `code/gitlab-cursor-webhook`): **comment**
  dated GET JSON of the `main` rule for **G3-9**, **G3-10**, and **G3-2**
  (same JSON answers observed **G3-8** / **G3-3**) on leftover `{iid}`
  **and** on `{p}` (not on closed `pulls/3`). Unauthenticated GET is 401;
  in-repo CI cannot do this. Activates this repo in Woodpecker (Allow pull
  requests on, agent online; retrigger if `{w}` opened first). Opens
  predecessor `{w}`, records `{w}` on `{p}` when opening it, and merges
  `{w}` under Decision 7 **(c)** (architecture executable sequence; **S2
  never merges `{w}`**; CAC #429 does not merge `{w}`). `{w}` is opened
  only after I6 attests; I6 fail does **not** open `{w}`. `{w}`’s planted
  official request (if any) is non-blocking only under the same I6 attest.
  Comments on `{p}` when `{w}` is on `main`, **then always** posts
  `S2-restart` on `{p}` quoting the S2-restart recipe and queues a CAC
  `implement` job for `{p}` with that prompt (happy path **and**
  post-STOP; CAC #429 does not). After STOPI6 / STOPA / STOPB /
  STOPDEADLOCK, follow the architecture **post-STOP sequence** (option B):
  **re-GET**; I6 fail again → STOPI6; still **(a)** / **(b)** → STOPA /
  STOPB; if **(c)** and no `{w}` → steps **1–2 only**; if **(c)** and
  `{w}` already open from deadlock → **reuse that PR**; comment `{w}` is
  on `main`; **then always** `S2-restart`. Do **not** re-enter WAITI6. Do
  **not** merge `{w}` from reuse before re-GET attests I6 **and** **(c)**.
  After merge-ready items 1–6 **and** item 7 **(c)** only (I6 fail,
  **(a)**, **(b)**, and deadlock are stop states, not land criteria),
  `@PlasticDigits` performs SHA-pinned `Do: merge` of `{p}` (**G3-4**;
  architecture step 5). CAC #429 does not merge `{p}`, `{w}`, or
  plant-check `{n}`.
- **S3 owners** (two Forgejo `@login`s on leftover `{iid}` **before** merge of
  `{p}`; closing `#3` / `{p}` does not assign S3):
  1. `@PlasticDigits` — protection GET comment.
  2. `@lifejkskla` — plant-check `{n}` + **G3-1 last**. Owner 2 **must be
     able to open a PR** in this repo. If `@lifejkskla` **cannot open** a
     PR at leftover-open time, S2 **must** replace this line with another
     real `@login` that can open a PR here **before merge**. If that login
     **later** cannot open `{n}`, a post-merge replacement comment on
     leftover `{iid}` is allowed. Land of `{p}` still does not wait on S3.
     Land fails if either line still contains `repo admin of`,
     `GCH implementer`, `<Forgejo`, or lacks a `@login`.

## Component / state / interface changes

| Surface | Change |
| --- | --- |
| `CODEOWNERS` (root) | Already removed on `main`. `{p}` does not restore it. |
| `docs/CODEOWNERS`, `.gitea/CODEOWNERS`, `.forgejo/CODEOWNERS` | Must remain absent (no empty file). |
| `docs/adr/0001-remove-catchall-codeowners.md`, `docs/architecture.md` | Copy from the independently accepted hex onto `{p}` so the merged tip holds S0 (`git diff <accepted> --` empty). Standing G3 contract lands with the README pointer. |
| Forgejo PR review interface | After land, a **dedicated** plant-check PR against `main` must not get an official CODEOWNERS team request. Closed `#3` / `#2` are not that PR. |
| Branch protection API | No write from this ticket. The six-row table is **this repo’s host target**, not forge INVARIANTS **2–7**, not a measured GET pasted here, not the `#3` body. `#3` does **not** own force-push (named #48). S2 does not GET. `@PlasticDigits` **comments** dated GET JSON: land fail-closes on **G3-9** / **G3-10** and on `enable_push != false` (**G3-2**); leftover-complete records observed JSON plus vs-target diff plus plant-check after **(c)** land. |
| `.woodpecker.yaml` / `.woodpecker/` | Must remain absent in the `{p}` diff. `{w}` adds **only** the pinned root `.woodpecker.yaml` (architecture executable **(c)** step 1) on a separate tree. Observed **(c)** uses that sequence, not this diff. |
| `.gitlab-ci.yml` | Unchanged. |
| Rust crates, Terraform, Docker, `.env.example`, gitleaks | Unchanged in the `{p}` diff. `{w}` required step is gitleaks against existing `.gitleaks.toml`; optional `cargo test` / `clippy` only on `rust:1.88-bookworm`. `{w}` must not change Terraform / Docker / tokens. |
| README | Mandatory on `{p}`: merge gate is **G3**, documented in `docs/architecture.md`; product map stays README. Relative links to ADR 0001 / architecture, which exist on that same tip. Keep the product overview. |
| Runbooks under `docs/` | Unchanged except adding `adr/` + `architecture.md`. Plant-check may add a throwaway non-runbook path under `docs/` (Tests item 4). |
| Leftover `{iid}` | New Forgejo issue (`pull_request` absent, `iid != 3`) in `code/gitlab-cursor-webhook` only, opened by **first-session** S2 before merge of `{p}`, two `@login`s, body quotes leftover-complete items 1–3. Restart S2 does **not** open another. Never call `#3` the leftover. |
| Named Woodpecker / drift DEPS | Predecessor `{w}` under Decision 7 **(c)** (authored, recorded on `{p}`, and merged by `@PlasticDigits`; S2 never merges `{w}`; operator Woodpecker activation is a precondition, not leftover `{iid}`); named **G3-8** DEPS if **(a)**; named **G3-3** DEPS if **(b)**; named infra DEPS if Woodpecker ACL/server is outside this repo (deadlock); **G3-9**/**G3-10**/**G3-2** mismatch. Not leftover `{iid}`. Not a ticket that clears **G3-8** / **G3-3**. |
| `{p}` body | First-session S2 comments the architecture land procedure. Restart S2 does **not** re-comment it as a new wait. `@PlasticDigits` records `{w}` when opening it (reuse if already open from deadlock **after** re-GET **(c)**), comments **deadlock** if statuses stay `[]`/`pending` after activate (**STOPDEADLOCK**), comments when `{w}` is on `main` (**(c)** only), **then always** posts `S2-restart` (happy path and post-STOP). I6 fail does not open `{w}`. Do not comment this onto closed `pulls/3`. |
| Closed `#3` / merged `pulls/3` | Historical S1 only. Not the land vehicle. Not leftover `{iid}`. Not a wait target. |
| CAC / Coolify / org team `maintainers` in org `code` | Unchanged. The team may keep existing; it simply is not planted as official review. Coolify **may** rebuild if the host is git-follow on `main` (not verified in-tree; `pull_request` / non-`main` also unverified); that is not a new deploy grant. `{w}` must not add a Coolify deploy step. CAC #429 does not merge `{p}`, `{w}`, or plant-check `{n}`. |

No runtime state, schema, or HTTP API.

## Affected invariants

IDs live in [`architecture.md`](../architecture.md). This ADR changes **G3-1**
(four-path absence) — already true on `19bb806`; `{p}` must not regress it.
It does not write protection JSON. The six-row table is
**this repo’s host target**, not forge INVARIANTS **2–7**, not the `#3` body.
Force-push stays #48. Land fail-closes on **G3-9** / **G3-10** and on
`enable_push != false` (**G3-2**) via repo-admin attest (not S2 GET).
Leftover-complete records observed JSON plus a written vs-target diff
**after (c)** land, plus plant-check, plus **G3-1 last**. Merge procedure
remains **G3-4** (`@PlasticDigits`; S2 never merges `{p}`). Standing **G3-4**
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
| Keep file, rely on `block_on_official_review_requests=false` | Requests still plant on every PR; drain noise; Renovate/agent PRs look like they need a human stamp; templates can re-teach the old gate. File is already gone; do not restore. |
| Replace `.*` with path owners | No second reviewer exists; same 405/422 if official-review is ever turned on; out of scope. |
| Add a second maintainer | Founder ops, not this implement. |
| Dismiss official requests from CAC | Forbidden by #388 as a substitute for policy reversal. |
| Direct-push the docs to `main` | Violates **G3-2**. Remaining files go through `{p}`. |
| Empty or comments-only CODEOWNERS | Forgejo still parses it. Absence is the contract. |
| Add `.woodpecker.yaml` in `{p}` | Different change (CI enablement). Missing statuses are pre-existing. Observed **(c)** unsticks via predecessor `{w}` authored and merged by `@PlasticDigits` (pinned architecture step-1 yaml), then S2-restart of `{p}`, not this diff. Drain-skip ≠ add yaml to `{p}`. |
| Copy `code/hello` `.woodpecker.yaml` onto `{w}` | Fails here (no `.opengrep.yml`) and, after `{w}` merges, a `main` push can hit Coolify (#297). `{w}` is the pinned architecture step-1 yaml only (`when` is **only** `pull_request`; required gitleaks; no Terraform / Coolify / Hetzner / compose / tokens / auto-deploy / `echo`-only). |
| Noop `echo` pipeline on `{w}` | Makes the standing Woodpecker gate a paper check after CODEOWNERS is gone. |
| `force_merge` or fake Woodpecker statuses to land `{p}` | Forbidden by **G3-4** / **G3-3**. Drain-skip ≠ `Do: merge`. |
| Dispatch land on “**G3-8** is `false` or status checks unset” | Collapses **(a)** and **(b)**. Classify only **(a)** / **(b)** / **(c)**. |
| Named host ticket that clears **G3-8** / **G3-3** to land `{p}` | Policy expansion / #297 / contradicts Rollout. Forbidden. Unstick is Woodpecker success on the product-tip SHA, or stop. Clearing the check is forge `#48` / founder. |
| Treat a leftover “enable/post” issue as Woodpecker DEPS | Not sufficient under **(c)**. Unstick is predecessor `{w}` that actually posts, then `GET .../statuses/{sha}` success. Yaml-on-the-PR is not enough; activate Woodpecker / Allow PRs / agent first. |
| Merge under **(a)** or **(b)** | **Stop.** Record the named DEPS. Do not `Do: merge`. Land of `{p}` is **(c)** only. |
| Merge under **(b)** after recording a DEPS iid only | Host **405** if observed `status_check_contexts` are not success, and land of `{p}` still must not take **(b)**. |
| Rewrite the standing gate as “Woodpecker only under **(c)**” | Issue `#3` keeps Woodpecker as the remaining gate. **(a)** / **(b)** are distinct **stop** reasons for land of `{p}`, not alternate merge paths. |
| Let first-session S2 wait for `{w}` (old item 2) or rebase on happy path | First session is **complete** after leftover + land comment. There is **no item 2**. **Only S2-restart rebases.** After `{w}` is on `main`, `@PlasticDigits` **always** posts `S2-restart`. |
| Let first-session S2 **and** S2-restart both rebase | **Only** S2-restart rebases; never both. |
| Let S2 merge `{p}` after leftover + admin comment + Decision 7 | **S2 never merges `{p}`**. `@PlasticDigits` does SHA-pinned `Do: merge`. |
| Let S2 or CAC #429 merge `{w}` | **S2 never merges `{w}`.** CAC #429 does not merge `{w}`. Merger is `@PlasticDigits` after statuses GET success on `{w}`-tip. `{w}`’s planted official request is non-blocking only under the same I6 attest. |
| Retry Vehicle **B** / restore `CODEOWNERS` / comment Decision 7 onto closed `pulls/3` / wait for I6 on closed `#3` / `Fixes #3` as close-gate | Vehicle **B** is dead. S1 is done. Leftover-complete lives on new `{iid}` (`iid != 3`, `pull_request` absent). Repair is `{p}` + leftover + I6 + `{w}`. |
| Treat empty draft plant-check GET as leftover-complete; open `{n}` ready-first; or undraft and wait 30s | Forgejo skips CODEOWNERS while `pr.IsWorkInProgress()`; API `draft` follows the title WIP prefix. Empty draft signals are the skip, not leftover-complete. `#3`’s known-plant sample is `"draft": false`. **Mandatory sandwich:** open **WIP/draft** → strip WIP / mark ready → **GET immediately** (pass GET `draft == false`, no WIP prefix) → **re-apply WIP immediately** → close unmerged in the same session. Do **not** open ready-first. Record the **ready** GET pair, not the re-WIP GET. Residual race ready → re-WIP is still open to a poller; `do-not-merge` is not a host-block; CAC #429 prose is not a mutex. If `{n}` merges, leftover-complete **fails**; revert via PR. No 30s wait. |
| Treat closed `#3`’s plant, `#2`, or the next natural PR as S3 | A dedicated post-merge PR is the evidence. Closed `#3` is not leftover-complete. |
| Vehicle **A**: docs-only PR from `cac-design-issue-3` onto `main` | Second merge vehicle. That branch is design transport, not `{p}`. |
| Wait on sibling `code/*` CODEOWNERS PRs / hello#15 | Wrong repo; no product iid dependency. |
| Open leftover `{iid}` in `PlasticDigits/*` or leave it without two `@login`s | Land criterion 5 would pass a wrong-repo or vacant issue. S2 names this repo and two `@login`s. |
| Treat Coolify rebuild after merge as leftover-complete | **G3-6**. Plant-check and observed-vs-target record are leftover-complete. A rebuild is not a #297 grant. |
| Merge plant-check `{n}` and call leftover-complete a pass | Leftover-complete **fails** (possible host rebuild / #297 incident). Close without merge in the same session; recovery under Tests item 4. |
| Let S2 GET protection or skip admin attest | S2 cannot GET (401). Skipping land fail-close on **G3-9** / **G3-10** / **G3-2** can merge into 405 or leave direct `main`. |
| Write the archival `PlasticDigits/gitlab-cursor-webhook` clone | CAC invariant 21. Wrong repo. |
| Require dated protection JSON in S0 | Freezes design behind a token this pass does not have. That GET is leftover/admin work. |
| Call leftover-complete a “prove” of **G3-2** / **G3-8** / **G3-3** / **G3-5** | Drift must not fail S3. Record JSON + vs-target diff instead. |
| `Fixes` / `Closes` leftover or Woodpecker iids on `{p}` | Merge would auto-close S3 / unstick tickets. Ban those trailers. `{p}` must not `Fixes #3` either (`#3` already closed; leftover-complete is `{iid}`). |
| Loop STOP* back through WAITI6 with early `{w}` reuse | Architecture land diagram forbids it. Post-STOP is re-GET → STOP* **or** open/reuse `{w}` → comment → `S2-restart`. `{w}` reuse is non-blocking only under I6. |

## Complexity added / removed

**Removed:** catch-all official-review robot on every diff (already on
`main`); operator dismiss step; false “CODEOWNERS is the trusted-PR gate”
story in this repo; Vehicle **B** / first-session rebase waiter (item 2).

**Added:** a small standing doc (this ADR + architecture **G3**) that lands on
`main` via `{p}` so later agents do not re-add `.* @code/maintainers`
as a merge requirement, and a leftover Forgejo issue in this repo that
survives merge of `{p}`. No new services, jobs, flags, or test harnesses. No
new pipelines **in the `{p}` diff**. Predecessor `{w}` adds the pinned
architecture step-1 Woodpecker file on `main`. Named predecessor
`{w}` / **G3-8** / **G3-3** / **G3-9**/**G3-10**/**G3-2** DEPS only when
observed JSON requires them. Do not add a ticket that clears **G3-8** /
**G3-3**.

## Migration

1. Fleet protection is owned by forge #48. This ticket does not PATCH. Do not
   treat a GET recorded on `code/hello` as proof for this repo. The six-row
   table is **this repo’s host target**; leftover-complete records **observed**
   JSON plus a vs-target diff.
2. **Leftover `{iid}` (first-session S2, land gate).** Before merging `{p}`,
   **first-session** S2 opens **one** follow-up Forgejo issue
   in **`code/gitlab-cursor-webhook` only**. Restart S2 does **not** open
   another.
   Do not open it in `PlasticDigits/cl8y-forgejo`,
   `PlasticDigits/cl8y-agent-control`,
   `PlasticDigits/gitlab-cursor-webhook`, or any other repo. Suggested title:
   `chore: leftover CODEOWNERS S3`. Title substring `Forgejo issue, not PR`
   is **not** a land gate. Body **must** name two `@login`s (below) and
   **quote** leftover-complete items 1–3 from this ADR. Record `{iid}` on
   `{p}` before merge. Prove with
   `GET /api/v1/repos/code/gitlab-cursor-webhook/issues/{iid}`:
   `pull_request` absent, `iid != 3`, repo `code/gitlab-cursor-webhook`.
   Closed [#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) is
   **not** this issue. Never call `#3` the leftover. Do not put `Fixes` /
   `Closes` `{iid}` on `{p}`. Do not `Fixes #3`.

   Body template (quote onto leftover `{iid}`):

   ```
   Leftover-complete for ADR 0001 after merge of product PR {p}.
   #3 is already closed and is not this issue. Closing {p} does not assign S3.
   Opened in code/gitlab-cursor-webhook before that merge.
   GET .../issues/{iid} must show pull_request absent, iid != 3.

   Owners (required before merge of {p}; two Forgejo @logins; land fails if a
   placeholder remains):
   1. Protection GET comment: @PlasticDigits
   2. Plant-check {n} + G3-1 last: @lifejkskla
      Owner 2 must be able to open a PR in this repo. If that login
      cannot open a PR at leftover-open time, S2 must replace this
      login before merge. If that login later cannot open {n}, a
      post-merge replacement comment on this issue is allowed. Land
      of {p} does not wait on S3.

   S2 pasted these endpoints; S2 does not GET; S2 never merges {p}:
   - GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections
   - GET /api/v1/repos/code/gitlab-cursor-webhook/branch_protections/main

   Repo admin comments dated GET JSON for G3-9, G3-10, and G3-2
   (enable_push == false) (same JSON answers observed G3-8 and G3-3 for
   Decision 7) on this issue AND on {p} before merge. Do not POST
   protection. If an I6 comment is posted that lacks dated GET JSON (or
   fails parse / omits G3-9 / G3-10 / G3-2) or that JSON shows G3-9 != 0
   or G3-10 is true or enable_push != false, that is a named local/host
   DEPS (not "wait on #48" with no ticket). No I6 comment yet: operator
   waits.

   1. Dated operator GET of this repo's `main` protection rule: comment
      observed JSON here. Not copied from code/hello. Write a vs-target
      diff against the six-row host target in docs/architecture.md.
      Leftover-complete records that JSON + diff. Drift is a #48 leftover,
      not a GCH PATCH, not a reason to restore CODEOWNERS, and not a
      leftover-complete fail. A GET recorded before a later template
      re-copy does not count.

   2. After the docs/README pointer is on `main`: dedicated plant-check PR {n}
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
      not a pass with a Coolify rebuild. Not closed #3, not #2, not the next
      natural PR.

   3. G3-1 last: test -f fails on CODEOWNERS, docs/CODEOWNERS,
      .gitea/CODEOWNERS, .forgejo/CODEOWNERS. Recorded after items 1 and 2.
      Owner: @login on line 2.
   ```

3. **Vehicle `{p}` (post-`19bb806`).** Land a **new** product PR `{p}` whose
   tip is S0 files + README G3 pointer (S1 no-op). Copy
   `docs/adr/0001-remove-catchall-codeowners.md` and `docs/architecture.md`
   from **the independently accepted hex**. Do **not** restore `CODEOWNERS`.
   Do **not** add `.woodpecker.yaml` in `{p}`. Do **not** `Fixes #3`.
   `cac-design-issue-3` stays the review/transport branch; do not open it as
   `{p}`; do not merge it as docs-only first. Do not merge a README that
   points at those paths until they exist on that tip. After ACCEPT, pin that
   hex on leftover `{iid}` and on `{p}`; `git diff <accepted> --` the two
   `docs/` files must be empty on `{p}`. Incomplete merged tip `022f4f5` /
   `19bb806` skipped merge-ready items 2–7; repair is `{p}` + leftover + I6
   + `{w}`, not a second delete. Do not merge `022f4f5` again.
4. Open PRs created while the file existed (`#2`, and `{w}` / `{p}` once
   opened **if** planted) may still show an official team request.
   Non-blocking **only after** the admin I6 comment in Actors (same attest
   for `{w}`; no CAC dismiss). Closed `#3` still shows `id: 209`; that is
   historical, not a land blocker to dismiss, not S3. If an **I6 comment is
   posted** that lacks dated GET JSON (or fails parse / omits **G3-9** /
   **G3-10** / **G3-2**) **or** that JSON shows **G3-9 ≠ 0** / **G3-10**
   `true` / `enable_push != false`, stop; record named DEPS; merge is 405,
   still review-gated, or would allow direct `main`. **No I6 comment yet →
   operator waits.** **S2 never merges `{w}`.**
5. Do not restore the file from `docs/templates/CODEOWNERS` in cl8y-forgejo;
   that template is owned by #48.
6. Woodpecker: architecture land procedure only **(a)**/**(b)**/**(c)**.
   Classify; do not collapse **(a)** and **(b)**. Merge of `{p}` only under
   **(c)**. Do not restore catch-all CODEOWNERS. Do not `force_merge`. Do
   not add `.woodpecker.yaml` in the `{p}` diff. Do not clear **G3-8** /
   **G3-3**. First-session S2 comments that procedure on `{p}` then is
   **complete**. Restart S2 does not re-comment it as a new wait. S2 never
   merges `{p}` or `{w}`. After `{w}` is on `main`, **always** `S2-restart`.

## Observability

Relative reads. Do not log tokens, hosts, or protection-script inventories. Do
not add a Forgejo admin token to Woodpecker, cargo tests, or Coolify.

**I6 protection GET (G3-9 / G3-10 / G3-2, admin attest; Integration stop
on fail, not land item 7).** Always applies. Before
merging `{p}`, **repo admin** of `code/gitlab-cursor-webhook` **comments**
dated GET JSON of the `main` rule showing **G3-9** (`required_approvals == 0`),
**G3-10** (`block_on_official_review_requests == false`), and **G3-2**
(`enable_push == false`) on leftover `{iid}` **and** on `{p}`. Same
endpoints as leftover-complete. S2 pastes the endpoints and does not GET.
Fleet #48 / `code/hello` is not this GET. Fail closed if an **I6 comment
is posted** that lacks dated GET JSON (or fails parse / omits **G3-9** /
**G3-10** / **G3-2**), or that JSON shows **G3-9 ≠ 0**, **G3-10** `true`,
or `enable_push != false`; that mismatch is a named local/host DEPS
(this-repo issue or cl8y-forgejo iid — not “wait on #48” with no ticket)
and **STOPI6**; do not open `{w}`; do not `Do: merge`; do not classify
through to **(c)**. **No I6 comment yet → operator waits.** Do not
`force_merge`. The same JSON answers observed **G3-8** / **G3-3** for
Decision 7.

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

Known-plant sample: closed PR
[#3](https://git.cl8y.com/code/gitlab-cursor-webhook/pulls/3) (must **fail**
this predicate; not S3 evidence). S3 must show a **new** PR of that shape
(`draft == false`, title with no WIP prefix) does **not** get `maintainers`
/ `official == true` `REQUEST_REVIEW`. Empty draft plant-signals are the
skip, not leftover-complete. `#2` is not S3 evidence.

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

`{n}` is the dedicated plant-check PR opened **after** `{p}` is on
`main` by leftover owner 2. Recipe: Tests item 4. Probe a throwaway path
under `docs/` that is not a runbook (example:
`docs/_plant-check-adr0001.md`). Forbid `.env.example`, Terraform, and
Docker as the probe. After land of `{p}`, **G3-9** is proven `0` and
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
with a Coolify rebuild. Closed `#3`’s own official request does not pass.
`#2` does not pass. “The next natural PR” does not pass. CAC #429 does not
merge `{n}`. No 30s wait.

**CI / deploy.** Land of `{p}` uses the architecture **land-of-`{p}`**
diagram only. Standing after land uses the **standing-after-land** diagram
only. Standing remaining CI gate is Woodpecker
`ci/woodpecker/pr/woodpecker`. This tree does not post Woodpecker today.
Empty `[]` on `19bb806` is that CI gap. Drain comments such as `drain skip:
no occupying job…` are **#429**, not a `{p}` failure. Coolify **may**
rebuild if the host is git-follow on `main` (**G3-6**, not verified in-tree;
`pull_request` / non-`main` also unverified); that is not leftover-complete.
`{w}` must not add a Coolify deploy step.

## Failure modes

| Mode | Handling |
| --- | --- |
| File deleted on `main` but docs/README still missing | Expected until `{p}` lands. Do not restore `CODEOWNERS` to “retry” Vehicle **B**. |
| README links ADR/architecture but those files are not on the same tip | 404 after merge. Land fails criterion 2. Copy both files onto `{p}` before merging README. |
| Copy left in `docs/`, `.gitea/`, or `.forgejo/` (including empty/comments-only) | Forgejo still loads the first existing path and may plant. Land fails **G3-1**; delete those paths too (none exist on current `main`). |
| Whole-tree `git grep` for `.* @` | Hits this ADR after a correct delete. Use the pathspec in Tests item 1. |
| cl8y-forgejo migrate/apply re-copies a template | Sister-repo race ([cl8y-forgejo#48](https://git.cl8y.com/PlasticDigits/cl8y-forgejo/issues/48) `_ensure_codeowners`). Out of this slice. If a later apply re-adds the file, leftover-complete is **not** done: delete again via PR, then one new dated leftover comment with all three leftover-complete items, **G3-1 last**. A GET or plant-check from before a re-copy does not count. Never direct-push `main`. |
| Plant-check → re-copy → stale-green GET | Void. Leftover-complete requires one dated leftover comment with (1) observed JSON + vs-target diff, (2) plant-check `{n}` + JSON, (3) four-path `test -f` fails — **G3-1 last** (or all three timestamps in one attest). |
| Official request leftover on closed `#3`, `#2`, or `{w}` / `{p}` | Non-blocking **only after** admin comments **G3-9** / **G3-10** / **G3-2** on leftover `{iid}` and `{p}` (same I6 attest for `{w}`; no CAC dismiss). If an **I6 comment is posted** that lacks dated GET JSON (or fails parse / omits those flags) **or** that JSON shows **G3-9 ≠ 0** / **G3-10** `true` / `enable_push != false`, do not land; named local/host DEPS; do not `force_merge`; optional human dismiss is not a CAC substitute. **No I6 comment yet → operator waits.** Not S3 evidence. Not a rollback signal. **S2 never merges `{w}`.** CAC #429 does not merge `{w}`. |
| Treating merge of `{p}` or closed `#3` as leftover-complete | Use leftover `{iid}` in this repo. Never call `#3` the leftover. |
| Leftover `{iid}` opened in the wrong repo, is a PR, is iid 3, or lacks two `@login`s | Land fails criterion 5. `GET .../issues/{iid}` must show `pull_request` absent, `iid != 3`, repo `code/gitlab-cursor-webhook`, two `@login`s. |
| Commenting Decision 7 onto closed `pulls/3` or waiting I6 on closed `#3` | Vehicle **B** is dead. Comment and attest on `{p}` + leftover `{iid}`. |
| Restart S2 opens a second leftover or re-comments the land procedure as a new wait | Violates land criterion 5 (one leftover `{iid}`). Use the S2-restart recipe: leftover already recorded on `{p}`; rebase only; stop for **G3-4**. |
| First-session S2 rebases, or both sessions rebase | **Only S2-restart rebases.** First session is complete after leftover + comment. |
| Plant-check matcher uses `team.name == "code/maintainers"` or requires `team.organization` on reviews GET | Misses the live plant (`team.name == "maintainers"`, `team.organization == null` on reviews). Use the Observability fail-closed pair. |
| Plant-check pass GET is still `draft == true` or title still contains `WIP`; leftover-complete claimed from an empty draft GET; `{n}` opened ready-first; probe is `.env.example` / Terraform / Docker / a runbook; no changed file; reviewers requested in the UI / `POST .../requested_reviewers`; or no `do-not-merge` comment | False pass (CODEOWNERS skipped while `pr.IsWorkInProgress()`) or false fail (manual team request). **Mandatory sandwich** (do not open ready-first). Residual race ready → re-WIP is still open to a poller; CAC #429 prose is not a mutex. Recipe fails closed; open a new probe. |
| Leaving plant-check `{n}` open ready | `do-not-merge` is not a host-block (**G3-9**=0, **G3-10**=false). Sandwich re-applies WIP then closes unmerged in the same session. |
| Merging the plant-check PR `{n}` | Leftover-complete **fails** (possible host rebuild / #297 incident). Recovery: revert throwaway path via PR; Coolify rebuild from `{n}` is #297. Close without merge in the same session. |
| Protection silently reverted to official-review true | Merge 405 returns. Out of this repo; re-apply via forge policy, do not `force_merge`. Not proven by scanners. |
| `enable_push` flipped true | **G3-2** regression (direct push). Land of `{p}` fail-closes if the dated GET shows `enable_push != false`. After land, refuse on later PRs. Not a leftover-complete fail (record-not-fail on post-land drift). Not a force-push claim. |
| Observed **(c)** and nothing posts Woodpecker | Owner of activation remains `@PlasticDigits`. If after activate + Allow PRs + agent online + retrigger, `GET .../statuses/{w-tip-sha}` is still `[]` or `pending`: comment **deadlock** on `{p}`. **STOPDEADLOCK** (nested under **(c)** / statuses GET; do not wait for `{w}`; do not `Do: merge`). Restart **re-GETs first**, then **reuses** the existing `{w}` only if that GET is **(c)**; do not open a second `{w}`. If Woodpecker ACL/server is outside this repo, record a **named** infra DEPS (not leftover `{iid}`, not a ticket that clears **G3-8** / **G3-3**). Land stays blocked until statuses success. `{w}` yaml is the pinned architecture step-1 file. A leftover “enable/post” issue is not sufficient DEPS. Not solved by restoring CODEOWNERS. Do not fake statuses. Do not add `.woodpecker.yaml` in the `{p}` diff. Do not clear **G3-8** / **G3-3**. |
| Merge `{p}` under **(a)** | Violates Decision 7. Record named **G3-8** DEPS. **Stop. Do not `Do: merge`.** |
| Merge `{p}` under **(b)** | Violates Decision 7. Record named **G3-3** DEPS. **Stop. Do not `Do: merge`.** |
| Treat I6 fail as classified **(c)** and open `{w}` | Same GET can be **(c)** on **G3-8** / **G3-3** and fail on **G3-9** / **G3-10** / **G3-2**. Fail edge is **STOPI6** (posted I6 comment that lacks dated GET JSON, fails parse, omits those flags, or shows **G3-9 ≠ 0** / **G3-10** `true` / `enable_push != false`). Do not open `{w}`. Do not `Do: merge`. Do not classify through to **(c)**. **No I6 comment yet → operator waits.** |
| Re-adding CODEOWNERS “for safety” in a follow-up | Violates **G3-1**. Reviewers must reject unless a new ADR allowlists path owners. |
| Observed flags differ from the six-row host target | #48 leftover. Not a GCH PATCH. Not a reason to restore `CODEOWNERS`. Not a leftover-complete fail. Land blocker for **G3-9** / **G3-10** / **G3-2**. |
| Rust / Terraform / Docker / token / runbook sneak into the MR | Fail review. |
| `{w}` copies `code/hello` deploy steps, attaches `push`/`main`, or a noop `echo` | `#297` (Coolify on `main` push) or a paper standing gate. `{w}` is the pinned architecture step-1 yaml only. Land of `{w}` fails review. |
| README treats `docs/architecture.md` as product architecture or stubs the product map | Fail S2. Two architecture docs must not collide. |
| Implement deploys Coolify, rotates tokens, or edits `autonomy.rs` / HMAC | Forbidden (#297). |
| S2 performs the protection GET or merges `{p}` or `{w}` | Forbidden. S2 pastes endpoints and **never merges `{p}` or `{w}`**. First session complete after leftover + comment. **Only S2-restart rebases** after the `{w}`-on-main comment. |
| `Fixes` / `Closes` leftover `{iid}` or Woodpecker/unstick iids on `{p}`; `Fixes #3` | Ban. Merge must not auto-close S3. `#3` is already closed. |
| Loop STOP* through WAITI6 / reuse `{w}` before I6 re-attest | Forbidden. Post-STOP is architecture option B. |

## Ordered implementation slices

| Slice | Work | Depends on |
| --- | --- | --- |
| **S0** | This design (ADR 0001 + architecture **G3**). Transport on `cac-design-issue-3`. Copy **the independently accepted hex** onto `{p}`; do not call this tip accepted until independent review says so. `cac-design-issue-3` is never the merge vehicle. Do not GET protection in S0. | None in `code/gitlab-cursor-webhook`. |
| **S1** | **Done** on `main` (`19bb806` / `022f4f5`). Four-path `test -f` fails. **No-op** on `{p}`. Do **not** restore `CODEOWNERS` to retry Vehicle **B**. If a later apply re-adds the file before `{p}` merges, `{p}` may delete it again as **G3-1** repair only. | Already merged. |
| **S2** | **First session.** Open `{p}`: copy S0 files; README pointer (keep README as product map; point at `docs/architecture.md` **only** for the merge gate (**G3**); relative links to ADR 0001 / architecture; do not imply CODEOWNERS is the trusted-merge gate). Keep the product overview. No Docker/Woodpecker/Terraform/Rust edits. Open leftover `{iid}` in **`code/gitlab-cursor-webhook` only**, before merge, with two `@login`s and the body template under Migration (paste the two protection endpoints). Owner 2 **must** be able to open a PR here: cannot-open at leftover-open → replace **before merge**; later-cannot → post-merge leftover comment. Comment the architecture land procedure onto `{p}`. Do not GET protection. **Never merge `{p}` or `{w}`**. After leftover `{iid}` + that comment, **this session is complete**. Not a waiter. Do not rebase. Do not `Fixes`/`Closes` leftover, unstick iids, or `#3`. Drain-skip / exit ≠ skip rebase, ≠ add yaml to `{p}`, ≠ `Do: merge`. | S0 files on `{p}`. S1 already on `main`. Admin land attest is **not** an S2 implement step. |
| **S2-restart** | **Always** after `@PlasticDigits` comments `{w}` is on `main` (happy path **and** option B). Leftover `{iid}` already recorded on `{p}` — do not open another; do not re-comment the land procedure as a new wait; do not re-enter first-session S2. **Only** this session rebases. Rebase immediately (`{w}` already on `main`) with the two yaml gates; stop for **G3-4**. Never merge `{p}` or `{w}`. Start: `@PlasticDigits` posts `S2-restart` on `{p}` quoting this recipe, then queues a CAC `implement` job for `{p}` with that prompt. CAC #429 does not launch it. | First-session leftover `{iid}` on `{p}`. `{w}` on `main`. I6 attested **(c)**. Named STOP* DEPS resolved and re-GET **(c)** when post-STOP. |
| **S3** | Leftover-complete on leftover `{iid}`: one dated leftover comment with (1) observed JSON + vs-target diff, (2) dedicated plant-check `{n}` + JSON closed unmerged, (3) four-path `test -f` fails — **G3-1 last**. Owners: `@PlasticDigits` (1), leftover owner 2 (2+3). Does **not** close `#3` (already closed). Closing `{p}` does not assign S3. | `{p}` merged to `main`. Tracked on leftover `{iid}` in this repo. |

`{p}` ships **S0 + README**. S1 is already on `main`. S2 first session
depends on S0 files being on `{p}`.

**`@PlasticDigits` (not a slice of implement):** **comment** dated GET JSON
for **G3-9**, **G3-10**, and **G3-2** on leftover `{iid}` **and** on `{p}`
before merge. That JSON also answers Decision 7 (**G3-8** / **G3-3**). I6
fail (posted comment) → do not open `{w}`. **No I6 comment yet → operator
waits.** Activate Woodpecker; open `{w}` only after I6 attests **(c)** and
record it on `{p}` (reuse if already open from deadlock **after** re-GET
**(c)**); merge `{w}` under **(c)** (**S2 never merges `{w}`**; CAC #429
does not merge `{w}`); comment **deadlock** if statuses stay `[]`/`pending`
after activate (**STOPDEADLOCK**); comment `{w}` is on `main` only under
**(c)**, **then always** post `S2-restart` and queue `implement` (happy
path **and** post-STOP; CAC #429 does not). After STOPI6 / STOPA / STOPB /
STOPDEADLOCK, follow the architecture **post-STOP sequence** (option B).
Then SHA-pinned `Do: merge` of `{p}` after merge-ready items 1–6 **and**
item 7 **(c)** only. **S2 never merges `{p}`.** CAC #429 does not merge
`{p}`, `{w}`, or plant-check `{n}`.

Merge of `{p}` only under Decision 7 **(c)** (`{w}` statuses success →
**S2-restart** rebase → statuses GET success → **G3-4**). I6 fail:
**STOPI6**, do not `Do: merge`, do not classify through to **(c)**. **(a)**
/ **(b)**: **STOPA** / **STOPB**, do not `Do: merge`. Do not add
`.woodpecker.yaml` in the `{p}` diff. Do not clear **G3-8** / **G3-3**.

Sister repos (not slices of this land, not local `DEPS` unless a named iid is
recorded under Decision 7 or **G3-9**/**G3-10**/**G3-2** mismatch): forge #48
protection+templates; CAC #429 autoland occupying job; hello#15 canary.

## Tests

In-repo CI cannot GET branch protection. Do not add a `cargo test` solely for
four-path absence (filter/provision tests are product; a whole-tree basename
check would also be the wrong contract). `cargo test` / `cargo clippy -- -D
warnings` still pass on `{p}`.

1. **Absence (land, G3-1).** On `{p}` and after merge, `test -f CODEOWNERS`,
   `test -f docs/CODEOWNERS`, `test -f .gitea/CODEOWNERS`, and
   `test -f .forgejo/CODEOWNERS` all fail. Optional content check, **pathspec
   those four paths only**:
   `git grep -nE '^\.\* @' -- CODEOWNERS docs/CODEOWNERS .gitea/CODEOWNERS .forgejo/CODEOWNERS`
   — no match is pass. Do **not** whole-tree `git grep` (the escaped form
   `git grep -n '^\\.\\* @'` matches nothing even while the catch-all file
   exists; the unescaped `git grep -nE '^\.\* @'` hits this ADR after a correct
   delete). Already true on `19bb806`; `{p}` must not restore the file.
2. **Existing scanners (land).** `cargo test` and `cargo clippy -- -D warnings`
   still pass on `{p}`. Gitleaks config is unchanged. Do not treat
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
4. **No new plant (leftover-complete).** After `{p}` is on `main`, leftover
   owner 2 runs this recipe on leftover `{iid}`:
   1. **G3-1** still holds; if a later apply re-copied the file, delete again
      via PR first.
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
   (**G3-6**). It is not a pass with a Coolify rebuild. Do not use closed
   `#3`. Do not use `#2`. Do not use “the next natural PR.” Who opens `{n}`:
   leftover owner 2 (must be able to open a PR here). Cannot-open at
   leftover-open time: S2 replaced **before merge**. Later-cannot: a
   post-merge replacement comment on leftover `{iid}`. Land of `{p}` still
   does not wait on S3. Who GETs protection (item 3): repo admin; not S2;
   not cargo; not Coolify. Leftover-complete attest order is Integration:
   **G3-1 last**. Do not treat a passing GET or plant-check from before a
   later `_ensure_codeowners` apply as done.
5. **Reject still blocks (doc-level).** Do not turn off
   `block_on_rejected_reviews` to “make autoland easier.”
6. **Diff guard (land).** `{p}` does not change Rust sources, Terraform,
   Docker, `.env.example`, runbooks, or add `.woodpecker.yaml`. `{p}` does
   not restore `CODEOWNERS`. `{p}` body / commits must not `Fixes` /
   `Closes` leftover `{iid}`, Woodpecker / unstick iids, or `#3`.
7. **I6 protection GET (stop vs attest; not land-of-`{p}` item 7).** Repo
   admin **comments** dated GET JSON of this repo’s `main` rule showing
   **G3-9**, **G3-10**, and **G3-2** (`enable_push == false`) on leftover
   `{iid}` **and** on `{p}` before merge. S2 does not GET. I6 fail is
   Integration **stop**, not a land tick: an **I6 comment posted** that
   lacks dated GET JSON (or fails parse / omits **G3-9** / **G3-10** /
   **G3-2**) **or** that JSON shows **G3-9 ≠ 0** / **G3-10** `true` /
   `enable_push != false` → **STOPI6**; do not wait for `{w}`; do not
   `Do: merge`; do not classify through to **(c)**. **No I6 comment yet →
   operator waits.** Pass (attest) is merge-ready item 6, not this fail
   path. Do not `force_merge`. Do not share this item’s number with
   merge-ready **(c)** (Integration item 7).
8. **README collision (land, S2).** `{p}` README still is the product map
   and points at `docs/architecture.md` only for the merge gate.
9. **Host context (Decision 7).** Same XOR as Rollout / the **land-of-`{p}`**
   diagram: **I6 fail else classify (a|b|c).** Do not tick I6 fail and
   **(c)** as parallel. A GET that is **(c)** on **G3-8** / **G3-3** and
   fail on **G3-9** / **G3-10** / **G3-2** is **STOPI6 only**. Deadlock is
   **not** an XOR sibling; nest it only under **(c)** / statuses GET.

   - **I6 fail (STOPI6):** I6 comment **posted** that lacks dated GET JSON
     (or fails parse / omits **G3-9** / **G3-10** / **G3-2**) **or** that
     JSON shows **G3-9 ≠ 0** / **G3-10** `true` / `enable_push != false`.
     Do not wait for `{w}`; do not `Do: merge`; do not classify through to
     `(c)`. **No I6 comment yet → operator waits.**
   - **Else classify:**
     - **(a):** named **G3-8** DEPS (not leftover `{iid}`); **STOPA**; do
       not wait for `{w}`; do not `Do: merge`.
     - **(b):** named **G3-3** DEPS (not leftover `{iid}`); **STOPB**; do
       not wait for `{w}`; do not `Do: merge`.
     - **(c):** architecture executable sequence: `@PlasticDigits`
       activates Woodpecker (Allow PRs on, agent online), predecessor
       `{w}` is the pinned root `.woodpecker.yaml` (`when` is **only**
       `pull_request`; required gitleaks; optional cargo only on
       `rust:1.88-bookworm`), `{w}` recorded on `{p}` when opened, `{w}`
       merged under statuses GET success (**S2 never merges `{w}`**; CAC
       #429 does not merge `{w}`), `@PlasticDigits` comments `{w}` is on
       `main` **then always** posts `S2-restart` (happy path and
       post-STOP). **Only S2-restart** new-pushes `{p}` (Woodpecker reads
       the **tree** of the new SHA; **gates after rebase:**
       `HEAD:.woodpecker.yaml` exists **and**
       `git diff origin/main -- .woodpecker.yaml` is empty; do not add
       yaml in `{p}` commits), `GET .../statuses/{new-product-tip-sha}`
       includes `ci/woodpecker/pr/woodpecker` in a success state before
       merge.
       - **Deadlock (STOPDEADLOCK, nested under (c) / statuses GET):** if
         after activate + Allow PRs + agent online + retrigger,
         `GET .../statuses/{w-tip-sha}` is still `[]` or `pending`:
         comment **deadlock** on `{p}`; do not `Do: merge`. Restart
         **re-GETs first**, then **reuses** the existing `{w}` only if
         that GET is **(c)**. If Woodpecker ACL/server is outside this
         repo, record a **named** infra DEPS (not leftover `{iid}`, not a
         ticket that clears **G3-8** / **G3-3**).

   Empty statuses on `19bb806` / `022f4f5` / `9f8dec8` are the CI gap, not
   a classification. A leftover “enable/post” issue is not sufficient
   DEPS. Land stays blocked until statuses success. Do not clear
   **G3-8** / **G3-3**. Slice S2 does not add `.woodpecker.yaml`.

## Rollout

- Merge vehicle: **new** `{p}` once that branch’s tip is S0 files + README
  G3 pointer (S1 already on `main`). `{p}` must not `Fixes #3`. Design-only
  `cac-design-issue-3` must not be opened as `{p}` and must not be merged
  first as docs-only. Do not use closed `#3` / merged `pulls/3`.
- Order up to I6 (shared stem): fleet protection already owned by #48 →
  copy S0 files from the independently accepted hex + README via `{p}`
  (S1 no-op) → S2 opens leftover `{iid}` with two `@login`s
  (cannot-open at leftover-open → replace **before merge**; later-cannot
  → post-merge leftover comment) and pasted endpoints, comments the
  architecture land procedure on `{p}` → first S2 session **complete**
  (not a waiter) → `@PlasticDigits` comments **G3-9** / **G3-10** /
  **G3-2** (and observed **G3-8** / **G3-3**) on leftover `{iid}` and
  `{p}`. **No I6 comment yet → operator waits.**
- Exclusive branches after that GET (same XOR as the **land-of-`{p}`**
  diagram; not a linear `→` chain):

  1. **I6 fail** (I6 comment **posted** that lacks dated GET JSON, fails
     parse, omits **G3-9** / **G3-10** / **G3-2**, or shows **G3-9 ≠ 0**
     / **G3-10** `true` / `enable_push != false`) → **STOPI6**. Named
     **G3-9** / **G3-10** / **G3-2** DEPS. Do not open `{w}`. Do not
     wait for `{w}`. Do not `Do: merge`. Do not classify through to
     **(c)** even if the same GET would also be **(c)** on **G3-8** /
     **G3-3**. **No I6 comment yet → operator waits** (not this branch).
     After those I6 DEPS resolve, `@PlasticDigits` **re-GETs** onto
     leftover `{iid}` and `{p}` (not a resume of a stale **(c)**). Then
     the post-STOP sequence (option B): if **(c)** and no `{w}` → steps
     **1–2 only**; comment `{w}` is on `main`; **then always**
     `S2-restart` (CAC #429 does not). Do **not** re-enter WAITI6.
  2. **Else classify:**
     - **(a)** or **(b)** → **STOPA** / **STOPB**. Named **G3-8** /
       **G3-3** DEPS (not leftover `{iid}`). Do not wait for `{w}`. Do
       not `Do: merge`. After those DEPS resolve, `@PlasticDigits`
       **re-GETs** (same as I6: include **G3-9** / **G3-10** / **G3-2**,
       not only **G3-8** / **G3-3**). If I6 fails on the re-GET, take
       branch 1. If still **(a)** / **(b)**, stop again. If **(c)**, take
       the next bullet. Post-STOP: steps **1–2 only** (no `{w}` yet) →
       comment `{w}` is on `main` → **then always** `S2-restart` (option
       B; CAC #429 does not).
     - **(c)** → architecture executable sequence (`{w}` merged under
       statuses success → comment `{w}` on `main` → **always**
       `S2-restart` rebase → statuses GET success). Then
       `@PlasticDigits` SHA-pinned `Do: merge` of `{p}` after
       merge-ready items 1–6 **and** item 7 **(c)** only (**S2 never
       merges `{p}` or `{w}`**) → leftover `{iid}` remains open (do not
       open a second) → leftover-complete **after (c)** as one dated
       comment with observed JSON + vs-target diff, plant-check
       (mandatory sandwich; record ready GET pair; close unmerged in the
       same session), then four-path `test -f` (**G3-1 last**).
       **Deadlock nested under (c)** (statuses stay `[]`/`pending` after
       activate): **STOPDEADLOCK**; do not `Do: merge`; after that DEPS
       resolves, **re-GET**; if **(c)** **reuse** the existing `{w}`;
       comment `{w}` is on `main`; **then always** `S2-restart`.
- Woodpecker: Decision 7 only. Merge of `{p}` only under **(c)** when the
  dated GET **attests I6 and** classifies **(c)**. Do not weaken **G3-3** /
  **G3-8** **host target** rows to land `{p}`; do not add the pipeline in
  the `{p}` diff; do not clear those rows; do not merge under **(a)** or
  **(b)** or I6 fail. `{w}` yaml is the pinned architecture step-1 file.
- Canary role: other `code/*` catch-all deletions may copy this pattern; this
  ADR does not merge those repos. hello#15 is not a gate.
- [#297](https://git.cl8y.com/PlasticDigits/cl8y-agent-control/issues/297): no
  deploy, spend, custody, or CAC policy expansion. Landing `{p}` does not
  authorize Coolify config changes, token rotation, Hetzner spend, or
  autonomy changes. `{w}` must not add Coolify / Terraform / Hetzner /
  compose / tokens / auto-deploy. This Coolify app **may** rebuild if the
  existing host is git-follow on `main` (not verified in-tree; `pull_request`
  / non-`main` also unverified); that is not a new grant and is not
  leftover-complete.

## Rollback

Do **not** restore `CODEOWNERS` as part of `{p}` repair. If a later change
must restore the previous `CODEOWNERS`, do it **via PR**, not direct `main`,
from `72133f5` (six-line catch-all). That re-plants official requests. It does
**not** by itself re-enable merge-block
(`block_on_official_review_requests`); restoring the 405 gate is a
forge-policy revert, founder-scoped, and is not a GCH rollback step.

Woodpecker files are untouched **by `{p}`**. `{w}` is a separate tree on
`main`; rollback of `{w}` is a separate PR if needed. Coolify / Terraform
rollback is unused for the `{p}` diff.

If S0 docs need revert, revert via PR together with README so relative links
do not 404.

## Integration completion criteria

### Land (S0 + README; S1 already on main) — merge of `{p}`

`#3` is already closed. `{p}` must not `Fixes #3`. Leftover-complete lives
on `{iid}`.

#### Merge-ready (the only land criteria)

All of items 1–6 **and** item 7 **(c)** must be true on the merged `{p}`
tip. This is what merging `{p}` completes. It does **not** wait for S3.
Merger: `@PlasticDigits` (**G3-4**) after items 1–6 **and** item 7 **(c)**
only. Item 7 **(c)** is `{w}` merged under statuses success →
**S2-restart** rebase → statuses GET success. Step 5 **is** the merge.
**S2 never merges `{p}` or `{w}`.** Standing **G3-4** after land does not
keep merge-ready items as the forever contract. I6 fail, **(a)**, **(b)**,
and deadlock are **not** land criteria; they are stop states below.
Checking off a named DEPS does **not** make 1–7 look complete. The
`022f4f5` merge skipped items 2–7; this list is the repair.

1. `main` has no CODEOWNERS file at the four Forgejo paths: `test -f` fails on
   `CODEOWNERS`, `docs/CODEOWNERS`, `.gitea/CODEOWNERS`, `.forgejo/CODEOWNERS`
   (**G3-1**). Already true on `19bb806`; `{p}` must not restore the file.
2. Merged tip contains `docs/adr/0001-remove-catchall-codeowners.md` and
   `docs/architecture.md` that satisfy
   `git diff <accepted> -- docs/adr/0001-remove-catchall-codeowners.md docs/architecture.md`
   empty, where `<accepted>` is the independently accepted hex pinned on
   leftover `{iid}` and on `{p}` after ACCEPT. README relative links to
   those paths resolve. Do not merge a README that points at those paths until
   they exist on that tip. Do not treat `62ffd58` or `6e0852c` as that hex.
3. README states the **G3** gate via `docs/architecture.md`, keeps the product
   map, and does not imply CODEOWNERS is what makes merge trusted. Product
   overview remains. No stub escape.
4. Diff does not change Rust, Terraform, Docker, `.env.example`, runbooks, or
   add `.woodpecker.yaml`. No restore of `CODEOWNERS`. No `force_merge`, no
   direct `main`, no CAC dismiss-as-merge, no Coolify/HMAC/`autonomy.rs`
   edits in the `{p}` diff. No `Fixes` / `Closes` of leftover `{iid}`,
   Woodpecker / unstick iids, or `#3`.
5. Leftover `{iid}` exists: `GET /api/v1/repos/code/gitlab-cursor-webhook/issues/{iid}`
   shows `pull_request` absent, `iid != 3`,
   `repository.full_name == "code/gitlab-cursor-webhook"`, body names two
   Forgejo `@login`s (`@PlasticDigits` and leftover owner 2) and quotes
   leftover-complete items 1–3, and states that closing `{p}` does not
   assign S3 and that `#3` is not this issue. Owner 2 must be able to open
   a PR here. Cannot-open at leftover-open time: S2 **must** replace owner 2
   **before merge**. Later-cannot: a post-merge replacement comment on
   leftover `{iid}` is allowed. Land of `{p}` still does not wait on S3.
   Fail if that GET does not match, if a placeholder (`repo admin of`,
   `GCH implementer`, `<Forgejo`) remains, or if either owner line lacks a
   `@login`, or if a second leftover was opened after a stop. Title
   substring `Forgejo issue, not PR` is **not** a land gate. Record
   `{iid}` on `{p}`. Never call `#3` the leftover. Never use closed `#3`
   as leftover `{iid}`.
6. **I6 attest (pass only).** `@PlasticDigits` has **commented** dated GET
   JSON of this repo’s `main` rule showing **G3-9** `== 0`, **G3-10**
   `== false`, and **G3-2** (`enable_push == false`) on leftover `{iid}`
   **and** on `{p}`. S2 did not perform that GET. An **I6 comment posted**
   that lacks dated GET JSON (or fails parse / omits **G3-9** / **G3-10** /
   **G3-2**), or that JSON shows **G3-9 ≠ 0**, **G3-10** `true`, or
   `enable_push != false`, is **not** this item: that is the I6-fail stop
   state below. **No I6 comment yet → operator waits**, not this item. Do
   not `force_merge`. Do not attest on closed `pulls/3` instead of `{p}`.
7. **(c) only.** `{w}` merged under statuses GET success → **S2-restart**
   rebase (**gates:** `HEAD:.woodpecker.yaml` exists **and**
   `git diff origin/main -- .woodpecker.yaml` is empty; do not add yaml
   in `{p}` commits) → `GET .../statuses/{new-product-tip-sha}` shows
   `ci/woodpecker/pr/woodpecker` success → **G3-4**. Empty statuses on
   `19bb806` / `022f4f5` / `9f8dec8` do not satisfy **(c)** (CI gap only).
   Do not fake the context. Do not add `.woodpecker.yaml` in `{p}`. Do not
   clear **G3-8** / **G3-3**. A leftover “enable/post” issue is not
   sufficient DEPS. Same procedure as architecture “One Woodpecker land
   rule”, Tests item 9, the **land-of-`{p}`** diagram, and the `{p}` body.

Green `cargo test` on `{p}` **before** merge is useful and not sufficient
for leftover-complete. Empty commit statuses on `19bb806` document the CI
gap; they are not leftover-complete and not a classification.

#### Stop states (not land criteria)

These complete **this land attempt**. They are **not** merge-ready. Do not
tick them as land of `{p}`. Do not `Do: merge`. Do not wait for `{w}`
(except **no I6 comment yet**, which is **not** a stop). After the named
DEPS resolves, `@PlasticDigits` **re-GETs** (include **G3-9** /
**G3-10** / **G3-2**, not only **G3-8** / **G3-3**) and follows the
architecture **post-STOP sequence** (option B): re-GET → STOPI6/STOPA/STOPB
**or** open/reuse `{w}`; comment `{w}` is on `main`; **then always**
`S2-restart` (CAC #429 does not). Do **not** re-enter WAITI6. Do **not**
re-enter first-session S2.

- **I6 fail (STOPI6):** I6 comment **posted** that lacks dated GET JSON
  (or fails parse / omits **G3-9** / **G3-10** / **G3-2**) **or** that
  JSON shows **G3-9 ≠ 0** / **G3-10** `true` / `enable_push != false`.
  Named **G3-9** / **G3-10** / **G3-2** DEPS. Do not open `{w}`. Do not
  classify through to **(c)** even if the same GET would also be **(c)**
  on **G3-8** / **G3-3**. **No I6 comment yet → operator waits** (not
  this stop).
- **(a) (STOPA):** named **G3-8** DEPS recorded (not leftover `{iid}`).
  Do not `Do: merge`.
- **(b) (STOPB):** named **G3-3** DEPS recorded (not leftover `{iid}`).
  Do not `Do: merge`.
- **Deadlock (STOPDEADLOCK, nested under (c) / statuses GET):** `{w}`
  statuses still `[]` or `pending` after activate + retrigger; comment on
  `{p}`; no `Do: merge`. Restart **re-GETs first**, then **reuses** the
  existing `{w}` only if that GET is **(c)**.

### Leftover-complete (S3) — leftover `{iid}` in this repo; survives merge of `{p}`

Require **one dated leftover comment** with all three items, **G3-1 last**
(or all three timestamps in one attest). A GET or plant-check from before a
later `_ensure_codeowners` apply does not count; re-delete via PR and write a
new comment. Closed `#3` does not assign S3. Closing `{p}` does not assign
S3. Pass is dated observed JSON + written vs-target diff + plant-check `{n}`
closed unmerged + **G3-1 last**. Recording vs-target drift is leftover-complete
**after (c)** land of `{p}` (the only merge path; same “after (c)” as
Rollout). Do not “prove” flags whose drift must not fail S3. Drift after that
is `#48`, not an S3 fail.

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
   Closed `#3`’s own official request does not count. `#2` does not count.
   “The next natural PR” does not count. CAC #429 does not merge `{n}`.
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
this ADR only records the in-tree file delete already on `main` and lands the
standing G3 contract via `{p}`. Independent review of this proposal is a later
gate. Design author must not write `DESIGN: APPROVE`.
