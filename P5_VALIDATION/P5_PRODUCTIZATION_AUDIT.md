---
title: "P5 Productization Audit"
doc_id: "FS-P5-AUDIT"
product: "FirmwareSight"
version: "1.0"
status: "CHECKPOINT_CONCLUDED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-03"
---

# P5 — Productization audit (prompt §4)

**Attestation.** This document was written before any product change. The working tree was clean at the
head named in section 0 (`git status --short` empty), and the audit itself touched no source, schema,
workflow, fixture, generated asset or governance field. Every number below comes from a command run on
this tree on 2026-10-03; nothing is recalled from a previous round. Where a fact lives only outside the
repository it is cited in env-var form, never by a user-specific absolute path.

**What changed after the checkpoint:** §5 governance was opened by the commit that lands this audit
(`active_task: P5_PRODUCTIZATION`, stage `P5`, state `IN_PROGRESS` in `BASELINE.yaml`, which also carries a
new `p5_execution:` block), and the prompt was archived under `10_AUDIT/SOURCE_PROMPTS/` with its hash
recomputed from the stored bytes and byte-compared against the delivered file. **No product code was
written by that commit**; the four owner answers recorded in section I are the plan its follow-on commits
implement.

## 0. Preflight (§2) and authority read (§3)

```
git fetch --prune origin      # no output change
git rev-parse HEAD            # d83175a3309a5ac5ec01995bdf7b74ec0ddb3d91
git ls-remote origin refs/heads/main   # d83175a3309a5ac5ec01995bdf7b74ec0ddb3d91
git branch --show-current     # main
git status --short            # (empty)
git diff / git diff --cached  # (empty)
git worktree list             # D:/study/Software/FirmwareSight d83175a [main]  (one worktree)
```

HEAD equals origin equals the `d83175a…` the prompt expects, so the remote did not move and §2 does not
stop the round. Governance was **deliberately not opened while the audit was being written** — §4 puts the
audit first and §5 opens the round only after it. `BASELINE.yaml` read `active_task: NONE` at every
measurement in this document, and the same commit that lands this audit sets the live fields to stage `P5`
and state `IN_PROGRESS` (§5), registers the prompt's hash in `10_AUDIT/SOURCE_PROMPTS/README.md` and adds
the `p5_execution:` block. Nothing else in §5 was done early: `baseline_version` stays `0.6.0`, no version
in any manifest was touched, and no tag or GitHub Release exists.

Authority read: `AGENTS.md`, `.ai/README.md`, `.ai/CURRENT_STATE.md`, `.ai/DECISIONS.md`,
`.ai/ACTIVE_TASK.md`, `.ai/HANDOFF.md`, `BASELINE.yaml`, `06_DELIVERY/06_STAGE_GATES.md`,
`06_DELIVERY/07_POST_MVP_CANDIDATE_ROADMAP.md`, `06_DELIVERY/08_MILESTONE_DELIVERABLE_MATRIX.md`,
`01_PRODUCT/01_PRD_MVP.md`, `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md`, `04_TECH/05`,
`04_TECH/14`, `04_TECH/15`, `04_TECH/20`, `G2_VALIDATION/G2_KNOWN_LIMITATIONS.md`,
`POST_G2_E2E_REMEDIATION/{REMEDIATION,FOCUSED_REVALIDATION,EXIT_CHECKLIST}_REPORT.md`,
ADR-0015/0016/0018/0023/0026, plus the inspection list in §3: `Cargo.toml`, `Cargo.lock`,
`apps/desktop/ui/pnpm-lock.yaml`, `tauri.conf.json`, the desktop `Cargo.toml`,
`.github/workflows/`, `crates/firmwaresight-storage/migrations/`, the storage query APIs,
`apps/desktop/ui/src/`, `apps/cli/`, `fixtures/`, `scripts/check.py`.

## A. Packaging and version identity

