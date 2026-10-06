---
title: "P5 Release Readiness"
doc_id: "FS-P5-RELEASE-READINESS"
product: "FirmwareSight"
version: "0.6.0"
status: "VALIDATED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 — release readiness states, as Commit F2 leaves them

Every line is a **state**, not an achievement. Several of these say "not authorised", and the point of
writing them down is that no one can later read a green CI run as a licence to ship.

| State | Value | What it means, and what would change it |
| --- | --- | --- |
| `SIGNING_READINESS` | **READY_NOT_EXECUTED** | The Windows package is built by CI and its artifact set is verified file by file, but nothing is Authenticode-signed. `signtool verify` was run against unsigned bytes and reported what unsigned bytes look like. Executing this needs a certificate, a key-management decision and an owner with a budget |
| `NOTARIZATION_READINESS` | **READY_NOT_EXECUTED** | The macOS package builds and its index reads correctly, but no notarization has been submitted. Notarization needs Apple credentials this project does not hold and has never asked for |
| `UPDATE_MODE` | **MANUAL_UPGRADE_READY** | There is no updater. Upgrading means running the newer installer over the older one; the store migrates on first launch and a pre-migration snapshot is written beside it first. The installed round trip — install, repair over a running app, uninstall, reinstall, reopen the same v5 store with no migration re-run — was measured on the F1 artifact. `AGENTS.md` §2 keeps an updater behind a signing and key-management ADR, and no such ADR exists |
| `OPEN_SOURCE_LICENSE_DECISION` | **PENDING_OWNER_CONFIRMATION** | The workspace is `license = "Proprietary"` with no root `LICENSE`. `AGENTS.md` §9 makes a licence change a human decision, and no P5 prompt let an agent make it. **No `LICENSE` file was added by any P5 commit** |
| `PUBLIC_DISTRIBUTION` | **NOT_AUTHORIZED** | Nothing here has been given to anyone outside this machine |
| `GITHUB_RELEASE` | **NOT_CREATED** | No release object exists for any P5 head |
| `TAG` | **NOT_CREATED** | No annotated or lightweight tag was pushed by P5. The `v1.2.3` tag that appears in the Gate evidence belongs to a **disposable synthetic repository** under `%TEMP%`, created by this round's project builder and removed with it — it is not a repository tag and must not be read as one |
| `PRIVATE_BETA` | **NOT_AUTHORIZED** | L11 — user comprehension with real people — is still open, with V0 at `0 / 8`. A beta is the instrument that would close it, and P5 has no user panel |
| `RC` | **NOT_AUTHORIZED** | No release candidate is declared. When F2 wrote this row `P5` was `IN_PROGRESS`; Commit F3 closed the stage `PASS_COMPLETE` on 2026-10-06 and the state above did not move — closing an engineering stage is not opening a release candidate |
| `GA` | **NOT_AUTHORIZED** | Not claimed, and not claimable by this commit |

## 1. What F2 added to the readiness picture

F2 did not move any state above. What it did was convert a set of *assertions about installability* into
*measurements of an installed build*, which is the difference between "ready" meaning something and meaning
nothing:

- the exact CI-built artifact installs, runs, repairs, uninstalls, reinstalls and uninstalls again on a real
  Windows machine, without ever elevating;
- an older store migrates under the installed binary and the user's History survives it, proved on 34
  measured checks;
- a support conversation can start from one exported file that carries no path in it;
- uninstalling does not remove a user's data, and the user is asked before anything could.

That is what `MANUAL_UPGRADE_READY` now rests on. Before F2 it rested on a local build and a shorter walk.

## 2. Boundaries that survive into any release decision

