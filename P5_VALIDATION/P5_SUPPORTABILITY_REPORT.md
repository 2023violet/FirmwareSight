---
title: "P5 Commit E supportability report"
doc_id: "FS-P5-COMMIT-E-SUPPORTABILITY"
product: "FirmwareSight"
version: "1.2"
status: "MEASURED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-05"
---

# P5 Commit E — supportability dispositions (prompt §48)

Final disposition for the fourteen rows §48 names, plus the one product defect this round's fixtures
found and fixed. Vocabulary is §48's: `CLOSED / REDUCED / CARRIED_FORWARD / OWNER_DECISION /
NOT_REPRODUCED`, with the row-specific phrasings §30, §31 and §37 ask for kept verbatim where they use
them.

Every row states what was measured, where the evidence lives, and what stays open. Evidence decides, as
§48 says. Compared against §48's own expected-direction list, **thirteen rows landed on the word the prompt
expected and one did not**: L15 went to the Architect instead of closing, because the condition §32 names for
that stop is the one the tree satisfies — the label is serialized in a portable schema, a report DTO, the
stored evidence rows and the bundle, so no "narrow contract-safe wording fix" exists inside Commit E. Two more
reached their expected word for a reason that changes what the row means: L8 is carried because the MAP facts
were measured to reach **no** product surface rather than assumed not to, and L17's residual is narrower than
the inherited sentence implies because `measure_workloads.py` already prints an explicit
`NOT RUN (workload missing; run scripts/gen_p0_workload.py)` row (line 121) instead of dropping the missing
workload, so the only
blind spot left is a newly-added file nobody has referenced yet.

**What changed in this file on 2026-10-04.** The closure normalization round touched two things here and
nothing else: L15's disposition moved from `CARRIED_FORWARD → ARCHITECT` to the canonical
`CARRIED_FORWARD` with the reason `LEGACY_WIRE_IDENTIFIER`, because the Architect answered that one stop with
**Option E** and the answer is now recorded in `P5_COMMIT_E_SCHEMA_DECISION.md` §11; and every §48 disposition
word still means what §48 defined, so **no row here is `CLOSED` that was not closed by evidence** — L15 in
particular stays open on purpose, because preserving a legacy identifier documents the risk rather than
removing it.

**What changed in this file on 2026-10-05.** Commit F1 touched two things here and nothing else, and neither
was a rewrite of what this round measured: L26's §2 section gained a dated position reading
`CLOSED — DECIDED_BY_ADR_0029_AND_IMPLEMENTED` with the counts that decided it, and the disposition table's
L26 cell keeps Commit E's `CARRIED_FORWARD TO COMMIT F` beside a pointer to that note instead of being
rewritten into a word this round never earned. **No other row moved, and no row became `CLOSED` that evidence
did not close.** L15 stays `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER` after F1, because what F1 closed is the
presentation residue Option E allowed it to close — `Details.tsx` now renders the legacy token as *ELF address
+ flags evidence* — while the identifier itself remains inside `analysis:1`, the report DTO, the stored rows
and the bundle. L4, L8, L12 and L23 are untouched by this round and stay exactly as measured.

## 1. Disposition table

| row | inherited state at start HEAD `bfbc1aa` | Commit E disposition | what decided it |
| --- | --- | --- | --- |
| L4 | one Windows host, one WebView2, 100 % DPI | **REDUCED**, residue **CARRIED_FORWARD** | matrix now names each platform's evidence class; no second host and no 125/150 % DPI exists here |
| L5 | two layouts, ELF only, DWARF unread | **REDUCED** | four new layout shapes + compiler variety + debug/no-debug pair; DWARF still unread by design |
| L8 | object/module attribution Unavailable by evidence | **CARRIED_FORWARD** | the MAP facts are parsed but reach no product surface; no narrow current path exists to expose them |
| L12 | two advisories accepted in `deny.toml` | **REVISITED / CARRIED_FORWARD** | local `cargo deny` green; the compatible upgrade measured does not exist |
| L14 | `update_goldens.py` key order, unverified | **CLOSED BY REVALIDATION** (§30) | dry-run twice, 14 goldens `unchanged`, no rewrite, tree clean afterwards |
| L15 | `ElfProgramHeader` source label | **CARRIED_FORWARD** — reason `LEGACY_WIRE_IDENTIFIER` | the label is serialized in three namespaces, so fixing it is a public contract change; §42's stop was **answered by the Architect on 2026-10-04 with Option E** — identifier preserved, meaning documented (`P5_COMMIT_E_SCHEMA_DECISION.md` §11). Not `CLOSED`: the name stays inaccurate inside `analysis:1` |
| L16 | `custom-protocol` needed by hand | **CLOSED** (§38) | package group 4/4 PASS on this host with `cargo tauri build -- --locked` and no manual feature flag |
| L17 | CI provisioning duplication, index blind spot | **REDUCED** (§35) | duplication already gone; the two canonical lists that exist were checked and are satisfied |
| L19 | "Object attribution" means two things | **CLOSED** (§33) | Analyze and Compare now name their own scope, each with a test |
| L20 | Compare prints a Core enum word | **CLOSED** (§34) | closed display mapping for all five current variants + Unknown-safe fallback, 4 tests |
| L23 | UI test races, six known instances fixed | **REDUCED** (§36) | 20/20 fresh-process runs green; bounded sweep read every candidate; one more timer flush removed with a mutation proof |
| L24 | `update_goldens.py` cannot rerun over its Windows leftover scratch | **CLOSED / NOT_REPRODUCED ON CURRENT HEAD** (§31) | second dry-run over the scratch the first one left, no `PermissionError [WinError 5]` |
| L25 | one dead `Apply filter` click, never reproduced | **NOT_REPRODUCED** (§37) | 30 legitimate Apply activations across both tables, 6 more through form submit, no failure |
| L26 | `SHA256SUMS` verifies only on the host that wrote it | **CARRIED_FORWARD TO COMMIT F** (§48) — **CLOSED by Commit F1 on 2026-10-05**, `DECIDED_BY_ADR_0029_AND_IMPLEMENTED`; read the note in §2's L26 section, the table cell keeps the word this round said | untouched by §1; this round neither ran the baseline generator nor changed how a baseline file is produced |

## 2. Row detail

### L4 — platform and display evidence

`P5_COMPATIBILITY_MATRIX.md` §Platforms records Windows x64 on the measured host as `SUPPORTED` (real
install, window, uninstall and reinstall of the packaged NSIS build from Commit D, WebView2
`Edg/154.0.4258.48`, 100 % DPI), Windows 10 x64 and Windows 11 x64 as distinct configurations as
`NOT_TESTED` (§39: "Windows 10: NOT_TESTED unless actually exercised" — this host reported build 19045,
so the family row is the measured one and nothing infers 11), and macOS plus Ubuntu LTS as
`CI_BUILD_ONLY`. What is **carried forward**: a second physical host, and 125 %/150 % DPI. Neither
exists in this environment, and §39 forbids inferring them.

### L5 — layout, ELF-only scope, DWARF

Reduced on the §27 terms, quoted as §27 requires them:

- layout evidence expanded — `p5-ram-exec` (code VMA-resident in writable RAM), `p5-extsram` (third
  writable region at `0x60000000`), `p5-dma-region` (`DMARAM` at `0x30000000`), `p5-long-preamble`
  (region table at byte 16,571), `p5-no-debug` (two absences), `p5-clang-arm` (compiler variety inside
  the frozen cohort);
- debug/no-debug acceptance proved — `a_build_without_debug_and_a_post_link_strip_are_two_different_facts`
  (17 symbols vs 0, both nv 48 / RAM 8);
- DWARF semantic analysis still not implemented — no code reads a `.debug_*` payload, and
  `debug_sections_cost_real_bytes_and_enter_neither_budget` fails if any capability claims DWARF;
- ELF remains the intentional scope — §6 keeps Keil and IAR out, and the matrix says so.

§27's warning is respected: more fixtures is a reduction, not a close. §27 also forbids adding DWARF
parsing to close a row, and none was added.

### L8 — object / module attribution

§28 asked for an inspection rather than a guess, and the inspection changed the answer's shape.
Measured:

- the MAP adapter **does** parse object-level facts — `MapEvidence.object_contributions` is non-empty
  for every committed MAP, asserted for `p2-diff` (`p0_acceptance.rs:143`) and for the long-preamble
  fixture (`p5_compat_fixtures.rs:473`);
- those facts reach **no product surface**: `grep -rn "object_contributions" crates/ apps/` returns the
  adapter and those two tests, and nothing else;
- the capability the CLI reports as `objectAttribution` is not derived from them at all — it is set by
  `crates/firmwaresight-artifact/src/pipeline.rs:228-247` to `Available` whenever `map_evidence` is
  `Some`, i.e. it means "a MAP was provided";
- Compare's separate question (`diff:1 objectChanges.available`) stays `false` with its recorded reason.

So there is no "narrow current path" to add a fixture and test onto: exposing per-object attribution
truthfully needs a new portable field, and §28 says that if closing L8 needs a new subsystem the
subproblem stops and returns to the Architect. **No synthetic attribution was created**, and no object
ownership was inferred from symbol names. Disposition: **CARRIED_FORWARD**, with the reason now
measured rather than assumed.

### L12 — dependency advisories

`cargo deny check licenses bans sources advisories` on this host at this tree:
`advisories ok, bans ok, licenses ok, sources ok` (exit 0, no `unused-ignored-advisory` warning, so both
accept entries still match what they name). Versions and paths, from `Cargo.lock` and `deny.toml:47-62`:

- `RUSTSEC-2024-0429` — `glib 0.18.5` (`Cargo.lock:1293-1295`), reached `gtk 0.18.2 ← tauri 2.12.0 ←
  firmwaresight-desktop`; unsoundness in `glib::VariantStrIter`, which P0 never calls, and glib is not
  compiled for the Windows or macOS targets.
- `RUSTSEC-2024-0370` — `proc-macro-error 1.0.4`, unmaintained, build-time only (a proc macro, not
  linked into the shipped binary), arriving through `glib-macros 0.18.5` in the same gtk-0.18 line.

Compatible upgrade availability, measured rather than asserted:
`cargo update -p glib --precise 0.20.0 --dry-run` fails with
`failed to select a version for the requirement 'glib = "^0.18"' … required by package 'gtk v0.18.2'
… of package 'tauri v2.12.0'`. Moving glib means moving gtk-rs, muda, tao and webkit2gtk — a change to
the frozen Tauri dependency architecture, which §29 forbids doing solely to clear an advisory. So:
**REVISITED / CARRIED_FORWARD**.

Two boundaries kept. The local green is only as fresh as this machine's crates.io index, so the CI
`deny` job stays the authority for the claim. And the sentence is **"dependency policy passes with
documented accepted risks"** — never "security clean", never "zero vulnerabilities".

### L14 and L24 — the goldens tool, revalidated as one pair

`scripts/update_goldens.py` was not modified (§30: "Do not modify simply because L14 exists"). It was
exercised twice in succession in no-confirm mode and produced, both times, 14 lines of
`unchanged golden\…` and `all goldens match current output.` — no semantic golden change, no key-order
rewrite (the writer is `json.dumps(document, indent=2, ensure_ascii=False, sort_keys=True)` at
`scripts/update_goldens.py:269`, and a nested form at `:280`), nothing written, and the repository
state afterwards was whatever the round had already staged and nothing more. **L14 = CLOSED BY
REVALIDATION**, with no code change, which is exactly the outcome §30 predicts.

L24 is the same pair of runs: the scratch tree the first run leaves behind existed when the second ran,
and the second run raised no `PermissionError [WinError 5]` — the cleanup path owns it
(`_force_removal` at `scripts/update_goldens.py:162`, used by `shutil.rmtree(..., onexc=…)` at `:175`
and the `onerror=` fallback for older interpreters at `:177`). No repository `.git`, no unrelated
`target/`, no user data was deleted, and the tool touched no goldens. **L24 = CLOSED / NOT_REPRODUCED
ON CURRENT HEAD.** No code change, so §31's "add a regression if code changes" does not apply.

### L15 — the source label: stopped, answered, carried forward with a reason

Audit row L15 asked whether `MemoryEvidenceBasis::ElfAddressAndFlags` showing `ElfProgramHeader` is
(A) user-facing wording or (B) serialized public portable evidence semantics. Measured answer: **(B)**,
in three namespaces at once — the `analysis:1` enum member `"elf.program-header"`
(`schemas/analysis.schema.json:616`, emitted at `crates/firmwaresight-report/src/dto.rs:139`), the
store's `evidence.source_type` column holding the Rust `Debug` name `ElfProgramHeader`
(`crates/firmwaresight-storage/src/db.rs:507`, read back at `query.rs:503`), and the shipped bundle's
`analysis.json` (`ANALYSIS_DOC_NAME = "analysis.json"`,
`crates/firmwaresight-core/src/domain/release.rs:431`, written at
`crates/firmwaresight-project/src/bundle.rs:802`) — plus 4 rows of `golden/cli/p0-basic-analyze.json`
and a verbatim passthrough in `apps/desktop/ui/src/Details.tsx:667`.

§32 says stop if changing `SourceType` moves the portable JSON, the bundle, the goldens or schema
semantics. It does, so **this subproblem was stopped and returned to the Architect** with
`P5_VALIDATION/P5_COMMIT_E_SCHEMA_DECISION.md`, which prices four options (display-only mapping,
additive enum member, corrective rename plus migration, or documented status quo) and names what each
one moves. Nothing in Commit E renamed an enum, bumped a schema version, or wrote a UI caption over the
top of the stored value. `CLOSED only if narrow contract-safe wording fix` (§48) is therefore not
claimed: the wording fix that is contract-safe is a subset of option A, and choosing it was not this
round's call.

**The call was made afterwards, and it is recorded here rather than in Commit E.** The Architect answered
with **Option E — legacy wire identifier preserved, accurate presentation / documentation** (§8–§12 of the
closure normalization prompt; decision text in `P5_COMMIT_E_SCHEMA_DECISION.md` §11):

- kept: `SourceType::ElfProgramHeader`, the wire token `"elf.program-header"`, the stored
  `ElfProgramHeader` rows, `analysis:1`, the Bundle `analysis.json` contract, the goldens, `SCHEMA_VERSION`,
  release identity and `ADR-0028`;
- refused: no `ElfSectionFlags`, no enum rename, no wire rename, no rewritten history, **no migration 0006**,
  no `analysis:2`, no golden byte moved;
- documented instead: `elf.program-header` is a **legacy `analysis:1` compatibility identifier**, and the
  accurate human meaning of `MemoryEvidenceBasis::ElfAddressAndFlags` is **ELF address + flags evidence** —
  which the Compare basis caption already says (`Compare.tsx:690`, `ELF address/flags evidence`);
- disposition: **`CARRIED_FORWARD` with reason `LEGACY_WIRE_IDENTIFIER`**, and explicitly **not** `CLOSED`,
  because §11 of that prompt forbids pretending the historical identifier became accurate. The risk is now
  bounded and defined; a true rename remains available only inside a future `analysis:2`, which is a
  major-contract decision no document here previews.
- **carried to Commit F as a named presentation item:** the Evidence Inspector renders the stored token
  verbatim (`Details.tsx:667`, fed by `db.rs:507` → `query.rs:503` → `details.rs:223`), so a user expanding
  such a row reads `ElfProgramHeader` rather than the accurate meaning. Fixing that is a display-only caption
  mapping in the closed-table shape L20 already used for the basis captions — product source, and therefore
  not this documentation-only round's to touch.

### L16 — package path and `custom-protocol`

`python scripts/check.py --only package` on this host: `PASS package/frontend deps`, `PASS
package/cli companion`, `PASS package/desktop package`, `PASS package/artifacts verified`, exit 0, no
SKIPPED step. The desktop step is `cargo tauri build -- --locked` (`scripts/check.py:389`) with **no**
`--features custom-protocol` anywhere in the command, run by `tauri-cli 2.12.1` (the pinned version),
and it produced `FirmwareSight_0.6.0_x64-setup.exe` (3.70 MiB) plus
`target/dist-package/FirmwareSight-0.6.0-windows-x86_64-nsis.exe`, `-cli-fwsight.zip`,
`SHA256SUMS.txt` and `artifact-metadata.json`.

The proof that the feature was enabled by the build rather than remembered by a human is the verifier's
own finding: `frontend embedded (read from target/release/firmwaresight-desktop.exe …)`, and the
installer's version resource reports `0.6.0` from the same workspace version. §38's "if green:
L16 = CLOSED" applies, and no package architecture was rewritten.

### L17 — the index blind spot

§35 scoped this to the remaining bounded risk: files a build or test uses that Git does not track. The
provisioning-duplication half was already reduced and was not redone.

The two canonical lists that exist were checked:

1. `fixtures/manifest.json` — all 56 paths exist and are tracked; this is the existing
   `drift/fixtures tracked` step (`scripts/check.py:187-216`) and it is green, including for the 31 new
   compatibility entries.
2. `apps/desktop/src-tauri/tauri.conf.json` — the bundle declares **no** `resources` and five icons
   (`icons/32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.ico`); measured against
   `git ls-files`, all five exist **and** are tracked, so there is nothing to add to the guard.

One set is deliberately untracked and must stay that way: `fixtures/generated/` (the guard ladder:
`guard-100mib/256mib/512mib/over-512mib.elf`, 1,447,035,904 bytes measured on disk here) is in
`.gitignore:15`, produced by `scripts/gen_p0_workload.py`, which says so in its own header, and is
consumed by `scripts/measure_workloads.py:42-45` for performance measurement rather than by the test
suite. A tracking rule would need an allowlist for exactly this class, which is where §35's
"do not build a general source-control linter" line is. This round also corrected the generator's own
estimate of that total, which read "~1.1 GiB" where the four files measure 1,447,035,904 bytes — a
docstring only, no behaviour change.

**L17 = REDUCED**, and the residual is narrower than it first looked, because the missing half of the
handling is already in place: `scripts/measure_workloads.py:117-123` prints an explicit
`NOT RUN (workload missing; run scripts/gen_p0_workload.py)` row for each absent workload instead of
dropping it from the table, and `:104-111` refuses outright when the release binary has not been built.
So a consumer of untracked inputs already says which input it lacked and which generator makes it.

What no gate step can see, stated as the residual risk: nothing asserts that the `fixtures/generated/`
class *stays* untracked (an accidental `git add -f` of a 512 MiB workload would be caught only by the
next person who notices the repository grew), nothing checks that a generated workload still matches its
recipe (only the generator knows), and CI never runs `measure_workloads.py`, so a recipe drift surfaces
during measurement rather than in a job. Closing that would mean a second manifest for deliberately
untracked artifacts — new governance surface, for a class that already fails loudly at its only
consumer. §35's "if no additional falsifiable set exists, L17 = REDUCED with residual risk documented"
is the disposition, and no general source-control linter was built.

### L19 — two questions, now named as two

§33's requirement was product-facing scope, not a rename of any contract. Measured against the current
tree:

- Analyze reports **`Object/module attribution`** for `capabilities.objectAttribution`
  (`apps/desktop/ui/src/Analyze.tsx`), the single-build question;
- Compare reports **`Object-level change attribution`** for `diff:1 objectChanges.available`, the
  pair question;
- `compare.test.tsx:791` `labels object attribution as the two different questions the pages answer`
  asserts both labels in one test, and `details.test.tsx:344`
  `names the attribution capability by the scope it actually reports` holds the Diagnostics surface to
  the Analyze scope.

**L19 = CLOSED.** No capability semantics changed: `objectAttribution` still means "a MAP was provided"
(which is why L8 above cannot be closed by editing a label).

### L20 — Compare no longer prints a Core enum word

Before: `weakest basis MapRegionAndElfLoad`. Now `apps/desktop/ui/src/Compare.tsx` carries a closed
display mapping — `EVIDENCE_BASIS_CAPTIONS` maps each variant's Debug name *and* its kebab wire alias to
one caption, and `basisCaption()` returns `'unknown'` for a missing basis and
`Unrecognized evidence basis` for a value the mapping does not contain — so the five current variants
read: MAP regions + ELF load evidence / configured regions + ELF load evidence / ELF address/flags
evidence / section-name heuristic / insufficient evidence. §34's rules all hold: no Core enum was
renamed, the vocabulary matches Analyze's, a future value cannot crash the page, and Unknown keeps its
Unknown meaning rather than becoming a claim.

Tests: `compare.test.tsx:752` (both MAP/region bases), `:763` (ELF and name-heuristic), `:773`
(insufficient plus an unknown future variant in the same run), `:783` (a missing basis stays `unknown`).
**L20 = CLOSED.**

### L23 — async reliability, swept and repeated

§36A static review over all seven UI test files (`compare`, `details`, `help`, `history`, `intake`,
`release` `.test.tsx` plus `format.test.ts`) for the five shapes it names. Method and result:

- every IPC start and every first loading commit is awaited (`openHistory()`, `analyzeOk()`,
  `findByRole`, `await within(…).findByText`), and the chained/second-wave tests resolve their deferred
  promises explicitly before asserting;
- a second pass over the chained-wave shape itself — tests with two or more interactions where nothing
  awaits after the second — returned two candidates, both in `release.test.tsx`, and both are local-state
  tests rather than wave tests: `:990` drives `submit.disabled` from form state (its one IPC assertion at
  `:998` sits behind a `findByRole` that already settled), and `:1497` clicks `Edit policy`, which only
  toggles the editor (`apps/desktop/ui/src/Release.tsx:361-366` flips `draft` from config awaited one line
  earlier). Neither pass found a product race or a test-only race;
- absence assertions were enumerated per file (20 + 6 + 9 + 9 + 12 + 33 = 89 across the suite) and
  filtered to those with **no** settle point between the interaction that could start work and the
  assertion. Three survived the filter and each was read:
  `details.test.tsx:511` (unit radio reformats data already awaited), `history.test.tsx:429` (the same,
  and the test additionally pins the read counts at 1 each), `release.test.tsx:1329` (a *disabled*
  button: the click must never start the write, so the absence is the contract and is synchronous by
  construction). No product race and no test-only race among them;
- one real timer dependency was found and removed: `history.test.tsx` settled its stale-reply assertion
  with `await new Promise(done => setTimeout(done, 0))`. That is a macrotask sleep standing between the
  interaction and the absence check, exactly the shape §36 forbids, so it is now
  `await act(async () => { await Promise.resolve(); })`. **Mutation proof F**: replacing
  `if (request.current !== id)` with `if (false)` in `src/History.tsx:94` reddens
  `paints only the newest answer when an older reply arrives last` with
  `expected <td title="stale.elf"></td> to be null`, and `History.tsx` was then restored byte-exact
  (digest `29022971…5ecea`, `git diff` empty). The test therefore catches the regression rather than
  passing by skipping the settle.

§36B repetition, after the final UI change: **20 fresh-process runs** of the complete suite
(`corepack pnpm test` in `apps/desktop/ui`, one process per run), `failing_runs=0`, each run reporting
`Test Files 8 passed (8)` and `Tests 217 passed (217)`, wall time **17–34 s** per run (479 s in total,
campaign 18:07:24Z to 18:15:25Z), environment `MINGW64_NT-10.0-19045` with `node v24.19.0`. Log:
`target/p5reliability/ui-20-runs-2.log` (local and untracked, which is why the numbers are recorded here and
the method beside them). A first campaign of the same 20 runs on an almost-identical tree is also green
(`ui-20-runs.log`, `failing_runs=0`, wall 15–26 s) but its count extractor was defeated by ANSI escapes, so
the campaign was re-run rather than restated; its per-run totals are the same 217 in 8 files. Direct count is
**217 tests in 8 files**; repetitions are not multiplied into it, per §53.

No sleeps, no retry wrappers, no timeout inflation: the only `setTimeout` in the suite was the one
above, no test or config raises `testTimeout` or a `waitFor` timeout, and no `retry` helper exists.
**L23 = REDUCED** — the residue is that jsdom still cannot drive a real keypress (see L25), and a
bounded sweep cannot prove the absence of a race the sweep's shape missed.

### L25 — `Apply filter`, revalidated with volume

§37 requires reliable automated interaction over both filter tables, at least 30 legitimate Apply
actions, and keyboard activation where supported. §55 asks for those three numbers in the attempt/success
shape, measured from the test's own two assertions (`expect([applied, bySubmit]).toEqual([30, 6])` and the
per-value `waitFor` that each filter reached the shell):

```text
Section Apply filter:   15 attempts / 15 success   (fireEvent.click on Apply filter, one value each)
Symbol Apply filter:    15 attempts / 15 success   (fireEvent.click on Apply filter, one value each)
Keyboard activation:     0 attempts /  0 success through a real keypress in this harness —
                        1 synthetic Enter keydown probed, it produced no request because jsdom does not
                        implement implicit form submission, so the honest value is
                        NOT_VERIFIED_THROUGH_A_KEYPRESS;
                        6 section applies succeed through the form-submit path a keypress would take
