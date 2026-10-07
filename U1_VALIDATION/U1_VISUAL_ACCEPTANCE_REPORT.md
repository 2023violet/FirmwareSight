---
title: "U1 Installed Visual Acceptance Report"
doc_id: "FS-U1-004"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Design"
last_updated: "2026-10-07"
---

# U1 — Push → CI read-back → exact artifact → real desktop visual acceptance

Continuation of `U1_UI_PRODUCTIZATION_CONVERGENCE`, delivered as 《FirmwareSight — U1 Continuation: Push → CI
Read-back → Exact Artifact → Real Desktop Visual Acceptance, Execution Prompt v1.1 — Architect Authorized》,
canonical unit **`U1_PUSH_CI_AND_INSTALLED_VISUAL_ACCEPTANCE`**. Section numbers below cite that prompt. v1.1
governs; v1.0 is its prior revision and is followed only where it adds something v1.1 does not contradict. Both
revisions are archived byte-for-byte (see §12).

This document records evidence and ends with a recommendation. **It does not approve the UI visually** — that
judgement is the Architect's, and §21 fixes the only status this round may set.

## 1. Start authority and what was already committed (§0–§1)

```text
git rev-parse HEAD                 7dc2ca80e5273b9aac48bb7ca216d24c2c1585ce   (local U1 tip)
git rev-parse origin/main          f481b78059c14e1c83d3ba18e082004b7de72ee2   (remote base, as §0 states)
git branch --show-current          main
git status --short                 empty
git diff / git diff --cached       empty
git worktree list                  one entry: D:/study/Software/FirmwareSight 7dc2ca8 [main]
git log --oneline f481b78..HEAD    f481b78 -> 8efe9c8 -> 7dc2ca8, nothing between
```

`8efe9c8` is the U1 convergence round (57 paths: 18 new UI files, 20 modified UI files, the additive
`MainWindowPage::Overview` page/title change in `apps/desktop/src-tauri/src/ipc.rs`, one Rust test file, five
`U1_VALIDATION/` documents and the governance pointers). `7dc2ca8` recorded the baseline artifacts at that head.
Both were local-only when this round began, so §1's precondition — "HEAD is the U1 tip, remote is one commit
behind, tree clean" — held, and no history operation of the forbidden kind was needed or used.

## 2. Pre-push diff audit (§2)

`git diff --stat f481b78..7dc2ca8` → **57 files changed, 4391 insertions(+), 1216 deletions(-)**;
`git diff --check` → no output, exit 0. The 57 paths classify as 23 added + 34 modified: UI presentation (46),
the additive window-page/title enum plus its Rust test (2), U1 documents (5), governance pointers (10 of the
modified set), and the two baseline-integrity artifacts.

