---
title: "B1-U2 System / Light / Dark Theme — Execution Report"
doc_id: "FS-B1-U2-REPORT-001"
product: "FirmwareSight"
version: "0.6.0"
status: "REPORT"
owner: "Engineering"
last_updated: "2026-10-11"
---

# B1-U2 — System / Light / Dark, on the mechanism B1-U1 already built

Executed on 2026-10-11 under `NEXT-04` (archived at
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_NEXT_04_B1U2_System_Light_Dark_Theme_2026-10-11.txt`, 8,193 bytes, 43 lines,
0 `CR`, sha256 `122469aa78adca3b24d1ae8101354ee797fd683f3c0e7772da818e937431dd92`), whose three start gates were
closed in order earlier the same day: B1-U1's independent `ACCEPT_WITH_EXPLICIT_LIMITS`
(`B1_VALIDATION/B1_U1_PREFERENCES_SETTINGS_REPORT.md` §10), the Owner-authorized fast-forward `47fde34..2cff702`
read back as CI run `38124118299`, and the design decision `ADR-0031`. The Owner's dispatch is the four-step
instruction of 2026-10-11: accept, push, decide the theme, then execute this unit — with no separate UI art pass
and B1-U3 to follow.

Status word this round proposes, and does not grant: **`LOCAL_B1_U2_GATE_PASS / ARCHITECT_REVIEW_PENDING`**.

## 1. changes — exact paths and intent

| Path | Change | Intent |
| --- | --- | --- |
| `apps/desktop/src-tauri/src/preferences.rs` | `SCHEMA_VERSION` 1 → 2; `ThemePref`; `SettingsFileV1`; `read_document` widened; `render`; `update_application_preferences` third word | Store the third choice without losing the first two |
| `apps/desktop/src-tauri/src/ipc.rs` | `PreferencesDto.theme` | Carry the chosen word, not the resolved palette |
| `apps/desktop/src-tauri/tests/preferences.rs` | 6 new cases + `prefs_all` | Prove the v1 read, the v2 write, the round trip, the new ceiling, and the hybrid refusal |
| `apps/desktop/src-tauri/tests/release_gate.rs` | the isolation case now stores a theme too, and checks all three words landed | T06: a palette is not evidence |
| `assets/design-tokens.json` | `color.dark` (30 roles), theme-relative focus reference, meta 0.2.1 → 0.3.0, changelog | The authority holds both palettes; no light value moved |
| `scripts/generate_design_tokens.py` | emits `:root` + `[data-fs-theme="dark"]`; measures 18 pairs per theme; refuses a palette whose role set differs | The gate, not a reviewer's eyes, decides contrast and parity |
| `apps/desktop/ui/src/theme.ts` | new | One place for choice vs. palette, testable without a window |
| `apps/desktop/ui/src/theme.test.ts` | new, 7 cases | The resolution rules, apart from React |
| `apps/desktop/ui/src/App.tsx` | theme state, `matchMedia` subscription, apply-before-paint effect, `chooseTheme`, reset re-applies the theme | The shell owns the palette like it owns the unit |
| `apps/desktop/ui/src/Settings.tsx` | three radios + a fact sentence; reset copy corrected | A control that does something, described truthfully |
| `apps/desktop/ui/src/styles/global.css` | `color-scheme` per theme | The native part of first paint |
| `apps/desktop/ui/src/ipc/bridge.ts`, `ipc/generated/PreferencesDto.ts` | third argument; regenerated binding | The wire matches the command |
| `apps/desktop/ui/src/settings.test.tsx` | 8 new theme cases; 3 existing assertions extended; the "no theme control" guard narrowed | The old contract updated because the requirement moved, not because it failed |

No Core, CLI, Gate, storage-schema, fixture, golden or schema file was touched. `ADR-0018` was not edited.

## 2. The two decisions that were not mechanical

**The version bump nearly shipped a data-loss experience.** `read_document` refused any version that was not
exactly its own, so raising `SCHEMA_VERSION` alone would have opened every existing user's file as *corrupted*:
defaults on screen, "the stored settings could not be read", and their working preferences demoted to
`settings.json.unreadable` on the next save. Nothing would have been destroyed, which is precisely why it is hard
to see — the failure presents as a recovery path that works. The reader is widened first (accept 1 and 2, resolve a
missing `theme` to `system`, rewrite nothing on read), and the writer only then moves. `ADR-0031` D2 records the
order; the tests record the behaviour.

**Contrast turned out to be owned by an export.** Measuring the frozen light palette found one pair under the floor:
`diff.removed` on `diff.removed.bg` at **4.4974:1** against 4.5, rendered today as Compare's "Removed" chip. The
value that clears it is `#FCEEEC`, and this round tried it. `crates/firmwaresight-report`'s own contract test,
`html_export_colours_come_from_the_frozen_token_set`, failed the gate: `#FBECEA` is printed into
`golden/reports/p2-diff.html` and `golden/reports/p4-release/release-report.html`, so the fix moves two committed
report artefacts — which `ADR-0031` D7 and `NEXT-04` §D both put outside this round. The token was reverted, the
floor stayed at 4.5, and the exception is enumerated in the generator with its measurement and its reason. The
generator also fails if the pair ever *passes*, so the exemption cannot outlive its reason quietly. This is an open
item for the Architect, not a closed one: **clearing it is a decision about two goldens.**

## 3. What is genuinely working

- Three words, one default, in the same document and the same command as the other two. No second mechanism.
- `system` follows the desktop live; `light`/`dark` ignore it and say so in the same sentence.
- A v1 document opens unchanged and gains `system`; the first save writes v2 with all three fields.
- The focus ring follows the palette: measured `#2563EB` in light and `#6BA1F5` in dark in the running window, from
  one theme-relative reference rather than a second constant.
- Every colour literal in the desktop UI still lives in the generated `tokens.css` — 62 of 62, none in a component
  stylesheet — so dark was a data change and not a sweep. That is `ADR-0018` D5 paying for itself.
- Nothing in the exported surface moved: goldens unchanged, `drift/goldens unchanged` green, and the Gate answered
  with the same identity in both palettes.

## 4. evidence — `NEXT-04`'s matrix, each with what decides it

| Row | Verdict | What decides it |
| --- | --- | --- |
| T01 system / light / dark, live follow, override, return | **PASS** for the choice logic; **`NOT_VERIFIED` for a real desktop flip** | `theme.test.ts` (7 cases: named palette ignores the platform, `system` follows, no-answer falls back to light); `settings.test.tsx` `keeps following after the desktop changes` and `stops following the desktop the moment the reader names a palette`, which drive a `matchMedia` subscription and fire its listener; in the real window, cold starts in both palettes with the OS genuinely in light, and the page reporting "System is following your desktop, which is set to light." Flipping the *actual* Windows setting was not done: it changes the Owner's desktop for the sake of a screenshot. |
| T02 persistence across pages and restarts, and the recovery contract | **PASS** | Real window: `theme: dark` stored, process killed, relaunched, attribute and `color-scheme` dark on the first read with no "saved" claim. Recovery paths unchanged from B1-U1 and re-run: `a_missing_file_loads_the_defaults…`, corrupted, future-version, `nowhereToStore`. |
| T03 light regression + dark parity at 1024×720 / 1440×900 | **PASS** at both sizes, real Win32 resize, **`NOT_VERIFIED` at 125% / 150%** | `target/n04_shots/`: `gate_findings_light.png` and `gate_findings_dark.png` at the same scroll offset on the same run — same wrap points, same rows, only colour differs; `release_gate_dark_1024x720.png`, `release_gate_dark_1440x900.png`, `settings_dark_1040.png`, `compare_dark_diff_1040.png`, `analyze_dark_coldstart.png`. Windows display scaling was not changed on the Owner's session, so the scaled case is not claimed. |
| T04 five states, diff rows, focus, disabled, overlays in both palettes | **PASS** | Photographed in dark: `⊘ Block 0 · ⚠ Review 0 · ◌ Unknown 2 · ✓ Pass 2 · — Not applicable 6`, each icon + label, Unknown hollow and neutral rather than accent. Diff chips and old/new/delta read in `compare_dark_diff_1040.png`. Focus ring measured through the real keyboard path in both palettes. |
| T05 WCAG 2.2 AA, reproducible | **PASS with one named, measured exception** | `python scripts/generate_design_tokens.py --contrast` prints 18 pairs × 2 palettes with four-decimal ratios; `check_palettes` fails the build on any new miss *and* on a stale exemption. Light: 17 pass, `diff.removed`/`diff.removed.bg` exempt at 4.4974. Dark: 18 pass, lowest enforced pair 5.4228. Forced-colors / high contrast **`NOT_VERIFIED`**; `prefers-reduced-motion` is inherited from B1-U1's global rule and was not re-photographed. |
| T06 a theme moves no analysis, identity, verdict or byte | **PASS** | `a_stored_preference_moves_no_gate_identity_no_verdict_and_no_policy_fingerprint` now stores `kib + history + dark` and asserts all three words are in the document before comparing two Gate runs. In the real window the Gate was run in light and again in dark: the same `gate-d19c48891d7931118d3cee622f7cf7ccaed02467d0a62b17ec3d770b25245d74`, the same REVIEW 7 / UNKNOWN 2 / PASS 13 / N/A 8. `drift/goldens unchanged` green. |
| T07 old file reads, rapid switching, unwritable folder | **PASS** for the first two; **unchanged limit** on the third | Real window: a hand-written `schema_version: 1` document opened, rendered KiB, and was *not* rewritten — still 55 bytes with no trailing newline — until a save produced v2. Six rapid alternating saves (B1-U1) still settle on the last choice. An ACL-read-only config folder is still not covered by an automated test; the Windows refusal itself was re-confirmed out-of-band with `icacls /deny` in a gitignored sandbox and then reset. |
| T08 the whole gate | **PASS** | `python scripts/check.py` → **17/17**, exit 0, on the staged tree: **1,050 Rust passed / 0 failed** (was 1,043; +7), **334 UI in 11 files** (was 319 in 10), `cargo-deny` advisories/bans/licenses/sources ok, baseline verifier PASS. Raw log: `target/n04_check_final.log`. |
| T09 real clicks and keys in a real window | **PASS as `DEV_RUNTIME_VERIFIED`** | `target/debug/firmwaresight-desktop.exe` at this head, driven over WebView2 CDP: theme radios clicked and arrow-keyed, `Tab` order walked (rail → General → units → theme → start page → reset, entering each radio group at its checked option), confirmation dismissed, screenshots viewed. **Not** an installed artifact — no CI-built installer exists for any B1 head — so this is not `INSTALLED_VERIFIED`. |
| T10 the boundary held | **PASS** | No new dependency, no Tauri plugin, no capability change, no network, no telemetry; `prefers-color-scheme` is a standard media query. The command count stays 37 and `drift/ipc bindings` regenerated exactly one file. |

## 5. findings reported, and the two this round had to fix

1. **Fixed here — a reset restored two of three settings.** `resetPreferences` re-applied `sizeUnit` and forgot
   `theme`, so a reset left the document on `system` while the window stayed dark until the next start: a page
   reporting a state it was no longer in. The new reset case caught it as a red assertion, not as a review comment.
2. **Fixed here — two sentences the change made false.** The Reset section still read "Puts the **two** settings
   above…" and the confirmation "Reset **both** settings to their defaults?", and the confirmation listed only units
   and start page. Now "every setting", "all three", and the theme is named among what is about to be discarded —
   asserted in the test, so the copy is load-bearing rather than decorative.
3. **Open, needs a decision — the 4.4974:1 pair.** See §2. Reverting the token was the correct call for this round
   and leaves a real, measured AA shortfall in the light palette that predates it.
4. **Open, inherited from `NEXT-03` — focus falls to `<body>`** when the inline reset confirmation is dismissed, in
   either direction, and `Escape` does not dismiss it. Unchanged by this round; still a keyboard-path gap.
5. **Open, inherited — the window's 720×480 minimum** in `tauri.conf.json` is below the 1024×720 floor v0.2 states.
6. **Not a defect, recorded so it is not mistaken for one** — the first `tabto` probe after a Vite hot reload
   reported focus on `body` for every press. Re-run against a stable page it walked correctly; the cause was the
   hot-reloaded document, not the app.

## 6. what this round deliberately did not do

No UI concept art, no redesign pass, no new component library, no per-page or per-window theme, no density or font
scale, no language control (that is B1-U3, still needing its own dispatch), no change to any exported artefact, and
no edit to `ADR-0018`. The seven accepted UI reference screenshots stay light and stay authoritative for layout and
density; `ADR-0031` D8 narrows what they are authoritative for, in writing.

## 7. git

- Base: `2cff702` (B1-U1's reviewed head, pushed and CI-green as run `38124118299`).
- This round is **two local commits, and neither is pushed**: `bf3d494` carries the authority — `ADR-0031` plus the
  current-spec surfaces it authorizes (`DESIGN.md` §4/§9, `AGENTS.md` 11, `BASELINE.yaml` `design.theme`) and the three
  archived prompts — and `bd62289` carries the unit: 21 source/token/generator paths and every report and governance
  record in this file. `NEXT-04` preferred at most two commits and that is what it got, with the ADR deliberately
  first: no theme code landed under a rule the documents still forbade.
- This paragraph names `bd62289` from inside `bd62289`, which is impossible, so it was written after: **the first
  version of this section said "one commit", and that was already false when it was written.** The correction is a
  docs-only successor of its own rather than an amend, because the alternative to stating a mistake is rewriting the
  commit that made it.
- Manifests regenerated from the staged index in `ADR-0029` order; `scripts/verify_baseline_artifacts.py` → **PASS at
  820 tracked / 818 sum entries / 949 tree lines**, against the 812 / 810 / 941 recorded at B1-U1's closure. The +8 in
  all three is exactly this round's eight new files — three archived prompts, `ADR-0031`, `theme.ts`,
  `theme.test.ts`, this report and its checklist — with `index blob mismatch`, `tracked but unlisted` and
  `listed but unindexed` all at 0.
- `origin/main` is still `2cff702`. The Owner's push authorization covered B1-U1's range and nothing beyond it, so this
  round has no remote CI of its own to read back, and none is claimed.
- **The raw evidence is not in the baseline.** Every screenshot and log this report cites — `target/n04_shots/*.png`,
  `target/n04_check_final.log`, `target/n04_check_docs_final.log`, the CDP driver and the resized-window PowerShell
  script — lives under `target/`, which `.gitignore` already excludes from the tracked set. They are reproducible by
  the tests named in §4, not downloadable from this repository, and the report does not claim otherwise.
- The gate was re-run on the amended tree before this sentence was committed, and it is the same 17 of 17 with the same
  1,050 / 334: a docs-only successor moves digests, not behaviour. That successor is the commit which contains this
  paragraph; its own hash cannot appear in the paragraph it creates, and no hash is invented for it here.

## 8. next

- **B1-U3 (zh/en)** is next in the Owner's sequence and needs its own dispatch; B1-U2's strings are inline in
  components, in the same habit every other page already has, which is what B1-U3's report will have to translate
  as one set.
- The Architect's open questions from this round, in priority order: the 4.4974:1 exemption and the two goldens it
  is tied to; the dismissed-confirmation focus loss; whether 125% / 150% scaling and a real desktop-theme flip are
  required before a Beta claim, since neither is verified here.
- An installed-artifact pass for B1 is still owed: every B1 unit so far is `DEV_RUNTIME_VERIFIED`, because no CI
  build of any B1 head has been installed.
