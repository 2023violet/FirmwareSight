---
title: "G2 Engineering Closure Report"
doc_id: "FS-G2-REPORT"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "G2_ENGINEERING_CLOSURE_AUDIT"
owner: "Engineering"
last_updated: "2026-10-01"
---

# FirmwareSight G2 Engineering Closure Report

Companion documents: `G2_EVIDENCE_MATRIX.md` (every requirement, classified and cited),
`G2_EXIT_CHECKLIST.md` (§34 box by box), `G2_END_TO_END_SMOKE_REPORT.md` (the CLI chain, the window,
parity, failure and recovery), `G2_KNOWN_LIMITATIONS.md` (the one canonical list). This report says
what was done and decided; the tables live in those four files.

## 1. Start state

`HEAD = origin/main = 055b54e9d6f0a9e53b8a5c65afe1f161bb5dcc95`, one worktree, clean. Run `36899128645`
on that head: attempt 1 completed / success, 7 of 7, and two further Windows UI executions green.
Governance at start: P0 PASS, G1 PASS on the P0 basis, P1–P4 PASS / COMPLETE, G2
`READY_FOR_ENGINEERING_GATE_REVIEW`, `active_task: NONE`, baseline 0.6.0, V0 `0 / 8` non-blocking,
licence pending the owner.

The audited tree became `e35cfe70aa0e8f273a75ac14b9dc62377483af36` during the round (G2-F1, §8). That
commit changes one test file and `SHA256SUMS`; no product source moved, so every product observation
below was made on product source identical to `055b54e`.

## 2. Authority

*FirmwareSight — G2 Product MVP Engineering Closure Audit, Execution Prompt v1.0 — Architect Reviewed*,
delivered as a file (SHA-256 `3c6ab83e11ce6a4c91a609dd0a6bc318e700dc0cbbccd3d4493eaf65c3e2bac9`), and
its *G2 Storage Path Semantics Clarification Addendum v1.0*, supplied inline. The addendum overrides
only the host-path persistence wording of §13 / §34 and the audit head. The exit definition is
`06_DELIVERY/06_STAGE_GATES.md` §G2 as amended by `ADR-0026`: engineering self-evidence, with every
user-panel metric `POST_MVP / NOT CURRENT GATE`.

## 3. P1–P4 stage audit

All four stages keep their verdicts on the current tree: each acceptance list is the frozen user story,
each pack has a verdict with its PARTIAL items stated, each has a green 7-of-7 remote run, and each
capability is exercised again by this round's tests and smoke. US-005 (CLI) has no stage of its own and
is covered across P2–P4. Matrix: M§2.

## 4. Current architecture

Five first-party library crates; Core zero-dependency, synchronous and filesystem-free; Project owns
config, the read-only bounded Git adapter and the one bundle engine both shells call; Storage alone
uses `rusqlite`; Report owns the projections. One documentation inaccuracy in the P4 pack was found and
is recorded rather than rewritten: `P4_BUNDLE_EXECUTION_REPORT.md` §1 says the desktop and CLI "own
staging, rename and rollback", while the source puts all three in `firmwaresight_project::bundle` and the
shells only call it. Matrix: M§6.

## 5. Public contracts

Six contracts at major 1, each closed at the top level with a required, pinned version; current outputs
validate through the in-repo contract tests and independently through `jsonschema`. Two path-shaped
fields are constrained by code rather than by schema, and both constraints were exercised. Matrix: M§4.

## 6. Storage and migrations

`SCHEMA_VERSION` 4; every upgrade path, the fail-closed future version, recoverable failed migrations,
foreign keys and the three immutability rules are proven by named tests and by triggers, and were
observed on the smoke store. Matrix: M§5.

## 7. Security and capabilities

The WebView holds `core:default` only; 23 use-case commands take no path, SQL, shell or Git argument; no
network API in first-party code; no HTTP client in the shipping graph on any of the three targets;
`cargo deny` passes with the two documented accepted advisories. Matrix: M§6. This is a policy-passing
dependency graph with documented accepted risks, not a claim of a security-clean tree.

## 8. Full local gate, and the finding it produced

On `055b54e`, before any write: `cargo fmt --check` 0, `clippy --workspace --all-targets --all-features
-D warnings` 0, one `cargo test --workspace` **769 passed / 0 failed / 0 ignored** (41 result lines, 33
non-empty), `pnpm install --frozen-lockfile` / `typecheck` / `lint` / `build` 0, `check.py --only drift`
6/6, `--only deny` 1/1 (`advisories ok, bans ok, licenses ok, sources ok`), `--only core-smoke` 3/3 —
and `corepack pnpm test` **exit 1: 1 failed / 154 passed (155)**.

