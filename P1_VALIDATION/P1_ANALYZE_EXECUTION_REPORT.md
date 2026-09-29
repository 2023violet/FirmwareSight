---
title: "P1 Analyze Execution Report"
doc_id: "FS-P1-001"
product: "FirmwareSight"
version: "0.1.0"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-29"
---

# P1 Analyze Execution Report

Task: `P1_ANALYZE_DETAILS` - capabilities 5 to 9 of the Analyze verb.
Authority: ADR-0026 (Open-Source MVP-First Delivery), the parent prompt *FirmwareSight Open-Source
MVP-First Governance Reset + P1 Analyze Completion v1.0*, and the *P1 Analyze Acceptance Closure v1.0*
addendum, which binds P1 `PASS` to the frozen US-001 criteria in `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md`.
Work started from `872ad7e` (on top of `be09c65`, whose remote run #12 `36520562718` is `success`, 7 of 7 jobs).

Every claim in §1-§8 below is `LOCAL PASS` unless it says otherwise: it ran on this machine and produced
the quoted result. Nothing in §1-§8 says `CI PASS`; the remote CI status of the pushed head is recorded
once, in §9, which the successor commit added.

## 1. What the reader can now do

```text
intake -> summary -> top contributors -> [Sections | Symbols | Evidence] -> Evidence Inspector
```

Sections, Symbols and Evidence are three tabs over one bounded query layer. Each tab filters, sorts
and pages through the shell; the reader's own screen never holds the whole table. The unit switch
(bytes / KiB) sits above the tabs and re-labels every byte figure without asking the shell anything.

## 2. Storage: the bounded query layer

`crates/firmwaresight-storage/src/query.rs` (new) owns the whole read path for this slice:

| Item | Value |
| --- | --- |
| Page size | `DEFAULT_QUERY_LIMIT = 100`, `MAX_QUERY_LIMIT = 500`, both clamped in Rust (`clamp_page`) |
| Degenerate input | `limit <= 0` means "not asked" and becomes the default; a negative offset clamps to the first page; an offset past the end is an empty page, not an error |
| Shapes | `Page<T> { rows, total, offset, limit, next_offset }`, `SectionRow`, `SymbolRow`, `EvidenceRow` |
| Facts | `Fact<u64>` / `Fact<String>` where the schema stores a reason column; `Option<u64>` where it does not (see §6) |
| Ordering | NULL-last in both directions, via `(col IS NULL)` as the leading `ORDER BY` term, with a deterministic tiebreak on the natural key |
| Filter | `LIKE '%' \|\| ? \|\| '%' ESCAPE '\'` with `%`, `_` and `\` escaped in the caller's text, so a filter is a substring test and never a pattern |
| Scope | every statement is keyed by `build_id`, resolved from `snapshot_id` first; an unknown snapshot is `ERR-STORAGE-4005` |

`crates/firmwaresight-storage/src/db.rs`: `build_id_for_snapshot` became
`SELECT id FROM builds WHERE snapshot_id = ?1 ORDER BY id LIMIT 1`. Without the `ORDER BY` the row
chosen depends on visit order, which is SQLite's business and not ours; `a_snapshot_id_always_resolves_to_the_same_build_row`
pins it.

`crates/firmwaresight-storage/src/lib.rs` re-exports the query types together with
`firmwaresight_core::domain::{evidence::EvidenceClass, identity::Fact}`, so a caller of the read path
needs no additional dependency.

Tests: `crates/firmwaresight-storage/tests/analyze_queries.rs`, 19 of them, written before the
implementation and each named for the production change that would break it. They run against the
real committed fixtures through the real write path, not against a mock database.

## 3. Shell: three use-case commands

`apps/desktop/src-tauri/src/details.rs` (new) adds `Session::query_sections`, `query_symbols` and
`query_evidence`, and the three `#[tauri::command]`s of exactly those names. They hop through
`spawn_blocking`, take a typed request DTO, and return either a typed page or the existing
`ErrorEnvelopeDto`.

Boundary properties, each with a test in `apps/desktop/src-tauri/tests/analyze_details.rs` (9 tests):

- addresses and offsets cross as the same `0x%08x` text the CLI prints; byte counts cross as plain
  numbers so the WebView, not Rust, chooses the unit;
- a section with no load evidence crosses as a reason and never as zero;
- the command layer applies the same page bounds as storage (`None` -> 100, `100_000` -> 500);
- evidence crosses as provenance and the serialized payload contains no repository root, no `C:\`,
  no `/home/` and no `/Users/`;
- an unknown snapshot id is a typed error with an operation id and a remediation, not a panic and not
  an empty table;
- `no_general_purpose_query_surface_was_added_to_the_shell` scans `lib.rs`, `details.rs`, `intake.rs`
  and `service.rs` for `run_sql`, `read_table`, `query_any`, `get_database` and `execute_shell` and
  asserts none appears, while the three commands are registered.

`apps/desktop/src-tauri/src/ipc.rs` gained 14 bound types (3 request, 3 page, 3 row, 3 sort, plus
`SortDirDto` and `EvidenceClassDto`). Two existing shapes were wrong in a way this slice would have
hidden: `BudgetDto.bytes`, `SectionRowDto.memorySize` and `SymbolRowDto.size` were exported to
TypeScript as `number` while genuinely able to be `null`. They are now `#[ts(type = "number | null")]`,
which is what let the UI render `Unknown` without a cast.

## 4. UI: the detail screen

`apps/desktop/ui/src/Details.tsx`, `Details.module.css` and `details.test.tsx` (21 tests).

The screen asks rather than slices: a filter, a sort, a class or a page offset becomes a request, and
the answer is what gets painted. `intake`-level rules carried over unchanged - no path is ever sent,
the summary stays on screen when a detail query fails, and the details are bound to the snapshot id
the shell last succeeded with.

Two pieces of state are deliberately separate: the contributor page and the active table page each
count their own requests, so an answer to one is never treated as stale for the other.

## 5. Defects this round found and fixed

Each was reproduced by a test before it was fixed; none was fixed by weakening a test.

| Defect | Root cause | Fix |
| --- | --- | --- |
| Section rows painted inside the symbol table for one frame | The page lived in its own state slot, so the render between a tab click and the effect that clears it paired `tab = symbols` with a sections page. `key={row.ordinal}` was then `undefined` and React's key check reported it | A page now carries the tab it answers (`type Loaded`), and the panel renders only when the two agree. Guard test: `never paints one tab rows under another tab columns` |
| `limit = 0` returned one row | `limit.clamp(1, MAX)` read "not asked" as "asked for one" | `clamp_page` treats `limit <= 0` as the default |
| Evidence query bound 5 parameters to 4 placeholders | The shared statement reused `?3`/`?4` for `LIMIT`/`OFFSET` while the class filter also needed a slot | The evidence statement numbers its own parameters (`LIMIT ?4 OFFSET ?5`); `rusqlite` surfaced this as `Wrong number of parameters` in the first RED run, not as a silent shift |
| Contributor rows wrapped their role onto a second line | `grid-template-columns` had two tracks and the row had three children, because the first track was borrowed from the evidence-inspector width token | Name and role share the flexible track; the byte count is right-aligned in an `auto` track |
| Inspector labels overlapped their values | The label track was `--fs-space-12`, a spacing token used as a column width, too narrow for `Classification` | `grid-template-columns: max-content minmax(0, 1fr)` on the list, with each pair `display: contents` so all values align on one edge |
| `Inspect` was off the right edge at the minimum supported window width | The value and locator columns are wider than 1024 px, and the table owns its own scroll axis, so the last column was the one clipped | The action moved to the first column; the inspector also moved above the table so the click has a visible result. Guard tests: `keeps the Inspect action in the first column…`, `places the inspector above the table…` |
| Chromium offered form-history autofill in the filter box | A bare `<input type="text">` inside a `<form>` | `autoComplete="off"`; the filter box has nothing worth autofilling. Guard test: `offers no browser autofill for a name typed to query firmware` |

## 6. Findings reported, not fixed

These are real and deliberately left alone, because fixing them needs an authority this task does not have.

1. **Two stored facts lose their reason.** `sections.file_offset` and `symbols.address` are nullable
   columns with **no** companion `*_unknown` reason column, unlike `load_address` / `memory_size` /
   `name`. A reason therefore cannot survive the write. The read layer types both as `Option<u64>`
   rather than `Fact<u64>` so the UI says `Unknown` without inventing a reason, and the UI shows no
   reason for those two cells. Adding the columns is a schema migration; `SCHEMA_VERSION` stays 2 in
   this slice and no `0003` was written.
2. **A stored `NULL` with no reason column reads back as `Unknown` with the text "no reason was
   recorded".** That string is produced at read time for the columns that *do* have a reason column
   but hold `NULL` in both fields - it names the absence rather than inventing a cause.
3. **The `drift/ipc bindings unchanged` step cannot see untracked files.** It is
   `git diff --exit-code`, so before the first commit of a new binding it passes trivially. For this
   round the invariant was verified directly instead: over the 25 files in
   `apps/desktop/ui/src/ipc/generated/`, `sha256sum -- * | sha256sum` gives
   `7fcc246a5a3b26aaf615692617bd7d1fead9a1604f415a6b746f0c48f3107943` before `cargo test
   -p firmwaresight-desktop` and the same value after it, and `git status --porcelain` on that
   directory reports nothing. An earlier revision of this row quoted
   `35e546ba031bb71a395ed6a2e91f6aaabad5a420abe4c6149c65d866302c4d33`; that value was measured at the
   time but the command that produced it was not recorded, so it could not be re-derived and has been
   replaced by the command above rather than left as a number no one can check. After the commit that
   ships these files, the gate is meaningful on its own.
4. **An `Apply filter` click did nothing once during the first smoke pass.** Reported honestly as
   unresolved. The `Why`: the WebView2 accessibility tree carried a `状态 建议可用` ("suggestions
   available") autofill node in every capture of that session, and after dismissing it with `Escape`
   the identical sequence - type, Apply, clear, Apply - worked in the shipped binary and is asserted
   by `clearing the filter and applying again asks for the whole table`. The two candidate mechanisms
   (the autofill popup swallowing the first click, or a click landing while the panel was unmounted
   mid-request) were not distinguished, so neither is claimed as the cause. The autofill attribute in
   row 7 of §5 removes one of them as a possibility; it is not evidence that it was the cause.
5. **A long `Unknown` reason makes a tall row.** `.debug_info`'s virtual-address reason ("this section
   is host metadata and is not loaded at runtime") wraps to several lines inside its column. The text
   is fully readable and nothing is hidden, which is what AGENTS.md 11 and the DESIGN.md Unknown rule
   require; a shorter rendering would drop a fact. Left as is, and recorded rather than silently
   polished.

## 7. Verification

| Gate | Command | Result |
| --- | --- | --- |
| Rust | `cargo test --workspace` | **184 passed / 0 failed / 0 ignored** |
| New Rust tests | counted from the same run | 19 `analyze_queries` + 9 `analyze_details` + 14 `ipc::export_bindings_*` = 42 |
| Rust totals | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean, through `python scripts/check.py` |
| UI | `corepack pnpm test` | **58 passed** - 21 `details`, 20 `intake`, 11 `bridge`, 6 `format`; no stderr |
| UI static | `corepack pnpm typecheck`, `corepack pnpm lint` | clean |
| Whole gate | `python scripts/check.py` | **14/14 steps passed**, including `drift/design tokens`, `drift/desktop icons`, `drift/ipc bindings`, `drift/ipc bindings unchanged`, `drift/goldens unchanged`, `deny/cargo-deny` (`advisories ok, bans ok, licenses ok, sources ok`) |
| Real window | `target/release/firmwaresight-desktop.exe`, 11,207,168 bytes, sha256 `82de3bbadebac5f8…` | see `P1_ANALYZE_DETAILS_SMOKE_REPORT.md` |

Test-count arithmetic against the previous record, so the growth is checkable: P1-A0's closure round
recorded 142 Rust tests. 142 + 28 new hand-written + 14 new ts-rs export tests = 184. UI: 31 recorded
at P1-A0, plus 21 in `details.test.tsx`, plus 3 in `bridge.test.ts`, plus 3 in `format.test.ts` = 58.

## 8. Boundaries held, and what this does not claim

| Boundary | Evidence |
| --- | --- |
| No migration | `SCHEMA_VERSION: i64 = 2` in `db.rs:14`; `migrations/` still holds only `0001_initial.sql` and `0002_evidence_keyed_by_build.sql`; `git status` shows that directory clean |
| No new dependency | `git diff apps/desktop/src-tauri/Cargo.toml` empty; `Cargo.lock` unmodified |
| No new design token | `assets/design-tokens.json` unmodified, still `0.2.1`; `drift/design tokens` PASS |
| No golden changed | `drift/goldens unchanged` PASS - the unit switch is presentation-only and the CLI is untouched |
| Baseline stays v0.6.0 | `product.baseline_version` reads `0.6.0` in both `872ad7e:BASELINE.yaml` and `HEAD:BASELINE.yaml`. `BASELINE.yaml` *was* edited this round - 15 key paths differ - and none of them is a version field: the edits are `active_task`, `product.status`, the three `validation` gate fields (`current_gate`, `next_authorizable_tracks`, `p1_status`) and the `p1_execution` results (`p1_stage_status`, `completed_on`, `implemented_from_head`, `desktop_smoke`, `tests_after_p1`, `open_findings`, `not_measured`, and the three `*_added: false` records). `p1_execution.no_new_baseline: true`, `baseline_version_stays: 0.6.0`; no `v0.7.0` tag |
| P2 not implemented | no Compare, Gate or Bundle surface exists; `offers no way into a stage this build does not have` still passes |

Not measured, unchanged from the state this task started with: peak RSS is `NOT MEASURED`, fuzzing is
`NOT RUN`, and the external V0 sample is still `0 / 8` - this slice added no participant and no
simulated one. Remote CI for the commit that ships this document had not run when §1-§8 were written and
is not claimed there; §9 records it after the push.

`P1 = PASS / COMPLETE` is decided in `P1_ANALYZE_EXIT_CHECKLIST.md`. `P2 = NEXT_AUTHORIZABLE_STAGE`;
this round does not implement it.

## 9. Remote CI of the shipped commits

The three commits left together in one push, so the gate ran once, on the head of that push. This
section is that run's record, added by the successor commit of `e63afaf` after the owner pushed - the
anti-recursion rule that held through P0 and P1-A0: no commit names a run of itself.

| Fact | Value |
| --- | --- |
| Governance commit, unpushed until this round | `872ad7e` - the MVP-First reset, 20 files, no source, test, schema, dependency or design token among them |
| Storage + shell commit | `f649afd` - 23 files: `query.rs`, `details.rs`, the two new Rust suites, 15 binding files |
| UI commit | `ae7759a` - 10 files: `Details.tsx`, `Details.module.css`, the three touched test files, `format.ts`, `bridge.ts`, `App.tsx` |
| Evidence + governance commit, the pushed head | `e63afaf` - 16 files: `P1_VALIDATION/`, the `.ai/` pack, `README.md`, `INDEX.md`, `BASELINE.yaml`, two delivery docs, the prompt registry, and both integrity artifacts |
| Remote CI Run #13 | `36556735551` on `e63afaf` - `completed`, `success`, 7 of 7 jobs |
| Jobs | Rust (windows-latest), Rust (ubuntu-latest), Desktop UI (windows-latest), Desktop UI (ubuntu-latest), Generated output drift, Dependency policy, macOS Core Smoke |
| Read it with | `gh run view 36556735551 --repo 2023violet/FirmwareSight` |

`872ad7e`, `f649afd` and `ae7759a` carried no run of their own, because the gate is per push and all four
commits arrived in one. Say it plainly: "every commit in this round was CI-verified" is not a claim this
record supports. What is supported is that the pushed head - which contains all of them - passed 7 of 7,
and that the same tree measured 14/14 locally before it was written.

Two claims in this pack did not survive re-checking while the run was being recorded, and both were
corrected rather than left standing:

- §8 said `BASELINE.yaml` was unmodified by this slice. It was modified: 15 key paths differ across the
  round. What is true is that none of them is a version field and `product.baseline_version` reads
  `0.6.0` at both ends, which is now what the row says.
- §6 finding 3 quoted a bindings directory hash of `35e546ba…`. The value was measured, but the command
  that produced it was never recorded, so nothing could re-derive it. It is replaced by the command and
  the value that do reproduce - `sha256sum -- * | sha256sum` over the 25 files in
  `apps/desktop/ui/src/ipc/generated/`, equal before and after `cargo test -p firmwaresight-desktop`.

A number no one can re-run is not evidence. That is the reason this section carries commands.

One more, so the rule is not just stated: the first draft of the `.ai/HANDOFF.md` rows above carried
40-character hashes for `f649afd` and `ae7759a` that `git rev-parse` did not agree with - written from
the short forms rather than read from the object store. They never reached a commit and are corrected
here as well as in place. Treat `git rev-parse` and `gh run view` as authoritative over any hash or run
id in this repository's prose, including this document's.
