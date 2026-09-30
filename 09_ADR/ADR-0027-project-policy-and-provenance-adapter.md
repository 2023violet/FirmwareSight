---
title: "ADR-0027 Project Policy and Provenance Adapter"
doc_id: "ADR-0027"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-29"
---

# ADR-0027 — Project Policy and Provenance Adapter

## Status

Accepted. Authored during P3 Release Gate under *FirmwareSight — P3 Release Gate MVP
Implementation, Execution Prompt v1.1 — Architect Reviewed*, which authorizes this ADR explicitly (§6).
It supersedes no earlier ADR: it adds a boundary that `AGENTS.md` 3 left open, and it operates inside
ADR-0023's five-state Gate semantics, ADR-0021's memory accounting and ADR-0022's portable-schema
strictness without changing any of them.

## Context

Gate needs four things that no existing crate owns and that no existing crate should own:

- `firmwaresight.toml` — load, validate, canonicalize, and save supported policy fields.
- Project-root-relative file evidence — Release Notes presence and digest.
- Workspace provenance — read-only system Git facts: repository availability, root, HEAD commit, exact
  tag, dirty state.
- Deterministic fingerprints — a policy hash that survives formatting, and a run id that changes when and
  only when an input that affects evaluation changes.

`AGENTS.md` 3 forbids Core from depending on Tauri, React/JS or SQLite and keeps it headless, synchronous
and dependency-free; `04_TECH/15` keeps `rusqlite` types inside storage; the Desktop shell is the Tauri
edge. Each of those crates therefore has a stated prohibition, and none of them has a stated role for
project files or a Git process. Putting the four responsibilities into Core would end Core's zero-dependency
property — `crates/firmwaresight-core/Cargo.toml:10-14` declares an empty `[dependencies]` on purpose.
Duplicating them in CLI and Desktop would produce two implementations of policy canonicalization and two
fingerprint rules, which is the exact failure mode `04_TECH/02` and `05_ENGINEERING/08` close.

`04_TECH/22_GIT_PROVENANCE_ADAPTER.md` already decided *how* to obtain Git facts (installed `git` CLI,
read-only, no `gix`) but not *which crate* runs it. `04_TECH/08_CONFIG_SPEC.md:59` defers the formal
validation shape to "实现阶段冻结" — this stage is that implementation stage.

## Decision

Authorize a fifth first-party library crate:

```text
crates/firmwaresight-project
```

**It owns, once, for CLI and Desktop together:** loading and validating `firmwaresight.toml`;
canonicalizing Gate policy; saving the policy fields it supports; discovering the project root from the
config; reading project-local release evidence; running the read-only system Git adapter; normalizing
those observations into Core-friendly facts; and computing the deterministic policy and context
fingerprints.

**It must never:** depend on Tauri, SQLite or Tokio; implement Gate rule semantics; parse firmware
artifacts; know anything about UI; make a network call. `firmwaresight-core` continues to own every Gate
semantic (ADR-0023), and continues to hold zero external dependencies — which fixes the boundary
precisely: the project crate turns the outside world into facts, Core decides what they mean, and the
SHA-256 digest of a run fingerprint is therefore computed in the project crate from Core's canonical
ordered input, never inside Core.

**Provenance boundary, restated because it is where a Gate can lie:** the adapter reports *workspace*
facts. `git.clean`, a HEAD commit and an exact tag describe the directory the release owner pointed
FirmwareSight at; they do not prove that the artifact under evaluation was built from them. The wording
`Firmware was built from this commit` remains forbidden, and only `artifact:<kind>:<sha256>` evidence
identifies the binary. This is ADR-0023's `UNKNOWN` rule applied to identity, and it is the same
distinction `04_TECH/24_BUILD_IDENTITY_EVIDENCE.md:15-17, 36-41` already draws.

**Degradation is a fact, not an error.** Git missing, not a repository, timing out, or returning nothing
readable yields normalized `Unknown` facts and never aborts Analyze, Compare or Gate. A rule that needs
the missing fact reports `UNKNOWN` and takes its effective severity from `gate.on_unknown` — the three-step
rule ADR-0023 established, which replaces the older "becomes Review/N/A according to policy" phrasing in
`04_TECH/22:48`.

**Configuration is frozen at schema_version 1** with the shape `04_TECH/08_CONFIG_SPEC.md` records and this
stage's `[gate]` plus `[gate.on_unknown]` sections — the per-rule policy
`04_TECH/08:66-67` required but never placed. Unknown keys in
major 1 are warnings, and because they are warnings they must be preserved: a save that cannot keep them
refuses with a typed error instead of silently dropping them.

## Dependency admission (measured, not assumed)

Every entry below was already resolved in `Cargo.lock` before this crate existed, is dual
`MIT OR Apache-2.0` or `MIT`, and declares a `rust-version` at or below the workspace's `1.98`.
Versions are the ones the lock already carries, per the prompt's "prefer an already-resolved compatible
version". `cargo deny check licenses bans sources advisories` runs for real in the `deny` group and is
re-run at stage close.

