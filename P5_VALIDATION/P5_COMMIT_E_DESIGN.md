---
title: "P5 Commit E design record"
doc_id: "FS-P5-COMMIT-E-DESIGN"
product: "FirmwareSight"
version: "1.0"
status: "DESIGN_RECORDED_IMPLEMENTED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-04"
---

# P5 Commit E — design record (prompt §4)

Written before any production edit, as §4 requires. Every fact below is read from a command run on this
host at the start HEAD named in §0, from a file and line in the tree at that same head, or from `gh`. No
row inherits its wording from the P5 audit without being re-measured against the current tree — which is
what §3's last two lines ask for, and two rows changed on that account (§1's L14 and §1's L15).

Sections 2 onward are written as the work they describe lands. This file's status stays `IN_PROGRESS`
until Commit E's own exit list is checked, and nothing here closes P5: §5 forbids a P5 verdict, §63 lists
what must stay open, and §22 stops at the end of this round.

## 0. Start authority, and the delta that moved it

**The prompt.** *FirmwareSight — P5 Commit E, Compatibility Fixtures, Support Matrix & Supportability
Closure, Execution Prompt v1.0 — Architect Reviewed*, canonical unit `P5_COMMIT_E_COMPATIBILITY_SUPPORTABILITY`.
SHA-256 `030ca28233b964958d6aea8e59a312c66ceb4d117273cba9e667950b4ba21148`, 53,915 bytes, 2,459 LF endings,
65 numbered sections. Archived as `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitE_Compatibility_Supportability_v1.0.txt`,
`cmp`-verified byte-identical to the file the owner delivered on 2026-10-04, hash recomputed from the
archived copy, and registered in that directory's README. `10_AUDIT/SOURCE_PROMPTS/README.md` now names
Commit D's v1.2 continuation `COMPLETE, HISTORICAL` and this prompt the current active authority.

**The remote had moved, and §2 says stop and classify before writing.** §0 and §2 fix the start at
`origin/main = 956e250`, Run `37204847474`, 10 of 10. Measured at preflight, after
`git fetch --prune origin`:

```text
HEAD                 bfbc1aa7d974dc67b85e4db39eed157f6dc044a1
origin/main          bfbc1aa7d974dc67b85e4db39eed157f6dc044a1   (same)
status --short       (empty)
worktree list        D:/study/Software/FirmwareSight  bfbc1aa [main]   (one)
git diff             (empty)   git diff --cached  (empty)
```

`git log --oneline 956e250..bfbc1aa` returns two heads, `57a904e` and `bfbc1aa`.
`git diff --name-status 956e250..bfbc1aa` returns ten files — `.ai/ACTIVE_TASK.md`, `.ai/HANDOFF.md`,
`.ai/README.md`, `INDEX.md`, `README.md`, `SHA256SUMS`, `P5_VALIDATION/P5_CI_AUTHORITY.md`,
`P5_COMMIT_D_DESIGN.md`, `P5_INSTALL_RECOVERY_REPORT.md`, `P5_MIGRATION_DECISION.md` — 98 insertions and
30 deletions. A path filter for `crates/ apps/ schemas/ fixtures/ golden/ examples/ .github/ Cargo*
deny.toml assets/ scripts/` returns **nothing**. So the delta is governance prose plus the integrity
artifact that records it: the behaviour tree this round builds on is the one §0 describes, byte for byte
where it matters. Nothing was rewound, no published history was touched, and no `reset`/`clean`/`stash`/
`restore` was run.

**The start HEAD is green, which is why the round may open here.** Run `37214675036` on `bfbc1aa` — read
with `gh run view 37214675036 --json status,conclusion,jobs`, not recalled — concluded `completed` /
`success` with **10 of 10** jobs green on the first attempt, every job read individually. `bfbc1aa` is
therefore Commit E's verified start HEAD, and §0's `956e250` row stands as the history it was true for.

**The tree, measured.** `cargo test --workspace` **854** (storage 131, desktop 222), UI **210** in 8 files,
`python scripts/check.py` **16 of 16**, `--only package` **4 of 4** — the figures §0 states, matching what
the repair chain has been green on. These are the numbers to beat; §50 allows them to grow and §53 forbids
inflating them with repetitions.

**Toolchain, re-measured rather than inherited from the audit.**

| Tool | Measured on this host |
| --- | --- |
| `arm-none-eabi-gcc` | `arm-none-eabi-gcc.exe (Arm GNU Toolchain 14.3.Rel1 (Build arm-14.174)) 14.3.1 20250623` |
| `arm-none-eabi-ld` | `GNU ld (Arm GNU Toolchain 14.3.Rel1 (Build arm-14.174)) 2.44.0.20250616` |
| `arm-none-eabi-objcopy` | GNU objcopy, same 2.44.0.20250616 build |
| `clang` | `clang version 22.1.8`, default `Target: x86_64-pc-windows-msvc` |
| `ld.lld` | `LLD 22.1.8 (compatible with GNU linkers)` |
| `llvm-objcopy` | present, `compatible with GNU objcopy` |
| `rustc` / `cargo` | 1.98.1 / 1.98.1 (`rust-toolchain.toml` pinned) |
| Python / pnpm | 3.13.15 / 12.6.0 (corepack) |

Nothing was installed to produce that table, and §8's "do not install a new compiler/toolchain
automatically" holds for the whole round.

