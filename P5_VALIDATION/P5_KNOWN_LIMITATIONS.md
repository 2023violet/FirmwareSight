---
title: "P5 Known Limitations"
doc_id: "FS-P5-LIMITS"
product: "FirmwareSight"
version: "0.6.0"
status: "IN_PROGRESS"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-05"
---

# P5 — known limitations, as the installed build leaves them

`apps/desktop/ui/src/Help.tsx:50` names this path to the reader, and until Commit F2 the file was not
there. The screen is honest about that: it lists four documents, marks three `shipped: false`, and says
an installed package carries a binary rather than a source tree. So creating this file fixes a pointer a
stranger could not follow without touching a line of product code — which Commit F2 is not allowed to do
(its own §35), and does not need to.

**What this list is.** Every row inherited from `G2_VALIDATION/G2_KNOWN_LIMITATIONS.md` (L1–L25) plus
everything P5 added (L26), each carrying its disposition **as of the F1 build measured on a real machine
in F2**. No closed row is dropped: the G2 file stays the historical record of `e35cfe7`, and this one is
the current answer. Where the two disagree, the disagreement is about the *tree*, not about the fact —
the G2 row is never rewritten, because it reports what was true on the audited commit.

**What it is not.** Not a security clearance ("dependency policy passes with documented accepted risks",
never "security clean"), not a release authorisation, and not a claim that P5 passed. `P5_RELEASE_READINESS.md`
carries the states, and Commit F3 owns the closure sentence this document is forbidden to write.

Disposition words are the ones P5 already uses: **CLOSED**, **CLOSED BY REVALIDATION**, **REDUCED**,
**CARRIED_FORWARD** (with a reason), **NOT_REPRODUCED**, **OWNER_DECISION**, **BY_DESIGN**.

## 1. Rows P5 closed

| # | Limitation | Disposition | What closed it, and what it does not cover |
| --- | --- | --- | --- |
| L6 | `sections.file_offset` reason lost | **CLOSED** | migration `0005_unknown_reasons` added `sections.file_offset_unknown`; the window now says *why* (".bss — the section has no file range; it occupies no bytes on disk"). F2 saw those sentences rendered by the installed binary, not by a test |
| L7 | `symbols.address` reason lost | **CLOSED** | same migration, `symbols.address_unknown` |
| L10 | No History surface | **CLOSED** | Commit C's History page over three bounded storage **read** APIs and no new table; F2 proved it in the installed build, including after the firmware folder had been moved away |
| L15 | `ElfProgramHeader` source label | **CLOSED for presentation**, **CARRIED_FORWARD for the wire** — see §3 | the display-only half is what closed |
| L16 | `custom-protocol` needed by hand | **CLOSED** | the package group builds 4/4 with `cargo tauri build -- --locked` and no manual feature flag |
| L19 | "Object attribution" meant two things | **CLOSED** | Analyze and Compare now name their own scope, each with a test |
| L20 | Compare printed a Core enum word | **CLOSED** | closed display mapping over the five current variants plus an Unknown-safe fallback |
| L21 | Window title fixed at "…- Analyze" | **CLOSED** | the title now tracks the page in Rust over a closed page enum. F2 read it off the installed window on all five pages: `FirmwareSight - Analyze / Compare / Release / History / Help` |
| L14 | `update_goldens.py` key order, unverified | **CLOSED BY REVALIDATION** | dry-run twice, 14 goldens `unchanged`, tree clean afterwards |
| L24 | `update_goldens.py` cannot rerun over its Windows leftover | **CLOSED / NOT_REPRODUCED ON CURRENT HEAD** | second dry-run over the scratch the first left, no `PermissionError [WinError 5]` |
| L26 | `SHA256SUMS` verifies only on the host that wrote it | **CLOSED** | `ADR-0029` decided it and Commit F1 implemented it: the root baseline now digests each path's canonical Git stage-0 index blob, and `scripts/verify_baseline_artifacts.py` runs inside the authoritative drift step, so all three CI platforms prove it |