| Crate | Locked version | License | rust-version | Status for P3 |
| --- | --- | --- | --- | --- |
| `toml` | `1.1.6+spec-1.1.0` | MIT OR Apache-2.0 | 1.85 | **new direct dependency** — already in the graph via `tauri-utils 2.10.0`; no new package enters the lock |
| `regex` | `1.13.1` | MIT OR Apache-2.0 | 1.65 | **new direct dependency** — already in the graph via `tauri-utils` / `urlpattern`; pulls no new package |
| `sha2` | `0.11.0` | MIT OR Apache-2.0 | 1.85 | reuse — already a direct dependency of `firmwaresight-artifact` |
| `thiserror` | `2.0.21` | MIT OR Apache-2.0 | 1.77 | reuse — already direct in artifact, storage and report |
| `tracing` | `0.1.44` | MIT | 1.65 | used **only if** a diagnostic actually needs it; an authorized dependency is not an installed one |
| `serde` | `1.0.229` | MIT OR Apache-2.0 | — | reuse — the existing DTO boundary |
| `firmwaresight-core` | path | `Proprietary` (workspace) | — | the only first-party dependency admitted |

Admission answers the four questions `AGENTS.md` 4 requires:

- *Which need requires it* — `04_TECH/08:38` specifies `[version] pattern` as a regex, and
  `firmwaresight.toml` is TOML. Neither is expressible with `std`, and hand-rolling a TOML reader or a
  backtracking matcher inside the product would be new, worse, unreviewed parser code — the same category
  `04_TECH/11:52` refuses with "No universal regex parser" for the MAP adapter.
- *Why std and existing deps are not enough* — Core and report have no TOML or regex capability and must not
  gain one; `object` parses ELF, not config; `rusqlite` is storage-only; `serde_json` is JSON.
- *License* — all admitted crates are MIT OR Apache-2.0 (or MIT), inside `deny.toml:107-124`'s allow list.
- *Maintenance and build impact* — both crates are actively released and already compiled into the
  dependency graph for every build today, so making them direct adds no new unit of work to CI; the
  binary and the shipped artifact set do not change, because the added code lives in a headless library
  crate that CLI and Desktop already link.
- *Trusted core* — **no**. `firmwaresight-project` is not Core: it touches the filesystem and spawns a
  read-only process. Core's zero-dependency, filesystem-free property is what this ADR protects.

`gix` stays banned (`deny.toml:86`), `anyhow` stays banned for first-party code (`deny.toml:99`), and
`rayon` stays banned (`deny.toml:87`) — the adapter is synchronous with a bounded timeout and needs no
executor.

## Alternatives considered

- **Gate semantics plus config parsing inside Core.** Rejected: it would force `toml`, `regex` and
  `sha2` into the crate `AGENTS.md` 3 defines as dependency-free, and it would give the domain knowledge
  of project directories and child processes.
- **Inside `firmwaresight-artifact`.** Rejected: that crate is about artifact bytes; project policy and Git
  are not artifact formats, and the pairing would make every config change a parser change.
- **Inside `firmwaresight-storage`.** Rejected: policy and provenance are not persisted facts of a build,
  and storage's SQLite boundary must not become the place that reads a filesystem or spawns `git`.
- **Inside the CLI, with the Desktop duplicating it.** Rejected: two canonicalization and fingerprint
  implementations would let CLI and Desktop disagree about a run id, which is the class of defect P2
  recorded when one portable document rendered two `layoutSource` values.
- **`gix` instead of the system client.** Rejected on the existing decision (`04_TECH/22:17`,
  `deny.toml:86`): a library parser of a repository the user owns is a wider failure surface than a
  read-only subprocess with a timeout.
- **`toml_edit` instead of `toml`.** Rejected for now: comment-preserving edits are exactly what would make
  "unknown keys are warnings, not data loss" cheap, but P3 does not promise comment preservation — it
  promises to refuse a save that would lose keys. Admitting a second TOML implementation to buy a feature
  nobody asked for is the wrong trade.

## Consequences

- The workspace has five library crates and one more admission question at each future stage: does this
  belong in the project boundary or in Core? The test stays the same — a fact that needs the filesystem or
  a process is never Core.
- `scripts/check.py`'s `CORE_PACKAGES` list gains `firmwaresight-project`, so `--only core-smoke` on macOS
  keeps covering the headless product without the Tauri shell. A new crate that is not added there is not
  covered by that job, and the omission is silent: the list is hand-maintained by design.
- `firmwaresight.toml` and its JSON Schema become public contracts the same way `diff` did: additive
  optional keys with contract tests, and a major version for any semantic change
  (`04_TECH/26:29-30`).
- Policy save is now a user-visible write to a file outside FirmwareSight's own directory, so the atomicity
  and no-silent-loss rules in this ADR are user-facing promises, not internal tidiness.
- Git availability is a capability that can degrade at any moment on the same machine, so `UNKNOWN` is a
  normal Gate outcome for three of the ten MVP rules, and the UI must render it as one.
- Nothing in this ADR resolves the license gap: `Cargo.toml:17` still reads `license = "Proprietary"`, the
  repository still has no `LICENSE` file, and `AGENTS.md` 9 puts a license change in front of a human.
  P3 records `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION` and changes nothing.

## Revisit trigger

Re-open this ADR when any of these happens: a second product verb needs project-level configuration beyond
Gate policy; a portable artifact-embedded build identity becomes readable enough to replace workspace
provenance; a dependency-admission answer above stops holding (a `toml` or `regex` advisory, or a
maintenance stop); `UNKNOWN` handling for Git rules needs a fourth disposition beyond `review | block`; or
comment-preserving config editing becomes a stated user requirement, which is the trigger for revisiting
the `toml_edit` rejection.
