---
title: "V1 Plan"
doc_id: "FS-V1-001"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Product / Research"
last_updated: "2026-10-06"
---

# V1 Plan

## 1. Question and hypothesis

V1 tests whether the productized MVP candidate works on someone who did not build it, using evidence they
already own. The hypothesis is deliberately narrow:

> A firmware engineer outside the project, given their **own** real build and no training, can reach a
> rendered Analyze result, get a first useful fact from it quickly, drive Analyze→Compare on two of their own
> builds, read the five Gate states without inventing certification meaning in them, discover at least one
> true thing about their own firmware they did not already know, and would open the tool again for a future
> release.

Six metrics carry that sentence, and only six (§15). Anything else is colour.

## 2. What is authorized (§4)

Open the V1 track; prepare recruitment, screening, consent and session protocol; preserve the exact F3
artifact for research; recruit real external participants; conduct real sessions; record anonymized
behavior, timing and quotes; calculate metrics; classify misunderstandings and findings; produce V1 evidence
and recommendations.

*(Dated amendment, 2026-10-08: the owner's inline V1 Cohort Re-freeze authorization moved the preserved research
artifact from F3's `11419727517` to the U1-accepted `11573661113`, and requires **both** sets of bytes to be
preserved — the new one because the cohort runs on it, the old one because it is immutable research history. See
`V1_COHORT_REFREEZE_RECORD.md`. The wording above is the delivered prompt's own §4 list and is kept as it was
received.)*

## 3. What is not authorized (§4, §33, §40)

No product feature implementation; no product redesign from participant preference; no schema, migration or
dependency change; no cloud, account, network or telemetry; no updater; no signing or notarization; no licence
selection; no pricing model, paid plans or invented price anchors; no E1/E2/E3/GX implementation; no B1 or
Private Beta; no RC or GA; no public release.

V1 **may recommend** CSV/Markdown/PDF export, SBOM/CVE, CI automation, stack analysis, watch mode, cloud sync,
accounts, team dashboard, AI, pricing, subscriptions, new Gate rules, new parser formats or E1/E2/E3/GX items.
V1 **may not implement** any of them without new Architect authority. ADR-0024's rule stays load-bearing:
candidate ≠ task.

## 4. Cohort and inputs (§8, §9)

Formal workflow metrics use inputs inside FirmwareSight's **measured** support boundary: GCC /
`arm-none-eabi-gcc` ELF, Clang ELF within the documented measured Arm cohort (`P5_VALIDATION/P5_COMPATIBILITY_MATRIX.md`),
and GNU ld MAP where available.

Keil and IAR participants are valuable **discovery** participants and must not be told those toolchains are
currently supported. If their real output cannot enter the supported input path, the session is classified
`DISCOVERY_OUT_OF_COHORT`: it contributes qualitative demand, workflow and toolchain evidence, and does not
enter a supported-workflow denominator unless that participant independently provides a supported artifact.
Unsupported-toolchain behaviour is not a usability defect.

Own-artifact rule (§9): a full-workflow participant brings at least one real own-project ELF, preferably with
its matching MAP, and preferably **two real builds of the same project** (baseline + target). Two valid
own-project builds are what put a participant in the Analyze→Compare denominator. One valid build still
contributes import activation, TTFV, Analyze understanding, the Gate comprehension interview, new-information
and return intent — and is excluded from M3's denominator, stated as an exclusion rather than smoothed into a
percentage. Repository and demo fixtures may **never** substitute for a formal own-artifact task.

## 5. Sample and batching (§12, §34)

Minimum formal external sample **N ≥ 8** eligible unique participants; target 12–15. Not counted: internal
project members; a second session from the same person as a new participant (follow-ups are `V1-P001-F1` and
do not raise unique N); sessions where the moderator taught before the task; simulated sessions;
invalid/unsupported inputs inside supported-workflow denominators. `N < 8` yields
`V1_INCOMPLETE_INSUFFICIENT_SAMPLE` with no forced PASS/FAIL.

| Batch | Who | Then |
|---|---|---|
| A | first 4 eligible participants | `analysis/BATCH_A_INTERIM_REVIEW.md`, recommendation only |
| B | participants 5–8 minimum | evidence commit, aggregate update |
| C (optional) | 9–12/15 | only if the Architect wants the depth |

Batch A detects catastrophic misunderstanding, a broken protocol, an artifact/cohort mismatch or an obvious
blocker. It does **not** complete V1, and four people are not a redesign input.

## 6. Session shape (§23, §25, §29)

Neutral wording only, T0–T8, in this order: install/launch → Analyze → evidence improvement → Compare →
release readiness → Gate semantics interview → Bundle handoff → History → Support/Diagnostics if relevant.
For T1/T3/T4/T6/T7 the moderator records first click, first navigation path, wrong turns, backtracks, stalls,
Help usage and interventions. No threshold is invented for first-click behaviour; it is described.

