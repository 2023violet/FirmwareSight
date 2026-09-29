---
title: "ADR-0026 Open-Source MVP-First Delivery"
doc_id: "ADR-0026"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Product / Architecture"
last_updated: "2026-09-29"
---

# ADR-0026 — Open-Source MVP-First Delivery

## Status
Accepted.

Supersedes the **sequencing conclusions** of `ADR-0020-validation-sequence.md` and
`ADR-0025-conditional-pre-g1-analyze-implementation.md`, and only insofar as those two make V0 external
validation a hard prerequisite for continuing MVP implementation. It does not erase, and explicitly
preserves:

- every V0 research artifact under `V0_VALIDATION/**`, including the protocol, the frozen `v0.1.0`
  clickable prototype, the screening and scheduling registers, and the `0 / 8` sample state;
- the P0 evidence pack and `P0 = PASS`;
- the P1-A0 evidence pack, its correctness closure, and `P1-A0 = COMPLETE`;
- every technical safety and privacy boundary: the Rust-side native dialog, the absence of any generic
  filesystem / shell / network capability or `read_file(path)`-style command, the no-host-path rule in
  normal IPC and UI output, the evidence-class rules (`Observed` / `Derived` / `Declared` / `Unknown`),
  the `UNKNOWN 不等于 PASS` rule of ADR-0023, and `AGENTS.md` 2 / 7 / 11.

Per `00_GOVERNANCE/03_DECISION_POLICY.md` ("ADR 一旦 Accepted 不修改结论；若变更，新增 ADR supersede 旧
ADR"), the superseded sentences stay in ADR-0020 and ADR-0025 with dated notes appended rather than being
rewritten.

## Context

FirmwareSight is an open-source project. Its current priority is a credible, complete, usable local MVP:
Analyze, then Compare, then Gate, then Release Bundle.

The sequence on disk says otherwise. `ADR-0020:31` gates P1 implementation on both V0 and P0 passing;
`ADR-0025` relaxed that only by carving out one bounded pre-G1 slice and then re-blocked everything after
it behind "V0 Batch A must have at least 4 eligible external sessions completed and an interim architect
review" (`ADR-0025:50-52`). `06_DELIVERY/06_STAGE_GATES.md:242,248-249` carries the same rule, and
`G1 = V0 PASS + P0 PASS` appears in twenty places across sixteen files.

V0 has sat at `0 / 8` eligible external sessions since it resumed. The blocker is real and is not an
engineering artifact: no participant has been met, and the coding side authors none. The last recorded
round was exactly that - Batch A was activated as the live task, the recruitment pack was verified ready,
and the round stopped with nothing to write because there were no humans to write about. Pricing,
willingness-to-pay and pilot signals sit in the same position: `BATCH_A_RESEARCH_PRICE_ANCHORS.md` states
that no concrete anchor has been decided, and none was invented.

So the project currently has a verified core chain, one shipped vertical slice of it, and a gate document
that makes the next usable piece of software depend on recruiting strangers for moderated sessions about a
prototype. That is not what an open-source MVP needs. What an open-source project can substitute for paid
cohort research is a real artifact people can run themselves and feedback that arrives after something
usable exists.

The distinction worth keeping: V0 was never only about *desirability*. It was also about whether users
comprehend the evidence model - Observed versus Declared, Unknown, Review versus Block, the purpose of a
Bundle. Dropping that question is a real cost, recorded below rather than argued away.

## Decision

1. **MVP implementation proceeds sequentially on engineering grounds**: P1 Analyze → P2 Compare →
   P3 Gate → P4 Release Bundle → G2 MVP Candidate. Each stage still needs its own architect prompt;
   this ADR changes what blocks them, not who authorizes them.
2. **P0 technical validation remains `PASS` and is sufficient to open MVP implementation.**
3. **For the current open-source MVP delivery, `G1 = P0 PASS`.** The formula is redefined, not merely
   re-satisfied: G1's technical content in `06_STAGE_GATES.md` (the verified chain, fixture reproducibility,
   parser behaviour on bad input, a non-blocking UI, a data model that can carry Compare/Gate) is unchanged
   and still has to hold. What is removed is V0 as a conjunct.
4. **V0 becomes `NON_BLOCKING_USER_FEEDBACK_TRACK`.** Its sample may legitimately remain `0 / 8`; that
   number no longer gates any P-stage.