Total legitimate Apply actions: 36 (30 click + 6 submit), against §55's minimum of 30.
```

- `compare.test.tsx:929` `applies every filter the two change tables are offered, and each one reaches
  the shell`: **30 click Activations** — 15 section-filter values and 15 symbol-filter values, each
  clicked and each verified to reach the shell as a request with that value, with the asserted count
  ending at exactly 30;
- the same test then drives **6 form-submit activations** with values the click loop never uses
  (`eeprom`, `watchdog`, `spi`, `i2c`, `crc`, `timer`), asserting `sectionApply.type === 'submit'` and
  ending at `[30, 6]`;
- keyboard activation: **NOT_VERIFIED_THROUGH_A_KEYPRESS**. A synthetic Enter `keydown` was tried first
  and produced no request at all, because jsdom does not implement implicit form submission — measured,
  not assumed. The path is covered by the submit activations above, and a genuine keypress needs an
  installed window;
- no intentionally disabled control was counted as a failure; nothing was found to reproduce, so no
  state capture or narrow fix was needed;
- `FOCUSED_COMMIT_E_PACKAGE_CHECK = NOT_RUN`. §47 makes the packaged walk optional. Running it would
  mean a fresh install and the owner-store isolation protocol (hash → park → backup → disposable store →
  check → restore → hash compare) for one keyboard assertion, and §64's installed journey is Commit F's.
  The packaged build itself was exercised this round by the L16 package group, but no packaged *app* was
  operated. Because no packaged app ran, the owner's store was never opened: measured here, not assumed —
  `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite` is 155,648 bytes with digest
  `d6e41034d9a7fdcdf5e72b4621ca3c8445c79dbbf7fd9aab70f5bf6bca836468` and last-write time
  2026-09-30T11:56:40, byte-identical to the value `P5_DIAGNOSTICS_RECOVERY_REPORT.md:35` recorded before
  this round began. §62's "owner DB untouched or restored byte-exact" is therefore satisfied by
  *untouched* rather than by a restore.

**L25 = NOT_REPRODUCED**, with the attempt counts and environment recorded above and in
`P5_COMMIT_E_DESIGN.md`.

### L26 — left exactly where Commit D put it

`CARRIED_FORWARD TO COMMIT F`, unchanged: §1 forbids touching it and §48 fixes the disposition. What
this round contributed is adjacent, not the decision — the fixture regeneration check proved the
compatibility binaries reproduce byte-identically here (§51), which is a fact about the fixtures, not
about `SHA256SUMS` for a `git archive` extraction on a different host. The blob-vs-declared-normalization
choice stays with whoever owns Commit F, and `scripts/generate_baseline_artifacts.py` was not run.

**L26 after Commit F1, 2026-10-05 — CLOSED, `DECIDED_BY_ADR_0029_AND_IMPLEMENTED`.** The paragraph above keeps
Commit E's word because it is Commit E's answer to §48, and `CARRIED_FORWARD TO COMMIT F` was the truthful
disposition when that round closed. Commit F decided it: `ADR-0029` settles that the root `SHA256SUMS` records
the SHA-256 of each path's canonical Git stage-0 index blob rather than its working-directory bytes, both
baseline scripts were rewritten to that rule, and `scripts/verify_baseline_artifacts.py` now runs inside the
authoritative drift gate — so the claim is checked on three platforms instead of agreeing with the host that
wrote the file. What made it a measurement rather than a decision by impression is the count: the
worktree-versus-blob difference read **19, 19, 14, 14, 17, 13, 13** across the seven P5 heads before it, and the
closure evidence is that `80b47c4`'s own manifest returns **13 FAILED** against a clean `git archive` of
`80b47c4` while F1's returns **693 OK, 0 FAILED** against a clean archive of the same kind. The plan, the seven
proofs and the raw numbers are `P5_COMMIT_F_DESIGN.md` §1 and §4–§7, and `.ai/DECISIONS.md` (F1). `ADR-0028` is
untouched: release identity still means the exact bytes observed on disk, and prompt §6 is why the two rules
stay in separate documents.

## 3. The product change this round's evidence forced

**An allocated section the parser does not recognise entered neither memory budget.** Discovered by
`p5-clang-arm`: clang emits `.ARM.exidx.text.main` (8 bytes, `SHF_ALLOC` set) inside the declared FLASH
region; `map_section_kind` treated "unrecognised kind" as "not allocated", so the image total reported
`Exact` at 50 while the file stored 58 allocated payload bytes and `unattributed` stayed empty.

- fix: `crates/firmwaresight-artifact/src/elf.rs` reads `SHF_ALLOC` from the section header flags and
  passes it to `map_section_kind`; write/execute derivation and the metadata-name net are untouched;
- classification rule respected: `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md:42` settles a custom section by
  segment/region evidence, never by name, and the charge carries basis `MapRegionAndElfLoad`;
- tests: `an_allocated_section_the_parser_does_not_recognise_is_still_charged` (8/0, totals 58/8) and
  `the_reported_image_footprint_equals_the_bytes_the_file_claims_are_loaded` — an invariant over all 11
  manifest ELFs, computed from raw `SHF_ALLOC`/`SHT_NOBITS` bytes rather than from the model's own flag
  derivation, because a version of that assertion built on the model's flags stayed green under the bug;
- mutation proofs: **A** (undo the fix) reddened both tests with `left: 50, right: 58`; **C** (perturb one
  recorded total, 184 → 185) reddened the external-SRAM test; **E** (read `SHF_WRITE` instead of
  `SHF_ALLOC`) reddened 8 tests; **B** (give the clang ELF the GCC binary) reddened the provenance test.
  Everything was restored byte-exact and re-run green;
- collateral: no golden moved (`git diff --exit-code -- golden` clean at the end of the round, and
  `drift` 7/7 including `drift/goldens unchanged`), and **ten of the eleven committed ELFs already
  satisfied the invariant**. That is measured, not inferred: walking every manifest ELF's section
  headers for an allocated, non-`SHT_NOBITS` section whose `sh_type` is outside `SHT_PROGBITS` returns
  exactly one hit in exactly one file — `.ARM.exidx.text.main`, `type=0x70000001`, 8 bytes, in
  `p5-clang-arm`. The other ten carry only ordinary `.text`/`.rodata`/`.data`, kinds the old
  classifier already knew, which is why the whole existing cohort stayed green over a real defect.

## 4. What this round did not do

No new dependency, no new capability, no new IPC command, no schema or migration change, no
network/telemetry surface, no signing or release action, no DWARF parser, no Keil/IAR/`lld` claim, no
performance redesign (§41 — the 519,179,252-byte workload figures stay carried forward as measured in
the post-G2 round: warm ~4.7–4.9 s, cold first read ~68.7 s, working set ~1,428 MB, no "Not Responding",
no crash), no fuzzing (§40: still blocked by ADR-0016 plus the pinned `1.98.1` toolchain, so the
no-panic claim keeps resting on `fixtures/malformed/*` and the four negative-input regressions, which
Commit E re-ran green without weakening), and no P5 verdict. `P5_PRODUCTIZATION_AUDIT.md` §I and §63
remain the list of what must stay open.