Session ids `V1-P001`, `V1-P002`, …; follow-ups `V1-P00X-F1`. Quality flags: `VALID`,
`VALID_WITH_INTERVENTION`, `DISCOVERY_OUT_OF_COHORT`, `EXCLUDED_PROTOCOL_CONTAMINATED`, `EXCLUDED_INTERNAL`,
`EXCLUDED_NO_CONSENT`, `EXCLUDED_NO_REAL_ARTIFACT`.

## 7. Code freeze during a cohort (§32)

One frozen artifact per formal cohort. **No silent product patch mid-cohort.** On S0 or S1: stop formal
sessions, preserve evidence, return to the Architect. An S2 may continue only if it does not invalidate the
tasks already measured, and is flagged for Interim Review. Any product code change creates a new cohort version
and a **new CI artifact**; pre- and post-hotfix metrics are never pooled.

## 8. Early stop triggers (§36)

Immediate Architect review on: any S0; any S1; a credible C1–C8 product risk; the same S2 independently
observed by ≥ 3 participants; the same misunderstanding independently observed by ≥ 3 participants; import
activation < 60 % after the first 4 valid sessions; Gate comprehension < 50 % after the first 4; any
privacy/consent breach; a research artifact provenance mismatch; or a participant receiving the wrong build.
Do not wait for N = 8 when the protocol or the product is invalid.

## 9. Privacy and data handling (§10, §41, §50)

Default collection excludes firmware binary bytes, MAP files, source code, private repositories, credentials,
customer/device identifiers, confidential release information and private Git remotes. Participant artifacts
stay on the participant's machine or a controlled research machine. The raw evidence root lives outside Git.
Repo-safe facts about an artifact are: `OWN_ARTIFACT = YES`, artifact type, toolchain category, ecosystem
category, size band, `MAP YES/NO` — and a full artifact SHA may stay private. No private absolute path enters
the repository. Before any session evidence is staged, §50's scan runs.

## 10. Commit sequence (§42, §43, §51)

| Commit | Content | Boundary |
|---|---|---|
| Activation (this one) | prompt archive, V1 pack, governance state | `V1_VALIDATION/**`, `BASELINE.yaml`, `.ai/*.md`, `README.md`, `INDEX.md`, `10_AUDIT/SOURCE_PROMPTS/**`, `DIRECTORY_TREE.txt`, `SHA256SUMS`, `06_DELIVERY/*.md` only where materially stale |
| Batch A evidence | 4 anonymized sessions + interim review | docs/evidence only |
| Batch B evidence | ≥ 8 participants, aggregates | docs/evidence only |
| Final evidence candidate | reports and recommendation | docs/evidence only |

Every pushed evidence commit still passes the authoritative 10-job CI, and none of them moves a product count:
**868 Rust / 225 UI in 8 files, gate 17, drift 8, package 4**. A docs-only round that changes one of those
numbers is a product change wearing a documentation diff. **Dated 2026-10-08, when the cohort was re-frozen:** the
numbers above are the activation head's, and they are kept as what §45 held still at V1's own commit. The cohort
build is now `41bb6a36`'s artifact `11573661113`, whose CI reports **868 Rust across 47 result lines / 291 UI in
9 files** on the same 17-step gate with the same 8 drift and 4 package steps and no required skip — the UI line grew
across the eight U1 rounds that followed this activation, and no Rust test moved or was deleted. From the re-freeze
onward, that is the figure an evidence commit must not move
(`BASELINE.yaml` `v1_execution.product_counts_required_unchanged.refrozen_guard`). Forbidden in all of them: `apps/**`, `crates/**`,
`scripts/**`, `fixtures/**`, `schemas/**`, `golden/**`, `migrations/**`, `.github/**`, `Cargo.toml`,
`Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json`, `deny.toml`,
`assets/design-tokens.json`. If a product path appears: **STOP**.

## 11. What V1 owns (§55)

V1 owns the evidence needed to resolve or reduce **P5 Known Limitation L11** — "user comprehension untested
(V0 0/8)", today `CARRIED_FORWARD, owner V1` in `P5_VALIDATION/P5_KNOWN_LIMITATIONS.md`. Before an Architect
verdict, the most this track may write for L11 is `READY_FOR_ARCHITECT_REVIEW`, and only once real evidence
exists to be reviewed. With zero sessions, L11 stays `CARRIED_FORWARD`; that is the honest state, not a
placeholder to be upgraded for appearances. V1 may inform roadmap priority; it does not activate E1/E2/E3/GX.

## 12. What V1 cannot prove even if it goes well (§56)

Market size, product-market fit, production safety, security certification, regulatory compliance, enterprise
readiness, all-platform runtime support, optimal pricing, long-term retention, team deployment, support
scalability, and Beta readiness by itself.

## 13. No automatic next stage (§39, §40)

Metrics do not self-issue a stage PASS. The Architect also reviews critical misunderstandings, S0/S1, cohort
concentration, protocol integrity and privacy integrity. This round may never write `V1 = PASS_COMPLETE`,
`B1_READY` or `PRIVATE_BETA`. Even a full V1 pass does not authorize B1: Private Beta / G4 suggests 20–50 users
and 3–5 teams, plus a beta support loop and a release-train decision of its own.
