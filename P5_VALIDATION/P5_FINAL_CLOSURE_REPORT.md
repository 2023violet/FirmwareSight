---
title: "P5 Final Closure Report"
doc_id: "FS-P5-FINAL-CLOSURE"
product: "FirmwareSight"
version: "0.6.0"
status: "VALIDATED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 — final closure report (Commit F3, 2026-10-06)

This is the stage's closing record: what was authorized, what was measured, what was deliberately left
unfinished, and what is now forbidden to read into the words above. It is concise on purpose — the measurements
live in the eleven documents it points at, and restating them here would create a second source of truth for
facts that already have one.

## 1. Start authority

Preflight ran before any write, as F3 §1 required: `HEAD = origin/main = a5ce7c43a1656966efbb9d2d99ea64d1aad27f97`,
the working tree clean at the head that closed Commit F2R, and the authoritative predecessor run read back from
the API rather than recalled — `37453402452` (run #71), attempt 1, `completed / success`, **10 of 10 jobs**,
each of the ten job names enumerated individually. The F3 prompt itself is archived with **both** digests,
because transport moves them and the repository records that instead of hiding it: delivered bytes SHA-256
`860e00976b242f15de1473455941140599a8a5cd639409521e7c38a28a64514c` (36,458 bytes, 1,821 CRLF lines), stored Git
blob `6cacd10ee1eff4a6c4ab90b4f9d51df987bad16cf94fe142961c0d37fd749c66` (34,637 bytes, 1,821 LF lines). The
byte delta is exactly the 1,821 carriage returns, the line-by-line comparison of the delivered file split on
CRLF against the stored blob split on LF returns identical for every logical line with zero differing lines, and
`.gitattributes` was not modified to make the two hashes agree — that file is an `AGENTS.md` §9 integrity
boundary. No reconstruction was needed: the delivered file existed and was copied verbatim.

## 2. P5 scope

P5's task was productization, not product: carry the G2-passed MVP candidate to a build a stranger engineer can
install, understand, use (Analyze / Compare / Gate / Bundle), inspect in local History and Diagnostics, recover
from, and uninstall or reinstall — without the dev team present. It added no verb, no crate, no schema major,
no dependency line to the product graph, no network capability and no telemetry. The four product verbs and
every technical baseline in `AGENTS.md` §2 held unchanged through eleven heads.

## 3. Commit A–F2R lineage

The complete head-by-head record, with every red and interrupted run kept, is
`P5_EXECUTION_REPORT.md` §7; the run-level authority is `P5_CI_AUTHORITY.md`. In shape: **A** audit and
governance, then migration `0005` after its written decision · **B** packaging on three runners, the job set
grown from seven to ten, and the `SKIP`-that-read-as-pass defect that made `check.py` fail on any skipped gate
step in CI · **C** onboarding, Help/About with the window title owned by Rust, and History over three bounded
read APIs with **no new migration** · **D** storage-owned `integrity_check`, the WAL-safe pre-migration
snapshot, the closed 41-key Diagnostics allowlist, the typed startup refusal and `ADR-0028` · **E** the
compatibility cohort (which found a real undercount and earned the narrow `SHF_ALLOC` fix), §48's fourteen
dispositions, and the L15 answer as Option E · **E normalization** the closed status and disposition
vocabularies · **F1** `ADR-0029` and the index-blob baseline, inside the authoritative drift step, plus L15's
display caption · **F2** the §64 installed journey on F1's own CI artifact and 34 measured migration checks ·
**F2R1** the three installed-UI fixes inside `apps/desktop/ui/src/**` · **F2R2** the read-back that revalidated
them on F2R1's own artifact and restored the owner's store byte-exact · **F3** this closure.

Four heads did not come back green on the first attempt and all four keep their rows: `9e3b1de` (6 of 7, a
pre-existing `compare.test.tsx` race it never touched), `0c031cd` (7 of 10, three package jobs whose skip was
caught by their own upload step), `3400981` (8 of 10, a clock race inside a test that commit wrote, repaired
test-only in `bccea88`), and `80b47c4` (9 of 10 on attempt 1 — Ubuntu rustup provisioning, killed before it
compiled anything — 10 of 10 on attempt 2). F2's `55fad63` took three attempts, the first two being zero-step
allocation cancellations, and `37262147348`/`37367382516` are recorded as two and three attempts rather than
smoothed into first-attempt green. Three heads have **no run of their own** because one push of two commits
starts one run at the tip: E1 `859648e`, F2's `d1dc61c` and F2R2's `31f10c7`.

## 4. Final engineering exit checklist

`P5_EXIT_CHECKLIST.md` §7 is the check, and §6's re-audit is what licensed this document: **nineteen areas
re-read, every required engineering item landing on `PASS`, `CARRIED_FORWARD_NONBLOCKING` or
`OWNER_DECISION_NONBLOCKING`, and no required item `BLOCKED`.** Compatibility is `PASS` *within the documented
cohort*; Design/Accessibility is `PASS_FOR_FROZEN_DESKTOP_SCOPE`, carried exactly from F2R and not upgraded;
Security/supportability is `PASS_WITH_DOCUMENTED_ACCEPTED_RISKS`, which is a different sentence from "security
clean" and the repository has never written the other one; Signing and Notarization are `PASS` as
`READY_NOT_EXECUTED`; License is `OWNER_DECISION_NONBLOCKING`.

One defect the re-audit found was in this repository's own paperwork rather than in the product: the exit
checklist asserted every L1–L26 row was accounted for, and counting `P5_KNOWN_LIMITATIONS.md` at `a5ce7c4` found
**25 of 26 — L12 had no row**, although the two accepted RustSec advisories it covers were carried honestly in
`P5_SUPPORTABILITY_REPORT.md` §2 and `P5_SECURITY_SUPPORTABILITY_REVIEW.md` §2. F3 added the row and corrected
the assertion instead of deleting the claim.

## 5. Installed acceptance authority

Installed evidence is F2's and F2R's, and F3 produced none: §24 forbids reinstalling in this round because F3
changes no product byte, so there would be nothing new to test and a real risk of disturbing the tree the
installed verdicts describe. The chain of custody stays as it was recorded — **F1 head `0bca373`, run
`37293381181`, artifact `11337963032`** for the §64 journey and the 34-check v4 → v5 migration; **F2R1 head
`fb5f628`, run `37431977428`, artifact `11397938806`** (`FirmwareSight_0.6.0_x64-setup.exe`, 3,886,598 bytes,
`efbc45a3…d5fd4`, installing `firmwaresight-desktop.exe` at 15,362,048 bytes / `2cf01a6d…670b`) for the
revalidation of the three corrected surfaces at 1024×720, 1056×799 and 1440×900. Both were installed **as CI
built them**: no local rebuild, no re-packaging, no dev server, no other installer.

## 6. F2R corrective authority

The narrow corrective the Architect inserted between F2 and F3 closed F2's three findings and nothing else:
`apps/desktop/ui/src/**` only, no token, no dependency, no capability, no contract, no Rust. The mechanism was
measured before it was changed and the rejected alternatives are recorded (`P5_F2R_UI_CORRECTIVE_DESIGN.md` §3,
§5, §7), the four mutation proofs each reddened their own test, the 20 fresh-process repetitions came back 20
green, and the honest trade at 1440×900 — a contained scroll of ~1.06× the pane replacing one-character-per-line
prose — is written down rather than smoothed. `L20`'s sequence (E `CLOSED` on Compare → F2 `REOPENED_BY_F2` on
Release → F2R `CLOSED_BY_F2R` across verified human-facing memory-basis surfaces → final `CLOSED`) is reconciled
with dates in `P5_SUPPORTABILITY_REPORT.md` §2 and `P5_KNOWN_LIMITATIONS.md` §8; Commit E's measurement was not
rewritten.