**What §0's closed rows are closed by, in the current tree.** `L6`/`L7` by migration 0005 —
`crates/firmwaresight-storage/migrations/0005_unknown_reasons.sql:29,31` adds `sections.file_offset_unknown`
and `symbols.address_unknown`, and `crates/firmwaresight-storage/src/db.rs:14` carries
`SCHEMA_VERSION: i64 = 5`. `L10` by the History page, `apps/desktop/ui/src/History.tsx` with
`history.test.tsx`. `L21` by the Rust-side title composed over the closed `MainWindowPage` enum,
`apps/desktop/src-tauri/src/ipc.rs:1260`. `L22` by `ADR-0028`, `status: BASELINE`. None of the four is
reopened here: §4 forbids reopening a closed row without evidence, and this round found none against them.

## 1. Rebase audit — the inherited rows against `bfbc1aa`

§4's list, one block per row, with the five fields it names. The starting dispositions §4 predicts are
confirmed or corrected below; the one that changed materially is **L15**, whose premise turned out to be
true of the *serialized contract* rather than of a label.

### L4 — platform and runtime evidence

- **CURRENT FACT.** Real runtime evidence exists on exactly one machine: this Windows 10 (19045) x64 host,
  WebView2 `Edg/154.0.4258.48`, 100 % DPI. macOS and Ubuntu have a *package* built by CI (jobs 8–10) and no
  window has ever run there — `macos-core` runs `check.py --only core-smoke`, which exercises headless
  crates only. `04_TECH/20_PLATFORM_SUPPORT.md:17-24` still puts Windows 10 x64 at "Yes if Tauri/WebView
  support validated" and macOS/Ubuntu LTS at Tier 2 CI/release.
- **STATUS AT COMMIT E START.** Partly reduced by P5: an installer was built on three runners and installed,
  run, uninstalled and reinstalled here (`P5_INSTALL_RECOVERY_REPORT.md`). `04_TECH/20` itself is a
  `version: 0.5.1` document and its Tier-1 clause is still written as a condition, not a result.
- **TARGET THIS ROUND.** Matrix rows only: state each platform with its evidence kind (`RUNTIME_MEASURED` /
  `CI_BUILD_ONLY` / `NOT_TESTED`), name the OS actually measured without inferring Windows 11, and keep the
  ARM64 rows as `04_TECH/20` states them. §39's vocabulary is the whole deliverable.
- **FILES / TESTS.** `P5_VALIDATION/P5_COMPATIBILITY_MATRIX.md` (new). No product file.
- **STOP.** No §64 journey (§1A): a packaged check is allowed only as
  `FOCUSED_COMMIT_E_PACKAGE_CHECK`, and 125/150 % DPI and a second host stay carried — there is no safe
  environment for them here, and inventing one is worse than the gap.

### L5 — layout, architecture and DWARF coverage

- **CURRENT FACT.** Four named ELF fixture sets exist (`fixtures/manifest.json`, 25 entries), all produced by
  `arm-none-eabi-gcc` + GNU ld, all Cortex-M. `grep -rln "clang" fixtures golden` still returns nothing, so
  the Clang half of the cohort claim has zero committed compiler-produced evidence. Measured against §13's
  eight cases: A partly (`p0-dual-region`, named `ROM`/`RAM`), B no, C no, D no, E no committed fixture,
  F yes (`-g` on every fixture), G no, H *already true in bytes* — `arm-none-eabi-readelf -l` counts
  **PT_LOAD = 2** on `p0-basic` and **3** on `p0-dual-region`, and no test asserts either number today.
- **STATUS AT COMMIT E START.** Open, unchanged since the audit wrote §F.
- **TARGET THIS ROUND.** Substantially reduce: real Clang-produced Arm evidence, new layouts for B/C/D, a
  genuine no-debug path for G, direct `PT_LOAD` assertions for H, and a truthful E disposition. §27 names the
  possible end state as `REDUCED`, and explicitly forbids closing it because fixtures multiplied — DWARF stays
  recognized-and-not-consumed, and ELF stays the intentional scope.
- **FILES / TESTS.** `scripts/gen_p5_compat_fixtures.py` (new), fixture dirs under `fixtures/elf/`,
  `fixtures/manifest.json`, and acceptance tests beside `crates/firmwaresight-artifact/tests/p0_acceptance.rs`.
- **STOP.** §6 bans Keil/IAR/TI COFF/HEX/BIN/UF2/Mach-O/PE and a DWARF semantic feature; §9 bans
  hand-edited or relabelled bytes; §27 forbids adding DWARF parsing to close a row.

### L8 — object / module attribution

- **CURRENT FACT.** The MAP adapter already parses per-object lines:
  `crates/firmwaresight-artifact/src/map.rs:38-45` declares `MapObjectContribution { input_section, address,
  size, object, line }`, and the only place anything reads them today is a test
  (`crates/firmwaresight-artifact/tests/p0_acceptance.rs:141-147`, which asserts an object ending in
  `main.o`). The domain has nowhere to put the fact: `crates/firmwaresight-core/src/domain/section.rs:89-107`
  has no object or module field, and `crates/firmwaresight-core/src/domain/diff.rs:498-506` returns
  `available: false` from a `from_inputs(_base, _target)` that ignores both arguments, with a reason that is
  true of the persisted model.
- **STATUS AT COMMIT E START.** Open, and now precisely bounded: the missing piece is not parsing, it is a
  place to carry the fact.
- **TARGET THIS ROUND.** Add a fixture and a test that prove a *Clang-compiled, GNU-ld-linked* MAP carries
  object-file contributions the adapter reads, so the record says what the evidence would support. Then
  carry the product claim forward: `L8 = CARRIED_FORWARD`, with the reason named as a portable-schema
  question rather than an absence of data.
