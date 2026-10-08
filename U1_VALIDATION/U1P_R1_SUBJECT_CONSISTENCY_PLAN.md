---
title: "U1P-R1 Overview gate-subject consistency: correction plan"
doc_id: "FS-U1P-R1-001"
product: "FirmwareSight"
unit: "U1P_R1_OVERVIEW_GATE_SUBJECT_CONSISTENCY"
written_before_code: true
start_head: "a2e6b3030163d36ad1d67252fd55d5ea9ab34d6c"
authority: "AGENTS.md 3 and 11 + DESIGN.md + assets/design-tokens.json (frozen) + ADR-0018 precedence chain + U1_VALIDATION/U1P_VISUAL_ACCEPTANCE_REPORT.md"
---

# FS-U1P-R1-001 — what is going to change, and what is not

Prompt: `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R1_Overview_Gate_Subject_Consistency_v1.0.txt`
(delivered 33,484 bytes / SHA-256 `0263e0fe92b7065fcc6a8cf1280630ff753eb2e4415ecb07fd40c72bd8e8d024`; the file
arrived with LF endings, so the stored blob `df5b2d0dcc28e2438861c26e00ae5b0369a3e8a3` hashes to the same value —
measured with `git show :path | sha256sum`, not assumed).

This closes **U1P-V2-01**. It is a subject-binding and presentation-correctness fix, not a redesign: U1P's six
first-viewport compositions and U1R's parse-failure state consistency stay exactly as the Architect accepted them.

## 1. The two things the page describes, and who owns each