## 7. Owner-data safety

F3 opened nothing of the owner's and installed nothing, so no parking was required (§24). The inherited state
is F2R's, and it was re-read rather than assumed: `OWNER_STORE_PARKED = YES` and `OWNER_BACKUP_HASH_MATCH = YES`
before F2R's install; `ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`,
`OWNER_STORE_OPENED_BY_F2R = NO` after it, with the parked copy verified against the original digests before it
was moved back and again afterwards. F2's earlier cycle carried the same three gates twice, and the owner's store
hashes (`d6e41034…` at 155,648 bytes, `e3b0c442…` at 0, `fd4c9fda…` at 32,768) matched byte for byte on both
passes. The uninstaller's "Delete the application data" option has **still never been exercised** — that folder
holds unrelated historical stores from earlier phases — and it stays recorded as a coverage boundary rather than
a pass.

## 8. Final known limitations

`P5_KNOWN_LIMITATIONS.md` is the canonical list: 26 rows, every one accounted for, and **no row became `CLOSED`
because P5 closed** (§11). Carried forward: L3 fuzzing (blocked by `ADR-0016` plus the pinned `1.98.1` toolchain,
so the no-panic claim rests on `fixtures/malformed/*`), L8 object/module attribution, L11 real-user
comprehension (V0 `0 / 8`, owner V1), L12 the two accepted advisories, L15's legacy wire identifier, and L9's
undrivable native `<select>`. Reduced with named residue: L4 (one host, 100 % scale, one WebView2), L5 (ELF
only, DWARF unread), L17, L23 (seven race instances fixed and proved by mutation; a differently-shaped wave may
still exist), and the performance rows L1/L2. `NOT_REPRODUCED`: L25. `OWNER_DECISION`: L13. `BY_DESIGN`: L18.
`CLOSED` rows stay visible with what they do not cover. Installed migration coverage is still **one path**
(v4 → v5) while the owner's real store sits at schema v2, so the chained v2 → v5 installed upgrade remains
unexecuted; the machinery behind it is integration-tested for v1–v4 → v5, v5 reopen and failure rollback.