- **FILES / TESTS.** `P5_COMPATIBILITY_FIXTURE_REPORT.md`, the new fixture's acceptance test.
- **STOP.** §28 stops and returns to the Architect if closing L8 needs a new MAP grammar or parser subsystem;
  this round established it needs neither. §42 stops it as written if a new portable field is the only path,
  and that is the case here — see §1's L15 note on the same collision. No object ownership inferred from
  symbol names, which §28 forbids and `section.rs` cannot represent anyway.

### L12 — dependency advisories

- **CURRENT FACT.** `deny.toml:48,56` ignore `RUSTSEC-2024-0429` (glib 0.18.5, unsound) and
  `RUSTSEC-2024-0370` (proc-macro-error 1.0.4, unmaintained), both informational-class per the keys written
  out at `deny.toml:39-41`. They are reachable only through the gtk `0.18` line that Tauri 2.12.0 requires.
- **STATUS AT COMMIT E START.** Accepted with reasons; `Dependency policy` green on the start HEAD's run.
- **TARGET THIS ROUND.** Re-measure: `cargo deny check licenses bans sources advisories` locally
  (cargo-deny 0.20.2 is installed here), name the current path, version, platform exposure and whether a
  compatible upgrade exists, and record `REVISITED / CARRIED_FORWARD` unless an upgrade inside the existing
  framework family keeps every package and runtime test green.
- **FILES / TESTS.** `deny.toml` only if a version moves; `P5_SUPPORTABILITY_REPORT.md` for the row.
- **STOP.** §29 forbids a Tauri/gtk replacement or a major framework jump taken to silence an advisory —
  that is an `AGENTS.md` §2 baseline change, not a dependency bump. And a local `deny` green is only as fresh
  as this machine's crates.io index: CI's `Dependency policy` job is the authority on yank and advisory state.
  The permitted sentence stays "dependency policy passes with documented accepted risks".

### L14 — `update_goldens.py` key order

- **CURRENT FACT.** This row is **stale as written**, and §30 asked to verify before touching, so it was
  verified. `scripts/update_goldens.py:264-269`'s `pretty()` ends with
  `json.dumps(document, indent=2, ensure_ascii=False, sort_keys=True)`, and the comment above it says the
  sorted order is what the committed P0 CLI goldens already use. `matches()` compares `.json` goldens by
  parsed content, so key order alone cannot mark one changed. The G2 row
  (`G2_VALIDATION/G2_KNOWN_LIMITATIONS.md:36`) cites `update_goldens.py:250`; the code is at 269.
- **STATUS AT COMMIT E START.** Nothing observed to fix; the row's own text already concedes it was "not
  re-verified by a dry run".
- **TARGET THIS ROUND.** §30/§54's method, not a code change: run the no-confirm dry run twice in succession
  on this host, then prove `git status --short`, `git diff -- golden` and `git diff -- fixtures` are clean.
  If that holds, `L14 = CLOSED BY REVALIDATION` with the command written down, and the script is not edited.
- **FILES / TESTS.** `P5_SUPPORTABILITY_REPORT.md`.
- **STOP.** §30 forbids reaching for `--confirm` to make a run look green, and §45 forbids regenerating
  goldens as a routine step.

### L15 — the `ElfProgramHeader` evidence source label

- **CURRENT FACT.** The G2 row's suspicion is correct, and it is bigger than a label. For
  `MemoryEvidenceBasis::ElfAddressAndFlags`, `crates/firmwaresight-artifact/src/pipeline.rs:284-289` reports
  `SourceType::ElfProgramHeader` with the rule text `"{section_locator} + elf:sh_flags"` — while the ELF
  reader in this crate never opens a program header: `crates/firmwaresight-artifact/src/elf.rs:239` sets
  `load_address: Fact::unknown("not present in the section header table")`, and a grep for
  `p_flags` / `program_headers` / `p_paddr` across `crates/firmwaresight-artifact/src/` returns no read at
  all. Section addresses come from `section.address()` (`elf.rs:232-235`), i.e. the section header table.
- **STATUS AT COMMIT E START.** Open, and **not** a wording-only item. The wire string
  `"elf.program-header"` is a literal in the public contract at `schemas/analysis.schema.json:616`, is
  emitted by `crates/firmwaresight-report/src/dto.rs:139`, and is carried by four evidence rows in
  `golden/cli/p0-basic-analyze.json`. No test asserts the literal, which means the only thing guarding it is
  the schema and the golden.
- **TARGET THIS ROUND.** Establish the fact in evidence and stop the subproblem. The fixture work makes it
  repeatable — the no-MAP path that selects `ElfAddressAndFlags` gets new fixtures driving it — and
  `P5_COMMIT_E_SCHEMA_DECISION.md` records the problem, the fixture evidence, the old contract, the proposed
  change, and the compatibility, Release Bundle and golden impact for the Architect to decide.
  `L15 = CARRIED_FORWARD (schema decision requested)`.
- **FILES / TESTS.** The decision document, plus the fixture assertions that pin current behaviour.
- **STOP.** §32 is explicit: if changing `SourceType` moves the analysis portable JSON, the Release Bundle
  analysis, the goldens, release identity or schema semantics, stop for Architect review — and it does. §42
  repeats the rule for any new portable field or renamed serialized enum. This round therefore does **not**
  patch a UI string over the top of it: a clearer label beside a `sourceType` that still says
  `program-header` would create a display that no longer matches the bytes a user can export and re-check,
  which is the opposite of §32's "preferred narrow closure". The G2 row also claimed the Release Bundle
  carries it; it does not — `golden/reports/p4-release/analysis.json` uses only `elf.file-header`, `map` and
  `filesystem` — and that correction belongs in the decision document, not in the G2 file.

