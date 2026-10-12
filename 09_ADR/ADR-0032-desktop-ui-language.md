---
title: "ADR-0032 Desktop User Interface Language"
doc_id: "ADR-0032"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-11"
---

# ADR-0032 — The Desktop Interface Speaks Two Languages, and Nothing Else Moves

## Status

**Accepted as a design decision for `B1-U3`, authorized by the Owner's dispatch of 2026-10-11 (`NEXT-06`) and by
`FirmwareSight｜完整产品能力、基础体验与专业工作流决策 v0.2` F02 and §3.**

It adds a third application preference and one version step to the preference document. It changes no product
judgement, no exported artefact and no wire contract outside the preferences command. `ADR-0018` and `ADR-0031` are
not edited here; `ADR-0031` D1's three theme words and `ADR-0031` D2's version rule are the pattern this record
follows, not a rule it revises.

## Context

v0.2 F02 lists `zh-CN/en/System 语言` as 同期必做: a Chinese-speaking firmware engineer must be able to read the tool,
and an international user must be able to operate it, with every main flow, error and Help surface translated, while
"稳定 code/JSON/run id/hash 不本地化" and "资源完全离线". v0.2 §3 fixes the shape of the preference
(`language`: `system` default / `zh-CN` / `en`, 仅本机 UI, 切换即时生效, unsupported platform language falls back to
English) and v0.2 §9 items 1, 6 and 7 make it an exit criterion: both languages complete on the main flows, and
Rust/Core depending on neither the settings file nor the UI language, with CLI JSON and exit codes identical across
two languages and three themes.

`NEXT-06` §1 refuses two easy answers. The catalog decision may not be settled by "zero dependency" as a slogan, nor
by "must be i18next"; and the schema number may not be guessed. Both are decided below with measurements.

## Decision

### D1 — Three words, one default, English as the floor

`LanguagePref { System, ZhCn, En }`, stored as the flat document key `language` beside `size_unit`, `startup_page`
and `theme`, with words `system`, `zh-CN`, `en`. `System` is the default. The UI reads the platform's own language
signal (`navigator.languages`, then `navigator.language`) and normalizes: a tag whose primary subtag is `zh` selects
the Simplified catalog; `en*` selects English; **anything else selects English and says nothing about the platform**.

`zh-HK` and `zh-TW` resolve to the Simplified catalog because that is the only Chinese catalog this product has. This
is stated rather than hidden: the catalog is 简体中文, and no sentence in the UI or in these documents may claim
Traditional support. A reader on a Traditional desktop sees Simplified Chinese and can pin `en` explicitly.

### D2 — A typed catalog in the repository, not `i18next`

The comparison, with the numbers taken on 2026-10-11 from the registry and from this tree:

| | typed catalog (this decision) | `i18next` + `react-i18next` |
| --- | --- | --- |
| New runtime packages | 0 | 5 — `i18next@26.4.3` (MIT, 0 deps) plus `react-i18next@17.0.16` (MIT) which pulls `@babel/runtime`, `html-parse-stringify`, `use-sync-external-store`. The UI ships 3 runtime packages today: `react`, `react-dom`, `@tauri-apps/api`. |
| Missing translation key | **Build failure.** The Chinese catalog is typed as `Record<keyof typeof english, string>` per namespace, and `pnpm typecheck` (`tsc --noEmit`) is already step 5 of the 17-step gate. | Runtime fallback to the key or to English, surfaced by a dev-mode console log; parity requires extra tooling (`i18next-parser`, a typescript plugin) to reach the same guarantee. |
| HTML in translated strings | Impossible by construction: values are plain strings rendered as text children. | `html-parse-stringify` exists precisely to let a translated string carry markup, which v0.2 forbids ("禁止…HTML 直接注入"). Adopting the library would mean documenting a capability the product rules out. |
| Offline | Source modules, bundled at build. | Same, if resources are local — but only if a remote backend is never enabled. |
| Interpolation, plurals, dates, key loading, namespaces beyond nine | `{token}` interpolation only, written here. **No plurals, no ICU, no date/number formatting.** | Complete, mature. |
| Maintenance surface | Two data files of ~300–450 keys each plus a ~120-line resolver, read by the same reviewer as the rest of the UI. | Two data files (same keys, JSON instead of TS) plus init config, a provider, a type plugin and five packages to track. |