5. **V0 may resume later for usability feedback**, and the existing instrument stays frozen and ready for
   exactly that: the `v0.1.0` prototype, `protocol/TASK_SCRIPT.md`, `sessions/TEMPLATE.md`, the registers.
   **`V0 PASS` is not required before P1, P2, P3 or P4.**
6. **Pricing, willingness-to-pay, commercial buyer path and team-pilot signals are not part of any MVP
   gate.** They move to `POST_MVP / OPTIONAL / NOT CURRENT GATE`, which affects the G2 metrics at
   `06_STAGE_GATES.md:98-99` and the G4 wording at `:146-147`; the historical text is marked, not deleted.
7. **Commercial validation is deferred** until explicitly re-authorized after MVP.
8. **No cloud, account, telemetry, AI, updater, commercial licensing or paid-tier work enters MVP.**
   This restates `AGENTS.md` 2 and 7 rather than adding to them; nothing in this ADR loosens them.
9. **Open-source does not weaken engineering quality.** Deterministic output, evidence correctness,
   security boundaries, cross-platform CI and a portable release output remain mandatory, as do the
   per-slice obligations already on file: real Windows desktop smoke, fmt / clippy / test, frontend
   typecheck / lint / test / build, IPC binding drift, goldens unchanged, `cargo-deny` actually executing,
   and no new dependency or design token without admission.

## Alternatives

- **A. Keep V0 gating engineering.** Rejected. It blocks MVP progress on external recruitment, makes the
  engineering schedule a function of screening and scheduling strangers, and delivers nothing to anybody
  in the meantime. It also inverts the value of an open-source project, whose users arrive after software
  exists rather than before.
- **B. Remove validation and testing along with the research gate.** Rejected. The thing that stops gating
  is *human-panel* validation, not verification. An open-source MVP still has to be deterministically
  correct, must not crash on supported input, and must keep its evidence claims honest; dropping CI would
  trade a governance problem for an engineering one.
- **C. MVP first, with later community and user feedback.** Accepted. Ship the complete local workflow
  against the evidence discipline P0 proved, then let real usage - open-source contributors and users
  included - inform iteration, keeping the V0 instrument frozen so a later round of formal sessions is
  still comparable with the protocol that already exists.

## Consequences

Positive: engineering effort goes into software people can actually run; the governance loop count drops
from "recruit, screen, moderate, review, authorize" per slice to "authorize, build, verify"; the full
Analyze → Compare → Gate → Bundle workflow exists sooner, which is also what makes later usability
feedback concrete rather than hypothetical; and an open-source community can test a real MVP instead of a
clickable prototype.

Negative and accepted:

- **Comprehension risk is carried forward, not resolved.** Whether real firmware engineers distinguish
  `Unknown` from `PASS`, or understand why a MAP is requested, remains untested. `G1` is now claimed on
  technical grounds alone, which is a weaker basis for exactly the claim ADR-0020 was written to protect.
- **UX may need iteration after MVP**, because the design decisions now being made in Sections, Symbols
  and the Evidence Inspector are informed by the frozen design contract and engineering judgment rather
  than by observed sessions. Mitigated by keeping the UI modular, tabs/panels reversible, and the V0
  instrument intact so it can be run against the finished product.
- **Commercial signal is now absent rather than pending.** No price anchor, no return-intent ratio, no
  pilot interest is collected; productization decisions after G2 will be made without it.
- `G1` now means two different things depending on the document's date: historical records (the P0
  promotion pack, the P1-A0 pack, the changelog) assert `G1 = V0 PASS + P0 PASS` and stay as written.
  Every reader must know that ADR-0026 changed the basis on 2026-09-29, which is why the current-truth
  documents are updated with dated notes rather than silently edited.
- A future productization round must re-derive its own authority; this ADR does not pre-approve it.

## Revisit trigger

- **After `G2` MVP Candidate**: reassess whether the shipped workflow needs the formal V0 protocol run
  against the real product, and with what cohort.
- **Before any productization or commercial strategy**: pricing, paid tiers and buyer path require a new,
  explicit authorization; this ADR defers them and authorizes nothing.
- **If the open-source direction changes** (dual licensing, hosted service, accounts, telemetry, paid
  tiers): the MVP scope in Decision 8 and the gate basis in Decision 3 are reopened together.
- **If evidence-model comprehension failure is observed** in any later feedback - users treating
  `Unknown` as safe, or a Review as an approval - reopen the relevant UI before adding depth on top of it.
- **If a slice appears that cannot be built without weakening a technical boundary** named in Status
  above: stop and write a new ADR instead of proceeding under this one.
