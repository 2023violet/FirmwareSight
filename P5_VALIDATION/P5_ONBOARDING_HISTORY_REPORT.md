---
title: "P5 Onboarding, Help and Local History Report"
doc_id: "FS-P5-ONBOARDING-HISTORY"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 — first-run onboarding, Help/About and local History (prompt §13, §14, §15, §16, §17, §18)

**Scope of this document.** It records what Commit C added, which command proves each claim, and what
Commit C deliberately does *not* close. The History page is implemented and gated locally; it has not
been operated in an installed application on a real desktop, and the CI run for this head is recorded in
`P5_CI_AUTHORITY.md` rather than here. Diagnostics (§19) is **not** in this commit, so the Help screen
says so instead of pointing at a surface that does not exist. Nothing below says `SUPPORTED`, `beta`,
`RC` or `GA`.

## 1. §13 — first-run onboarding, as a list rather than a wizard

One component feeds both surfaces that show the guidance: `apps/desktop/ui/src/GettingStarted.tsx`
exports `GettingStartedList()` (the seven answers as a definition list) and
`GettingStartedPanel({ onDismiss })` (the same list with the way out). Analyze renders the panel in its
empty state; Help renders the list. A fork into two copies of the same seven sentences is the failure
mode this shape prevents, and `help.test.tsx` pins it by reading the `term` nodes off both surfaces and
comparing them to one array.

| §13 requirement | Where it lives | What proves it |
| --- | --- | --- |
| Seven answers, in order | `GettingStartedList()` | `help.test.tsx` → *answers all seven, in the same words Analyze uses* asserts the `dt` list equals `['What FirmwareSight does', 'First action', …, 'Where a project's policy enters']` on Analyze **and** on Help |
| Dismissible | `GettingStartedPanel` → *Hide this* | *is a panel and not a gate* clicks it and asserts the region is gone while the controls remain |
| No modal trap | The panel is a `<section>` inside `<main>`, never a `<dialog>` | the same test asserts `panel.closest('dialog')` is null, no `alertdialog` exists, and focus starts on *Choose firmware artifact* with Analyze still reachable |
| Dismissal is the reader's, for the session | `gettingStartedHidden` lives in `App.tsx`, above the page switch | *keeps the guidance hidden for the session, across a page change* hides it, walks to History and back, and asserts it does not return — then finds the words on Help |
| No account, no internet, no analytics | Nothing in the component calls the shell beyond the window title | `getAppIdentity`/`listHistory*` are the only reads in this commit and none of them is reachable from the panel |
| No compulsory wizard before Analyze | Analyze's own controls are unchanged and above the panel | `intake.test.tsx`'s 22 tests pass without one line of its interaction changed |

`§13`'s flow requirement names four steps; the panel renders them as one ordered list
(`Analyze → Compare → Release → Bundle`) and the sentence under it says what each step is *for*, which
is the difference between a tour and an orientation.

## 2. §14 — Help/About, and why the window title moved to Rust

`apps/desktop/ui/src/Help.tsx` shows: the identity block, what the product can read, the local-first
statement, the seven getting-started answers, and where the four documents live. It takes no browser and
needs no network, and it renders **zero `<a>` elements** — §14 permits optional external links and this
build has none worth giving, so the screen states the folder path in text instead of a URL.