### L16 — `custom-protocol` packaging

- **CURRENT FACT.** `apps/desktop/src-tauri/Cargo.toml:15` still declares
  `custom-protocol = ["tauri/custom-protocol"]`, and the requirement is satisfied in practice because no user
  path goes through a bare `cargo build`: the official package group runs the Tauri package path, and CI's
  three package jobs built and attached installers on all three platforms from `1055242` forward.
- **STATUS AT COMMIT E START.** Likely closed by the package path, which is exactly what §38 says to verify
  rather than assume.
- **TARGET THIS ROUND.** Verify, do not rewrite: run `python scripts/check.py --only package` and show the
  build frontend → Tauri package path → automatic `custom-protocol` chain requires no manual feature flag.
  If green, `L16 = CLOSED` with the command beside it.
- **FILES / TESTS.** `P5_SUPPORTABILITY_REPORT.md`; `scripts/check.py`'s package group as it stands.
- **STOP.** §38: no package architecture rewrite.

### L17 — CI provisioning duplication and the index blind spot

- **CURRENT FACT.** The provisioning half is already resolved — the apt block was reduced when packaging
  landed, and §35 says do not redo it. The remaining half is precisely characterizable in code:
  `scripts/check.py:185-205`'s `untracked_fixture_paths` reads `fixtures/manifest.json` and compares it
  against `git ls-files`, in **one direction only**. A file the working tree has and the manifest does not
  name is invisible to the gate. The function's own docstring records why it exists at all — `**/target/`
  ate half of the P2 fixture pair and every local run was green.
- **STATUS AT COMMIT E START.** Reduced; the blind spot is now a named, bounded direction-of-proof gap.
- **TARGET THIS ROUND.** Add only what is falsifiable and already has a canonical list: this round's new
  fixture sets are the test for the existing rule, and the residual risk (an ignored source file no manifest
  names) is stated in words, not papered over. `L17 = REDUCED` with the residual named.
- **FILES / TESTS.** `P5_SUPPORTABILITY_REPORT.md`; the manifest entries for the new fixtures.
- **STOP.** §35 forbids a rule that makes every untracked file an error, and forbids building a general
  source-control linter.

### L19 — "Object attribution" means two things

- **CURRENT FACT.** Two pages print the same two words for different scopes:
  `apps/desktop/ui/src/Analyze.tsx:372` renders `<CapabilityRow term="Object attribution"
  value={capabilities.objectAttribution} />` for one build's capability, and
  `apps/desktop/ui/src/Compare.tsx:738` renders the same term for the object-level delta between two builds,
  driven by `objectChanges.available` at `:741-743`. No UI test asserts the phrase today.
- **STATUS AT COMMIT E START.** Open. The fact behind each label is correct; the label is ambiguous.
- **TARGET THIS ROUND.** §33's distinct labels on the two UI strings, and UI tests that guard the
  distinction so a later edit cannot collapse them back. `L19 = CLOSED` if the tests land.
- **FILES / TESTS.** `Analyze.tsx:372`, `Compare.tsx:738`, `apps/desktop/ui/src/compare.test.tsx`.
- **STOP.** No Core capability semantics change: `capabilities.objectAttribution`, `diff:1`
  `objectChanges`, `crates/firmwaresight-core/src/domain/capability.rs:112` and the report renderers
  (`render.rs:130`, `release_render.rs:864,876`, whose text `release_schema_contract.rs:1183` asserts) stay
  as they are. Those are golden- and contract-guarded, and §33 asks for product-facing wording only.

### L20 — Compare prints a Core enum word

- **CURRENT FACT.** `apps/desktop/ui/src/Compare.tsx:695-697` renders
  `weakest basis {side.weakestEvidenceBasis ?? 'unknown'}`, and that field is a Rust `Debug` name:
  `crates/firmwaresight-core/src/domain/diff.rs:405` writes `format!("{basis:?}")`. The full variant set is
  five, `crates/firmwaresight-core/src/domain/memory.rs:165-176`: `RegionConfigAndElfLoad`,
  `MapRegionAndElfLoad`, `ElfAddressAndFlags`, `SectionNameHeuristic`, `Insufficient`. Analyze already speaks
  in kebab labels for the same ladder (`report/src/dto.rs:160-168`). The Debug words are load-bearing in
  bytes elsewhere: three Rust tests assert `"MapRegionAndElfLoad"`
  (`apps/cli/tests/p2_golden.rs:219-222`, `src-tauri/tests/compare_ipc.rs:242`,
  `storage/tests/compare_candidates.rs:445`) and `golden/core/p2-diff.json` and
  `golden/reports/p2-diff.html` carry them.
- **STATUS AT COMMIT E START.** Open.
- **TARGET THIS ROUND.** A closed display mapping over all five variants in the UI, aligned with Analyze's
  vocabulary, plus a test per variant and a test for an unknown future value that must not crash and must
  fall back to `Unrecognized evidence basis` while keeping UNKNOWN semantics. `L20 = CLOSED` for the window
  if that lands; the diff **HTML** report keeps printing the serialized word, and that stays a named
  residual in `P5_SUPPORTABILITY_REPORT.md`, because `diff_render.rs:282-291` feeds a golden.
- **FILES / TESTS.** `Compare.tsx:695-697` and its test file.
- **STOP.** §34 forbids renaming Core enum values, and §45 forbids regenerating goldens to follow a wording
  change. The serialized `weakest_basis` field is not touched.

### L23 — the UI async race family