Measured performance truth, unchanged (§12): a valid ELF of **519,179,252 bytes** with about **2,020,073
symbols**, warm Analyze **~4.73–4.89 s**, cold first read **~68.7 s**, peak observed working set **~1,428 MB**,
never "Not Responding", never a crash. Near-500-MiB UI is `MEASURED`; first-use-under-60 s is **not proved at
this workload** and stays environment-sensitive; RSS is measured for the tested workload only. No claim
that performance was optimized, no claim that a memory target was achieved and no claim that every large file
finishes inside a stated window appears anywhere in this repository.

## 9. Release readiness

`P5_RELEASE_READINESS.md` §5 holds the ten states, and F3 moved none of them:

```
SIGNING_READINESS            = READY_NOT_EXECUTED
NOTARIZATION_READINESS       = READY_NOT_EXECUTED
UPDATE_MODE                  = MANUAL_UPGRADE_READY
OPEN_SOURCE_LICENSE_DECISION = PENDING_OWNER_CONFIRMATION
PUBLIC_DISTRIBUTION          = NOT_AUTHORIZED
GITHUB_RELEASE               = NOT_CREATED
TAG                          = NOT_CREATED
PRIVATE_BETA                 = NOT_AUTHORIZED
RC                           = NOT_AUTHORIZED
GA                           = NOT_AUTHORIZED
```

`MANUAL_UPGRADE_READY` rests on measurements, not on intent: the installed round trip — run the newer installer
over the older, the store migrates on first launch behind a named pre-migration snapshot — was walked on two
CI-built artifacts, with repair over a running app, uninstall, reinstall and a reopen at schema v5 with no
migration re-run. `createUpdaterArtifacts: false` is untouched, there is no endpoint and no certificate, and
`AGENTS.md` §2 keeps an updater behind a signing and key-management ADR that does not exist.

## 10. License boundary

No licence decision exists, and F3 did not make one. `license = "Proprietary"` stands in the root `Cargo.toml`,
there is no root `LICENSE`, and no P5 commit added one: `AGENTS.md` §9 puts a licence change in front of a human
and the P5 prompt's §54 forbade this round from choosing one. The consequence is stated rather than worked
around:

```
PUBLIC OPEN-SOURCE REDISTRIBUTION CLAIM = BLOCKED BY OWNER LICENSE DECISION
```

That is the **only** `BLOCKED` phrase in the P5 record, and it is a claim about redistribution, not a required
engineering item — which is why §6's re-audit could pass with it in place. FirmwareSight must not be described
as a finished licensed open-source release by anything in this stage's evidence.

## 11. Final local validation

Run on the final staged F3 tree and reported as measured, not as expected. Logs: `F3` evidence root outside the
repository (`GATE_FULL_1.txt`, `GATE_GROUPS.txt`, `CLIPPY_TEST.txt`, `FRONTEND.txt`, `FMT_CHECK.txt`,
`ALLOWLIST_F3.txt`, `PRODUCT_BYTES_IMMUTABLE_F3.txt`).

| Check | Result |
| --- | --- |
| `git diff --check` / `git diff --cached --check` | exit 0, no output (four files needed their trailing blank line trimmed first — `git diff --cached --check` flagged them, and the fix was whitespace only) |
| `cargo fmt --all -- --check` | exit 0, no output |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0, no warnings |
| `cargo test --workspace` | **868 passed / 0 failed / 0 ignored** over 47 test-result lines — the count §5 requires unchanged |
| `corepack pnpm install --frozen-lockfile` | exit 0 |
| `corepack pnpm typecheck` / `lint` | exit 0 / exit 0 |
| `corepack pnpm test` | **225 passed in 8 files** (`Test Files 8 passed (8)`), colours stripped before counting |
| `corepack pnpm build` | exit 0, emitting the same content-hashed assets F2R1 built (`index-R0fRHss-.css`, `index-DCexUFJu.js`) — the machine's way of saying no product byte moved |
| `python scripts/check.py` | **17 of 17 steps PASS**, exit 0: `rust/fmt`, `rust/clippy`, `rust/test`, `frontend/install`, `frontend/typecheck`, `frontend/lint`, `frontend/test`, `frontend/build`, `drift/design tokens`, `drift/desktop icons`, `drift/baseline integrity`, `drift/ipc bindings`, `drift/ipc bindings unchanged`, `drift/fixtures tracked`, `drift/version identity`, `drift/goldens unchanged`, `deny/cargo-deny`. **No `SKIP` and no `FAIL` anywhere in the log** |
| `--only drift` / `--only deny` / `--only core-smoke` / `--only package` | **8/8**, **1/1**, **3/3**, **4/4**, each exit 0, no skip |
| `python scripts/verify_baseline_artifacts.py` | `RESULT PASS` — tracked **710**, manifest entries **708**, `index blob mismatch 0`, `listed but unindexed 0`, `listed but untracked 0`, `tracked but unlisted 0`, `duplicate entries 0`, `unmerged index entries 0`, sums sorted, no CRLF in either artifact, no host path |
| §25 allowlist, proved from the staged diff | 30 changed paths, **0** under `apps/` `crates/` `scripts/` `fixtures/` `schemas/` `golden/` `migrations/` `.github/`, and none of `Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json`, `deny.toml`, `rust-toolchain.toml`, `assets/design-tokens.json` |
| Product-byte immutability, proved from tree OIDs | `apps` `dd8cccfc…`, `crates` `0c933323…`, `apps/desktop/ui/src` `8e893af3…`, `scripts` `76c1e1cb…`, `.github` `174bd58f…`, `assets/design-tokens.json` `b818d113…`, `Cargo.lock`, both lockfiles, `tauri.conf.json` and `deny.toml` **all identical** between `a5ce7c4` and the staged F3 tree. Migrations live under `crates/`, so an identical `crates` tree also proves `0001`–`0005` and SQLite schema 5 did not move |

Two proofs are structurally impossible to include in this table from the inside: the clean detached worktree at
the F3 SHA and `git archive` of it, because neither can address a SHA that does not exist until this commit is
written. They run after the commit, against its exact SHA, and are reported to the Architect; §28 and §29
require them, §34 forbids buying them a further commit.