The identity block is asked for once, with no argument, from `support::get_app_identity`:
`productName` and `binaryName` from `app.package_info()`, `identifier` from `app.config()`, the platform
from `std::env::consts`, the schema version from `SCHEMA_VERSION`, and the store **file name** only. The
front end holds no copy of any of it: until the reply arrives all seven rows read `not reported`
(`Help.tsx`'s `told()`), because a default written into the WebView would be a second source of truth —
the exact drift D1 spent a decision removing.

**The title fix.** The known limitation was a fixed window title. `tauri.conf.json` can only carry a
static string, and granting the WebView `core:window:allow-set-title` would be a capability change
(`AGENTS.md` §9 reserves that for a human) *and* a channel from a file name into a window property. So
`App.tsx` tells the shell which page moved, over `support::set_window_title(MainWindowPage, AppHandle)`,
and Rust composes `"FirmwareSight - <Page>"` from a closed five-variant enum. The command is synchronous
and runs on the main thread, which is where a window property belongs.

Two tests hold that boundary in `apps/desktop/src-tauri/tests/history_reads.rs`:

- *the_window_title_comes_from_a_closed_page_set_with_nothing_to_inject* — the wire form of
  `MainWindowPage` is exactly the five strings, and no request field carries a path separator;
- *the_title_fix_took_no_new_capability* — reads `capabilities/main.json` and asserts the window `main`
  still holds exactly `["core:default"]`, with no `allow-set-title` and no shell or filesystem permission.

`bundle.resources` is `None`, so the installed package ships no documents. Rather than print a pointer a
stranger cannot follow, Help flags three of its four document rows `not carried inside the installed
package`, and `help.test.tsx` counts them: four rows, three flagged, and the one unflagged row is Getting
Started because it is on the screen.

## 3. §15–§17 — History, on reads the storage layer did not have

§18 asked History to be built from facts already persisted, and it was: **no migration was added for
History**. The five migrations are unchanged from Commit B (`0005_unknown_reasons.sql` came from the
numeric-Unknown work, not from this). What did not exist was a *read* path — `firmwaresight-storage`
could list Compare candidates but nothing could page over Gate runs or release records. That is what
Commit C added.

- `crates/firmwaresight-storage/src/history.rs` — `HistoryQuery` plus
  `list_history_builds` / `list_history_gate_runs` / `list_history_releases`, all returning the existing
  `Page<T>`. The build table delegates to `list_compare_candidates` rather than re-writing its SELECT.
  Scope is one closed predicate (`b.state = 'COMPLETE' AND b.project_id IN (?)`), sort is
  `created_at DESC, id ASC` (and the snapshot/run/release id as the tie-break), and the ceiling is
  Compare's: default 25, maximum 100.
- `crates/firmwaresight-storage/src/query.rs` — one `search_clause` helper, shared with the candidate
  list, that binds the filter text as a **value** inside `'%' || ? || '%' ESCAPE '\\'` and escapes `%`,
  `_` and `\` through the existing `like_pattern`. A filter is never concatenated into SQL.
- `apps/desktop/src-tauri/src/history.rs` — three `async` commands on `spawn_blocking`, each fixing
  `project_ids` to the local project so the WebView cannot widen the scope, and clamping an
  out-of-range number to the ceiling instead of letting it wrap.

The page shows three tables (`Builds`, `Gate runs`, `Release records`), each with its own filter, offset
and request counter; opening a row reveals the full ids and digest in a detail row; and the subhead says
in plain words that these are persisted facts and not the artifact now open on Analyze (§17's last
requirement). No control on the page writes: it issues three reads and nothing else, and there is no
delete, edit, re-run or accept affordance for one to be mistaken about.

### §16 required content, row by row

| §16 field | Rendered from | Test |
| --- | --- | --- |
| build: file name, shortened snapshot id, shortened SHA-256 with full text on open, stored time, architecture, nonvolatile state+value, RAM state+value | `CompareCandidateDto` | UI *lists the three stored populations the shell returned*, *gives the whole digest back when the row is opened, and takes it away again*; storage `history_rows_carry_a_file_name_and_never_a_path`, `an_unknown_footprint_reads_as_unknown_and_never_as_zero` |
| Gate run: run id, build, baseline if any, disposition, counts by state, stored time | `HistoryGateRunRowDto` | UI *shows every state count, including the ones that are zero*, *names a run with no baseline as none, and never invents one*; storage `gate_rows_report_the_stored_disposition_and_the_counts_by_state` |
| release: release id, version, build, Gate run, manifest SHA-256, stored time | `HistoryReleaseRowDto` | UI *opens the ids a stored Gate run and a release record are cited by*; storage `release_rows_carry_the_version_the_digest_and_the_run_that_qualified_them` |
| deterministic sort and a bounded page, both Rust's | `Page<T>` as returned by the read | storage `history_build_rows_are_scoped_and_newest_first`, `gate_rows_follow_the_stored_order_of_time_then_identity`, `history_page_size_is_a_server_side_ceiling`; desktop `a_history_page_is_bounded_by_rust_not_by_the_caller` |
| must not expose: artifact absolute path, project root, bundle destination, database path | (nothing to expose) | desktop `no_history_page_ever_carries_a_host_path` walks every string field of the serialized page; UI *shows no host path anywhere on the page* |
| must work when the firmware bytes are gone | rows read from `builds`/`artifacts`, with no filesystem access on either side | storage `history_build_rows_survive_the_artifact_being_deleted`; the UI asserts the sentence on the screen instead of touching a file |

### The filter scope decision

The stored `artifacts` row keeps the intake directory, and it is the obvious thing a search box might
match. It was excluded on purpose: a filter that searched a stored directory would turn the search box
into an oracle for "which folders on this machine hold firmware", which is a question §16 forbids and §19
keeps behind an allowlist. History filters therefore search identity columns only — build id, snapshot
id, artifact SHA-256, architecture for builds, and the id/version/digest columns for the other two, each
a closed list in `history.rs`.

Two storage tests make that a fact rather than an intention: `a_history_filter_is_text_and_not_like_syntax`
(a `%` typed into the box must match a literal percent, not a wildcard) and
`a_filter_searches_identity_columns_and_never_a_directory`, which seeds real builds into a real temporary
directory and asserts that filtering by that directory's own name returns zero rows while filtering by a
stored digest returns one.

## 4. What was measured

One command reproduces everything in this section.

```
python scripts/check.py
```

Result on this host, 2026-10-03: **16 of 16 steps passed** — `rust/fmt`, `rust/clippy`, `rust/test`,
`frontend/{install,typecheck,lint,test,build}`, `drift/{design tokens,desktop icons,ipc bindings,ipc
bindings unchanged,fixtures tracked,version identity,goldens unchanged}`, `deny/cargo-deny`.

| Surface | Before | After |
| --- | --- | --- |
| Rust tests, whole workspace (`rust/test`) | 775 | **812 passed, 0 failed** |
| — of which `crates/firmwaresight-storage/tests/history_reads.rs` | (none) | 19 |
| — of which `apps/desktop/src-tauri/tests/history_reads.rs` | (none) | 11 |
| UI tests (`frontend/test`) | 160 | **200 passed** |
| — of which `history.test.tsx` / `help.test.tsx` | (none) | 24 / 16 |

Bounded at the population §49 names, measured by
`cargo test -p firmwaresight-storage --release --test history_reads -- --nocapture history_stays_bounded_at_productization_scale`
— which seeds 100 builds, 100 Gate runs and 50 release records through the real writers and asserts the
page sizes, the tie-break order and that page two repeats nothing from page one:

```
HISTORY SCALE builds=100 gate=100 releases=50:
  build page 467.9µs, gate page 541.5µs, release page 248.8µs, filtered gate page 443.8µs
```

The repository defines no latency SLA for History, so those are printed and not asserted. They are
release-profile timings over a freshly written store in a temporary directory; the same test on the debug
profile reports 889.6µs / 1.0764ms / 485.4µs / 810.1µs, which is the number a developer sees locally.

## 5. Mutation proofs: that the new assertions bite

Each row is a deliberate defect introduced into the component, the one command that caught it, and the
state after reverting. Run from `apps/desktop/ui`.

| # | Defect | Caught by |
| --- | --- | --- |
| 1 | `applyFilter` stops resetting the offset | *returns to the first page when a filter is applied from a later one* — request became `{offset: 25, filter: "app"}` |
| 2 | The request guard is disabled (`request.current !== id` → never true) | *paints only the newest answer when an older reply arrives last* — `stale.elf` reappeared under a `two` filter |
| 3 | `dispositionBadge`'s default branch returns PASS | *calls an unmappable stored word unknown instead of a verdict* |
| 4 | The opened row truncates the SHA-256 instead of printing it | *gives the whole digest back when the row is opened, and takes it away again* |
| 5 | `useBoundedPage` sends `filter: null` (i.e. filters the page it holds) | three tests: *sends a filter as a request…*, *does not filter on a keystroke*, *returns to the first page…* |
| 6 | A `<button>Delete row</button>` is added to the Builds table | *offers no write, delete or re-run control and no link out* |
| 7 | `Help.tsx` renders `0.6.0` before the shell answers | *says it has not been told instead of showing a version it kept* |

After the seventh, `npx tsc --noEmit`, `npx eslint .` and `npx vitest run` were re-run on the reverted
tree: typecheck silent, lint silent, 200 of 200 passing.

## 6. What Commit C leaves open

- **History in an installed app.** The page has been exercised under jsdom and through the Rust boundary
  tests, not yet by a person in the packaged binary. That belongs to the §38/§64 install acceptance
  (`P5_INSTALL_RECOVERY_REPORT.md`), which was written before this page existed.
- **Diagnostics.** §14 asks where to find it; the honest answer is that it is not in this build, and the
  Help screen says so. Commit D owns §19 and the store-path visibility that goes with it.
- **A recency index.** None was added: the scale measurement above shows the page reads in well under a
  millisecond at the §49 population, so an index would be a migration without a user-visible reason.
  `P5_MIGRATION_DECISION.md`'s reasoning is unchanged by this commit.
- **The window title on a restore-from-minimize.** Rust sets the title on every page move, so the value
  is correct for the page shown; a real-desktop check that Windows' own title-bar behaviours (Aero
  snap, taskbar grouping) do not rewrite it is part of the install acceptance above.