- **CURRENT FACT.** Six instances found and fixed test-only, the sixth caught by the *local* full gate and
  not by CI; the seventh was the storage-side clock race that reddened `3400981` and closed at `bccea88`.
  The honest residue the audit names is that absence assertions still settle by timing, and that a wave with
  a different shape from the ones already closed may exist. `apps/desktop/ui` has 8 test files and 210 tests.
- **STATUS AT COMMIT E START.** Reduced, not closed.
- **TARGET THIS ROUND.** §36's bounded sweep on the current tree: static review for the five shapes it names,
  then 20 complete UI-suite runs in fresh processes, recording suite size, repetitions, failures, wall time
  and environment. Any failure is classified test-side or product-side and repaired causally.
  `L23 = REDUCED` after the sweep.
- **FILES / TESTS.** `P5_SUPPORTABILITY_REPORT.md`; a test file only if the sweep finds a race.
- **STOP.** §36 forbids sleeps, retry wrappers and timeout inflation — the same three fixes this repository
  has already rejected. And §53 forbids reporting 20 × 210 as a test count.

### L24 — Windows scratch cleanup in the golden updater

- **CURRENT FACT.** The fix has already landed: `scripts/update_goldens.py:162-170` defines `_force_removal`,
  which `os.chmod(path, stat.S_IWRITE)` and retries, its docstring naming L24 and the read-only
  `.git/objects` files under its own scratch; `:173-177` dispatches `onexc=` on 3.12+ and `onerror=` below.
  The p4 scratch at `target/update_goldens-p4` is deliberately never removed at the end of a run, so every
  run re-enters that path — which is what makes §31's "twice" the right test.
- **STATUS AT COMMIT E START.** Code present, outcome unmeasured on this head.
- **TARGET THIS ROUND.** Exercise it twice and read the result: no `PermissionError [WinError 5]`, owned
  scratch cleaned, tracked tree unchanged. `L24 = CLOSED / NOT_REPRODUCED ON CURRENT HEAD` if so.
- **FILES / TESTS.** `P5_SUPPORTABILITY_REPORT.md`.
- **STOP.** §31: never delete the real `.git`, an unrelated `target/` folder, or user data. The scratch is
  under `target/` and owned by the script; that boundary is the whole safety argument and it is not widened.

### L25 — one dead `Apply filter` click

- **CURRENT FACT.** `apps/desktop/ui/src/Compare.tsx:1575-1577` is a `<button type="submit">` inside the
  `<form>` at `:1538-1544` whose `onSubmit` prevents default and calls `onApply(draft)`; the text input at
  `:1566-1573` carries `autoComplete="off"`, and the button is never disabled. Applying a section filter
  reaches `setFilter` at `:1056` and a symbol filter at `:1297`, both refetching through the effect keyed on
  `[diffId, filter, kind, sort, direction, offset]`. That yields a concrete, testable explanation for the
  historical observation: **an Apply whose draft value did not change emits no IPC call at all**, so it looks
  exactly like a dead click while behaving correctly.
- **STATUS AT COMMIT E START.** Open as `UNRESOLVED, never reproduced`; the post-G2 283-case round never saw
  it recur.
- **TARGET THIS ROUND.** §37/§55: at least 30 legitimate Apply actions across both filters with the value
  changing each time, plus keyboard activation, recording N attempts / N successes per filter and the
  environment. If nothing fails, `L25 = NOT_REPRODUCED` with the count. If something fails, capture state
  and fix narrowly.
- **FILES / TESTS.** `apps/desktop/ui/src/compare.test.tsx`, whose house idiom is `fireEvent` plus
  `waitFor` over a mocked bridge (no `user-event` dependency anywhere in the repo, and adding one would be a
  new npm dependency under §46).
- **STOP.** One unit-test click does not close this (§37). A packaged check is allowed only as
  `FOCUSED_COMMIT_E_PACKAGE_CHECK` under §47's store protocol, and never called the §64 journey.

### L26 — root `SHA256SUMS` is host-relative

- **CURRENT FACT.** Unchanged and measured: `scripts/generate_baseline_artifacts.py` hashes working-copy
  bytes, `core.autocrlf=true` with `.gitattributes` `text eol=lf` leaves 17 tracked files differing from a
  clean checkout, `verify_baseline_artifacts.py` prints `RESULT PASS` on the host that wrote it, and no
  `check.py` step and no workflow job runs it.
- **STATUS AT COMMIT E START.** Open, deliberately.
- **TARGET THIS ROUND.** Nothing. §1B forbids the fix in every form — not blob bytes, not line-ending
  normalization, not wiring the verifier into CI as if the semantics were decided, not an `ADR-0028` edit —
  and §57 says to keep generating under the current convention so the gate stays internally consistent while
  explicitly not claiming cross-checkout reproducibility.
- **FILES / TESTS.** `P5_SUPPORTABILITY_REPORT.md` records `CARRIED_FORWARD TO COMMIT F`.
- **STOP.** This row is the clearest trap in the round: a "drive-by fix" here would change how a baseline
  artifact is produced, which is a governance decision this prompt reserved elsewhere on purpose.

---

## 2. Fixture design (prompt §9, §10, §13)

Written as the work landed, which is what §4's "sections 2 onward" asks for.

**One generator, standard library only.** `scripts/gen_p5_compat_fixtures.py` is the whole recipe: no new
Python dependency (§46 speaks of crates and npm packages, but a generator needing `pyelftools` would still
be a new dependency for the repository), and `--force` is the only way to overwrite an existing fixture —
its `guard()` refuses to regenerate silently, because a re-recorded hash without a recorded reason is how
goldens rot.

