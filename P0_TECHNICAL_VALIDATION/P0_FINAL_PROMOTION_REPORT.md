---
title: "P0 Final Promotion Report — v0.6.0 Baseline Closure"
doc_id: "FS-P0-022"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Final Promotion Report — v0.6.0 Baseline Closure

Scope: the architect prompt *FirmwareSight P0 — Final Promotion / v0.6.0 Baseline Closure v1.0 —
Architect Signed*. One act: promote P0 to `PASS` and freeze
`FirmwareSight_Project_Baseline_v0.6.0` as the **P0 Technical Foundation Baseline**.

This round changed no product source. It changed baseline, governance, audit and evidence documents,
and it regenerated the two integrity artifacts. Had promotion required a source change, the prompt's own
guard says the round stops as `PROMOTION BLOCKED BY NEW ENGINEERING DEFECT` and no `PASS` is written;
that branch was not taken, and §20 of this report's evidence below is the command that shows why.

## 1. The decision, by the architect

```text
Decision        P0 = PASS
Baseline        v0.6.0 — FirmwareSight_Project_Baseline_v0.6.0, the P0 Technical Foundation Baseline
Signed by       FirmwareSight P0 — Final Promotion / v0.6.0 Baseline Closure v1.0 — Architect Signed
Date            2026-09-28
V0              unchanged: DEFERRED / NOT YET EVIDENCE-VALIDATED, 0 / 8 eligible external sessions
Formal G1       NOT CLAIMED
P1              NOT AUTHORIZED
Architecture    no change; no ADR required
```

The execution side did not choose this status; it recorded the one the architect signed, and the
distinction matters because the same pack refused `PASS` while the evidence looked sufficient two rounds
ago. `BASELINE.yaml` now reads `status: PASS`,
`product.baseline_version: 0.6.0`, `current_gate: G0_PASS_V0_DEFERRED_P0_PASS_G1_NOT_CLAIMED`,
`next_authorizable_tracks: [V0_workflow_prototype_validation]` and `active_task: NONE`.

## 2. Basis for the decision

Four things, each of which was measured before it was cited:

1. **Remote CI Run #3** — `36399805005`, event `push`, head `1cd6309`, conclusion `success`,
   **7 of 7 jobs green**.
2. **Remote CI Run #4** — `36402637251`, event `push`, head `5e58f77`, conclusion `success`,
   **7 of 7 jobs green**.
3. **A real desktop window** in the shipping configuration — `P0_DESKTOP_SMOKE_REPORT.md`.
4. **The local gate** — `python scripts/check.py`, re-executed by this round; §20 below.

Both runs were read back from GitHub rather than from this repository's own reports:

```
$ gh run view 36399805005 --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,jobs
{"conclusion":"success","head":"1cd6309a09313a0a900a83cb12e054e1a3d7c5e3","id":36399805005,
 "njobs":7,"allsuccess":true}
$ gh run view 36402637251 --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,jobs
{"conclusion":"success","head":"5e58f778aad35f33188b96d0b8873401a31ccc3c","id":36402637251,
 "jobs":["Dependency policy","Rust (ubuntu-latest)","Rust (windows-latest)","macOS Core Smoke",
         "Generated output drift","Desktop UI (ubuntu-latest)","Desktop UI (windows-latest)"],
 "status":"completed"}   # every job conclusion: success
```

## 3. Three HEADs, kept distinct

| Role | HEAD | What it contains |
| --- | --- | --- |
| Engineering-validated | `1cd6309a09313a0a900a83cb12e054e1a3d7c5e3` | The last commit carrying the source the gate measured. Run #3 executed it. |
| Architect-reviewed | `5e58f778aad35f33188b96d0b8873401a31ccc3c` | `1cd6309` plus governance, audit and evidence documentation only. Run #4 executed it. |
| Promotion | the child of `5e58f77` created by this round | Baseline, governance, audit, P0 evidence, `DIRECTORY_TREE.txt` and `SHA256SUMS`. No production source. Find it with `git log -1 --format=%H` on the tree this file ships in; a file cannot contain its own commit's hash, so it is named by relationship rather than by value. |

The `1cd6309 → 5e58f77` diff was machine-checked rather than eyeballed:

```
$ git diff --name-only 1cd6309..5e58f77 | grep -E '^(crates|apps|fixtures|golden|migrations)/|^Cargo\.lock|^pnpm-lock\.yaml|^deny\.toml$|^scripts/check\.py$|^\.github/workflows/'
$ git diff --name-only 1cd6309..5e58f77 | wc -l
15        # every one of them a .md document or BASELINE.yaml — no production source moved
```

## 4. Run history, including the failures

Four runs produced the green. None of the red ones is edited, deleted or relabeled.