The decisive fact is the second row, and it is testable rather than stylistic: this repository's gate already fails on
a type error, so a language that compiles is a language that has every key. `i18next` would move that check from the
compiler to a runtime log or to an added tool, in exchange for machinery this product does not use — there is no
plural in the interface, because the counts are already rendered as `Removed 22` / `22 项移除` with the number
supplied by data, and no date is formatted by the UI at all (timestamps are Core's, shown verbatim).

The cost is owned, not glossed: **grammatical agreement, plural rules and localized number/date formatting are out of
scope for this design.** The `Revisit trigger` below says what would force a change of heart.

### D3 — `SCHEMA_VERSION` 2 → 3, reader widened first

`ADR-0031` D2 established the order and this follows it exactly, because the same trap is still there: `read_document`
selects a shape by version and treats anything below its floor as damaged, so raising the number before widening the
reader would open every existing `schema_version: 1` and `schema_version: 2` document as corrupt and demote a
person's real settings to `settings.json.unreadable` on their next save.

- The reader accepts **1, 2 and 3**. Versions 1 and 2 parse their own shape and gain `LanguagePref::default()` —
  `system` — with nothing written.
- Above this build's own version: `FutureVersion`, refused on save with `ERR-PREFS-8004`, bytes untouched.
- Below 1: damaged, as before.
- A write produces `schema_version: 3` with all five fields. A read never writes; the first save after an upgrade is
  the only thing that moves a document forward.

The version number is not a guess: 2 is what `apps/desktop/src-tauri/src/preferences.rs` carries at this head, and 3
is the next integer in the sequence the file already documents.

### D4 — Nine namespaces, and Overview is named

`common`, `analyze`, `compare`, `gate`, `release`, `history`, `settings`, `help`, `errors`. v0.2 §97 lists exactly
these and leaves Overview unplaced; this record places it: **Overview reads `common` plus the `gate`/`analyze`
vocabulary it already borrows from Core's words, and gets no tenth namespace**, because a namespace that duplicates
two strings is how two screens start disagreeing about one word.

Keys are dotted, lowercase, semantic (`release.bundle.limits`), never the English sentence itself — an English
sentence as a key means a copy edit is a schema change. Two checks run in CI: every English key has a Chinese value
and every Chinese key exists in English (compile-time, D2), and every `{placeholder}` set matches across the two
languages (a test that walks both catalogs). No translated value is ever concatenated into a sentence: a string that
needs a variable takes a named placeholder, which is also what keeps a file name, an error code or a real engineering
number intact.

**Details reads `analyze`, and a tenth namespace was proposed during implementation and refused.** The detail tables
are the same analysis's rows seen from another angle; a screen that switches from the report to its own tables must not
cross a vocabulary boundary, and a namespace holding one page's three strings is where two screens start disagreeing.
The one shared sentence that could not live in either place is the pager line: `components/Table.tsx` owns
`common.pager.*` and the calling page supplies the noun, because a Chinese noun inside an English sentence is two
languages in one line. A DOM id built from that noun's English word stays English — an id does not change language.

### D5 — What does not translate, in either direction

`PASS` / `REVIEW` / `BLOCK` / `UNKNOWN` / `N/A` and `Observed` / `Derived` / `Declared` / `Unknown` stay exactly as
Core emits them — they are vocabulary, not prose, and `stateWords.ts` already refuses to invent a third mapping.
Chinese may *explain* a state beside the word; it may not rename it. Also untranslated, in both languages: hashes,
snapshot / run / release ids, addresses, sizes and other numbers, source paths, compiler symbols, section names,
rule ids, project and build names, error codes, and every exported document.

Three consequences of that rule were decided while implementing, and are named here because each one looks like an
oversight to a reader who has not met it:

- Compare's filter chips keep `Added` / `Removed` / `Changed` in Chinese, because the cells they filter render Core's
  own word. A translated chip that selects an untranslated row is a control that lies about what it filters.
- The evidence-class `<option>` labels stay `observed` / `derived` / `declared` / `unknown`, because the value sent to
  Rust through that select *is* the word. Translating the label would make the control read one thing and send another.
- The language control's own options are `System` / `English` / `简体中文` in both catalogs. A language is named in its
  own language so a reader who cannot read the interface can still find the one they can.

`format.ts` keeps emitting `Unknown`, `bytes` and `KiB` outside the catalog for the same reason: they are the state
word and the two units, not prose.

### D6 — Errors are keyed by code, and the engine's own words stay visible

A Rust `message` is never translated by splitting or re-parsing it. The UI composes its operational sentence from
`errors.<CODE>` with controlled parameters, and keeps three things on screen regardless of language: the code
(monospace, selectable), the diagnostics id (monospace), and the engine's own message under a label that says it is
the engine's technical detail. An unknown code falls back to a factual sentence — FirmwareSight reported this code —
followed by the same technical detail, which is accurate rather than reassuring. No host absolute path crosses into
any of it, which `ADR-0018`-era privacy rules already hold for the desktop shell.

This is the boundary that keeps `NEXT-06` §20 and the STOP condition about "大量暴露未经翻译的英文操作文案" from
contradicting each other: **window chrome, section headings, labels, buttons, empty and loading states, and the
per-code diagnosis are translated; the engine's own message, details and remediation are displayed, labelled, and not
rewritten.** The line is drawn there and not further: a translated *remedy* per code would mean writing the UI's own
second copy of Rust's remediation table, which is what `error.rs` warns against, so a Chinese window shows the
engine's next step in English under a Chinese label, and this round reports that as a limit rather than describing
every visible line as localized.

**Two consequences of that boundary had to be decided rather than discovered at review time.**

1. *The Rust tables stay the single source.* `crates/firmwaresight-artifact/src/error.rs` says its `user_message()` is
   shared by both surfaces because "a second copy of this table in the UI is how a parse failure starts meaning two
   different things". A catalog keyed by code does not answer that objection by re-authoring the engine's English
   sentence: it adds a line in the reader's language and leaves the engine's line on screen beside it. Because both
   are shown for the same envelope, a divergence between them is visible on the panel instead of silent — which is
   what makes a per-code UI sentence safe to hold in the front end at all.
2. *Coverage is counted, not claimed.* Every code Rust can produce has one diagnosis line in
   `apps/desktop/ui/src/i18n/en/errors.ts` — measured at 59 codes in `crates/` plus `apps/desktop/src-tauri/src`, and
   the same 59 keys in the catalog, with no code in one list and missing from the other — and `errors.unknown` covers
   anything a later build emits that this one has never seen. The commands that produced that count are recorded in
   the round's report, because "every code is covered" is the kind of sentence that goes stale silently.

Where one code covers two conditions — `ERR-INPUT-0001` is both "no artifact at that path" and "that selection is no
longer available" — the catalog sentence states the fact they share and the engine's own line carries the difference,
rather than the UI guessing which of the two happened.

### D7 — Language never reaches a fact

`language` is read by the React shell and by nothing else. It does not enter Core, `GateContext`, snapshot identity,
`policy_sha256`, Gate run id, release id, a bundle manifest, a portable report, CLI JSON, an exit code or an error
code. The desktop's own isolation test stores all four preferences and asserts the two Gate runs answer with one
identity, as `ADR-0031` D7 does for the palette. Goldens and the self-contained `release-report.html` keep their
existing fixed language and byte identity: a person's UI language must not change what another person receives.

### D8 — It switches live, persists across a restart, and says which it is

The choice applies immediately (v0.2 §3), is stored on the same command as the other three, and survives a restart.
`<html lang>` follows the resolved language, because a screen reader told `en` while reading Chinese is a defect, not
a detail. A `NEXT-05` lesson is carried deliberately: `ADR-0031` D4 claimed a palette lands before first paint and
`NEXT-05` measured that it does not. This record makes no such claim. The language is applied when the shell mounts,
after the stored document has been read, and any pre-mount text is English — the build's own source language. That is
the contract, and it is what the tests assert.

**`system` is live for colour and not live for language, and the two are not the same promise.** A palette has a host
signal a window can subscribe to (`matchMedia` emits on an OS theme change); a desktop language has none — no browser
event reports a `navigator.languages` change while a page is open. So `system` re-reads the platform at two moments
and says so: when the shell mounts, and when the choice returns to `system`. A person who changes their desktop
language sees the interface follow at the next start. This is the honest contract `NEXT-06` §A demands be recorded
rather than papered over, and the Settings sentence is written to describe the language this window resolved to, not
to claim it is tracking the desktop.

## Alternatives

**`i18next` / `react-i18next`.** Credible, mature, MIT-licensed, and the option v0.2 permits. Rejected on the two
measured facts in D2 — five new runtime packages into a three-package UI, one of which exists to parse markup the
product forbids — and on the fact that the guarantee this product needs (no key may be missing) is stronger as a
compile error than as a runtime fallback. Reopening it is cheap: the catalogs are plain data and the call sites use
one `t()` function.

**Server-side or downloaded locale resources.** Rejected outright: v0.2 requires 完全离线, `AGENTS.md` 7 forbids new
host access, and the product has no network path to fetch from.

**Status quo — English strings inline, Chinese added per page later.** Rejected: it is how a product ends up with two
UIs that disagree about what a state means, and it fails v0.2 §9 item 1 on the first read.

**Storing the resolved language instead of the choice.** Rejected for the same reason `ADR-0031` D3 rejects storing a
palette: it would turn every desktop language change into a preference write and would silently overwrite a choice
the person actually made.

## Consequences

Positive: a Chinese firmware engineer can run every main flow in their own language, including error recovery and
Help; a missing translation cannot ship, because it will not compile; the local-first and offline boundary is
untouched; and no judgement, identity or exported byte depends on who is reading the screen.

Negative, and accepted: two catalogs of several hundred keys each are real maintenance, and every new string now has
to be written twice — which is the correct cost, not a tax. No plural or ICU machinery, so a future string that needs
grammatical agreement will expose this design's limit. The engine's technical detail stays English in a Chinese
interface, which is honest and inspectable but not fully translated. And the UI's test suite, which asserts English
copy in 819 places, now depends on the resolver landing on English by default; that dependency is deliberate and is
asserted directly rather than assumed.

## Revisit trigger

Reopen D2 when any of these is true: a third language is authorized; a string needs plural or gendered agreement that
`{count}` cannot express; translated text ever needs to contain markup; the catalog's key count makes a full
type-check slower than the gate can tolerate; or an audit finds missing keys reaching users despite the compile-time
parity. Reopen D1 only when a real reader on a Traditional-language desktop asks for one — at which point the answer
is a second Chinese catalog, not a claim about the first.