**Four linker scripts, one source per case.** `ram-exec.ld` (FLASH plus writable RAM, with `.fast_text
> RAM AT> FLASH`), `extsram.ld` (adds `EXTSRAM (xrw) : ORIGIN = 0x60000000, LENGTH = 512K`),
`dma-region.ld` (adds `DMARAM (rw) : ORIGIN = 0x30000000, LENGTH = 16K`) and `plain.ld` (the two-region
baseline the compiler and debug cases share). Each case got its own `source/main.c` deliberately: three
fixtures once shared one source, and the extra sections fell to GNU ld's orphan-placement rules, so a
reader could not tell a script decision from a linker guess. After the split, every allocated section in
every committed ELF is one the fixture's own script names.

**Why `arm-none-eabi-ld` for the final link rather than the `gcc` driver.** The driver pulls in `crt0` and
start-up code from its default libraries, which would (a) add sections none of these scripts place and (b)
make the `PT_LOAD` count a property of toolchain defaults instead of the script under test. The compile step
is still `arm-none-eabi-gcc` or `clang`, so the compiler-identity claim stays about the code, and the link
plus the MAP stay about the layout. `-e main` is the price of a `crt0`-free link, and it is why every
fixture's entry is `0x08000001` — a thumb-mapped address, which the identity test then reads back out of the
header rather than trusting this sentence.

**The Clang probe (§8, §22).** This host's `clang 22.1.8` defaults to `x86_64-pc-windows-msvc`, so a bare
`clang` invocation produces a PE and would have proved nothing about the cohort. The path that works without
installing anything is `clang --target=arm-none-eabi -ffreestanding -fno-builtin -mcpu=cortex-m4 -mthumb`
for the compile, then the Arm toolchain's `arm-none-eabi-ld` for the link and the MAP. That is §22's outcome
**A** — a real Arm Clang fixture, committed and green — so the wording is
`Clang Arm ELF = SUPPORTED for the tested target/layout cohort`, and the layout stays `plain.ld` on purpose:
the compiler is the only variable between `p5-clang-arm` and the GCC fixtures sharing its script.

**Three guards that are the provenance rule (§9).**