| Field | Value | Where measured |
| --- | --- | --- |
| `productName` | `FirmwareSight` | `apps/desktop/src-tauri/tauri.conf.json:3` |
| identifier | `com.firmwaresight.desktop` | `tauri.conf.json:5` |
| Tauri app version | **`0.1.0`** | `tauri.conf.json:4` |
| workspace version | **`0.1.0`** | `Cargo.toml:14` (`[workspace.package]`) |
| desktop crate version | `version.workspace = true` | `apps/desktop/src-tauri/Cargo.toml:3` |
| CLI crate version | `version.workspace = true` | `apps/cli/Cargo.toml:3` |
| UI package version | **`0.1.0`** | `apps/desktop/ui/package.json:4` |
| `license` / `publish` | `Proprietary` / `false` | `Cargo.toml:17,18` |
| `edition` / `rust-version` | `2024` / `1.98` (pinned `1.98.1`) | `Cargo.toml:15,16`, `rust-toolchain.toml` |
| Tauri / tauri-build / codegen | `2.12.0` / `2.7.0` / `2.7.0` | `Cargo.lock:3445,3496,3517` |
| runtime / wry / utils | `2.12.0` / `2.12.0` / `2.10.0` | `Cargo.lock:3616,3641,3667` |
| dialog plugin | `tauri-plugin-dialog 2.8.0` (direct) | `apps/desktop/src-tauri/Cargo.toml:44` |
| `tauri-plugin-fs 2.6.0` | **transitive only**, pulled by the dialog plugin | `Cargo.lock:3585` (dialog's dep list), no direct reference in any `Cargo.toml` |
| `bundle.active` | **`false`** | `tauri.conf.json:29` |
| `bundle.targets` | `"all"` | `tauri.conf.json:30` |
| bundle icons | `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.ico` | `tauri.conf.json:31-37`, `apps/desktop/src-tauri/icons/` (5 files, no `.icns`) |
| window | title `FirmwareSight - Analyze`, 1040×760, min 720×480 | `tauri.conf.json:15-18` |
| CSP | `default-src 'self'; … connect-src 'self' ipc: http://ipc.localhost; object-src 'none'; base-uri 'self'; frame-ancestors 'none'` | `tauri.conf.json:25` |
| frontend build hook | `beforeBuildCommand: "pnpm build"`, `frontendDist: "../ui/dist"` | `tauri.conf.json:9,10` |
| `custom-protocol` | still required: `[features] custom-protocol = ["tauri/custom-protocol"]` | `apps/desktop/src-tauri/Cargo.toml:14-15` |

**CLI version identity is a single source and it is `0.1.0`.** `crates/firmwaresight-artifact/src/pipeline.rs:22`
binds `FWSIGHT_VERSION = env!("CARGO_PKG_VERSION")`, so the artifact fingerprint, the CLI `--version`
and the `fwsight_version` field of a Snapshot all carry the same string. Run first-hand on the current
debug binary:

```
$ target/debug/fwsight.exe --version
fwsight 0.1.0
```

`--help` prints `Usage: fwsight.exe <COMMAND>` (clap uses the binary file name) and its long description
still advertises the technical slice: "`analyze`, `diff`, `gate` and `release prepare` are implemented;
`project doctor` is a later phase and is not available" (`apps/cli/src/main.rs:57-59`). Exit codes are
already typed and documented at `apps/cli/src/main.rs:45-50` (Gate REVIEW 4, BLOCK 5, bundle/export 6).

**The §7 premise is right about the value and wrong about the scope.** The prompt states Tauri config
contains `"version": "0.1.0"`; it does, and so do the workspace and the UI package. It is *not* a single
straggler to bump: `0.6.0` exists only as a **document baseline** (`BASELINE.yaml:7 baseline_version: 0.6.0`,
`CHANGELOG.md` front matter and its `## 0.6.0` entry), never as an artifact version. §67's "one identity"
goal therefore requires a decision about how many version strings to unify — see **D1** in section I.

**Current CI package artifacts: there are none.** `.github/workflows/` holds exactly one file,
`p0-check.yml` ("P0 verification gate"), with 5 job keys that expand to the 7 jobs every recent run
reports: `rust` (ubuntu + windows matrix), `frontend` (matrix), `drift`, `deny`, `macos-core`
(`.github/workflows/p0-check.yml:18,70,93,130,145`). No step installs `tauri-cli`, no step runs
`tauri build`, no step calls `actions/upload-artifact`. So `bundle.active: false` is consistent with
reality: nothing in this repository has ever produced an installer, and the only shipping binary so far
was a hand-run `cargo build --release --features custom-protocol` executed outside CI during the
post-G2 round.

**Checksum namespaces.** Three different `SHA256SUMS`-like artifacts already coexist and P5 must keep
them distinct: the repository manifest `SHA256SUMS` at the root (595 entries, regenerated by
`scripts/generate_baseline_artifacts.py`), the Release Bundle's own `SHA256SUMS` inside a bundle
(`crates/firmwaresight-report/src/release_render.rs:739` states the bundle carries its file indexes and
"no temporary or hidden file"), and — new in P5 — distribution checksums over installer artifacts (§10).
`fixtures/manifest.json` is a *fourth* digest registry with its own schema
(`firmwaresight-fixtures-1`, 25 files, generator-attributed).

## B. Onboarding and first-use guidance

**What a new user sees on a cold start.** Three rail entries and nothing else:

```
const PAGES = [{ key: 'analyze', … 'Analyze' }, { key: 'compare', … 'Compare' }, { key: 'release', … 'Release' }]
```
(`apps/desktop/ui/src/App.tsx:33-40`, consumed at `App.tsx:43`). The shell comment at `App.tsx:6` is
explicit that this build has "no History and no Settings tab, because an empty placeholder you can click
is not an honest 'not built'".

Per-page cold states, verbatim:

- Analyze with nothing analyzed: `Nothing has been analyzed in this session yet. Choose an artifact and run Analyze.` (`apps/desktop/ui/src/Analyze.tsx:222-226`, `aria-label="No analysis yet"`).
- Compare with fewer than two builds: `Analyze another firmware build before comparing.` plus a `Go to Analyze` control (`apps/desktop/ui/src/Compare.tsx:322-330`).
- Release with no policy loaded: `No project policy is loaded. A run judges FirmwareSight's stated default policy, and the run below shows which values those were.` (`apps/desktop/ui/src/Release.tsx:323-327`).

So each page does fail honestly, but **there is no path from "installed" to "what are these five verbs
and what may I conclude from them"**. Concretely missing:

- no About / Help / version display anywhere in the UI (grep over `apps/desktop/ui/src` finds no
  `About`, `Getting started` or `Diagnostics` surface; the only diagnostic-adjacent string is
  `Diagnostics ID {operationId}` inside an error panel, `apps/desktop/ui/src/Details.tsx:698`);
- no first-run guidance of any kind — the window title itself is static
  (`tauri.conf.json:15`), so the app never tells the user which page they are on (L21);
- status semantics (`PASS / REVIEW / BLOCK / UNKNOWN / N/A`, the five states AGENTS.md §11 freezes) are
  explained nowhere in the window; a user meets `Unknown` first as a neutral grey row;
- project-policy guidance exists only as **document prose** in `04_TECH/14`/`01_PRODUCT/01` and as the
  default-policy footnote above; the window never says what a budget does or why a BLOCK happened beyond
  the finding text;
- the words on screen are still partly Core vocabulary, not user vocabulary (e.g. `Analyze.tsx:305-306`,
  `Compare.tsx:695,799-806`, `Release.tsx:691-692` emit enum-shaped strings — the same class as L15/L20).

The shared component inventory is four files (StateBadge, ErrorPanel, table pager, size-unit switch). The
design-system gaps §28-§32 name — L19 label ambiguity, L20 basis wording, L21 title, L25 the dead filter
click, plus a danger action, metric card, dialog, help callout, diagnostics block and history row — have
no existing implementation to audit; they are greenfield inside the frozen token set, which AGENTS.md §11
constrains (no new tokens, no shadow/gradient, five-state icon+label).

## C. History read model

**Tables that already exist** (`crates/firmwaresight-storage/migrations/`):

| table | migration | key columns relevant to History |
| --- | --- | --- |
| `projects` | 0001 (`:10`) | `id`, `name`, `created_at` |
| `builds` | 0001 (`:16`) | `id`, `project_id`, `snapshot_id`, `normalization_version`, `created_by_fwsight`, `state IN ('IMPORTING','COMPLETE','FAILED')`, `created_at` |
| `artifacts` | 0001 (`:31`) | `build_id`, **`path`**, `kind`, `sha256`, `byte_size`, `parser_id`, arch/bitness/endianness, `entry_point`+`entry_unknown`, `build_id_note`+`build_id_unknown` |
| `sections` / `symbols` | 0001 (`:52`, `:74`) | detail rows, PK-scoped to one build |
| `evidence` | 0001 (`:88`) | `field`, `classification IN (observed,derived,declared,unknown)`, `source_type`, `source_locator`, `raw_value`, `rule`, `confidence` |
| `gate_runs` | 0003 (`:21`) | `build_id`, `baseline_build_id`, `policy_sha256`, `overall_effective_severity`, `created_at`, id format `gate-`+64 hex |
| `gate_findings` / `gate_finding_evidence` | 0003 (`:41`, `:80`) | 10 findings per run, ordinal-ordered |
| `accepted_reviews` | 0003 (`:92`) | `run_id` + reviewer text |
| `release_records` | 0004 (`:37`) | `build_id`, `gate_run_id`, `release_version`, `manifest_sha256`, `created_at`, id format `release-`+64 hex |

Indexes that exist for History: `idx_builds_project`, `idx_builds_snapshot`, `idx_artifacts_build`,
`idx_artifacts_sha256`, `idx_evidence_build`, `idx_evidence_field`, `idx_gate_runs_build`,
`idx_gate_runs_created`, `idx_release_records_build (build_id, created_at, id)`, `idx_release_records_run`.
**There is no index on `builds.created_at`**, which is exactly the column a recency-ordered History list
sorts on (`0001_initial.sql:28-29` covers `project_id` and `snapshot_id` only).

**Read APIs today** (all public methods in the storage crate):

- `db.rs:60 open`, `:72 open_in_memory`, `:118 migrate`, `:187 connection` (raw `&Connection` escape hatch),
  `:196 import_snapshot`, `:251 summary(build_id)`, `:315 build_id_for_snapshot(snapshot_id)`;
- `query.rs:214 query_sections`, `:270 query_symbols`, `:326 query_evidence` — paged, but each is scoped
  to **one build** by its query struct;
- `gate.rs:214 persist_gate_run`, `:292 gate_run_by_id`, `:334 accept_review`,
  `:395 accepted_reviews_for_run`, `:428 load_gate_facts(snapshot_id)`, `:518 evidence_id_for_field`;
- `release.rs:80 persist_release_record`, `:132 release_record_by_id`.

The 24 IPC commands confirm the same shape from the shell side
(`apps/desktop/src-tauri/src/lib.rs:927-951`: 2 fixture/summary, 4 intake, 3 detail, 5 compare,
5 release/gate, 3 bundle). `release::get_gate_run` and `compare::list_compare_candidates` are the only
history-ish reads, and `list_compare_candidates` enumerates builds for *picking a baseline*, not for a
History view.

**Answer to §4 C: minimal History is implementable WITHOUT migration, with two named exceptions.**

Without a new table or column, a History screen can list `builds` (project, `created_at`, state,
`created_by_fwsight`), join `artifacts` for sha/kind/size, and reach `gate_runs` / `release_records`
per build — the data is all there and every row a History needs is already persisted (post-G2 read them
back from a real store). New storage **read** APIs are required (a builds listing, gate runs by build or
by recency, release records by recency) plus one index if the ordering must be cheap.

Two things cannot be done without a migration, and neither is a History convenience:

1. **`builds.created_at` has no index** (`0001_initial.sql:28-29`) — a `CREATE INDEX` is additive, so it
   is a migration, not a query change.
2. **Artifact display name is derivable, not stored**: `artifacts.path` holds the full user-chosen
   location (L18, by design, local-only), so a History row must call `display_name()`
   (`apps/desktop/src-tauri/src/service.rs:49-53`) on read rather than persist a second copy. This one
   needs **no** migration; it needs care not to widen the path boundary.

Content-derived identity also constrains the design honestly: re-analyzing the same bytes adds no
`builds` row, and re-running the Gate with the same context adds no `gate_runs` row (post-G2 finding,
recorded in `POST_G2_E2E_REMEDIATION/REMEDIATION_REPORT.md`), so a History list must not present itself
as an event log. It is a list of distinct facts, and `created_at` is not a freshness signal.

**The L6/L7 reason gap is the migration question, and it is not about History.** `sections.file_offset`
(`0001_initial.sql:65`) and `symbols.address` (`:79`) are the only two of the seven nullable *numeric*
columns in the schema with no companion `*_unknown` reason column: `entry_point`, `virt_addr`,
`load_addr`, `mem_size` and `size` all have one (`:43-44`, `:61-68`, `:80-81`), as does every nullable
text column (`:46`, `:56`, `:70`, `:78`). The write path is the cause: `optional_fact_u64` (`crates/firmwaresight-storage/src/db.rs:567-569`) returns
`fact.value().copied()` and **drops the reason**, whereas its sibling `optional_string` (`:571-575`)
and `optional_fact` (`…563`) keep it. Therefore:

- the reason for a numeric `Unknown` does not exist anywhere in the store — not in `builds`, not in
  `evidence` — so no read-side API can recover it; and
- §17's PRESERVE UNKNOWN discipline is currently violated for exactly these two columns, which is L6/L7.

**Migration decision (audited conclusion, not an implementation):** migration `0005` adding
`sections.file_offset_unknown` and `symbols.address_unknown`, plus the `builds.created_at` index, is the
narrow additive change that closes L6/L7. It is *not* "0005 merely to make History easier", which §18
forbids; History itself needs zero schema change. Per §18 this is written up as
`P5_MIGRATION_DECISION.md` **before** any schema code, and it is one of the checkpoint items (**D2**).
Existing precedent for the risk profile: `storage.rs:476 an_a_version_one_database_is_upgraded_without_
losing_its_evidence`, `gate_history.rs:314 a_failed_gate_migration_leaves_the_previous_schema_intact_and_
recoverable`, `release_records.rs:441 an_acceptance_written_before_the_upgrade_is_still_there_after_it`.

## D. Diagnostics, errors, health visibility, privacy boundary

**Typed error vocabulary exists and is stable.** 48 distinct codes across 12 domains, measured by
`grep -rhoE '"ERR-[A-Z]+-[0-9]{4}"'`:

| domain | range | distinct codes |
| --- | --- | --- |
| FORMAT / GUARD / INPUT | 0001–0002, 0001 | 3 |
| PARSE | 2001–2002 | 2 |
| MAP | 3001–3002 | 2 |
| STORAGE | 4001–4010 | 10 |
| DIFF | 5001–5002 | 2 |
| EXPORT | 6001 | 1 |
| BUNDLE | 6101–6114 | 14 |
| CONFIG | 7001–7008 | 8 |
| GIT | 8001 | 1 |
| INTERNAL | 9001–9004 | 4 |

Each envelope carries `code`, `message`, `operation_id`, `details`, `remediation`
(`apps/desktop/src-tauri/src/lib.rs:763-768`), and the UI shows `Diagnostics ID {operationId}`
(`apps/desktop/ui/src/Details.tsx:698`). So the *raw material* §19-§20 asks for is already machine-shaped.

**What does not exist:**

- **No Diagnostics surface at all.** No page, no panel, no export, no copyable bundle of the above.
  Errors appear only inline, in the screen that produced them, and are lost on navigation.
- **No log file.** The desktop crate has no subscriber and writes nothing: grep over
  `apps/desktop/src-tauri/src/*.rs` finds only `tauri_plugin_dialog` uses, no `tracing`/`log`/`println`.
  The CLI does use `tracing` (`apps/cli/Cargo.toml`) but the shipped desktop path emits nothing to disk.
- **No DB health visibility.** `PRAGMA integrity_check` appears **nowhere** in the repository
  (`grep -rn "integrity_check" crates apps` → no hits). The only health-ish guard is refusing a *newer*
  schema without touching it: `db.rs:120-127` returns `UnsupportedSchemaVersion { found, supported }`
  with the comment "Never delete and recreate: that is how history gets destroyed by an older binary",
  and the paired message tells the user to restore a backup
  (`crates/firmwaresight-storage/src/error.rs:61`) — a message that currently has no backup feature to
  point at. `PRAGMA` settings that the schema assumes are verified by
  `storage.rs:374 the_connection_carries_the_pragmas_the_schema_assumes`.
- **No app/version/WebView visibility.** Nothing in the UI shows the app version, the schema version,
  the Tauri/WL version, or the store path. `FWSIGHT_VERSION` reaches a Snapshot record but never the
  window. This is the §20 gap, and it is cheap to close with data already in the process.
- **Git availability is reported as a reason, not as a status.** `crates/firmwaresight-project/src/git.rs:221-245`
  produces the exact set "git is not installed" / "did not answer within {n}s" / "produced more output
  than the Gate reads" / "this directory is not a git repository", and the Gate fails closed to
  UNKNOWN (`ERR-GIT-8001`). Correct behaviour, invisible in aggregate: a user cannot ask "is Git
  available to this app at all?" without reading a Gate finding.

**Privacy boundary as it stands (and it must survive P5).** The local-first line is enforced in code, not
just prose: `service.rs:49-53` reduces any path to a file name for display; `lib.rs:772-784` `redact()`
walks the located paths and replaces each occurrence inside a diagnostic string with its file name;
`artifacts.path` never reaches the IPC DTOs, the UI, Gate locators, `release_records` or a bundle
(adjudicated in the G2 Storage Path Semantics Clarification Addendum, recorded as L18 at
`G2_VALIDATION/G2_KNOWN_LIMITATIONS.md:40`). CSP is restrictive with no remote origin
(`tauri.conf.json:25`), `connect-src` is `ipc:` only, and `tauri-plugin-fs` is transitive-only.
P5 §20 requires **positive-control** tests for this (a path that must not appear), not only the absence
assertions already present — that is the part of Diagnostics that is genuinely new work.

## E. Reliability and recovery

**Migration paths supported today:** 1→2→3→4 applied in version order, each in its own transaction
(`db.rs:23-28`, `:132` `apply`, `:139` per-migration `transaction()`), and an unknown newer version
refused without reset (`db.rs:120-127`). Three regression tests cover upgrade-does-not-lose,
failed-migration-leaves-previous-schema, and pre-upgrade-row-survives (cited in section C).

**Transaction boundaries:** five write paths, all explicit — `db.rs:139` (migration), `db.rs:206`
(import), `gate.rs:240` (gate run + findings + evidence), `gate.rs:376` (accept review),
`release.rs:110` (release record). Import is crash-safe by ordering: the row is inserted at `'IMPORTING'`
(`db.rs:220`) and flipped to `'COMPLETE'` only as the last statement before `tx.commit()`
(`db.rs:237-244` with its comment "nothing can observe COMPLETE with partial content"). A build left at
`IMPORTING`/`FAILED` is therefore the documented signature of an interrupted write — which History must
surface rather than hide.

**What is missing:**

1. **No backup/restore at all.** `grep -rn "backup" crates/firmwaresight-storage/src` hits only the
   wording inside one error message (`error.rs:61`). There is no `Database::backup`, no `VACUUM INTO`,
   no export. §22 requires a documented user-recoverable backup before P5 can tell a stranger "your
   history is safe"; today that sentence would be false.
2. **No integrity check and no startup health step.** `project doctor` is named in the CLI help as not
   available (`main.rs:57-59`), and nothing runs `PRAGMA integrity_check` on open. §23's corrupt-DB path
   (do NOT auto-delete) has no current implementation to fail closed *into* — a corrupt file surfaces as
   a rusqlite open error wrapped by `Session::open` (`lib.rs:921-922`), formatted as
   `{err} (code {stable_code})`, i.e. an `ERR-STORAGE-*` envelope with no remediation specific to
   corruption.
3. **Stale staging/backup siblings are never reclaimed.** The bundle write is correct in-process:
   `crates/firmwaresight-project/src/bundle.rs:401-420` creates a unique sibling staging dir
   (`sibling(parent, "staging")`, named with `std::process::id()` plus a process-local counter,
   `bundle.rs:1252-1276`), writes the payload, verifies it, then swaps, calling `discard(&staging)` on
   every failure path; `bundle/tests.rs:133-139` asserts no staging or backup sibling survives a success
   and `:215` asserts the same for the overwrite path. But nothing scans for an orphan left by a process
   that was killed mid-write, so a crash between create and rename leaves a
   `.firmwaresight-staging-*` folder in the user's chosen parent until a human deletes it. That is a
   §23/§45 "recoverable, explainable" item, not a data-loss item.
4. **Compare export leaves a temp file only if the rename fails, and it is removed** —
   `apps/desktop/src-tauri/src/compare.rs:321` `let _ = std::fs::remove_file(&temp);`. Honest current
   state: single call site, best-effort cleanup, untested for the power-loss case.
5. **Store location is fixed and carries a legacy name.** `lib.rs:919-922` opens
   `app_data_dir()/firmwaresight-p0.sqlite` — on Windows that is
   `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite`. Nothing creates, migrates, locks or
   rotates it beyond `create_dir_all`, and there is no user-facing path display. A "-p0" filename in a
   productized app is a §H wart with a data-continuity cost attached to fixing it (see **D6**).

**Previous E2E failure/recovery evidence** (this is what §4 E asks for, and it is the strongest asset P5
inherits): the post-G2 round ran the real binary through 283 registered cases and closed
`PASS_WITH_FINDINGS` with S0 = 0, S1 = 0 and 6 expectation failures, and its three findings were then
remediated and re-validated — `POST_G2_E2E_REMEDIATION/REMEDIATION_REPORT.md` and
`FOCUSED_REVALIDATION_REPORT.md` record F001 (pending-selection attribution), F002 (foreign-destination
confirmation) and F003 (GNU ld banner sniff) each with a failing regression first, a mutation proof, 30
fresh-process repetitions and a focused real-desktop re-validation on a freshly built binary; both fix
commits carried 7/7 remote CI on the first attempt. Recorded behaviours that P5 must not re-break:
`ERR-BUNDLE-6106/6107` refusal paths, `ERR-BUNDLE-6114` (both builds must be analyzed this session),
identical re-run adds no row, the >512 MiB guard refuses in ~1.10 s with no working-set growth, and the
519 MB case parses warm in ~4.7–4.9 s (`REMEDIATION_REPORT.md:141-143`).
Full frames and registers stay outside the repository at
`%TEMP%\FirmwareSight-PostG2-E2E-20261002-020701` and
`%TEMP%\FirmwareSight-PostG2-E2E-Remediation-20261002`, as those briefs required.

## F. Compatibility, cohort and fixtures

**Supported cohort, unchanged by this audit:** GCC/Clang ELF + GNU ld MAP. Keil/IAR remain unsupported;
no P5 work here widens that.

**Actual committed fixtures** (`fixtures/manifest.json`, schema `firmwaresight-fixtures-1`,
25 files across 6 named sets; dirs `fixtures/elf/{p0-basic,p0-dual-region,p2-diff/{base,target}}`,
`fixtures/generated`, `fixtures/malformed`, `fixtures/project/p4-release`):

| set | files | what it proves |
| --- | --- | --- |
| `p0-basic` | 2 | real linked ELF, default newlib script, no MAP |
| `p0-dual-region` | 4 | controlled `ROM (rx)` + `RAM (rwx)` script, `.data`/`.ota` VMA-in-RAM LMA-in-ROM, plus its GNU ld MAP |
| `p2-diff-base` / `p2-diff-target` | 6 + 6 | a changed pair with MAP + `.ld` + source each |
| `malformed` | 4 | `empty.bin`, `truncated-elf.bin`, `sparse-elf-header.bin`, `wrong-magic.bin` (+ README) |
| `generated` | 4 | guard ladder `guard-100mib/256mib/512mib/over-512mib.elf` |
| `p4-release` | 3 | a project folder: `firmwaresight.toml`, `docs/RELEASE_NOTES.md`, `fixture.toml` |

**Actual toolchain, measured:** every ELF/MAP fixture carries
`name = "arm-none-eabi-gcc"` and
`linker = "GNU ld (Arm GNU Toolchain 14.3.Rel1 (Build arm-14.174)) 2.44.0.20250616"`
(`fixtures/elf/p0-basic/fixture.toml:9,11`; `p0-dual-region/fixture.toml:9,11`), built
`-g -Os -mcpu=cortex-m4 -mthumb` (`scripts/gen_p0_fixtures.py:146`). `grep -rln "clang" fixtures golden`
returns **nothing**. So the "GCC/**Clang** ELF" half of the cohort claim has **zero** committed
compiler-produced evidence, and the architecture coverage is Cortex-M/ARM only. `gen_p0_fixtures.py:1-16`
also states the provenance rule this round must keep: fixtures are real linker output (PRE-0.5/PRE-0.6),
generated with **relative** paths on purpose because GNU ld writes the object paths it was given into the
MAP, and committed so parser tests need no toolchain.

**Local toolchain reality (first-hand, decides §26 feasibility):** this host has
`clang version 22.1.8` (LLVM, default `x86_64-pc-windows-msvc`), `ld.lld`, `wasm-ld`, `llvm-objcopy` in
`/c/Program Files/LLVM/bin`, `gcc 16.2.0` / `x86_64-w64-mingw32-gcc` (MinGW-W64 ucrt) in `/c/mingw64/bin`,
and `arm-none-eabi-gcc 14.3.1` (Arm 14.3.Rel1). No MSVC toolchain directory exists, so
`link` on PATH is Git Bash's coreutils `link`, **not** a linker. Consequence for §26: a Clang-produced
ELF needs an explicit ELF target plus `ld.lld` (or GNU ld from MinGW) — achievable here without
installing anything, and it must still be generated by a committed recipe, never by hand-editing bytes.

**§26 layout cases against current coverage:**

| case | covered today? | evidence |
| --- | --- | --- |
| A classic FLASH + RAM | **partly** — same shape, named `ROM`/`RAM` | `p0-dual-region.ld:12-14` |
| B executable-in-RAM section | **no** | no fixture places `.text` in a writable region |
| C external SRAM region | **no** | only ROM/RAM in `MEMORY` |
| D DMA / non-cacheable custom region | **no** | — |
| E large discarded-section / long MAP preamble | **no committed fixture**, and this is exactly F003's class: the parser now reads a banner at any offset (`crates/firmwaresight-artifact/src/map.rs` `is_gnu_ld` scans the whole text), proved by a regression, but the repo has no long-preamble MAP to keep it proved | post-G2 F003 fix, `REMEDIATION_REPORT.md` |
| F ELF with debug info | **yes** — `-g` on every fixture; DWARF is recognized, not consumed (L5) | `gen_p0_fixtures.py:146` |
| G ELF stripped / no debug | **no** | no `objcopy -S` recipe exists |
| H multiple loadable segments | **implicitly** — `p0-dual-region` carries VMA≠LMA pairs, so ≥2 `PT_LOAD` by construction, but no test asserts the segment count | `p0-dual-region.ld:23-27`, `fixture.toml:22` |

So §26 is six-and-a-bit new fixture recipes, all inside the cohort, all producible on this machine, each
needing the six attributes §26 lists (toolchain identity, recipe, hash, expected facts, expected memory
accounting, expected basis, expected Unknowns) and a `fixtures/manifest.json` entry — and the tracked-
manifest guard `scripts/check.py:187-216` (`drift/fixtures tracked`) will refuse any pair that is
committed half, which is the failure mode that already bit this repo once (`.gitignore` eating
`fixtures/elf/p2-diff/target/`, documented at `check.py:164-186`).

**Platform support evidence:** `04_TECH/20_PLATFORM_SUPPORT.md:18-20` puts Windows 10 x64 at Tier 1
"Yes if Tauri/WebView support validated" and macOS (Apple Silicon and Intel) plus Ubuntu LTS x64 at
Tier 2 "CI/release". The validation clause is now partly satisfied by the post-G2 round on this host
(one Windows 10 19045 x64 machine, WebView2 `Edg/154.0.4258.48`, 100 % DPI only, four window sizes with
the 2560 case labelled `VIRTUAL_WINDOW_SIZE_ONLY`). There has never been a macOS or Linux **window**:
`macos-core` compiles and tests the headless crates (`p0-check.yml:145`, `scripts/check.py:140-161`),
which §27 explicitly forbids calling "supported" — `CI_BUILD_ONLY` is the honest status, and that is also
what §42 allows for macOS/Linux packaging.

## G. Known-limitation disposition (§4 G, §60 expectations)

Classifications below are the **plan** answer to §4 G (`MUST_CLOSE_P5 / SHOULD_CLOSE_P5 / CARRY_FORWARD /
OWNER_DECISION`). The §60 close vocabulary (`CLOSED / REDUCED / CARRIED_FORWARD / OWNER_DECISION /
NOT_REPRODUCED`) is the outcome and gets written at close into `P5_KNOWN_LIMITATIONS.md`. Source of
truth for the inherited list: `G2_VALIDATION/G2_KNOWN_LIMITATIONS.md:23-47` (25 rows; physical order is
L1…L22, L24, L25, L23).

| # | inherited state (G2) | §60 expectation | P5 classification | audited note |
| --- | --- | --- | --- | --- |
| L1 | Peak RSS NOT MEASURED | update from post-G2 evidence | **SHOULD_CLOSE_P5** | already **reduced in fact**: `REMEDIATION_REPORT.md:143` records ~1,428 MB working set at the 519 MB workload. It is a *working-set* figure for one workload on one host, not a peak-RSS counter, and the canonical G2 row was never rewritten — the re-basing belongs in `P5_KNOWN_LIMITATIONS.md` |
| L2 | 500 MB in-window NOT MEASURED | update from post-G2 evidence | **SHOULD_CLOSE_P5** | now measured (warm ~4.7–4.9 s, cold ~68.7 s, never "Not Responding", `REMEDIATION_REPORT.md:141-143`); "first-use <60 s" stays `PARTIAL / environment-sensitive`, and cold-first-read numbers must never be quoted as typical |
| L3 | Fuzzing NOT RUN (needs nightly) | *not listed in §60* | **CARRY_FORWARD** | blocked by policy, not by effort: ADR-0016 plus the pinned `1.98.1` toolchain (`rust-toolchain.toml`) mean `cargo-fuzz` cannot run without a toolchain-policy change. The no-panic claim keeps resting on `fixtures/malformed/*` |
| L4 | one Windows host, one WebView2, 100 % DPI | reduce with package matrix; DPI may stay carried | **MUST_CLOSE_P5 (partially)** | owner accepted a real per-user install/uninstall/reinstall on this host; that directly satisfies the `04_TECH/20:18` "validated" clause for Win10 x64 Tier 1. 125/150 % DPI and a second host stay **CARRY_FORWARD** — no safe environment here |
| L5 | two layouts, ELF only, DWARF unread | reduce via fixture expansion | **SHOULD_CLOSE_P5** | section F maps §26 A–H: 6.5 cases missing, all producible locally; also the place a Clang-produced ELF enters, which the cohort already claims |
| L6 | `sections.file_offset` reason lost | resolve or carry with reason | **MUST_CLOSE_P5** | root cause located: `db.rs:567-569` drops the reason; column lacks a paired `*_unknown` (`0001_initial.sql:65`) while 5 sibling numerics have one. Needs migration 0005 → **D2** |
| L7 | `symbols.address` reason lost | as L6 | **MUST_CLOSE_P5** | same cause (`0001_initial.sql:79`); same migration |
| L8 | Object/module attribution Unavailable by evidence | may carry, evidence-dependent | **CARRY_FORWARD** | PRD P0-3 makes it conditional on evidence; a MAP with real object lines could change this, but inventing an adapter is out of scope |
| L9 | P2 step 27 same-pair lock not mouse-verified | *not listed in §60* | **CARRY_FORWARD** | native `<select>` popup is not drivable by this harness; covered by `compare.test.tsx` + IPC tests |
| L10 | No History surface | **History should close** | **MUST_CLOSE_P5** | section C: implementable with new read APIs and (for recency) one index; no new table |
| L11 | User comprehension untested (V0 0/8) | *not listed in §60* | **OWNER_DECISION / V1** | §60 does not ask P5 to close it and P5 has no user panel; the roadmap puts V1 own-artifact/real-user validation on its own architect prompt |
| L12 | `RUSTSEC-2024-0429`, `-0370` accepted in `deny.toml` | revisit advisory status | **SHOULD_CLOSE_P5** | re-run `cargo deny` on the round's head; local green is only as fresh as this machine's crates.io index, so CI is the authority. Wording stays "dependency policy passes with documented accepted risks" — never "security clean" |
| L13 | `license = "Proprietary"`, no root `LICENSE` | OWNER_DECISION | **OWNER_DECISION** | `Cargo.toml:18`; AGENTS.md §9 and §54 both put this in front of the owner. P5 does not choose a license and does not add a `LICENSE` file |
| L14 | `update_goldens.py` key order, not re-verified | **should close** | **SHOULD_CLOSE_P5** | `scripts/update_goldens.py:250` uses `sort_keys=True`; needs one recorded dry-run-equivalent verification, goldens themselves are guarded by tests |
| L15 | `ElfProgramHeader` source label | should close if narrow | **SHOULD_CLOSE_P5** | `crates/firmwaresight-artifact/src/pipeline.rs:286` — a label fix, no semantics change |
| L16 | `custom-protocol` still required | close through official package path | **SHOULD_CLOSE_P5** | §8's requirement "a user must not need to remember `--features custom-protocol`" is satisfied only by `cargo tauri build`/CI packaging; today it is a hand-run requirement (`apps/desktop/src-tauri/Cargo.toml:14-15`). The gate's `clippy --all-features` covers compilation, not packaging |
| L17 | CI provisioning duplication, index blind spot | reduce | **SHOULD_CLOSE_P5** | the apt prerequisite block is deliberately copied into `rust` and `drift` (`p0-check.yml`, comment "Deliberately a copy rather than a shared script"); `drift/fixtures tracked` remains the only index-reading step (`scripts/check.py:164-186,207-215`). §42 takes CI from 7 to 10 jobs, so the duplication has to be resolved before a third copy appears |
| L18 | local DB retains chosen artifact location | *not listed in §60* | **CARRY_FORWARD (by design)** | adjudicated G2 addendum; `0001_initial.sql:34` `path TEXT NOT NULL` is the storage of record and the display/IPC/`redact()` chain keeps it local. P5 must add positive-control tests, not change the column |
| L19 | "Object attribution" means two things | should close | **MUST_CLOSE_P5** | wording scope: Analyze `capabilities.objectAttribution` vs Compare `diff:1 objectChanges.available` |
| L20 | Compare prints a Core enum word | should close | **MUST_CLOSE_P5** | `weakest basis MapRegionAndElfLoad` vs Analyze's `map-memory-configuration+elf-load` |
| L21 | Window title fixed at "FirmwareSight - Analyze" | should close | **MUST_CLOSE_P5** | `tauri.conf.json:15`; nothing in the UI sets a title, and it is now the cheapest way to tell a user which page they are on |
| L22 | Line endings move a release's identity | document / ADR if semantics change | **OWNER_DECISION (architect)** | with `core.autocrlf=true` a checkout rewrites LF notes to CRLF, and the Release Notes digest is inside the Gate run id, so two checkouts of one commit can disagree; it fails closed (`git.clean` BLOCK). §32 requires `P5_RELEASE_IDENTITY_ADR_DRAFT.md` and a STOP — see **D3** |
| L23 | UI test races, 4th instance found | reduce through test audit | **SHOULD_CLOSE_P5** | §35 asks 20 repetitions per UI suite; four known instances are fixed and mutation-proved, so the residual risk is the fifth one nobody has lost yet |
| L24 | `update_goldens.py` cannot rerun over its own Windows leftover scratch | should close | **SHOULD_CLOSE_P5** | `PermissionError [WinError 5]` on a read-only `.git/objects` file; tooling-only, outside the shipped product and outside the gate |
| L25 | one dead `Apply filter` click, never reproduced | close as NOT_REPRODUCED or fix | **CARRY_FORWARD → target `NOT_REPRODUCED`** | post-G2 Tier A/B drove the filter pages repeatedly with no recurrence (`autoComplete="off"` removed the leading suspect); needs one documented non-repetition count before claiming it |

Carried forward from the post-G2 round itself: the harness limit set (Tier B `B2/B4/B9 PARTIAL`, `B3` two
`NOT_RUN` targets — all harness, not product), DPI 100 % only, `HARNESS-INCIDENT-16` (a close loop that
sent `WM_CLOSE` to every visible Tauri window and caught the owner's unrelated desktop pet; process
survived, selector now filters on the owning image name). Nothing from the remediation round is silently
dropped: F001/F002/F003 are all fixed, regression-tested and re-validated, and the round's own record is
in `POST_G2_E2E_REMEDIATION/`.

**Two doc/code conflicts this audit surfaces rather than resolves.** The lifecycle document asks for a
dark theme while ADR-0018 and AGENTS.md §11 freeze "MVP Light only" — a P5 code choice would be a silent
baseline change, so it is **CARRY_FORWARD**. And §60's list omits L3, L9, L11 and L18 while requiring a
disposition for *every* inherited limitation; this table gives all four one anyway, marked as such.

## H. Release readiness

| dimension | current state | what P5 must add |
| --- | --- | --- |
| package version source | three independent `0.1.0` strings (`Cargo.toml:14`, `tauri.conf.json:4`, `ui/package.json:4`) plus a document baseline `0.6.0` (`BASELINE.yaml:7`) | one decision (**D1**) then one source of truth; `FWSIGHT_VERSION` already follows the crate, so choosing the workspace propagates to CLI, fingerprint and Snapshot |
| install path | none; `bundle.active: false` (`tauri.conf.json:29`), no CI packaging step | Windows per-user NSIS (owner accepted a real install here) + macOS/Linux as `CI_BUILD_ONLY` (§42) |
| uninstall behavior | undefined — nothing to uninstall | must be *measured*, not asserted: whether `%APPDATA%\com.firmwaresight.desktop\` survives NSIS uninstall, and then documented honestly (§53 data policy) |
| user-data behavior | `app_data_dir()/firmwaresight-p0.sqlite` (`lib.rs:919-922`), created implicitly, never shown, never backed up | keep-or-move decision (**D6**); backup per §22; explicit statement of what uninstall preserves |
| manual upgrade readiness | "install the newer build over the older one" is untested; schema upgrade is tested (`storage.rs:476`, `gate_history.rs:314`, `release_records.rs:441`) | install-replace-reinstall acceptance (§41 A–L, §53) plus the §21 migration matrix over a *real* store |
| signing readiness | **nothing** — no certificate, no key, no signer hook | `READY_NOT_EXECUTED` (§11). Never a fake certificate, never a committed private key, never a claim of "signed" |
| notarization readiness | nothing; macOS packaging itself is unbuilt | `READY_NOT_EXECUTED`, and any macOS artifact stays `CI_BUILD_ONLY` |
| update readiness | no updater, no plugin, no endpoint; `AGENTS.md` §2 pins that model | `UPDATE_READY_MANUAL` (§12) — a documented manual upgrade path only; enabling an updater requires a signing/key ADR first |
| documentation gaps | 19 end-user docs (§36) exist as *required titles*; none is written. Developer docs exist and are thorough | the full set, plus the license row stays `PENDING_OWNER_CONFIRMATION` and the security row stays "documented accepted risks" |
| artifact identity hygiene | Release Bundle ≠ Application Package (§47); three+ SHA256SUMS namespaces (§A above) | distribution checksums kept physically and nominally separate from bundle sums, with a test that says so |

## I. Decisions this audit cannot make — checkpoint items

These are the only items that blocked the first line of product code. Everything else in sections A–H is
either measurable or already decided by an authority file. **D1, D2, D4 and D6 were put to the owner on
2026-10-03 and are answered below; D3 and D5 are not the agent's to make** (D3 goes to the architect with
an ADR draft per §32, D5 is an AGENTS.md §9 owner decision that §54 forbids this round from choosing).

**D1 — Which version string wins.** §7 forbids bumping beyond `0.6.0`, and §67 wants one identity, but
the artifacts all say `0.1.0` and nothing named a package exists yet. Options: (a) set the workspace,
Tauri and UI versions to `0.6.0` so a package matches the promoted baseline (recommended: no artifact has
ever shipped, so nothing is re-versioned retroactively); (b) keep `0.1.0` and declare `0.6.0`
document-only (then every installer filename, `--version` and About panel contradicts the roadmap's
language); (c) a new number, which would be a baseline change and needs the architect.
**Owner decision 2026-10-03: (a) — unify on `0.6.0`.** The workspace version becomes the single source, so
`Cargo.toml:14`, `tauri.conf.json:4` and `apps/desktop/ui/package.json:4` all read `0.6.0`, and
`FWSIGHT_VERSION` / `fwsight --version` follow automatically through `env!("CARGO_PKG_VERSION")`
(`crates/firmwaresight-artifact/src/pipeline.rs:22`) with no second constant to keep in step. §5's
"do not automatically bump product version" is not overridden by this: `baseline_version` in
`BASELINE.yaml` stays `0.6.0`, and this change makes the *artifacts* match it rather than promoting a new
baseline. No tag, no GitHub Release.

**D2 — Migration 0005.** Required to close L6/L7 (numeric `Unknown` reasons are dropped at write,
`db.rs:567-569`), plus the `builds.created_at` index for History ordering. Additive-only, matching the
established pattern of the six paired `*_unknown` columns. Per §18 this gets
`P5_MIGRATION_DECISION.md` before any code, and the matrix in §21 runs over real v1–v4 stores.
**Owner decision 2026-10-03: write the decision document, then add migration `0005`.** L6/L7 are therefore
in scope to close, and `P5_MIGRATION_DECISION.md` is the next deliverable after governance opens — no
schema file is written before it.

**D3 — Release identity vs line endings (L22).** §32 mandates `P5_RELEASE_IDENTITY_ADR_DRAFT.md` and a
STOP for the architect if the semantics change. The audit's position: fail-closed behaviour is correct
today and the Gate did catch the CRLF case; changing it (e.g. normalizing the digest input) alters
content-derived identity, which is an ADR-level move under AGENTS.md §2. Draft, then stop.

**D4 — How far the package matrix goes without a second machine.** §8 asks for ≥1 installable Windows
artifact and ≥1 CI package each for macOS and Ubuntu; §27/§42 allow `CI_BUILD_ONLY` for the latter two,
and §41 requires real Windows install evidence (owner has accepted a per-user install on this host, with
the live store parked, hash-verified and restored). macOS packaging also needs an `.icns` that does not
exist (`icons/` has 5 files, none `.icns`), and NSIS/AppImage targets need bundle config that is
currently `active: false`. Confirm: build-and-archive on CI, never "supported".
**Owner decision 2026-10-03: Windows with real install evidence here, macOS and Ubuntu as
`CI_BUILD_ONLY`.** So P5 adds the packaging jobs, archives their artifacts and writes the compatibility
matrix with those two rows in `CI_BUILD_ONLY` — never `SUPPORTED` — and the Windows rows are backed by a
measured install → use → uninstall → reinstall cycle on this host, with the live store parked and
re-hashed. 125/150 % DPI, a second hardware host and any macOS/Linux runtime claim stay carried forward.

**D5 — License.** Remains `Proprietary` with no root `LICENSE` (`Cargo.toml:18`), and §54 forbids this
agent from choosing MIT/Apache-2.0/GPL/AGPL/MPL. `OPEN_SOURCE_LICENSE_DECISION =
PENDING_OWNER_CONFIRMATION` carries into every P5 document.

**D6 — The legacy store filename.** `firmwaresight-p0.sqlite` is where real user history already lives
on this host. Renaming it is cosmetic but touches data continuity for an already-installed build; the
audit recommends **keeping the file name** for the productized release and documenting it, rather than
adding a silent data move — but that leaves "p0" in a stranger-visible path, so it is the owner's call.
**Owner decision 2026-10-03: keep `firmwaresight-p0.sqlite`.** No rename, no data move, no compatibility
shim; instead the file name becomes *visible and explainable* — Diagnostics and the install/recovery docs
state the store path in env-var form, so a user can find and back up the file the product actually writes.

## J. What P5 can and cannot prove on this environment

Can prove here, first-hand: a real Windows per-user install → Analyze → Compare → Gate → Bundle →
History → Diagnostics → uninstall → reinstall cycle on the shipping binary; the §21 migration matrix over
v1–v4 stores created by the previous binary; crash/restart and lock cases; backup/restore round-trips;
path-leak positive controls; fixture expansion for all eight §26 layout cases using the three installed
toolchains; 20 repetitions per UI suite; 30-run fresh-process soak patterns already validated in the
post-G2 round; and remote CI authority (this environment auto-pushes, so `gh run list --branch main` is
read after every commit and prose never claims "not pushed").

Cannot prove here: any macOS or Linux *runtime* (no such machine; those rows are `CI_BUILD_ONLY`),
125/150 % DPI (this host runs 100 % and changing the host scaling is out of bounds), a second hardware
host, signed/notarized artifacts (no certificate by design), real-user comprehension (V1), and fuzzing
(L3, toolchain policy).

## K. Next step, as authorized

§4's "No code before this audit is written" is satisfied, and the owner's chosen cadence was **audit
first, then checkpoint before any product code**. The checkpoint ran on 2026-10-03: D1 unify artifacts on
`0.6.0`, D2 write `P5_MIGRATION_DECISION.md` then add migration `0005`, D4 Windows with real install
evidence plus macOS/Ubuntu `CI_BUILD_ONLY`, D6 keep `firmwaresight-p0.sqlite`. D3 (release identity vs
line endings) and D5 (license) stay where the prompt puts them — architect and owner.

In order, the writes are: §5 governance opening (`active_task: P5_PRODUCTIZATION`, stage `P5`, state
`IN_PROGRESS`, `baseline_version` unchanged at `0.6.0`, no tag, no GitHub Release) together with the
archive of this prompt under `10_AUDIT/SOURCE_PROMPTS/`, registered against the hash measured from its own
bytes — `722125f5aa68e324ba1dea4826f66d8392acab9ad9a015c4919ed5ade471e0ae`, 73,722 bytes, 3,442 lines, read
from the owner's Downloads folder, `%USERPROFILE%\Downloads\FirmwareSight_P5_Productization_v1.0.txt` —
**done, in the commit that carries this audit**; then `P5_MIGRATION_DECISION.md`; then §6's workstreams in
the §61 commit sequence (B packaging and release metadata, C onboarding and local history, D diagnostics
and recovery, E compatibility fixtures and supportability, F documentation and release readiness), each
reviewable on its own and each followed by a fetch plus a re-read of HEAD and `origin/main`.
