---
title: "P3 Release Gate Exit Checklist"
doc_id: "FS-P3-EXIT"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "P3_RELEASE_GATE"
owner: "Engineering"
last_updated: "2026-09-30"
---

# P3 Release Gate exit checklist (prompt §69)

Forty boxes, in the order the prompt lists them. Each carries the thing that settles it — a command
output, a test name, a smoke step, or a `gh` query — because a checked box with no source behind it
is not evidence.

| # | Box | Verdict | What settles it |
| --- | --- | --- | --- |
| 1 | Run #18 implementation tree green | ✅ | `gh run view 36648718199` → `headSha 4a77ea19f62077cd8980a3cf0ce01712eb25657f`, `completed / success`, **7 of 7 jobs** |
| 2 | Run #19 latest successor closure HEAD green | ✅ | `gh run view 36665007523` → `headSha 32b23aa78323d315f6643f04c2343f75576d483f`, `completed / success`, **7 of 7 jobs** (macOS Core Smoke, Rust windows/ubuntu, Dependency policy, Desktop UI ubuntu/windows, Generated output drift). Run #17 `36596452341` at `c7fc2a34` stays in history as a real failure |
| 3 | ADR-0027 | ✅ | `09_ADR/ADR-0027-project-policy-and-provenance-adapter.md`, committed in `6e997f1` before any of the code it authorises |
| 4 | `firmwaresight-project` | ✅ | New crate: `config`, `git`, `version`, `fingerprint`, `evidence`, `fixtures`; 68 unit tests + 8 schema-contract tests |
| 5 | dependency admission | ✅ | ADR-0027 admits exactly `regex 1.13.1`, `serde 1.0.229`, `sha2 0.11.0`, `thiserror 2.0.21`, `toml 1.1.6` for that crate; `deny/cargo-deny` → 1/1 executed for real over licenses, bans, sources, advisories; nothing was installed merely because it is authorised |
| 6 | Core remains IO-free | ✅ | `crates/firmwaresight-core/Cargo.toml` `[dependencies]` is **empty**; `grep -rn "std::fs\|std::process\|File::open\|tokio::" crates/firmwaresight-core/src` → no match; `#![forbid(unsafe_code)]` still present |
| 7 | `firmwaresight.toml` v1 | ✅ | `schema_version = 1` is required and versioned; `ERR-CONFIG-7001..7008` cover every refusal |
| 8 | project-config schema | ✅ | `schemas/project-config.schema.json` + `crates/firmwaresight-project/tests/config_schema_contract.rs` (8 tests) |
| 9 | safe save | ✅ | `write_config` stages beside the target, reads back and swaps; a save that would drop unrecognised keys is refused with `ERR-CONFIG-7007`, and `release_gate.rs::unknown_keys_are_listed_and_a_save_that_would_drop_them_is_refused` proves the bytes on disk are untouched |
| 10 | policy saved to config | ✅ | `save_policy_in_project` / `save_policy_as_new`; `a_policy_saved_with_no_project_creates_the_file_the_loader_reads` and the `save_policy_as_new` → `policy_sha256(&GatePolicy::default())` assertion |
| 11 | read-only Git + timeout | ✅ | `crates/firmwaresight-project/src/git.rs` (14 tests) spawns argv arrays, never shell strings; only read subcommands; a timeout yields Unknown facts (`a_failed_head_probe_leaves_every_fact_unknown`) |
| 12 | Git absence → Unknown | ✅ | Smoke step 35 (`git.clean UNKNOWN`, "this directory is not a git repository"); CLI smoke "no repository is UNKNOWN at its policy disposition, never clean" |
| 13 | 5 factual states | ✅ | PASS / REVIEW / BLOCK / UNKNOWN / N-A all observed on screen in one run set (smoke steps 11–20, 29, 33, 35) |
| 14 | effective severity separate | ✅ | `FindingState` vs `EffectiveSeverity` (ADR-0023); every row prints both, and the schema refuses an illegal pairing (`the_schema_refuses_a_state_pairing_that_adr_0023_forbids`) |
| 15 | UNKNOWN preserved | ✅ | UNKNOWN is never folded into REVIEW or PASS: step 36 shows `effective severity: REVIEW` on an `UNKNOWN` state, and the aggregate says REVIEW, not BLOCK |
| 16 | stable rule IDs | ✅ | `GateRuleId::ALL` — ten ids in one canonical order, reused by Core, storage, CLI and UI; smoke step 21 shows them as the row keys |
| 17 | deterministic run id | ✅ | CLI: "the same input twice is byte-identical" (sha256 `89405df9c436a0af`); cross-surface: desktop and CLI both produced `gate-a64631b2…0ff152cc` for the same project, builds, HEAD and policy |
| 18 | all 10 MVP Gate rules | ✅ | Smoke steps 11–20 evaluate each one; counts row reads `Block 0 Review 1 Unknown 0 Pass 8 Not applicable 1` = 10 |
| 19 | additive 0003 | ✅ | `crates/firmwaresight-storage/migrations/0003_gate_history.sql` adds four tables and no destructive statement; `a_fresh_database_carries_the_gate_tables_at_version_three` |
| 20 | v1/v2 upgrade tests | ✅ | `a_version_one_database_is_upgraded_without_losing_its_evidence`, `a_version_two_database_gains_the_gate_tables_and_keeps_its_history`, `a_failed_gate_migration_leaves_the_previous_schema_intact_and_recoverable`, `an_unknown_newer_schema_is_refused_rather_than_reset`, `foreign_keys_are_enforced_on_the_connection` |
| 21 | immutable GateRun | ✅ | No UPDATE path exists for `gate_runs` / `gate_findings`; `storing_the_same_run_again_is_a_dedupe_not_a_duplicate` and smoke step 22 (still 1 run / 10 findings after a second click) |
| 22 | immutable Review acceptance | ✅ | `accepted_reviews` is insert-only with `PK(run_id, finding_id)`; smoke step 25 states it on screen: *"An acceptance is a record: it cannot be edited or deleted…"* |
| 23 | only REVIEW acceptable | ✅ | `ERR-STORAGE-4008` (`only_a_review_can_be_accepted`), refused before any write (`an_unaccepted_review…`/`an_acceptance_needs_a_name_and_a_reason_and_a_second_one_is_refused` → 4010/4007/4008/4009); smoke step 37: no accept control on an UNKNOWN row |
| 24 | accepted review keeps state REVIEW | ✅ | Smoke step 24 on screen and in the row: still `('REVIEW','REVIEW')` after `lin.we` accepted it |
| 25 | `gate-results` v1 | ✅ | `schemas/gate-results.schema.json`; CLI smoke "jsonschema validates gate-results v1 → 0 errors"; 13 tests in `gate_schema_contract.rs` |
| 26 | `accepted-reviews` v1 compatible | ✅ | `schemas/accepted-reviews.schema.json` with its own contract section in `gate_schema_contract.rs:308` |
| 27 | CLI `gate` | ✅ | `fwsight gate` with human and `--json` output, 18 tests in `apps/cli/tests/gate_cli.rs`, 18/18 smoke steps |
| 28 | exit 0/4/5 | ✅ | 0 on the all-PASS project, 4 on REVIEW, 5 on BLOCK (dirty workspace and missing notes), 6 on usage — all four observed in the CLI smoke |
| 29 | Desktop Analyze / Compare / Release | ✅ | Smoke steps 1–4; `intake.test.tsx` asserts Bundle, History, Settings, SBOM, Pricing and Cloud are absent |
| 30 | config load / save | ✅ | Smoke steps 5–6 through the real OS dialog; the save paths are box 9/10 |
| 31 | Gate findings | ✅ | Smoke steps 11–21: rule id, state, effective severity, summary, remediation and locators for all ten |
| 32 | review acceptance | ✅ | Smoke steps 23–26, including the disabled-until-complete button and the audit block |
| 33 | budgets | ✅ | Smoke steps 16–17: actual, limit, headroom, state and evidence basis for FLASH and RAM |
| 34 | baseline growth | ✅ | Smoke step 18: `256 → 376, +120 over the 100 B threshold`, and the growth row's new side equals the budget row's actual bytes |
| 35 | last-good | ✅ | `Release.tsx:470` — *"The record below is the last run that succeeded (…). It is not the result of the attempt that failed."* — plus `record_note` on a restored record; a refused acceptance leaves the run readable and unchanged |
| 36 | no path leak | ✅ | Smoke step 39: 147 persisted text fields + 2 CLI documents, 0 hits for the temp root, the smoke marker, any backslash or any drive path; 0 locators outside the six schemes; `projects` stores no root column; storage's CHECK rejects a separator in a locator |
| 37 | fmt / clippy / tests | ✅ | `cargo fmt --all -- --check` exit 0 · `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean · `cargo test --workspace` **556 passed / 0 failed / 0 ignored, 28 executable suites** (+6 empty doc-test suites). An earlier revision of this row said 715 in 42: that summed the gate log, where the `drift` group re-runs the desktop crate, and double-counted its 159 tests. See `P3_GATE_EXECUTION_REPORT.md` §4 |
| 38 | pnpm gates | ✅ | `install --frozen-lockfile`, `typecheck` (`tsc --noEmit`), `lint` (`eslint .`), `test` (**135 passed, 6 files**), `build` — run from `apps/desktop/ui`, where those scripts live, not from the repository root |
| 39 | drift / deny / core-smoke / full gate | ✅ | `check.py --only drift` 6/6 · `--only deny` 1/1 · `--only core-smoke` 3/3 · `python scripts/check.py` **15/15** · `git diff --check` exit 0. The `deny` row is a **local** green on a stale index: the first CI pass over the pushed tree (`36774472141`) failed that same step on a crate yanked the day of closure, and `Cargo.lock` moved to `yoke-derive 0.8.4` in response — §4.1 of the execution report |
| 40 | Windows smoke | ✅ | §61's 40 steps walked on the shipping `custom-protocol` binary in Run 3 over a clean validation DB — `P3_GATE_DESKTOP_SMOKE_REPORT.md` |

## Where this checklist is not total

- **Step 30 of the smoke, not box 21**: an older run cannot be re-opened *through the window* because
  the Release page has no run-id input. The command and its persistence are tested; the surface is
  not. Surfacing history is P4's page (§64 forbids building it here).
- **The gate was 13/14 before it was 15/15, and that is the record.** `python scripts/check.py` failed
  `frontend/test` on `compare.test.tsx > the change tables > keeps a size that was never recorded as
  Unknown with its reason` — three tests queried a table row synchronously while `Compare.tsx:1303` was
  still rendering its `Loading symbol changes…` state inside the same region, so the awaited region
  lookup resolved against an empty table. A test-side race in P2's file, not product behaviour and not a
  P3 surface — but this round's gate is what caught it. The queries are now awaited with the assertions
  unchanged, ten consecutive runs of that file are green where one in five had failed, and the `15/15` in
  box 39 is the re-run after the fix rather than the earlier run restated.
- **A second race of the same class surfaced only on CI, after that fix.** Run `36779715108` on `893a635`
  failed `Desktop UI (windows-latest)` → `frontend/test` with `AssertionError: expected 2 to be +0` at
  `compare.test.tsx:731`, while the ubuntu job passed the same file and eight local runs had passed it.
  The KiB test sampled `querySectionChanges` call counts before the change tables' own first fetches had
  landed, so the page's loading was charged to the radio click. Fixed by sampling after both tables
  resolve — the guard `details.test.tsx:352` has carried since P1 — with the delta assertions unchanged,
  and verified by mutation: a real `querySectionChanges` call on the unit toggle fails at exactly that
  line (`expected 4 to be 3`), after which `Compare.tsx` was restored byte-identical. Defect J in
  `P3_GATE_EXECUTION_REPORT.md` §7. Its successor push settled the question: Run `36783457030` on
  `02e8a81` concluded `completed / success` with **7 of 7 jobs**, `Desktop UI (windows-latest)` included.
- **The closure commits were first recorded as unpushed, then pushed, and CI now says something.** At
  the time box 39 was written, `origin/main` was still `32b23aa` and no CI run covered P3 code; §67
  forbids writing a future CI run into the commits that would trigger it, so nothing was predicted.
  The commits were pushed after closure, `origin/main` became `219178af195569ec6b13728d84d992ef78df8c04`,
  and run [`36774472141`](https://github.com/2023violet/FirmwareSight/actions/runs/36774472141) completed
  `failure` with **6 of 7 jobs green**: Rust on both platforms, Desktop UI on both platforms, the macOS
  core smoke and the generated-output drift check all passed, and only `Dependency policy` failed on
  `error[yanked]` for `yoke-derive 0.8.3` — a crates.io yank published at 2026-09-30T13:19:39Z, after
  the local `deny` step had run against an older index. Nothing in this checklist's product evidence
  depends on that crate's patch level, and the lockfile moved to `0.8.4` in a successor commit — which
  run `36779321479` then verified at **7 of 7 jobs green**, Dependency policy included. The third push,
  `893a635`, went red again on a different job: `Desktop UI (windows-latest)`, on the race two bullets
  above. §4.1 of the execution report carries all three runs.

## Verdict

```
P3 = PASS / COMPLETE
P4 = NEXT_AUTHORIZABLE_STAGE
G2 = NOT REACHED
```

STOP.