## 2. Rows P5 reduced but did not eliminate

| # | Limitation | Disposition | Residue that is still true |
| --- | --- | --- | --- |
| L1 | Peak RSS not measured | **REDUCED** | a ~1,428 MB *working set* figure exists for the 519 MB workload on one host. That is a working-set reading, not a peak-RSS counter, and no memory ceiling is claimed for any workload |
| L2 | 500 MB in-window not measured | **REDUCED** | measured warm ~4.7–4.9 s and cold ~68.7 s, never "Not Responding"; "first use under 60 s" stays **PARTIAL / environment-sensitive**, and the cold number must never be quoted as typical |
| L4 | One Windows host, one WebView2, 100 % DPI | **REDUCED** | the matrix now names each platform's evidence class. Still no second host, no 125/150 % DPI, and — new in F2 — see §4 about WebView2 |
| L5 | Two linker layouts, ELF only, DWARF unread | **REDUCED** | the Commit E cohort added layout shapes, compiler variety and a debug/no-debug pair. DWARF is still recognised and not consumed |
| L17 | CI provisioning duplication, index blind spot | **REDUCED** | the duplication is gone; the two canonical lists that exist were checked |
| L23 | UI test races in the same shape | **REDUCED** | seven instances fixed and proved by mutation, 20/20 fresh-process runs. Another may exist that no run has lost yet |
| L22 | Line endings move a release's identity | **DECIDED, NOT ELIMINATED** | `ADR-0028` settled that release identity is the exact on-disk bytes, so the behaviour is now the documented rule rather than a surprise. The fail-closed protection stays policy-dependent: `require_clean_git = false` makes `git.clean` N/A and lets the id move silently |

## 3. L15, in two halves, because it is two different claims

The identifier `ElfProgramHeader` (storage) / `elf.program-header` (wire) names the memory basis taken
from ELF program headers. It is serialised in three namespaces at once, so renaming it is a public
contract change, not a label fix. The Architect answered the stop on 2026-10-04 with **Option E**: keep
the identifier, document the meaning, and fix what the user is shown.

- **Presentation — CLOSED.** `Details.tsx` captions the token as **"ELF address + flags evidence"**. F2
  proved this in the *installed* binary rather than inheriting the unit test: the target ELF was analysed
  deliberately without its MAP, which forces the basis to `ElfAddressAndFlags` and therefore forces that
  evidence row on screen. The caption was there.
- **Wire — CARRIED_FORWARD, reason `LEGACY_WIRE_IDENTIFIER`.** `analysis:1`, the stored value, the DTO,
  the goldens and the Release Bundle all still carry the old identifier, and `04_TECH/23` §7 documents
  what it means. Anyone who reads "closed" from the caption alone is wrong, which is why the row is split
  here instead of being filed once.

## 4. Carried forward into the next stage

| # | Limitation | Disposition | Why it is still open |
| --- | --- | --- | --- |
| L3 | Fuzzing not run | **CARRIED_FORWARD** | blocked by policy, not by effort: `ADR-0016` plus the pinned `1.98.1` toolchain mean `cargo-fuzz` cannot run without a toolchain-policy change. The no-panic claim keeps resting on `fixtures/malformed/*` |
| L8 | Object / module attribution | **CARRIED_FORWARD** | the MAP facts are parsed but reach no product surface; no narrow current path exists to expose them without a feature decision |
| L9 | Same-pair lock in the window never mouse-verified | **CARRIED_FORWARD** | the native `<select>` popup is not drivable by this harness. F2's own driver is real `SendInput` mouse and keyboard, and it still could not reach that popup — which is evidence for the limitation, not against it |
| L11 | User comprehension untested (V0 0/8) | **CARRIED_FORWARD, owner V1** | P5 has no user panel and §60 never asked for one. Whether people read `Unknown`, `Review` and a MAP request correctly remains unverified |
| L18 | Local database retains the chosen artifact location | **BY_DESIGN** | adjudicated by the G2 Storage Path Semantics Clarification Addendum. `artifacts.path` is the storage of record and the display/IPC/redaction chain keeps it local. F2 adds a related observation below, not a contradiction |
| L25 | One dead `Apply filter` click, never reproduced | **NOT_REPRODUCED** | 30 legitimate Apply activations across both tables plus 6 through form submit, no failure |