**G2-F1.** `release.test.tsx` › *Release bundle* › "puts focus on the decision it just asked for"
expected `document.activeElement` to be the Replace button and received Export. `Release.tsx` moves
focus in a passive effect keyed on `confirm`, so the button exists one turn before it is focused, and
the test read focus synchronously after `findByRole`. The test came from `6b7e23e` (P4). Unfixed rate
on this machine: `release.test.tsx` alone 29/30 green; the named failure came from the full suite.
Reproduced deterministically by deferring the product's `focus()` 25 ms in a temporary test-only
mutation. Fixed by `await waitFor(() => expect(document.activeElement).toBe(replace))`, with the
expected element and every other assertion unchanged. Proofs, each restored byte-for-byte by SHA-256:
the deferred-focus mutation now passes; removing `confirmRef.current?.focus()` from `Release.tsx` fails
the repaired test at the `waitFor`. Target 30/30, file 30/30 (56/56), full suite 15/15 (155/155),
`check.py` 15/15. Committed alone as `e35cfe7` (prompt §41); Run `36906482900`, attempt 1, 7/7. CLOSED.

On the fixed tree the gate was re-run: `check.py` **15/15**. The other two `document.activeElement`
assertions in the suite were read and are not the same race (`compare.test.tsx:943`, `:958` follow a
`waitFor` on a request the same effect issues after focusing; `:1113` focuses directly).

**G2-F3.** `scripts/update_goldens.py` without `--confirm` exited 1 in `release_subject()`:
`shutil.rmtree(target/update_goldens-p4)` raised `PermissionError [WinError 5]` on a read-only Git
object left by an earlier run. No tracked file changed. The updater is not part of the gate or of CI;
recorded as L24 and not fixed here.

## 9. Clean detached worktree

`git worktree add --detach D:/fsg2wt e35cfe7`: clean, and no `node_modules`, `target` or `dist`. In it,
`python scripts/check.py` ran **17/17** in 401 s — the two extra steps are the frontend install and build
the gate performs when `dist/` is absent — with 769/0/0 Rust, 155/155 UI, `deny` all ok, drift
including `fixtures tracked` and `goldens unchanged`, and `--only core-smoke` 3/3. Afterwards: 0
tracked changes, 0 untracked non-ignored files, `Cargo.lock` and `pnpm-lock.yaml` unchanged. **No
dependency on an ignored or untracked file.** Removed: `git worktree remove --force` deregistered it and
left only empty directories and 467 pnpm junctions, every target inside `D:\fsg2wt` and 0 files
(verified with PowerShell before deleting); `git worktree list` shows the main worktree only.

## 10.–15. CLI and desktop smoke, parity, failure, determinism, portability

`G2_END_TO_END_SMOKE_REPORT.md`, in that order. In one line each: the CLI chain exits 0 five times,
twice, byte-identically, without touching the store; the desktop walks S1–S43 on the shipping binary;
every cross-surface fact is equal, three of them byte-for-byte (Diff JSON, Diff HTML, the whole
bundle); dirty Git, no Git, a stale source after preview and a malformed file all fail closed with an
explicit code and leave nothing half-written; every identity and document is stable across producers and
destinations; the relocated bundle answers every question with project, source, store and app absent.

## 16. PRD success metrics

M§9. One metric is **NOT_MEASURED** and stays so: the 500 MB working set in the window. Per the
prompt's §27 adjudication it does not block G2, because the 512 MiB guard is enforced, the G2 fixtures
are reliable and no hang or crash was observed — and it is not claimed as proven.

## 17. Known limitations

`G2_KNOWN_LIMITATIONS.md`: 25 rows, none blocking. Inherited gaps were kept; the new ones are the
`artifacts.path` storage fact (L18, adjudicated), the Object-attribution wording (L19), the Compare enum
label (L20), the fixed window title (L21), line endings moving a release id (L22), the race shape (L23),
and the updater on Windows (L24).

## 18. Licence gap

`license = "Proprietary"` in the root `Cargo.toml`, no root `LICENSE`,
`OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION`. Nothing was chosen or added. **Technical MVP
Candidate ≠ open-source licensing completed.**

## 19. Dependency / RustSec state

`cargo deny` on a refreshed index: `advisories ok, bans ok, licenses ok, sources ok`, with
`RUSTSEC-2024-0429` (glib 0.18.5) and `RUSTSEC-2024-0370` (proc-macro-error 1.0.4) accepted in
`deny.toml` with reasons and five revisit triggers. No new advisory; no policy change.

## 20. Exit checklist verdict

`G2_EXIT_CHECKLIST.md`: every box settled, the persistence rows as the addendum's §4 states them.

## 21. Git and integrity

One product-tree commit this round, `e35cfe7` (test only). The closure commit adds `G2_VALIDATION/`
(five files), archives the two delivered prompts byte-for-byte under `10_AUDIT/SOURCE_PROMPTS/`,
registers them with the P4 governance-consistency prompt and the addendum, moves the live governance
fields to `G2_ENGINEERING_CLOSURE_AUDIT` / local pass, and regenerates `DIRECTORY_TREE.txt` then
`SHA256SUMS` last.

## 22. Remote CI

Pending until pushed. The closure commit's run is recorded by its successor; the successor's own run is
external evidence (prompt §43).

## 23. Final local verdict

```text
G2 LOCAL PASS / READY FOR REMOTE CLOSURE
```

Not `G2 = PASS`: that needs the closure commit's own 7/7 and one successor's 7/7. Not production-ready,
not beta, not a release candidate, not signed or installable, not user-validated, not commercially
validated, not security-clean, and not licensed open source until the owner chooses a licence.