Every forbidden surface was asked for by name — `crates/**`, `schemas/**`, `migrations/**`, `fixtures/**`,
`scripts/**`, `.github/**`, `Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `deny.toml`,
`tauri.conf.json`, `capabilities/**`, `assets/design-tokens.json`, `styles/tokens.css` — and the answer for all
of them is `NONE_TOUCHED`. The only non-CSS source change is the one the prompt's additive window allows.

**A defect this audit found in our own prose.** `BASELINE.yaml`, `.ai/ACTIVE_TASK.md`, `.ai/DECISIONS.md` and
both U1 documents said the desktop surface registers **27** Tauri commands. Reading the registry at three
revisions (`git show <rev>:apps/desktop/src-tauri/src/lib.rs`, counting `generate_handler![…]` entries) gives
**30 at `08fdfcb`, 30 at `f481b78` and 30 at `7dc2ca8`** — U1 added none and removed none. The material claim,
"U1 introduces no command", is true and stays true at 30 = 30; the number 27 predates P5 Commit C, which
registered the three history list commands. Corrected here and in every path §21 allows. `BASELINE.yaml` is
outside that allowlist, so the correction there is **reported to the owner rather than made silently**:
`u1_execution.new_commands_added: 0` is right, and its trailing comment "the desktop command count stays 27" is
wrong.

## 3. Complete local validation before the push (§3)

Each command run at `7dc2ca8` in this round, with the value it printed:

| Command | Measured | Expected | Forced? |
| --- | --- | --- | --- |
| `cargo fmt --all -- --check` | clean, exit 0 | clean | no |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 | exit 0 | no |
| `cargo test --workspace` (one invocation) | 47 result lines, **868 passed**, 0 failed, 0 ignored | 868 | no |
| `corepack pnpm install --frozen-lockfile` | exit 0 | exit 0 | no |
| `corepack pnpm typecheck` / `lint` | exit 0 / exit 0 | pass | no |
| `corepack pnpm test` | **9 files, 236 passed** | 236 in 9 | no |
| `corepack pnpm build` | exit 0 | exit 0 | no |
| `python scripts/check.py` | **17 of 17 steps PASS, 0 SKIP** | 17 | no |
| `… --only drift` / `--only deny` | 8 of 8 / 1 of 1 | — | no |
| `… --only core-smoke` | **3 of 3** | — | no |
| `… --only package` | **4 of 4, 0 SKIP** | — | no |
| `python scripts/verify_baseline_artifacts.py` | RESULT PASS, 750 tracked paths, 748 entries | PASS | no |

`--only core-smoke` and `--only package` had never been run in the earlier U1 round, which is why
`U1_VALIDATION/U1_VALIDATION_REPORT.md` §5 says so and quotes no figure for either. They were run here, and the
figures above are theirs. The `package` group builds a local release installer; §6 forbids using it as the
product under visual test, and it was not used — see §5 of this report.

## 4. Push (§4)

A normal fast-forward of `main`, no rebase, no squash, no amend, no force:

```text
git push origin main        f481b78..7dc2ca8  main -> main
git ls-remote origin main   7dc2ca80e5273b9aac48bb7ca216d24c2c1585ce
```

## 5. Exact remote CI, read back by head SHA (§5)

```text
gh run list --workflow "P0 verification gate" --commit 7dc2ca8…   37609108402
gh run view 37609108402  head=7dc2ca80e5273b9aac48bb7ca216d24c2c1585ce
                         workflow="P0 verification gate"  event=push
                         number=74  attempt=1  status=completed  conclusion=success
```

Ten jobs, each read individually, all `success` on attempt 1 — no rerun was used, and no run was invented for
`8efe9c8` (that commit was never pushed on its own; it reached the remote as an ancestor of the tip):

| Job | Conclusion | Job id |
| --- | --- | --- |
| Generated output drift | success | 112751746796 |
| Desktop UI (ubuntu-latest) | success | 112751746904 |
| Desktop UI (windows-latest) | success | 112751746942 |
| Rust (ubuntu-latest) | success | 112751747007 |
| Package Ubuntu | success | 112751747013 |
| Dependency policy | success | 112751747115 |
| macOS Core Smoke | success | 112751747129 |
| Rust (windows-latest) | success | 112751747177 |
| Package macOS | success | 112751747235 |
| Package Windows | success | 112751747245 |

**10 of 10.** Nothing in this round was installed or captured before this condition was met.

## 6. The exact Windows artifact, and what was installed (§6)

```text
artifact id            11477857379      name FirmwareSight-0.6.0-windows-x86_64   expired false
zip                    5,540,175 bytes  sha256 cf1a6c848f6abebb6148146e33c8758fc99dc0fce27e27044f45f06349d8b3b5
                                        ^ equals the digest actions/upload-artifact printed for this upload,
                                          read from the Package Windows job log
inside the zip         SHA256SUMS.txt checked with sha256sum -c: installer OK, cli zip OK
artifact-metadata.json git_commit 7dc2ca80e5273b9aac48bb7ca216d24c2c1585ce, rustc 1.98.1, node v24.21.0,
                       tauri-cli 2.12.1, runner win25-vs2026, unsigned, updater artifacts false
NSIS installer         3,892,287 bytes  sha256 372631c367b34dc5c985025fb498d56dd8c70af6b31e33c8b9eaae6fd506f6f8
installed executable   15,366,144 bytes sha256 afdc528b97dcdce147183f272e4fdd5a97325855103213d69348e7c4398070a0
```

The installer was driven with real mouse input through the NSIS wizard (Welcome → Choose Install Location →
Choose Start Menu Folder → Installation Complete), no silent flag. The three sizes are quoted separately on
purpose: the container is 5,540,175 bytes, the installer inside it is 3,892,287, and the payload the bundler
hashed is neither of them. The installed executable's own digest differs from the metadata's `payload_sha256`
because the bundler rewrites the 27-byte `__TAURI_BUNDLE_TYPE_VAR_*` token in the packed copy; that documented
packaging behaviour is recorded in the artifact identity file rather than smoothed over.

Substitutions this round refused: F3's V1 cohort artifact `11419727517`, any F2/F2R artifact, the local release
installer this round's own `--only package` step produced, `cargo run`, `cargo tauri build`, `pnpm dev`/Vite, and
any source-tree execution.

## 7. Owner-store safety (§9)

```text
python target/u1_park.py snap     BASELINE.json: three files, sizes 155648 / 0 / 32768, digests recorded
python target/u1_park.py park     baseline_matches_live: true
                                  backup digests match the originals: YES
                                  OWNER_STORE_PARKED = YES
… the whole installed round ran with the owner's bytes outside the live path …
python target/u1_park.py restore  ORIGINAL_DB_RESTORED = YES
                                  ORIGINAL_DB_SHA_MATCH  = YES
                                  OWNER_STORE_OPENED_BY_U1 = NO
```

What "opened" means here, stated precisely because the round produced a subtlety: with the owner's three names
absent, the installed product **created a fresh store at the default location** (4,096-byte main file plus a WAL
that grew to 622,152 bytes). Those bytes are this round's disposable live app data, not the owner's — recorded
with digests in `13_owner_restore/PRE_RESTORE_LIVE.txt`, each one verified not to match the pre-round baseline
before it was removed by name. The owner's own bytes never sat at the path the product reads during the round,
were never opened, and came back identical in size and digest with their original mtimes. The 14 unrelated
entries in that folder (earlier phases' historical stores) were untouched throughout, and the uninstaller's
"delete the application data" option was never clicked.

## 8. Screenshots, manifest and hashes (§10–§12)

Evidence root, outside Git, printed in full in §14 below. Six primary states at **1440×900 client area**, nine
responsive captures (five at 1024×720, four at 1056×799), plus the working probes the driver took while walking
the product. `00_authority/SCREENSHOT_MANIFEST.tsv` carries one row per image: page and state, the size
requested of `SetWindowPos`, the size read from that PNG's own IHDR, capture time, bytes, sha256, the installed
executable's digest, the installer's digest, run id, artifact id and head SHA — so any single image can be
traced to the exact bytes it certifies.

Geometry is never claimed from the request: the capture command resizes, reads back `GetWindowRect` and the
client rectangle, and refuses to write a file when the measured size is not the requested one. Pixels come from
a screen grab of exactly the client rectangle, so the top edge of every image is the product's own bar — the
same rectangle as the 1440×900 mockups.

## 9. Responsive sanity (§12, small sizes)

At 1024×720 (`R01`–`R05`) and 1056×799 (`R06`–`R09`), against the five things §12 requires:

- **no unreadable collapse** — held. The rail, the header, the chips and the tables keep their shapes; nothing
  overlaps or truncates text inside a component.
- **no critical clipped action** — held. `Details`, the pagers, `Run Gate`, `Compare`, `Analyze` and the policy
  buttons are all on screen at both sizes.
- **no accidental page-wide overflow** — held. The only horizontal scrolling belongs to a wide table inside its
  own scroll container (`R05`, `R07`), not to the page.
- **navigation usable** — held, with one finding: the rail travels with the page rather than staying fixed
  (U1-V1-05), so on a long page a person scrolls their navigation away.
- **primary actions reachable** — held.

The one cost at small sizes is that a wide table's last column sits behind that table's own scrollbar at rest
(U1-V1-08). Recorded as a finding, not waived.

## 10. Functional smoke on the same bytes (§13)

Full record in `12_smoke/FUNCTIONAL_SMOKE.md`. Summary: Overview opens and every page was reached by real input
with the window title read back after each click; Analyze selected a file through the real dialog, produced a
success result (19 sections, 52 symbols), rendered its Sections and Symbols tables and switched tabs, and
produced a genuine parse failure (`ERR-PARSE-2002`, diagnostics id `op-5cf8-18dc3bd7c4dd9e34`) on a truncated
ELF without inventing a result; Compare rendered a valid pair, its deltas, its "What moved" counts, its
contributor lists and both change tables; Release Gate rendered the five states and its actions were exercised
end to end through a real bundle export; History rendered both stored lists and its `Details` action expanded
real rows; Help/Diagnostics is present with its facts and its export control. **No S0/S1/S2 functional
regression**, so §13 does not stop visual closure.

Compare has no export control in this build — measured by grep over `apps/desktop/ui/src`, which finds export
only on Release and Help — so §13's "where currently supported" is satisfied by the two that exist.

## 11. Visual findings and the convergence matrix (§14–§16)

`U1_CONVERGENCE_MATRIX.md` in the review pack rates the eight required dimensions across the implemented pages
using only MATCH / CLOSE / PARTIAL / GAP, with REFERENCE / CURRENT / GAP for each row and the screenshot named
as the evidence. Nothing in it is rated from CSS.

Severity register, per §16:

- **U1-V2-06** — during a failed parse the capability chips still read "ELF supported / MAP provided" in green
  above a row that says `MAP: Not provided` (`S06`, `R02`, `R06`). Rated V2, not V4: the page states the truth
  one block lower and no fact is asserted falsely; rated V2, not V1: the unqualified green row is the first
  thing a person reads on that screen. **Not self-waived. Returned to the Architect.**
- **U1-V1-01/02/03/04/05/07/08** — Overview's stacked state cards pushing metrics below the fold; Analyze
  leading with fact lists instead of its tables; no paired right-hand detail on Analyze, Compare, Release Gate
  or History; the page heading "History" against the rail and title "Bundle & History"; the scrolling rail;
  `RankBar` labels running name and role together (`.debug_infoDebug`); wide tables' last column behind a
  table-scoped scrollbar at small sizes.
- **U1-V0-09** — the product bar's project chip is absent before a project config is loaded and present after,
  so the same screen reads differently across pages.
- **No U1-V3 and no U1-V4 finding was observed**, which is the condition §21 puts on making an evidence commit.

## 12. Reference-only capabilities (§7, §12, §15)

`S07` was **not** captured, because the product does not have the dependency / declare / re-scan workflow the
mockup shows. `10_reference_only/S07_REFERENCE_ONLY.md` states the measured reason (no command among the 30
registered does this; no storage for a declaration exists; a screenshot would need invented rows), records the
disposition `NOT_IMPLEMENTED_IN_U1`, and assesses only the reusable grammar — tabs, the UNKNOWN glyph, the
"not present" cell, the real modal path — against surfaces that do exist. The same treatment is recorded for
`Link repository`, `Mark N/A`, History's bundle actions, the theme toggle and card elevation.

## 13. V1 interlock (§8, §23)

`V1` stays `IN_PROGRESS` / `RECRUITMENT_READY` with **0** eligible external sessions, and participant execution
is `PAUSED_FOR_ARCHITECT_UI_REVIEW`. This round ran no session, recruited no participant, and did not re-freeze
or re-download V1's cohort artifact — F3 head `08fdfcb710f78f8084bfcf614dc508c8b6e7e25b`, run `37475580080`,
artifact `11419727517` — which remains the Architect's decision. `U1` was not renamed to `B1`; `B1` stays NOT
AUTHORIZED, and U1 claims no stage.

## 14. Where the evidence lives

```text
C:\Users\16429\AppData\Local\Temp\FirmwareSight-U1-Visual-Acceptance-20261007T1037Z
  00_authority/   preflight, method, machine snapshot, CI run id, both prompt revisions with their delivered
                  digests (v1.1 99012d9c0e5ae9080b357fc41b1d4843c623a6417524db43240b9c07cd72b3ca,
                  v1.0 c6ab8ca327d0e0d5c6abb7c2475163a749f745e247393f00350cbf718339b081), SCREENSHOT_MANIFEST.tsv
  01_artifact/    the downloaded zip, its SHA256SUMS and metadata, the installer, INSTALL.json
  02_owner_backup/ BASELINE.json, PARK.json, backup/ (independent copy), parked/ (emptied by the restore),
                   RESTORE.json / OWNER_RESTORE.json
  03_references/  the seven mockups as copied, MOCKUP_SHA256SUMS.txt
  04..09, 11      S01-S06 and R01-R09, plus the working probes
  10_reference_only/ S07_REFERENCE_ONLY.md
  12_smoke/       FUNCTIONAL_SMOKE.md and the SM01-SM14 captures it cites
  13_owner_restore/ UNINSTALL_raw.txt, UNINSTALL.md, PRE_RESTORE_LIVE.txt, OWNER_RESTORE.txt, OWNER_RESTORE.json
  14_review_pack/U1_VISUAL_REVIEW_PACK/  the §18 deliverable
```

## 15. Recommendation

`U1 = READY_FOR_ARCHITECT_VISUAL_REVIEW`.

The strongest statement this evidence supports: **the U1 desktop UI was built, pushed, published by CI,
installed from the bytes CI produced from that commit, and inspected on a real screen at three window sizes;
it works, it is visibly closer to the reference direction than the pre-U1 shell on the structural dimensions
the audit lists, and it has one material presentation defect (U1-V2-06) plus eight smaller ones that a designer
should adjudicate before U1 is called converged.**

It is **not** "visually approved", not `PASS_COMPLETE`, not `DESIGN_COMPLETE`, not `MOCKUP_MATCHED`, not a
before/after pixel comparison (the owner chose after-only evidence, so the before capture is recorded as
`NOT_CAPTURED`), not V1, not B1, not RC, and not GA.