| Fact | Field | Owner | Moves when |
|---|---|---|---|
| The artifact held by the shell right now | `App.selection: SelectionDto \| null` → `SelectionDto.selectionId` | `App.tsx:115`, written only by `Analyze`'s `onSelectionChange` | a file is chosen, a MAP attached, a MAP removed |
| The analysis the session last proved | `App.lastGood: AnalysisSummaryDto \| null` → `lastGood.identity.snapshotId` | `App.tsx:116`, written only by `onLastGoodChange(outcome.value, selection.selectionId)` on a successful `analyzeSelection` | an analysis **succeeds** — never on failure (U1R's contract) |
| Which selection earned that analysis | `App.analyzedSelectionId: string \| null` | `App.tsx:118`, set in the same call as `lastGood` and never on its own | exactly when `lastGood` moves |
| The last Gate run this session made | `App.gateRun: GateRunDto \| null` → `gateRun.snapshotId` (the judged target) | `App.tsx:120`, set only by `Release`'s `onRunChange` | a run is made or an acceptance is recorded |
| The policy loaded now | `App.project: ProjectContextDto \| null` → `project.policySha256` | `App.tsx:119`, set only by `Release`'s `onProjectChange` | a project policy is loaded |

`Analyze` already applies the same identity rule for its own pending badge at `Analyze.tsx:136`:

```ts
const pendingSelection = selection !== null && selection.selectionId !== analyzedSelectionId;
```

The defect is that **`App.tsx` passes `analyzedSelectionId` to `Analyze` (line 167) and not to `Overview`
(lines 151–160)**, so `Overview` cannot ask the one question its dominant element depends on: *is the run in my
hand about the build I am describing?* `Readiness` (`Overview.tsx:174–269`) renders the verdict strip whenever
`gateRun !== null` and reads `summary` only for the MAP-attach hint. That is the whole root cause, and it is a
missing input to a presentation decision — not a wrong calculation, not a missing domain fact.

## 2. The authoritative identities, and what must never be used instead

Available today, from generated DTOs, with no new field and no new command:

- `GateRunDto.snapshotId: string` — **the judged target**. `snap-<artifact sha256>-<normalization version>`
  (`crates/firmwaresight-core/src/domain/build_snapshot.rs:35`).
- `AnalysisSummaryDto.identity.snapshotId: string` — the analyzed build, same grammar.
- `GateRunDto.baselineSnapshotId: string | null` — the **comparison base**, not the judged target.
- `GateRunDto.runId`, `createdAt`, `counts`, `findings`, `overallEffectiveSeverity`,
  `dispositionEffectiveSeverity` — the run's own facts, taken as they are.
- `GateRunDto.policySha256: string` and `ProjectContextDto.policySha256: string` — both real 64-hex fingerprints
  (`apps/desktop/src-tauri/tests/release_gate.rs:435` asserts the length; a default-policy run fingerprints the
  default policy, `release_gate.rs:1437`).
- `GateRunDto.projectName: string | null` — `null` means the run's policy is identified **by fingerprint only**
  (a default run, or a record read back from history — see the `PolicyLabel` branch at
  `apps/desktop/src-tauri/src/release.rs:377`).

Forbidden as substitutes, all of them, because each can be equal while the subjects differ: truncated ids,
`artifact.fileName`, `createdAt`, `git.headCommit`, `artifact.byteSize`, `baselineSnapshotId`. The comparison is
`gateRun.snapshotId === summary.identity.snapshotId`, full values, and the baseline is never the target.

## 3. The presentation state, derived read-only in one place

One derived value, no new state, no persistence, no recomputation of anything Core decides:

```
scope(gateRun, summary, selection, analyzedSelectionId, project) ->
  | 'noRun'              gateRun === null
  | 'selectionPending'   selection !== null && selection.selectionId !== analyzedSelectionId
  | 'noAnalysis'         summary === null
  | 'otherBuild'         gateRun.snapshotId !== summary.identity.snapshotId
  | 'otherPolicy'        project !== null && project.policySha256 !== gateRun.policySha256
  | 'current'            otherwise
```

Precedence and why: a run that does not exist is a different emptiness from a run about something else, so
`noRun` is first. `selectionPending` beats `otherBuild` because it is a claim about what the reader is **holding
now** — and both branches are neutral, so nothing is lost by ordering; the subordinate block still names both
ids. `noAnalysis` before `otherBuild` because with no analysis in the session there is no current snapshot to
mismatch against. `otherPolicy` last among the neutral states, because a policy mismatch only modifies the
answer once the subject question has been settled.

| State | Prompt | Prominent answer | What else is on screen |
|---|---|---|---|
| `current` | §5.A | the run's own PASS / REVIEW / BLOCK / UNKNOWN strip, unchanged | **full judged snapshot id inside the verdict region**, run id + timestamp kept, counts kept, accepted-review wording kept, scope sentence kept |
| `otherBuild` | §5.B | neutral `UNKNOWN` badge — *Not assessed for this build* | fact sentence naming **both** the judged id and the displayed id; the retained run's verdict shown **subordinate and labelled for the other build**; CTA navigates to Release Gate; scope sentence kept |
| `selectionPending` | §5.C | neutral — *Not assessed for the selected artifact* | "selected X, not analyzed yet"; retained last-good analysis and the old run stay visible, unmistakably scoped to their own build; nothing cleared |
| `noAnalysis` | §5.D | neutral — *No analysis in this session* | "a stored run judged *<full id>*, and there is no analyzed selection here to pair it with"; Release Gate stays reachable |
| `otherPolicy` | §5.F | neutral — *Not assessed under the policy loaded now* | the run's own policy fingerprint and the loaded one, both stated as they are; the old result retained, never re-labelled; no invented policy, no new query |
| `noRun` | §5.E | unchanged empty state | unchanged navigation; no synthesized PASS, no zero counts presented as a run |

§5.H is the invariant all five neutral rows obey: **no unqualified current PASS, and no verdict sentence at all,
in any neutral state.** §5.G is untouched by construction: `Release` renders the run it made (`Release.tsx:254`
feeds `onRunChange` from its own `snapshotId` choice), `History` reads stored rows with their own ids, and only
`Overview` changes.

## 4. Smallest truthful shape, deliberately chosen

- **The full judged snapshot id is rendered visibly, not hidden.** `snap-<64 hex>-<version>` is ~72 characters;
  at the metadata size in `Overview.module.css` `.inputDetail` (already mono, secondary colour,
  `overflow-wrap: anywhere`) it is one wrapped line inside the panel. This satisfies §5.A directly — a value that
  is never truncated needs no recovery path — and it means **no screen-reader-only class, no `title`-only
  channel, and no new CSS for the subject line.** Hover-only text is not "retrievable" for a keyboard reader, so
  it was rejected on that ground rather than on taste.
- The neutral answer reuses `StateBadge variant="chip" state="UNKNOWN"`, whose hollow dashed glyph in neutral grey
  is the design system's own "cannot be evaluated" mark (`StateBadge.tsx:106-119`); `N/A` is wrong here because
  this is not a decision to omit an input, and `REVIEW` would be a Gate word this page is not entitled to use.
- Findings lists, counts and the accepted-review sentence move **inside** the subordinate retained-run block in
  the neutral states, still generated from the same `gateRun` fields, so nothing is deleted or falsified.
- One new CSS class, `.pastRun` (hairline top border + secondary colour + metadata size, all existing tokens),
  which is exactly the "minimum required neutral/past-run styling" §6 allows. `AGENTS.md` 11 forbids magic values;
  every declaration cites a `--fs-*` token.
- **No** new component, **no** new token, **no** change to `Panel`, `Chip`, `StateBadge`, `Layout`, `Analyze`,
  `Compare`, `Release`, `History`, `Details`, `stateWords.ts`, `ipc/**` or any Rust file.

## 5. Explicitly out of this unit

`U1P-V1-01` (wrapped band's empty filled cell at 1024), `U1P-V1-02` (Compare result not surviving navigation) and
`U1P-V0-01` (Analyze table header at the fold) are named by §6 as **not to be touched here**; they stay open for a
separate judgement after product integrity is repaired. U1R's last-good retention is preserved rather than
simplified: this corrective adds one derived read, exactly as U1R did, and deletes no retained state.

## 6. Tests, and what must be red before the fix

`overview.test.tsx` gains T1–T12 (§7 of the prompt). Every test asserts inside the `Can we ship now?` region, on
DTO-shaped fixtures, using text a screen reader would read.

| # | Fixture | Assert |
|---|---|---|
| T1 | selection current, `gateRun.snapshotId === summary.identity.snapshotId`, no project | verdict shown; **full judged snapshot id present and exactly equal to the run's**, not the truncated form |
| T2 | `gateRun` PASS for `snap-B`, `summary` for `snap-A` | no `Clear —` sentence anywhere in the region; `Not assessed for this build`; both ids distinguishable in text |
| T3 | `gateRun` BLOCK for B, summary A | neutral current answer present; the BLOCK words appear only inside the labelled historical block |
| T4 | REVIEW with an accepted acceptance, UNKNOWN counts, subject matching | unchanged from today's wording, including "accepted review(s) counted" and the aggregate-without-them clause |
| T5 | matching snapshots **but** `selection.selectionId !== analyzedSelectionId` | neutral; PASS not inherited; retained analysis still named |
| T6 | `summary: null`, `gateRun` present | no current-build verdict; run identified as a stored/historical judgement by full id |
| T7 | `gateRun: null` | no run, no verdict, CTA still navigates (regression guard on the existing empty state) |
| T8 | mismatch, then the same view with `gateRun.snapshotId` matching the new current analysis | normal verdict returns without a reload |
| T9 | a second `gateRun` for the newly analyzed snapshot | verdict normal and names the new judged id; the old run's id is gone from the region |
| T10 | subject matches, `project.policySha256 !== gateRun.policySha256` | not presented as the current-policy answer; both fingerprints shown; no third fingerprint invented |
| T11 | `gateRun.baselineSnapshotId === summary.identity.snapshotId` while `gateRun.snapshotId` differs | mismatch **is** detected: the guard reads the target, never the baseline |
| T12 | any matched state | ELF/MAP/Git band and the four key figures unchanged, and both header actions plus the CTA only call `onOpen` |
| Wiring | `App.tsx` | a render-level check that `Overview` receives `analyzedSelectionId`, so the guard cannot be silently un-fed again |

**Red-before-green:** T2, T3, T5, T6, T10 and T11 must fail on `a2e6b30`, where `Readiness` has no subject guard
at all; the wiring check must fail because the prop does not exist yet. T1/T4/T7/T8/T9/T12 pass before and after —
they are the "do not weaken what was accepted" guards.

**Mutation proof:** with the corrective in place, restore the unconditional `gateRun !== null → verdict` path and
show T2 and T5 go red; then restore byte-exact and prove zero residue by digest.

## 7. Installed evidence this plan commits to producing

BEFORE is immutable where it exists: `%TEMP%\FirmwareSight-U1P-Visual-Acceptance-20261008T001110Z\07_smoke\FINDING_U1P-V2-01_overview_verdict_subject_1440x900.png`
(the mismatched Overview on U1P's bytes). If that root has disappeared, it is recorded `UNAVAILABLE` rather than
re-created from a newer build, which is what §12 requires.

AFTER, on the exact U1P-R1 CI artifact, four states on two widths plus one recommended width:
`R1_S01_Matched_1440x900`, `R1_S02_Mismatch_1440x900`, `R1_S03_UnanalyzedSelection_1440x900`,
`R1_S04_MatchedAfterNewGate_1440x900`, `R1_R01_Matched_1024x720`, `R1_R02_Mismatch_1024x720`, and the recommended
`R1_R03_Mismatch_1056x799`. Each records the measured client rect before and after the grab, the product EXE
digest, the timestamp and the scenario id. Scenarios S1–S4 use two real distinct analyzed snapshots from the
committed fixtures; S5 runs only if the installed product genuinely supports loading a second real project policy
in one session, and is recorded `NOT_EXECUTED` with the reason if it does not.

No second 16-capture pass, no regeneration of U1P's evidence from a new head, and no product edit after the
captures — the screenshots certify the bytes they came from.
