---
title: "ADR-0025 Conditional Pre-G1 Analyze Implementation"
doc_id: "ADR-0025"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Product / Architecture"
last_updated: "2026-09-28"
---

# ADR-0025 — Conditional Pre-G1 Analyze Implementation

## Status
Accepted.

Partially supersedes `ADR-0020-validation-sequence.md`. The superseded text is exactly one clause —
`P1 Product MVP implementation 只有两者都 PASS 后开始。` — and nothing else in ADR-0020. Its
parallel-validation decision, its consequences and its revisit note all stand.

## Context
ADR-0020 put V0 (workflow prototype validation) and P0 (technical vertical slice) in parallel and
gated P1 on both passing. P0 has since closed `PASS` at `v0.6.0` with real cross-platform evidence; the
last measured remote verification is Run #6 (`36419864513`) on the consistency-closure commit `7d2f38a`,
`success` with 7 of 7 jobs.
V0 has not: it is an active external-validation track waiting for real participants, at `0 / 8`
eligible external sessions, and the missing input is human participants, which no engineering work can
manufacture.

That leaves the project with two unsatisfying readings of ADR-0020. Read one way, every engineering
step after P0 waits on recruitment, so the engineering schedule is set by how fast strangers can be
screened and scheduled. Read another way, "P0 passed, carry on" opens P1 through P4 as a block, and
the project builds the Compare, Gate and Bundle surfaces on top of a workflow nobody has validated
with a real user — the precise risk ADR-0020's own Consequences section exists to prevent, and the
risk that turns into sunk cost when the workflow finally is tested.

The narrow question is therefore not "may we implement" but "which implementation is honest to build
before the workflow is validated, and what does it commit us to".

## Decision
Adopt **bounded parallelism**.

1. After `P0 PASS`, the architect may authorize a limited, reversible, low-coupling **Pre-G1 Analyze
   slice** without waiting for `V0 PASS`.
2. This prompt authorizes exactly one such slice: **P1-A0 — Real Artifact Intake + Analyze Summary**.
   The user outcome is that a person can choose their own ELF file through a native dialog, optionally
   attach a GNU ld MAP, and have FirmwareSight analyze it with the already-validated Core, artifact
   pipeline and storage. Nothing else.
3. **P1-A0 stops at P1-A0.** It is not `P1 PASS`, it does not close P1, and it does not authorize
   P1-A1 or any later slice.
4. A further Pre-G1 slice requires a new architect prompt. Before the next engineering authorization,
   **V0 Batch A must have at least 4 eligible external sessions completed and an interim architect
   review of that evidence**. Sessions are counted from real participants; the coding side authors none.
5. `P2 Compare`, `P3 Release Gate` and `P4 Release Bundle` remain **not authorized**.
6. **`Formal G1 = V0 PASS + P0 PASS` is unchanged** (`06_DELIVERY/06_STAGE_GATES.md`), and G1 remains
   **NOT CLAIMED**. This ADR moves when implementation may start; it does not move what a stage gate
   requires, and `g1_claimed` stays `false`.
7. P1-A0 is a development slice. `baseline_version` stays **0.6.0**; no `v0.7.0` is created by it.
8. The intake surface is a plugin-system change, and `00_GOVERNANCE/03_DECISION_POLICY.md` lists
   插件系统 as ADR-requiring, so it is decided here rather than left implicit: the desktop shell adopts
   **`tauri-plugin-dialog`** for native file selection, as already permitted by
   `04_TECH/11_DEPENDENCY_BASELINE.md` ("dialog plugin is acceptable for explicit user file selection").
   The consequence is specific: dialogs are opened from Rust-side use-case commands, the WebView keeps
   no filesystem, shell or network surface, no generic `read_file(path)` or `get_any_path` command is
   added, and the capability is extended only if the build genuinely requires a narrow dialog
   permission. Full admission evidence (license, advisories, Tauri 2.12 compatibility, native build and
   capability impact) belongs in the slice's execution report, and `cargo-deny` must stay green.

## Alternatives
- **Wait for V0 8/8 before any P1 work.** Rejected: it makes engineering throughput a function of
  recruitment throughput, and P0's own evidence already proves the Analyze chain end to end. It also
  buys nothing — the risk it guards against is building *unvalidated workflow depth*, which a single
  intake slice on the validated pipeline does not create.
- **Open P1 through P4 now, on the strength of P0 PASS.** Rejected: it accumulates sunk cost in Compare,
  Gate and Bundle before anyone has watched a real firmware engineer use the workflow, which is the
  failure mode ADR-0020 was written to avoid.
- **Stay fixture-only until V0 finishes.** Rejected: the P0 desktop can only analyze two committed
  fixtures, so the one thing V0 participants would most need to do — bring their own firmware — is
  impossible, and the prototype evidence would be weaker for it.
- **Implement intake with a hand-rolled path text field.** Rejected on security grounds: it would put
  arbitrary path input in the WebView, which `AGENTS.md` 7 and `04_TECH/05_SECURITY_PRIVACY.md` forbid,
  and it is worse UX than the native dialog the dependency baseline already allows.

## Consequences
Positive: the validated pipeline becomes reachable by a real user on a real artifact; V0 participants,
when they arrive, meet a product that accepts their files instead of two fixtures; the work is confined
to the desktop shell's intake boundary, so it is reversible by deleting commands rather than by
unwinding data models; and the sequencing rule becomes explicit and auditable instead of a reading
disagreement.

Negative and accepted: the repository now carries two identifiers that the frozen gate documents do
not define — "Pre-G1" and "P1-A0" — so every reader must check this ADR rather than infer from
`06_DELIVERY/06_STAGE_GATES.md`; a green P1-A0 could be misread as P1 progress, which the governance
text has to keep stating is not the case; the desktop shell gains one plugin dependency and its
transitive closure, which must be re-justified at each Tauri upgrade; and implementing intake before
workflow validation carries a real, if small, chance that V0 finds the intake affordance is wrong and
it has to be rebuilt.

## Revisit trigger
- V0 Batch A reaches 4 or more eligible external sessions: the interim review then decides P1-A1, not
  this ADR.
- V0 evidence contradicts the Analyze-first workflow (for example participants do not begin by
  selecting an artifact): the intake design is reopened before any further slice.
- A Pre-G1 slice is requested that touches Compare, Gate, Bundle, portable schema, storage semantics,
  or any capability beyond the dialog: it needs its own ADR, not an extension of this one.
- The dialog plugin would require a broad filesystem, shell or network capability to function: stop and
  reopen the intake architecture instead of granting it.
- V0 reaches `PASS`, or P0 were ever to be reopened: the pre-G1 carve-out loses its reason to exist and
  ADR-0020's original ordering applies again.