| Run | ID | HEAD | Conclusion | Detail |
| --- | --- | --- | --- | --- |
| #1 | `36360310447` | `f9b8ccb` | **failure** | 2 of 6 green — Rust (windows), Rust (ubuntu), Generated output drift, Dependency policy red |
| #2 | `36378384225` | `ebda52d` | **failure** | 6 of 7 green — only `Generated output drift` red, at `drift/ipc bindings` |
| #3 | `36399805005` | `1cd6309` | success | 7 of 7 green |
| #4 | `36402637251` | `5e58f77` | success | 7 of 7 green |

`BASELINE.yaml` keeps all four under `remote_ci_runs`, with `last_remote_ci` on Run #4 and Runs #1–#2
reachable as `previous_remote_ci` / `first_remote_ci`. Full per-job evidence:
`P0_CI_REPORT.md`.

## 5. What the tree is

104 Rust tests across six workspace members (four Phase-0 library crates — `firmwaresight-core` with
zero dependencies, `-artifact`, `-report`, `-storage` — plus the `fwsight` CLI and the Tauri shell), and
19 UI tests across three files. Core stays headless and synchronous; the UI does not reimplement parser,
diff or gate logic; the IPC boundary is typed and generated from the Rust side.

- **Storage**: `rusqlite + bundled`, migrations, transactional import. **Schema version 2**, reached by
  `crates/firmwaresight-storage/migrations/0002_evidence_keyed_by_build.sql`, which rebuilds `evidence` on `(build_id, id)` because version 1
  keyed it on `id` alone — contradicting `04_TECH/15` §4's `Build 1─N Evidence`. The defect was found by
  the authorized real-window launch, not by a test, and the two regression tests were written before the
  fix. The upgrade was then replayed against the real database the earlier launch had left in the user
  profile. `P0_STORAGE_REPORT.md`, `P0_DESKTOP_SMOKE_REPORT.md`.
- **Determinism**: CLI JSON is golden-tested; untrusted-input intake runs
  `stat → size guard → streaming SHA-256 → magic detect → single read → parse`, with the 512 MiB guard
  measured from both sides of the boundary.
- **Evidence discipline**: memory accounting reproduced by hand from `readelf` under ADR-0021's
  two-budget model; real ARM ELF/MAP fixtures with recorded provenance, so no test needs
  `arm-none-eabi-gcc`.

## 6. Engineering regression executed by this round

The promotion prompt requires re-running the gate before the freeze, on the documentation tree, to prove
that the promotion round broke nothing it did not intend to.

```text
$ python scripts/check.py                  14/14 steps passed — rust 3, frontend 5, drift 5, deny 1
$ python scripts/check.py --only core-smoke   3/3 steps passed
$ git diff --check                          clean (exit 0)
```

Step-by-step results: `rust/fmt`, `rust/clippy`
(`--workspace --all-targets --all-features -- -D warnings`), `rust/test` **104 passed / 0 failed** across
16 result groups; `frontend/install` (`--frozen-lockfile`), `typecheck`, `lint`, `test`
**19 passed (19)**, `build`; `drift/design tokens`, `drift/desktop icons`, `drift/ipc bindings`,
`drift/ipc bindings unchanged`, `drift/goldens unchanged`; `deny/cargo-deny`. The core smoke reports
**87** tests, which is the 104 minus the desktop shell's 17 — the same selection CI runs on macOS.
**0 mandatory steps reported `SKIPPED`.**

`git diff --check` was *not* clean on its first execution: it flagged trailing whitespace on one
`BASELINE.yaml` value and on the `INDEX.md` baseline line, both written by this round. Both were fixed
and the check re-run; the two lines above are the second execution's result.

Per the prompt, the desktop window was not reopened and the large-workload performance numbers were not
re-measured: no runtime source changed in the promotion round, so re-launching would produce a second
observation of the same build rather than new evidence.

## 7. Supply chain and the accepted advisories

```
$ cargo deny check licenses bans sources advisories        (cargo-deny 0.20.2)
advisories ok, bans ok, licenses ok, sources ok
```

`RUSTSEC-2024-0429` (`glib 0.18.5`, unsound) and `RUSTSEC-2024-0370` (`proc-macro-error 1.0.4`,
unmaintained) are recorded as **ACCEPTED AS EXPLICIT P0 TRANSITIVE RISK / NOT A P0 PROMOTION BLOCKER**,
which is the architect's disposition, not the executor's:

- Both reach the graph through the gtk-rs `0.18` line that Tauri `2.12.0` requires, and
  `cargo update -p glib --precise 0.20.0` fails against `gtk = "^0.18"`.
- `proc-macro-error` is a host-only build dependency; no first-party code calls the affected API of
  either crate.
- Five revisit triggers are recorded in `P0_DEPENDENCY_REPORT.md` and `P0_KNOWN_LIMITATIONS.md`: a
  compatible fixed Tauri/gtk-rs line becoming available, a material change in classification or
  severity, first-party code exercising the affected API, the P5 dependency/security review, and a Linux
  release-candidate security review.
- **No ADR**, because no architecture choice changed. An ADR becomes required if Tauri is replaced,
  dependencies are forked, or the frozen desktop dependency family moves.