## 5. Owner decision, not engineering debt

| # | Item | State |
| --- | --- | --- |
| L13 | `license = "Proprietary"` in the root `Cargo.toml`, no root `LICENSE` | **OWNER_DECISION**. `AGENTS.md` §9 puts a licence change in front of the owner and the F prompts forbid an agent from choosing one. No `LICENSE` file was added in P5 |

## 6. What Commit F2 added to this list

F2 changed no product code, so it could not close a code row — but it measured things no earlier round
had, and three of them belong here rather than only in the acceptance report.

| # | Item | Disposition | Evidence |
| --- | --- | --- | --- |
| F2-1 | Installed migration coverage is **one path**: synthetic v4 → v5. Fresh→v5 and v1/v2/v3→v5 are storage-integration proofs (`every_older_file_backed_schema_is_snapshotted_at_the_version_it_was_found` loops `for found in 1..=4`), not installed-binary proofs | **BY_DESIGN, bounded honestly** | `P5_MIGRATION_RECOVERY_REPORT.md` §4 |
| F2-2 | The owner's real store on this machine is at **schema v2**, so the chained v2 → v5 path that a returning pre-P5 user would take has never been executed by an installed build | **CARRIED_FORWARD — INSTALLED_PATH_UNCOVERED** | `17_owner_restore/RESTORE.txt` and `RESTORE_second_pass.txt`, both reading `schema_version 2` |
| F2-3 | WebView2 is a hard runtime dependency and this host already carries it. "Runs without Rust, Cargo, Node, pnpm or Vite" is evidenced as *does not use them*, never as *would fail without them*; a machine with no WebView2 is still untested | **CARRIED_FORWARD — CLEAN_ENVIRONMENT_UNTESTED** | `P5_INSTALL_RECOVERY_REPORT.md` §5 |

Two smaller facts that a reader could otherwise take the wrong way:

- **The install directory after uninstall is not stable across runs.** Four observed uninstalls: one
  left `%LOCALAPPDATA%\FirmwareSight` behind **empty**, three removed it entirely. No rule has been
  worked out, so the user-facing wording says the directory is normally removed and an empty one may
  remain as installer residue — it does not promise either outcome. See
  `P5_INSTALL_RECOVERY_REPORT.md` §8.
- **The uninstaller does offer to delete local data.** It presents "Delete the application data",
  unchecked by default. An earlier round recorded that no such question was asked; on the F1 CI artifact
  the question is there. F2 never exercised the checked path, because that folder also holds unrelated
  historical stores from earlier phases — recorded as a coverage boundary, not a pass. See
  `P5_INSTALL_RECOVERY_REPORT.md` §8.

## 7. Findings F2 raised that are not limitations

Everything F2 observed is classified in `P5_DESKTOP_ACCEPTANCE_REPORT.md` §6. None is S0 or S1, so §27's
stop does not apply, and none was patched — a silent patch would have invalidated the F1 installer as the
candidate under test. Three are product presentation or layout observations for whoever owns them next
(narrow-window section-table text collapse; a clipped "Details" button at the default width; the Release
page's Evidence basis column printing the raw Core enum word `MapRegionAndElfLoad` while every other page
prints human text, which is L20's shape reappearing on a surface L20 did not cover). The rest belong to
the test driver, and they are listed there under their own heading rather than quietly dropped.