## 12. F3 governance commit

One commit, preferred message `P5: close productization after installed and corrective acceptance`, containing
governance, evidence and indexing only. §4's boundary was proved from the staged diff rather than asserted:
allowed families are `BASELINE.yaml`, `.ai/*.md`, `README.md`, `INDEX.md`, `06_DELIVERY/*.md` (only where the
stage state was materially stale — `06_STAGE_GATES.md`'s P5 line), `P5_VALIDATION/*.md`,
`10_AUDIT/SOURCE_PROMPTS/*` and `DIRECTORY_TREE.txt` / `SHA256SUMS`. **Zero** hits under `apps/`, `crates/`,
`scripts/`, `fixtures/`, `schemas/`, `golden/`, `migrations/` or `.github/`, and no change to `Cargo.toml`,
`Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json`, `deny.toml`, `rust-toolchain.toml` or
`assets/design-tokens.json`. F3 is not allowed to "just fix one last thing", and it did not.

The baseline followed `ADR-0029`'s order: edit → inspect → stage → regenerate `DIRECTORY_TREE.txt` from the
stage-0 index → stage it → regenerate `SHA256SUMS` from stage-0 blob bytes → stage it → run the independent
verifier → run the gate → inspect staged → commit. Working-directory bytes are not the authority. Push is a
normal fast-forward; no force, no history rewrite, no branch deletion, and the temporary detached worktree
created for §28 is removed afterwards with only that worktree removed.

## 13. F3 remote CI — external, and pending inside this commit

```
F3 remote CI = PENDING_EXTERNAL_EVIDENCE
FINAL EXTERNAL ACCEPTANCE REQUIRES F3 REMOTE CI
```

The SHA is not knowable until this commit exists and the run is not knowable until it is pushed, so the commit
that carries the closure sentence cannot certify its own remote verdict. That is the same asymmetry F1, F2,
Commit E and F2R2 each met and named rather than papered over. §19 forbids fabricating the run here; §34
forbids a further commit written only to record it, so there is no F4 whose content is a number. §33 fixes the
failure rule: **if F3's remote run fails because of repository content, the external `P5 = PASS_COMPLETE` is not
accepted**, the issue is fixed forward in a corrective successor, a content failure is never re-rolled until a
green attempt appears, and a pure provisioning failure may justify one rerun only with the log read first and
both attempts left in the record. The twelve items §32 requires of that read: the ten named jobs each read
individually, Rust 868 and UI 225 / 8 files confirmed from the runners' own logs, drift including baseline
integrity, packages building, verifying and uploading, and no required gate skipped — a conditional
Linux-prerequisite step skipping on a Windows runner is the job's design, not a gate skip.

## 14. Verdict

**`P5 = PASS_COMPLETE`** · **Productization = `ENGINEERING_COMPLETE`** · **stage = P5** ·
**`active_task = NONE`** · **product = `MVP_CANDIDATE`** · narrative **FirmwareSight Productized MVP Candidate**
· **baseline = `0.6.0`** · `G2 = PASS`, P0/P1/P2/P3/P4 unchanged, no retroactive re-statement ·
**final external acceptance requires F3 remote CI**.

It means the MVP was productized to P5's engineering scope, evidenced by an installable package pipeline on
three platforms, a Windows real installed journey, migration with backup and recovery, onboarding, History,
Diagnostics, a compatibility cohort, repository baseline integrity, an installed-UI corrective closed on its
own CI-built artifact, data preserved across uninstall and reinstall, owner-data safety, release-readiness
documentation and a final engineering gate.

It does **not** mean market fit, real-user validation, beta quality, RC quality, GA readiness, a signed release,
a notarized release, an automatic update path, all-platform runtime validation, WCAG certification, all-DPI
validation, a selected licence or a public open-source release. And it authorizes no next track: V1 is the
likely next decision and needs a new architect prompt. §37 ends the round here.