| Boundary | Where it is written |
| --- | --- |
| Installed migration covers one path (v4 → v5); the owner's real store is at v2, so the chained v2 → v5 installed upgrade has never run | `P5_MIGRATION_RECOVERY_REPORT.md` §4 |
| The uninstaller's "Delete the application data" option was never exercised | `P5_INSTALL_RECOVERY_REPORT.md` §8 |
| WebView2 is an untested negative — this host already carries it | `P5_SECURITY_SUPPORTABILITY_REVIEW.md` §5 |
| One host, one DPI setting, one WebView2 version (`154.0.4258.53`) | `P5_DESKTOP_ACCEPTANCE_REPORT.md` §7 |
| The dependency policy passes **with documented accepted risks**, which is not the same sentence as "security clean" | `P5_SECURITY_SUPPORTABILITY_REVIEW.md` §2 |

## 3. What would have to be true before any of the NOT_AUTHORIZED rows could move

Not a plan — P5 does not own the next stage — but the preconditions are worth naming so the states are not
read as arbitrary:

- **Signing / notarization**: a certificate and key-management decision, recorded as an ADR, because
  `AGENTS.md` §2 treats the signing and updater model as baseline and §9 treats a key change as a
  human-confirmed operation.
- **Updater**: the same ADR, and it is the same decision — an unsigned updater is not an updater.
- **Private beta**: a V1 user-validation track with real participants, which closes L11, plus a licence
  decision if the artefact leaves the machine.
- **Public distribution / RC / GA**: a licence, a release identity policy (`ADR-0028` already fixes what an
  identity *is*, not what is published), and a stage gate that P5 does not own.

## 4. The sentence this file exists to prevent

Nothing in P5's evidence supports shipping this build. It supports saying: the artifact CI built at head
`0bca373` installs and behaves as documented on a real Windows machine, its data is safe across the whole
cycle, and the engineering stage can now be **reviewed**. Reviewing is not releasing, and the next decision is
the Architect's, not this commit's.

*(written by Commit F2 on 2026-10-05, when its own last clause ended in `F3 = READY_FOR_ARCHITECT_REVIEW`. That
marker is gone from the repository: the Architect issued F3, F3 ran, and §6 below records where each state
stands now. The paragraph above is left intact because it is what F2's evidence supports, and it still is.)*

## 5. Where the ten states stand after Commit F3 (2026-10-06)

F3 is the governance closure head. It changed no product byte, so it moved no state above — and it re-read each
one rather than assuming that because the stage passed, something downstream of it did.

```
SIGNING_READINESS              = READY_NOT_EXECUTED
NOTARIZATION_READINESS         = READY_NOT_EXECUTED
UPDATE_MODE                    = MANUAL_UPGRADE_READY
OPEN_SOURCE_LICENSE_DECISION   = PENDING_OWNER_CONFIRMATION
PUBLIC_DISTRIBUTION            = NOT_AUTHORIZED
GITHUB_RELEASE                 = NOT_CREATED
TAG                            = NOT_CREATED
PRIVATE_BETA                   = NOT_AUTHORIZED
RC                             = NOT_AUTHORIZED
GA                             = NOT_AUTHORIZED
```

Ten states, all ten exactly as F2 recorded them. F3 executed none of them, created none of them and opened
none of them, and `P5 = PASS_COMPLETE` is not evidence about any of them: a stage closing means its own scope
was met, not that a distribution decision was taken.

The one sentence §14 requires, and the one that follows from `OPEN_SOURCE_LICENSE_DECISION` rather than from
anything technical:

```
PUBLIC OPEN-SOURCE REDISTRIBUTION CLAIM = BLOCKED BY OWNER LICENSE DECISION
```

That is the only `BLOCKED` in the P5 record, and it is a claim about redistribution, not a required engineering
item — which is why §6's re-audit could pass with it in place. FirmwareSight is a **proprietary MVP candidate**
with a complete local-first engineering stage: `license = "Proprietary"` stands in the root `Cargo.toml`, there
is no root `LICENSE`, and no P5 commit added one. It is not a finished licensed open-source release, and the
absence of that sentence is not an oversight for an agent to repair.

What would change any of this is stated in §3 and remains the owner's and the Architect's, not a future
round's: a certificate and key-management ADR before signing or an updater, Apple credentials before
notarization, a real-user V1 panel before a beta, a licence before redistribution, and a stage gate P5 does not
own before RC or GA.