- `deny.toml` carries the ignores with their reasons and `unused-ignored-advisory` stays at its default
  `warn`, so an ignore that stops matching reports itself.

Nothing in this baseline may be described as "zero vulnerabilities", "security clean" or CRA-compliant.
`07_COMPLIANCE/00_COMPLIANCE_BOUNDARY.md` forbids the claim and §1 of the promotion prompt repeats it.

## 8. Known limitations that promotion does not remove

```text
Peak RSS              NOT MEASURED — reason and the measurements that do exist: P0_PERFORMANCE_REPORT.md
Fuzz campaign         NOT RUN — cargo-fuzz was never authorized for P0
Desktop evidence      one Windows 10 host, one WebView2 153.0.4234.48, 100% scaling, two launches
Platform coverage     macOS runs the core smoke only; no desktop UI or smoke on macOS
Linker layouts        GNU ld MAP plus one dual-region ELF layout
Open UI findings      capability labels show Core's enum words; eight 1px borders have no token
                      (closing the second needs a frozen assets/design-tokens.json bump)
Visual defect         the `Partial` badge label breaks mid-word at one column width — recorded, not fixed
CI duplication        two jobs install the same ten apt lines on purpose
Exit-code coverage    CLI codes 0/2/3 exercised; 4/5/6 are not registered, because no code path returns them
```

`v0.6.0` also does not implement Compare, the deterministic Release Gate verdict surface or the Release
Bundle, and it ships no installer, package, signature, notarization or updater. Those are later phases,
not deferred parts of this one.

## 9. What the version does and does not mean

`v0.6.0` is the **P0 Technical Foundation Baseline**. It is not V0 `PASS`, not G1, not an MVP, not P1
authorization, not a commercial stable release, not user-value validation. Product scope is inherited
verbatim from v0.5.0 — `PRODUCT_BASELINE.md` §12 carries the dated note, and no product verb, artifact
class or gate semantic moved.

`G1 = V0 PASS + P0 PASS` (ADR-0020, `06_DELIVERY/06_STAGE_GATES.md`). Only one half is satisfied, so
`g1_claimed_by_p0_pass: false`. V0 remains `0 / 8` eligible external sessions
(`0 / 4–5` Batch A), no `V0_VALIDATION/**` file was touched, and no participant data was created. P1
requires its own authorization prompt, which has not been issued.

This is a **project baseline promotion, not a software release**: no GitHub Release, no tag (the
repository has no tag convention — `git tag -l` returns nothing, and none was invented here), no
installer, no signing, no notarization, no updater metadata.

## 10. Integrity artifacts

`DIRECTORY_TREE.txt` and `SHA256SUMS` were stale by construction: they describe the frozen v0.5.1
package (247 files, listed in `manifest.txt`), and nine governance/execution documents had drifted since.
Regenerating them is part of the promotion act, and the order matters — the tree lists the files the
manifest hashes, and the manifest does not hash itself.

Both artifacts are generated from the tracked file set, so nothing gitignored can enter them: no
`target/`, `node_modules/`, `dist/`, `fixtures/generated/` workload, `gen/schemas/`, `*.sqlite` or
`.firmwaresight/`. Root ordering is directories first then files, each sorted case-insensitively,
matching the layout the v0.5.1 tree used; paths are repository-relative with `/`, LF-terminated, and no
absolute host path appears.

```text
$ python <one-off generator using git ls-files -z>     # 387 tracked files -> 466 tree lines
$ python <one-off hash generator>                      # 386 SHA256SUMS entries, SHA256SUMS excluded
$ sha256sum -c SHA256SUMS                              # exit 0
$ python <independent verifier>                        # 386 ok, 0 mismatch, 0 duplicate, no self-entry
```

`manifest.txt` is deliberately **not** regenerated: it is the v0.5.1 delivery's package list, and
rewriting a frozen package manifest to match a later tree is exactly the kind of after-the-fact
fabrication this pack has avoided. The two artifacts that do describe this baseline are the tree and the
checksums, and both are stamped by this round.

## 11. Prompt provenance

The promotion prompt, like the two remediation prompts before it, was **supplied inline as text**; there
is no original file in `10_AUDIT/SOURCE_PROMPTS/`, so no SHA-256 of it can be recorded and no
"byte-exact copy" is claimed. `10_AUDIT/SOURCE_PROMPTS/README.md` states this in the entry's own words.
The two earlier prompts that did arrive as files remain archived and hashed there, including the P0 slice
prompt at `60a59196708a53592be4d828a0c1275cf74b3bfad0bdc6455f863ab38d32929b`.

## 12. What the next agent may and may not do

`active_task: NONE`. The legitimate next moves are a real V0 participant source, or an explicit P1
sequencing prompt from the architect. Read `.ai/HANDOFF.md` first: it lists the HEADs, the runs, the
schema version, the accepted advisories and the measurement gaps, and it says plainly that an empty task
queue is not an invitation to invent work.

Pushing this commit belongs to the repository owner.
