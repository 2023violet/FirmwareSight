---
title: "B1-U3 Design Checklist"
doc_id: "FS-B1-U3-DESIGN-001"
product: "FirmwareSight"
version: "0.6.0"
status: "REPORT"
owner: "Design"
last_updated: "2026-10-11"
---

# B1-U3 — design checklist for the second language

`AGENTS.md` 11 binds every UI change PR to `templates/DESIGN_CHECKLIST_TEMPLATE.md`. This round changed which
words a screen says and nothing about how a screen looks: no stylesheet, no token, no component geometry and no
colour was touched. Every ⚙ line below was run as a command against this tree on 2026-10-11 and the command is
printed beside the verdict, so the next reader re-runs it instead of trusting this file.

## What is on screen

| Surface | Change | Kind |
| --- | --- | --- |
| `Settings.tsx` → Appearance | one more radio group (`System` / `简体中文` / `English`) below the theme group, with the sentence that names what the choice does not change | new control group |
| every page and shared component | the same text, now read from a catalog, and a Chinese catalog beside it | copy source |
| `components/Table.tsx` | the pager sentence and the sort label are read from `common`; the noun inside the sentence comes from the calling page | shared component |
| `components/ErrorPanel.tsx` | the diagnosis is looked up by stable error code, and the engine's own words stay visible under a label | rebuilt panel |
| `<html lang>` | follows the resolved language | document attribute |

No `.css` file, `assets/design-tokens.json`, `StateBadge.tsx`, `format.ts`, layout rule, spacing, radius,
typography, table geometry, focus geometry, motion or shadow changed in this round.

## Don'ts (一票否决) — none hit

| Line | Verdict | Command |
| --- | --- | --- |
| ⚙ No gradient / glass / neon | 0 hits in the stylesheets, and none added | `grep -rniE "linear-gradient\|backdrop-filter" apps/desktop/ui/src --include=*.css` → 0 |
| ⚙ No colour literal outside the generated file | 0 added | `git diff --cached -- apps/desktop/ui/src \| grep -E "^\\+" \| grep -oiE "#[0-9a-f]{3,8}"` → empty |
| ⚙ No stylesheet touched at all | none | `git diff --cached --name-only \| grep -E "\\.css$\|design-tokens"` → empty |
| ⚙ Two-theme compliance (`ADR-0031`) | unchanged; the gate's own token step still refuses a contrast miss or a stale exemption | `scripts/check.py` step `drift/design tokens` → PASS |
| State is never colour-only | unchanged — `StateBadge` was not edited, and the five words still render icon + label in both languages | live window: badge tally identical in `en` and `zh-CN` (`REVIEW` 1, `UNKNOWN` 2, `PASS` 2, `N/A` 8) |
| ⚙ No invented value | every figure on a screen is a token or a catalog placeholder; no number was baked into a copy string | `grep -nE "': '[^']*[0-9]" apps/desktop/ui/src/i18n/*/*.ts` reviewed; the only digits are inside machine words (`SHA-256`, `schema_version`) |
| UI does not re-implement parser / diff / gate | this round added no derivation; `stateWords.ts` only gained a parameter so one run cannot be worded two ways | `git diff --cached --stat` on `stateWords.ts`: signature and call sites, no new computation |
| Forbidden marketing register | 0 hits | `grep -riE "guaranteed\|fully compliant\|zero risk\|intelligent detection" apps/desktop/ui/src/i18n/` → 0 |
| No host path reaches the WebView | 0 catalog value carries a drive letter, a URL or a separator-shaped string | `grep -rnE "'[A-Za-z]:\\\\" apps/desktop/ui/src/i18n/*/*.ts` → 0, and `settings.test.tsx` asserts the refusal |

## Do's that this round actually had to answer

| Line | Verdict | Evidence |
| --- | --- | --- |
| Five states complete and correct | PASS in both languages: the words are Core's and are never renamed, only explained | `catalog.test.ts` "keeps the machine vocabulary in both languages"; the Getting-started panel counts `PASS` 1 / `REVIEW` 1 / `BLOCK` 1 / `UNKNOWN` 2 / `N/A` 1 identically in the live window in both languages |
| `Unknown` stays neutral, with a fact sentence and a next step | unchanged wording, now in two languages | `overview.unknownValue` / `details.unknown` keep the word; the sentences around them are catalogued |
| Error shows What happened / Why we know / What to do / Diagnostics ID | PASS, and the four parts survive the language switch | `ErrorPanel.tsx` plus the `errors` namespace (59 codes of 59 measured in Rust, plus the `unknown` fallback that says it is unknown) |
| Mono for numbers, hashes, versions, timestamps, paths | unchanged: `format.ts` was not edited, and no catalog value carries a number that should be mono | the mono classes are on the elements, not in the strings |
| Empty state offers an exit | unchanged behaviour, translated | `overview.emptyAction` / `intake.chooseArtifact` in the live window: 选择镜像文件, 选择固件镜像文件 |
| Text stays readable at the frozen sizes | PASS at inner 1024×721 and 1440×901, both languages, all seven pages: `scrollWidth == clientWidth` on the document and on `main` | `target/n06_pages_zh.json`, `target/n06_pages_en.json`, 15 captures in `target/n06_shots/` |
| No markup in a copy string | PASS | `grep -nE "': '[^']*[<>]"` over both catalogs → 0, and `catalog.test.ts` "carry no markup" |
| No sentence assembled from fragments | PASS by construction: one value per sentence with `{placeholder}` parameters, and the placeholder sets are compared key by key | `catalog.test.ts` "same placeholders in both languages" and the 918-key walk |

## One observation, recorded rather than fixed

At the 1024 floor the History builds table is wider than its wrapper in **both** languages, so 非易失 and RAM sit
behind the wrapper's own horizontal scroll — the behaviour `history.test.tsx` protects. Chinese makes those two
cells taller (精确值 above a wrapped byte count) without clipping anything outside the scroll area. It is not a
regression and no layout rule was changed to chase it; it is written here because the screenshots show it.