1. *Compiler identity comes from the artifact.* `comment_of()` reads `.comment` out of the built ELF, and the
   generator refuses to write a clang record whose own bytes do not name clang ("refusing to commit a fixture
   whose compiler identity is a claim rather than an observation"). Mutation **B** below is the proof this
   guard has teeth.
2. *No private machine path.* `assert_no_host_paths()` scans every textual output against `^[A-Za-z]:[\\/]`,
   `/Users/`, `/home/` and `/tmp/` before anything is recorded, and the build runs inside the fixture
   directory with relative arguments. It fired during development on an earlier draft of this generator,
   which is why the recipe passes `source/main.c`: GNU ld writes the object paths it was given straight into
   the MAP.
3. *A fixture must still be the thing it claims.* The long-preamble case re-measures its own banner offset on
   every run and exits if it ever falls under the 4096-byte floor — "generating more dead code is the fix,
   weakening an assertion is not". Its dead code lives in `.text.*` and is removed by `--gc-sections`, so the
   preamble is linker output. And the recorded link command now carries `--gc-sections` because an earlier
   draft recorded a flag the call did not pass: a provenance command that differs from the command actually
   run is a false record, which is the one failure mode this round cannot afford.

**What was deliberately not committed.** `with-debug.elf`, the intermediate the strip recipe produces, is
removed once `objcopy -S` has run, so the record cannot imply a third claim about a third file. `build/` is
deleted after each link. `fixtures/manifest.json` stays the only hash registry (the p0 convention §11 says to
reuse where possible): a `fixture.toml` names files and derivations, the manifest carries digests, and
`drift/fixtures tracked` is what pairs them.

## 3. The defect the evidence found, and the fix that was authorized

`p5-clang-arm` was linked, the generator derived its expectation from `readelf` plus the MAP's region table
(**58** image bytes), and the product answered **50**. The missing 8 bytes were `.ARM.exidx.text.main`:
`SHF_ALLOC|SHF_LINK_ORDER`, `SHT_ARM_EXIDX` (`0x70000001`), inside the declared FLASH region — bytes a real
linker put in the load image.

`crates/firmwaresight-artifact/src/elf.rs` decided allocation by elimination:

```rust
let alloc = named_role.is_none()
    && !matches!(kind, SectionKind::Unknown | SectionKind::Debug | SectionKind::DebugString
                    | SectionKind::Note | SectionKind::Metadata | SectionKind::OtherString);
```

For an unrecognized kind that reads "the parser does not know what this is" as "this is not loaded", and the
section then entered **neither** budget while `nonvolatile` still reported `Exact` with `unattributed` empty.
That is the shape `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md:42` forbids — a custom section is settled by segment
and region evidence, never by guessing from its name — and it is worse than a wrong number, because the model
presented an exact total built on an incomplete sum.

Three ways to carry it were on the table: document it as a limitation, make it visible by filling
`unattributed`, or read the bit. The owner chose the narrow fix in this round, and the choice stands on
evidence rather than preference: `SHF_ALLOC` is in the section header the parser already holds, so the fix
*removes* an inference instead of adding one. `map_section_kind` now takes `is_alloc` from that bit; `write`
and `execute` keep their existing derivation (the finding was about allocation, and moving those would
reclassify things no evidence here asked to move); and the metadata-name net still overrides the bit, which is
what this function was originally written to protect.

The invariant test is the part worth reading twice. Its first version summed the model's own `flags.alloc`,
so undoing the fix left it green — an assertion that agrees with the bug it exists to catch. It was rewritten
to parse `SHF_ALLOC` and `SHT_NOBITS` out of the section headers directly (`raw_allocated_payload`; ELF32
header offsets `e_shoff` at `0x20`, `e_shentsize` at `0x2E`, `e_shnum` at `0x30` — an earlier draft read
`e_shoff` from `0x28` and failed with a range panic, which is how the offsets were confirmed) and to run over
**every** manifest ELF rather than only the one that caught the bug. Alongside it,
`the_provenance_record_and_the_product_answer_the_same_numbers` reads each record's two figures back out of
the TOML and holds the analyzer to them, so neither the record nor the product can drift on its own.

Scope held: one function and one call site in one crate. No schema, no stored shape, no Gate semantics, no
capability change; no golden moved; and the other ten committed ELFs already balanced — measured by walking
each one's section table for an allocated, non-`SHT_NOBITS` section whose `sh_type` is outside
`SHT_PROGBITS`, which returns exactly one hit in exactly one file.

## 4. Mutation proofs (§44)

Each mutation was applied, its suite run, then the file restored and verified by digest.

| id | mutation | observed failure |
| --- | --- | --- |
| **A** | `elf.rs`: revert to the kind-elimination rule | `an_allocated_section_the_parser_does_not_recognise_is_still_charged` and `the_reported_image_footprint_equals_the_bytes_the_file_claims_are_loaded`, with `left: 50, right: 58` |
| **B** | hand the clang fixture a copy of the GCC binary, keeping the clang record | `the_clang_fixture_names_clang_in_its_own_bytes_and_the_gcc_fixtures_name_gcc` |
| **C** | perturb one recorded total, 184 to 185 | `an_external_sram_region_is_observed_from_the_map_and_charged_to_runtime_ram` |
| **E** | read `SHF_WRITE` where the fix reads `SHF_ALLOC` | 8 tests across the workspace, both budgets for several fixtures |
| **F** | `apps/desktop/ui/src/History.tsx:94`, `if (request.current !== id)` to `if (false)` | `paints only the newest answer when an older reply arrives last`: `expected <td title="stale.elf"></td> to be null` |

**F** exists because the reliability sweep changed a test's settle primitive rather than its assertion: the
proof is that the edited test still catches the regression it describes, which is what makes the new flush
known to be sufficient and not merely quieter. `History.tsx` was restored to digest
`29022971f963082d618ae9178771eaee5b0279dccdd10e8f229b3fa266e5ecea`, `git diff` empty.

## 5. Supportability probes, and what each one decided

| row | probe | result |
| --- | --- | --- |
| L8 | `grep -rn "object_contributions" crates/ apps/`, plus `pipeline.rs:228-247` | parsed by the adapter, consumed by two tests and nothing else; the capability is a MAP-presence proxy, so there is no narrow current path to expose it → **CARRIED_FORWARD**, nothing synthesized |
| L12 | `cargo deny check licenses bans sources advisories`; `cargo update -p glib --precise 0.20.0 --dry-run` | policy green with both accepts still matching; the upgrade fails on `glib = "^0.18"` required by `gtk v0.18.2` of `tauri v2.12.0` → **REVISITED / CARRIED_FORWARD** |
| L15 | read every surface that serializes a `SourceType` | three spellings across schema, store and bundle, plus a verbatim UI passthrough → **STOP**, `P5_COMMIT_E_SCHEMA_DECISION.md` |
| L14 / L24 | `update_goldens.py` in no-confirm mode twice, over the scratch its own first run left | 14 lines `unchanged` and `all goldens match current output.` both times, no `PermissionError [WinError 5]`, `Cargo.lock` and goldens untouched → **CLOSED BY REVALIDATION** and **CLOSED / NOT_REPRODUCED ON CURRENT HEAD** |
| L16 | `python scripts/check.py --only package` | 4/4 PASS, `tauri-cli 2.12.1`, command `cargo tauri build -- --locked`, verifier reports `frontend embedded` and installer version `0.6.0` → **CLOSED** |
| L17 | `drift/fixtures tracked` over 56 manifest paths; the `tauri.conf.json` bundle list; the `fixtures/generated/` policy | all manifest paths tracked; `resources` empty and 5 icons present and tracked; the workload ladder is untracked by design and its consumer prints `NOT RUN (workload missing; run scripts/gen_p0_workload.py)` → **REDUCED**, narrow residual |
| L19 / L20 | `compare.test.tsx:752,763,773,783,791` and `details.test.tsx:344` | both wording rows carry tests naming both scopes and every current basis variant → **CLOSED** |
| L23 | the sweep in §6, then the 20-run campaign | 89 absence assertions filtered to 3 candidates, all read and classified; one timer flush removed with proof **F** → **REDUCED** |
| L25 | `compare.test.tsx:929` | 30 click Activations plus 6 submit Activations, zero failures; the keyboard path measured as not drivable in jsdom → **NOT_REPRODUCED** |
| L4 / L5 / L26 | matrix rows, fixture expansion, §1's boundary | **REDUCED**, **REDUCED**, **CARRIED_FORWARD TO COMMIT F** |

## 6. The reliability sweep, and the campaign on the final tree

§36A over the seven UI test files, in two passes, one per risky shape.

**Chained and second-wave interactions.** Filter: tests with two or more interactions where **nothing**
awaits after the second one — the shape that would let a synchronous assertion read state the later wave has
not yet set. Two survived out of the suite, and both are local-state tests rather than wave tests:

- `release.test.tsx:990` `refuses to submit without a name and a reason` — the two `fireEvent.change` calls
  drive `submit.disabled`, which is derived from form state; the one IPC call in the test is asserted absent
  at `:998`, after `findByRole` at `:993-996` has already settled the wave that could have made it;
- `release.test.tsx:1497` `names every field the page offers` — `Edit policy` at `:1502` only toggles the
  editor (`Release.tsx:367` flips `draft`; the form renders config already awaited at `:1501`), so the DOM
  inventory that follows reads a committed render, not a pending response.

**Absence assertions before the effect settles.** Filter: enumerate assertions of the form *this must not be
present / must not have been called* (89 across the suite), then keep only those with **no** await,
`waitFor`, `findBy` or `act` between the interaction that could start work and the assertion. Three survived,
and each was read:

- `details.test.tsx:511` — a unit radio reformats rows already awaited; `fireEvent` commits the re-render
  before the next statement, so the absence races nothing;
- `history.test.tsx:429` — the same shape, and the test additionally pins `buildsMock`, `runsMock` and
  `releasesMock` at one call each, so a spurious second wave would fail it for another reason;
- `release.test.tsx:1329` — the button is `disabled`, so "the write never starts" is the contract and is
  synchronous by construction.

Neither pass found a product race or a test-only race. One real finding came out of the same reading: `history.test.tsx` settled its stale-reply assertion with
`await new Promise((done) => { setTimeout(done, 0); })`. That is a macrotask sleep standing exactly where §36
forbids one, so it became `await act(async () => { await Promise.resolve(); })`, and mutation **F** is the
proof the test still catches the regression. No sleeps, no retry wrappers and no timeout inflation remain:
`grep -rn "setTimeout" apps/desktop/ui/src/*.test.ts*` returns nothing, and no test or config raises
`testTimeout` or a `waitFor` timeout.

Campaign on the final UI tree — `corepack pnpm test` in `apps/desktop/ui`, a fresh process per repetition,
log `target/p5reliability/ui-20-runs-2.log`. The log is local and untracked (`target/` is ignored), so the
record is the numbers, and the method is stated with them rather than implied:

```text
command:   corepack pnpm test in apps/desktop/ui (vitest run, fresh process per repetition)
host:      MINGW64_NT-10.0-19045  node=v24.19.0
started:   2026-10-04T18:07Z      ended: 2026-10-04T18:15:25Z
per run:   Test Files 8 passed (8)   Tests 217 passed (217)   exit=0     (x 20)
repetitions: 20    failing runs: 0    wall time per run: 17 s to 34 s
```

Direct count is **217 tests in 8 files**; 20 repetitions, **0 failing runs**, wall 17–34 s; repetitions are
not multiplied into a canonical total (§53). The wall range is wider than the earlier campaign's 15–20 s
because other commands ran alongside it on this host, and the number recorded is the one the log holds
rather than the one that would read better.

An earlier 20-run campaign on a near-identical tree was also green (`ui-20-runs.log`, `failing_runs=0`), but
its count extractor was defeated by ANSI escapes, so the campaign was re-run rather than restated — the first
attempt is recorded here as a tooling failure, not as evidence. The extraction fix (strip ANSI, then match
`^ *(Test Files|Tests) `) is what produced the per-run counts above.

## 7. Security, dependency and capability review (§46)

What the round's diff contains, by surface:

| surface | change |
| --- | --- |
| Rust dependencies | none. `Cargo.toml` and `Cargo.lock` unchanged; the `--dry-run` probe confirmed the lock was not written |
| npm dependencies | none. `apps/desktop/ui/package.json` and `pnpm-lock.yaml` untouched; no `user-event`, no `jest-dom` |
| IPC surface | no command added, removed or widened; the L19/L20 work is presentational text over existing payloads |
| Capabilities, CSP, Tauri permissions | untouched — `capabilities/main.json`, `tauri.conf.json` and every plugin config are unchanged |
| Filesystem access | no new read path in the product. The generator is a repository tool: it writes only under `fixtures/elf/p5-compat/` and `fixtures/manifest.json`, runs only when a maintainer invokes it, and refuses to record an absolute host path |
| Schema, migration, release identity | none. `SCHEMA_VERSION` stays 5, no migration 0006, `analysis:1` untouched — which is why the L15 subproblem stopped instead of being patched |
| Network, telemetry, updater | none, and none was needed to build fixtures |
| Git access | read-only throughout: no history rewrite, no tag, no release, no push beyond §58's candidate |
| Product code | one function plus one call site in `elf.rs`, and two display-text maps in the UI |

The full local gate and `cargo clippy --workspace --all-targets --all-features -- -D warnings` are run at the
candidate per §50, and their numbers go in the final report rather than being asserted here.

## 8. Boundaries held (§1, §42, §47, §63, §64)

Not done, on purpose: the §64 installed Analyze → Compare → Gate → Bundle → History → Diagnostics journey
(Commit F), any L26 change in any form, any P5 closure document or verdict, any DWARF consumer, any
Keil/IAR/`lld` claim, any performance redesign, any fuzzing, and any licence choice —
`OPEN_SOURCE_LICENSE_DECISION = PENDING_OWNER_CONFIRMATION` carries through every document here. The L15
subproblem stopped mid-round and returns to the Architect with `P5_COMMIT_E_SCHEMA_DECISION.md`. Measured
against §48's expected-direction list, thirteen rows landed on the word the prompt expected and **L15 is the
single row that did not** — it went to the Architect because §32's own stop condition is the one the tree
satisfies. Two rows kept their expected word for a reason that changes what the row means, and the report says
so instead of smoothing it over: L8 is carried because the MAP facts were measured to reach no product surface,
and L17's residual is narrower than the inherited sentence implies because the one consumer of that untracked
class already prints which input it lacked and which generator makes it.
