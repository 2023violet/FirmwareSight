---
title: "G2 Exit Checklist"
doc_id: "FS-G2-EXIT"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "G2_ENGINEERING_CLOSURE_AUDIT"
owner: "Engineering"
last_updated: "2026-10-01"
---

# G2 — exit checklist (prompt §34, persistence rows as corrected by the addendum §4)

Each box is settled by something that ran. `M§n` is a section of `G2_EVIDENCE_MATRIX.md`, `S<n>` a step
of `G2_END_TO_END_SMOKE_REPORT.md`, `L<n>` a row of `G2_KNOWN_LIMITATIONS.md`. Audited tree:
`e35cfe70aa0e8f273a75ac14b9dc62377483af36`.

## Workflow

- [x] **Analyze works** — CLI exit 0 ×2 sides; S3–S10.
- [x] **Compare works** — `fwsight diff` exit 0; S11–S20.
- [x] **Gate works** — `fwsight gate` exit 0, PASS; S21–S28.
- [x] **Bundle works** — `fwsight release prepare` exit 0; S29–S33.
- [x] **CLI whole chain works** — five commands, non-interactive, twice, no desktop process, no store side effect.
- [x] **Desktop whole chain works** — S1–S43 on the `custom-protocol` binary `84a26a05…`.

## Cross-surface

- [x] **artifact facts match** — M§3; S4, S6.
- [x] **SnapshotIds match** — full strings compared.
- [x] **memory facts match** — NV / RAM, layout, basis, dual-accounted, excluded bytes.
- [x] **Diff semantics match** — `diff.json` and the HTML export byte-identical across surfaces.
- [x] **Gate run id / facts match** — `gate-e692…`; 10 of 10 findings with equal rule id, state, effective severity and summary.
- [x] **release id matches** — `release-2a7849c0…`.
- [x] **manifest matches** — `85e53c18…`; the whole bundle `diff -r` identical.

## Correctness

- [x] **deterministic outputs** — M§3.
- [x] **supported fixtures parse** — 5 of 5.
- [x] **malformed input fails closed** — exit 3 with `ERR-FORMAT-0001` / `ERR-PARSE-2002`, no panic; desktop scenario F keeps the last good report and adds no build.
- [x] **unsupported input explicit** — text, Intel HEX and raw BIN refused with `ERR-FORMAT-0001`; a non-MAP `--map` with `ERR-MAP-3001`.
- [x] **Unknown never silently PASS** — scenarios B and U; the stored-state CHECK; `unknown_state_is_never_rewritten_and_never_counts_as_pass`.

## Persistence

- [x] **schema v4** — `db.rs:14`; smoke store migrations 0001–0004.
- [x] **migration upgrade tests green** — fresh, v1, v2, v3 → v4; future version refused; failed migration recoverable (M§5).
- [x] **immutable Gate / Review / Release records** — triggers plus named tests (M§5); observed: re-run dedupe, no second release record (O4).
- [x] **local-only `artifacts.path` is still the intentional P0 / P1-A0 storage behaviour** — `04_TECH/15` §7; `P1_A0_EXECUTION_REPORT.md`; M§7 row 1.
- [x] **raw artifact bytes are not stored in SQLite** — no `BLOB` column; 0 BLOB cells.
- [x] **artifact source path does not cross normal IPC / UI** — M§7 rows 2–4, 8; 23 commands take no path.
- [x] **artifact source path does not enter deterministic portable contracts** — M§7 rows 5, 6, 9–14.
- [x] **Gate evidence locators contain no host path** — M§7 row 7.
- [x] **`release_records` contain no source / project / destination path** — M§7 row 16.
- [x] **Release Bundle contains no host path outside the copied artifact's own immutable bytes** — M§7 rows 9–15 and §5.
- [x] **no new path persistence was introduced** — the only path-like column in 0001–0004 is `artifacts.path`.

## Bundle

- [x] **portable** — S39–S41.
- [x] **independent reader green** — 64/64 (desktop, CLI), 59/59 relocated.
- [x] **hashes verify** — S36.
- [x] **offline report** — S37.
- [x] **source / project / DB unnecessary** — S40–S41.

## Reliability

- [x] **direct Rust tests green** — one `cargo test --workspace`: 769 passed / 0 failed / 0 ignored.
- [x] **direct UI tests green** — `corepack pnpm test`: 155 passed in 6 files, after G2-F1.
- [x] **full `check.py` green** — 15/15.
- [x] **clean detached worktree green** — 17/17 at `e35cfe7`; Cargo.lock and pnpm-lock unchanged; no ignored or untracked dependency.
- [x] **shipping build green** — `cargo build --release -p firmwaresight-desktop --features custom-protocol`, exit 0, 182 s.
- [x] **real Windows G2 smoke green** — every step and scenario PASS.
- [x] **current remote CI green** — Run `36906482900` on `e35cfe7`, attempt 1, 7/7. The closure commit's own run is recorded by the successor (prompt §43).

## UI

- [x] **Minimum Credible Product** — complete, consistent, labelled; K, W, S; clarity items L19–L21 are post-MVP.
- [x] **no blocker** — every task completed in the window.
- [x] **no path leak** — page sweeps clean at S22, S30, S33 and after export.
- [x] **states not colour-only** — icon plus label everywhere; UNKNOWN hollow and neutral.

## Governance

- [x] **P1 PASS** · [x] **P2 PASS** · [x] **P3 PASS** · [x] **P4 PASS** — M§2.
- [x] **known limitations explicit** — `G2_KNOWN_LIMITATIONS.md`, 25 rows.
- [x] **user metrics not fabricated** — M§1 last row; V0 `0 / 8`.
- [x] **license gap explicit** — L13.
- [x] **accepted security risks explicit** — L12; M§6.

## Scope

- [x] no P5 · [x] no installer · [x] no signing · [x] no updater · [x] no cloud · [x] no accounts ·
  [x] no telemetry · [x] no AI · [x] no commercial or pricing work — the only source change of this
  round is one test file (`e35cfe7`).

## Verdict

Local verdict, as written in the evidence commit `f75cbc5`:

```text
G2 LOCAL PASS / READY FOR REMOTE CLOSURE
```

Remote closure: the product tree `e35cfe7` (Run `36906482900`) and the evidence head `f75cbc5` (Run
`36948719972`) are each 7 of 7 on their first attempt. Therefore:

```text
G2 = PASS
Product MVP = ENGINEERING COMPLETE
Product state = MVP CANDIDATE
```

The 500 MB UI metric stays **NOT_MEASURED** (M§9) and is not counted as proven by this verdict.
