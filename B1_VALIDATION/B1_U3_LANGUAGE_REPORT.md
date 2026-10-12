---
title: "B1-U3 zh-CN / en Interface Language — Execution Report"
doc_id: "FS-B1-U3-REPORT-001"
product: "FirmwareSight"
version: "0.6.0"
status: "REPORT"
owner: "Engineering"
last_updated: "2026-10-11"
---

# B1-U3 — the whole interface in two languages, on the mechanism B1-U1 and B1-U2 already built

Executed on 2026-10-11 under `NEXT-06` (archived at
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_NEXT_06_B1U3_ZH_EN_Internationalization_2026-10-11.txt`, 8,797 bytes, 57 lines,
0 `CR`, sha256 `5d6bde626f63ecf280d275998c113b209ef1de5ceb4269d42f445c99ac6b2dca` — the digest recomputed against the
delivered file, `cmp`-identical to it). Its four entry gates were closed in order before the first line was written:

| Gate | What it required | How it closed |
| --- | --- | --- |
| A | B1-U2 independently accepted, with the limits itemised | `NEXT-05` returned `ACCEPT_WITH_EXPLICIT_LIMITS` with limits L1–L5 at `e0b8fc4`. Its report is `target/n05_next05_report.md`, gitignored on purpose: `NEXT-05` §授权边界 forbids modifying a tracked file, so the verdict was never written into `B1_VALIDATION/`. The Owner's own adjudication of the same day is archived and tracked at `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_B1U2_Architect_Review_2026-10-11.md` |
| B | A push the Owner authorized, and CI read back at the new head | `2cff702..e0b8fc4` fast-forwarded on the Owner's word; remote run `38160046797` read 10 of 10 green |
| C | An explicit dispatch for this unit, not an inferred one | The Owner's four-step instruction of 2026-10-11, step 3: implement the complete Chinese/English interface directly, with no separate product-direction study and no UI concept art |
| D | A live pointer that matches the task actually being run | `.ai/ACTIVE_TASK.md` is not `NONE`; it named B1-U3 as the next unit and `NEXT-06` is the authorization the pointer needs; the pointer paragraph for this round is appended in §7 below |

Status word this round proposes, and does not grant: **`LOCAL_B1_U3_GATE_PASS / ARCHITECT_REVIEW_PENDING`**.

## 1. changes — exact paths and intent

67 paths, +6,259 / −1,392 — the code, the catalog, the tests, `ADR-0032`, the three archived prompts, this report and its checklist, and the three governance documents. Nothing in `crates/` was touched: no Core, CLI, Gate, parser, diff, storage-schema,
fixture, golden or migration file moved (§2 boundary, verified in §4 below).

### The store and the wire

| Path | Change | Intent |
| --- | --- | --- |
| `apps/desktop/src-tauri/src/preferences.rs` | `SCHEMA_VERSION` 2 → 3; `LanguagePref {System, ZhCn, En}` with a closed word set; `SettingsFileV2`; `read_document` widened to accept 1, 2 and 3; `render` writes the fourth choice; `update_application_preferences` takes the language | Store the fourth choice without losing the first three |
| `apps/desktop/src-tauri/src/ipc.rs` | `PreferencesDto.language` | Carry the *chosen* word, never the language the platform answered with (`ADR-0032` D1) |
| `apps/desktop/src-tauri/tests/preferences.rs` | v1→v3 and v2→v3 read cases, the language round trip over every offered word, an older document that writes nothing on open, the first save that writes four choices, and two hybrid refusals | The reader widened before the writer moved, proved in both directions |
| `apps/desktop/src-tauri/tests/release_gate.rs` | the isolation case stores and checks the language too | A language is not evidence |
| `apps/desktop/src-tauri/tests/bundle_ipc.rs` | `a_stored_language_moves_no_bundle_byte_no_manifest_digest_and_no_release_id` | The claim §E asks for, at the only place it can be checked: the exported bytes |
| `apps/desktop/ui/src/ipc/bridge.ts`, `ipc/generated/PreferencesDto.ts` | fourth argument; regenerated binding | The wire matches the command |

### The catalog and its runtime

| Path | Change | Intent |
| --- | --- | --- |
| `apps/desktop/ui/src/i18n/en.ts`, `zh.ts` | new: composition points, `Catalog`, `Namespace` | English is the key authority; Chinese is typed against it |
| `apps/desktop/ui/src/i18n/en/*.ts`, `zh/*.ts` | new: nine namespaces per language (analyze, common, compare, errors, gate, help, history, release, settings) | A missing or extra Chinese key is a `tsc` error, not a runtime log |
| `apps/desktop/ui/src/i18n/index.tsx` | new: `translate`, `fill`, `placeholders`, `knowsError`, `I18nProvider`, `useI18n`, and the fallback recorder | One lookup, one interpolation rule, and a leak that cannot hide |
| `apps/desktop/ui/src/i18n/language.ts` | new: `resolveLanguage`, `pickPlatformLanguage`, the stored-word codec | `system` resolves against `navigator.languages` at mount, and is not claimed live |
| `apps/desktop/ui/src/i18n/types.ts` | new: `CatalogFor<'ns'>`, `CatalogShape` | Breaks the en↔zh type cycle the per-namespace split needs |
| `apps/desktop/ui/src/i18n/language.test.ts` | new, 15 cases | Resolution, precedence and the closed word set, apart from React |
| `apps/desktop/ui/src/i18n/catalog.test.ts` | new, 15 cases | Key parity, no empty value, placeholder parity, no markup, the two languages actually differ, machine vocabulary survives, and a walk of all 918 keys in both languages with per-template parameters |
| `apps/desktop/ui/src/test/render.tsx` | new | A page renders inside the provider, English by default so 800+ existing locators stay valid |

### The screens

`Analyze.tsx`, `App.tsx`, `Compare.tsx`, `Details.tsx`, `GettingStarted.tsx`, `Help.tsx`, `History.tsx`,
`Overview.tsx`, `Release.tsx`, `Settings.tsx`, `stateWords.ts`, `evidenceBasis.ts`, and the shared
`components/{Chip,ErrorPanel,SizeUnitSwitch,Table,TopBar}.tsx` now read their words from the catalog: 867 `t()`
calls across those files. `Settings.tsx` gained the language control and the sentence that states what it does not
change. `stateWords.ts`'s verdict builder takes the lookup as a parameter, so the gate page and the Overview page
cannot state one run two ways. `evidenceBasis.ts` kept its routing table and gave its six captions to `common`.

Tests extended: `settings.test.tsx` (+9 language cases), `details.test.tsx` (+2), `history.test.tsx` (+2),
`overview.test.tsx` (+2).

## 2. The decisions that were not mechanical

**The catalog was chosen on measured numbers, and neither slogan won.** `NEXT-06`'s 架构判定 1 forbids resting the
choice on "zero dependency" and forbids assuming `i18next` is required. What was measured: the string surface per
namespace (the inventory in `target/n06_string_inventory.md`, gitignored scratch), the cost of a runtime library at
this boundary (a new dependency, a new namespace of its own, and a lookup the type system cannot see), and the fact
that this project's failure mode is not "a language is missing" but "one screen says PASS while another says
UNKNOWN". A typed in-repo catalog makes the second impossible at compile time: `CatalogFor<'ns'>` types the Chinese
file against the English one, so a missing key fails `tsc --noEmit`, which is step 5 of the gate. `i18next` would
have made a missing key a log line. That is the whole of the argument, and it is `ADR-0032` D2.

**`system` turned out to be two different promises.** Colour has a change signal (`matchMedia`), so a `system`
palette follows the desktop while the window is open. Language has none: there is no `navigator.languages` event.
The implementation resolves the language at mount and when the choice returns to `system`, and the Settings sentence
says the reader's desktop language is applied at the next start. `ADR-0032` D8 records the asymmetry, and
`settings.test.tsx` refuses a page that claims a live desktop language.

**The schema number was not guessed.** 3 is 2's document plus one field, and the reader was widened to accept 1, 2
and 3 *before* the writer moved. Widening in the other order would have opened every existing file as damaged and
demoted it to `.unreadable` on the next save. Two tests exist only to hold that order honest, and one of them
(`a_version_one_document_carrying_a_language_is_refused_whole`) refuses a hybrid rather than understanding half of
it.

**A tenth namespace was proposed and refused.** `Details` was going to get its own; it reads `analyze`, because it is
the same analysis's rows and a reader must not meet two vocabularies for one artifact (`ADR-0032` D4). Nine
namespaces, both languages, 918 keys each.

**The shared pager sentence was split, not translated twice.** `components/Table.tsx` wrote
``Showing ${offset} to … of ${total} ${label}`` around a noun its callers supplied, which is one line in two
languages the moment a Chinese noun arrives. The sentence moved to `common.pager.*` and the noun stayed with the
page that owns the rows. `History.tsx`'s DOM id keeps the English-stable word it was built from, because an id does
not change language.

**Machine vocabulary is not translated, and three places say why in the file itself.** The five Gate states, the four
evidence classes, `KiB`, `ELF`, `MAP`, `Git`, `SHA-256`, rule ids, ids, digests, paths, numbers and error codes are
data or Core's words (`ADR-0032` D5). Two consequences were decided rather than drifted into: Compare's filter chips
keep `Added`/`Removed`/`Changed` because the cells they filter render Core's own word, and the Evidence-class
`<option>` labels keep `observed`/`derived`/`declared`/`unknown` because the value sent to Rust *is* that word —
translating the label would make the control read one thing and send another.

**The error boundary was rebuilt around the code, not the sentence.** `NEXT-06` §4 forbids translating Rust's
English by splitting and reassembling it. `ErrorPanel.tsx` now looks the stable code up in the `errors` namespace
(59 codes, 59 of 59 measured against `grep` over `crates/` and `apps/`, plus an `unknown` fallback that says so),
renders the controlled Chinese sentence for it, and keeps the engine's own words in a clearly-labelled detail row.
`knowsError(code)` exists so the page can state whether it recognises the code instead of guessing.

## 3. Verification

| Class | Result | Evidence |
| --- | --- | --- |
| SOURCE_PRESENT | **PASS** | start `HEAD e0b8fc4` (`git rev-parse`), one worktree, no stash; the round's work lands on top as `9ccd3f0` plus the docs-only successor `40ce0b6`, and `git status --porcelain` is empty with `main` `ahead 2` |
| LOCAL_TEST_PROVEN | **PASS** | python scripts/check.py on the final staged tree: **17 of 17, exit 0, no SKIP** — **1,056 Rust passed / 0 failed / 0 ignored across 49 `test result:` lines** (1,050 at `e0b8fc4`; the six new cases are not `cfg`-gated, so 1,056 is the projected POSIX figure too — **POSIX remains NOT MEASURED on this host, which has no POSIX runner**), **379 UI tests in 13 files** (334 in 11), clippy and fmt clean, 37 commands, baseline verifier PASS at **852 tracked / 850 sum entries / 984 tree lines** (820 / 818 / 949 at B1-U2's close; the +32 are this round's catalog, ADR, prompts, report and checklist) |
| DEV_RUNTIME_VERIFIED | **PASS** | `target/debug/firmwaresight-desktop.exe` against the Vite dev server, driven over WebView2 CDP: three cold starts, every page walked in both languages, 15 captures in `target/n06_shots/` (12 Chinese at inner 1024×721 and 1440×901, 3 English for the comparison) taken after a real Win32 `MoveWindow` resize |
| STORE_UPGRADE_ON_A_REAL_FILE | **PASS** | The Owner's `settings.json` was `{"schema_version":1,…}` at 03:58 today. The window read it, kept both words, and wrote `{"schema_version":3,"size_unit":"bytes","startup_page":"analyze","theme":"system","language":"en"}`. No `.unreadable` sibling was created. The file was restored byte-for-byte from `target/n06_backup/` afterwards |
| INSTALLED_VERIFIED | **NOT_VERIFIED** | No CI-built installer exists for any B1 head. A dev binary is not installed proof, and no B1 unit gets that word |
| REMOTE_CI_PROVEN | **NOT_VERIFIED** | Nothing in this round has been seen by a runner: §工作步骤 5 stops at a local commit |

What the running window showed, in numbers rather than adjectives:

- **Every page in Chinese, seven of them.** `总览 / 分析 / 对比 / 发布门禁 / 发布包与历史 / 设置 / 帮助`, each with
  `<html lang="zh-CN">`. The longest Latin run on those screens was a file name, an identifier or the config
  directory name — no English sentence.
- **A language switch moved no fact.** One Gate run, made in English: `gate-b64afd679c0e3d81f7b7b033c546647602f918415524e1300785c45979c0d0c3`.
  After switching to Chinese the same page showed the same id, the same three digests
  (`c6d0feed...7def4e62`, `c4182c5b...f0ce2a61`), and the same badge tally — `REVIEW` 1, `UNKNOWN` 2, `PASS` 2,
  `N/A` 8 — while the sentence around them read 需要决定 —— 0 条 finding 在等待一次明确的接受.
- **The five state words survive identically.** The Getting-started panel counts `PASS` 1, `REVIEW` 1, `BLOCK` 1,
  `UNKNOWN` 2, `N/A` 1 in both languages; only the explanation after each word is translated.
- **Two live switches, both directions.** Choosing 简体中文 re-wrote the whole window without a reload (the note read
  设置已保存。); choosing English did the same. A cold start then came back in the stored language twice — `en`, then
  `zh-CN` — and the first start of the day, with `system` stored, resolved to `zh-CN` from this desktop.
- **No overflow at either size, in either language**: `scrollWidth == clientWidth` on the document and on `main` for
  all seven pages at 1024×721 and 1440×901.

## 4. The STOP list, item by item

| `NEXT-06` forbids | How this round shows it did not happen |
| --- | --- |
| Language changing any Core / Gate / Release data, evidence identity or a portable contract | The bundle test compares two exports that differ only in the stored language: same artifact bytes, same `SHA256SUMS`, same manifest digest, same release id. In the window, the run id and digests above. `grep` finds `LanguagePref` only in `apps/desktop/src-tauri/src/preferences.rs` and three of that crate's own test files, and nowhere under `crates/`; the command count stays 37, because the language rides the preferences command B1-U1 already registered |
| An old `settings.json` not safely recoverable | v1 and v2 documents read, keep their words, and gain defaults; a refused future version is left alone; the Owner's real v1 file upgraded with no `.unreadable` copy |
| A future-version file overwritten | `a_document_from_one_version_ahead_is_refused_and_left_alone`, and the reader's `Some(seen) if seen > SCHEMA_VERSION => Err(Unreadable::Future)` |
| A damaged file unconditionally deleted | The `.unreadable` keep-before-replace path is B1-U1's and was not touched; no test that relies on it was weakened |
| Missing translations undetectable | `CatalogFor` makes them a compile error; the fallback recorder makes any runtime miss visible, and `details`/`history`/`overview` Chinese tests assert `fallbacks()` is empty |
| 中文界面实际上依旧大量暴露未经翻译的英文操作文案 | Zero hardcoded text nodes remain in any non-test `.tsx` (measured per file); the Chinese catalog has all 918 keys; the live window's seven Chinese screens contain no English sentence; `evidenceBasis.ts`'s six captions were moved into the catalog this round precisely because they were the last shared prose that leaked |
| Network-downloaded strings, telemetry, accounts, new host file access | No dependency added (`Cargo.toml`, `package.json`, `pnpm-lock.yaml` untouched); no new command; the only file named to the WebView is `settings.json`, by name, as before |
| Labelling an AI self-review as Architect ACCEPT, or passing JSdom captures off as desktop acceptance | §3 keeps `DEV_RUNTIME_VERIFIED` and `LOCAL_B1_U3_GATE_PASS` separate from `INSTALLED_VERIFIED` and from any Architect word; the window evidence is the running app, and the jsdom results are quoted as tests |

## 5. Limits, and what this round did not do

1. **One layout observation, not a defect.** At the 1024 floor the History builds table is wider than its wrapper in
   both languages, so the 非易失 and RAM columns sit behind the wrapper's horizontal scroll — the behaviour its own
   test protects. Chinese makes those two cells taller (精确值 plus a wrapped byte count) without clipping anything
   outside the scroll area. Recorded here because the screenshots show it and a reader will ask.
2. **`apps/desktop/ui/src/format.ts` still emits `Unknown`, `bytes` and `KiB`.** All three are the state word and the
   two units, i.e. D5 vocabulary, and are deliberately not in the catalog. The `evidenceBasis.ts` captions that sat
   beside them were prose and did move.
3. **The status strip counts with `String(count)`, not `format.ts`'s grouping.** Both call sites count Gate findings,
   bounded by the rules a policy holds. A byte figure and a finding count are different facts and this round refused
   to bind them.
4. **One Compare key has no test locator** (`section.pagerLabel` is rendered but no assertion names it). The catalog
   walk covers it; the DOM does not.
5. **A scratch-instrument artifact, recorded so nobody chases it as a product bug.** The page walk's `stateWords`
   field reported five words in the Chinese capture and one in the English one for the same page. A direct count of
   the same page in both languages reports the identical tally. The field is dedup over `textContent` inside a
   gitignored driver; the product measurement is the second one.
6. **Not done, by instruction:** no push, no B1-U4, no B2, no V1 work, no Release run. The three `NEXT-05`
   correctives (C1 needs the Owner's word because it moves two report goldens, C2, C3) are still open and are
   untouched here.
7. **`V1`'s frozen GitHub Artifact records expire 2026-10-22** and still need their own private archival. This round
   neither touched V1 nor folded that job into B1-U3; the reminder is carried in `.ai/CURRENT_STATE.md` and repeated
   in the commit's own handoff line.

## 6. Design checklist

`templates/DESIGN_CHECKLIST_TEMPLATE.md` was walked as `B1_VALIDATION/B1_U3_LANGUAGE_DESIGN_CHECKLIST.md`. The points that
carry weight here: no numeric value was invented (every figure is a token or a catalog placeholder), the five states
stay icon + label in both languages, mono still carries numbers/hashes/ids/paths, `Unknown` still renders neutral with
a fact sentence, and no gradient, glow, shadow-on-panel or colour-only state was introduced. Chinese adds no new
typographic scale: it inherits the frozen 1024×720 and 1440×900 measurements and overflowed neither.

## 7. Handoff

The next unit is B1-U4 or the `NEXT-05` correctives, at the Owner's choice; this round proposes nothing beyond it.
Nothing here is pushed. `origin/main` stays at `e0b8fc4`, and the Owner's authorization covered B1-U1's and B1-U2's
ranges only. V1's frozen artifacts expire 2026-10-22 and still need their own private archival.
