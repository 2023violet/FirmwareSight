---
title: "P5 Security and Supportability Review"
doc_id: "FS-P5-SECURITY-SUPPORTABILITY"
product: "FirmwareSight"
version: "0.6.0"
status: "IN_PROGRESS"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-05"
---

# P5 — security and supportability, as the installed build leaves them

Commit F2 changed no product, tooling, dependency or capability code, so this review adds no new
guarantee. What it can do is report what the **shipping artifact** was observed to hold open on a real
machine, which is a different evidence class from a source read.

**The sentence this document will never write is "security clean."** The accurate form, used since G2 and
still true: *the dependency policy passes with documented accepted risks.*

## 1. Attack surface, as measured on the installed binary

| Surface | Measured state | Why it matters |
| --- | --- | --- |
| Network | the running app owned **no TCP endpoint** (`netstat -ano` filtered on its PID returned nothing); no HTTP client, no server, no telemetry, no updater plugin | the local-first claim is a measurement here, not a policy statement |
| WebView origin | `http://tauri.localhost/`, the packaged custom protocol — not a dev server, and no `node`/`vite`/`cargo` in the process tree | a shipped build cannot accidentally be serving from a development port |
| Shell / filesystem capability | capabilities are minimal and use-case oriented; the web frontend gets no general shell and no general filesystem grant. Every file F2 reached was reached through a **native picker the user drove** | the "read arbitrary file" pattern `AGENTS.md` §7 forbids is absent by construction and was exercised through real dialogs |
| Privilege | `installMode: currentUser`; installed to `%LOCALAPPDATA%\FirmwareSight`, registered under `HKCU`, and **UAC was never prompted** across three installs, two repairs and three uninstalls. No `HKLM` key exists before or after | nothing in P5 writes a machine-wide location |
| Unsafe code | project-authored `unsafe` remains forbidden in the core, and the core stays headless: no Tauri, no React, no SQLite, no Tokio types in `firmwaresight-core` | the Core-first boundary in `AGENTS.md` §3 held through the whole stage |
| Panic on user input | no panic was produced by any user-supplied path in F2, including a deliberately MAP-less analysis and a renamed source tree. The no-panic claim still **rests on `fixtures/malformed/*` regression coverage, not on fuzzing** — L3 stays open, blocked by toolchain policy rather than by effort | the honest version of "robust" |

## 2. Dependency policy

`cargo deny` reports `advisories ok, bans ok, licenses ok, sources ok`. Two advisories are **accepted with
written reasons** in `deny.toml`, and neither is silenced by widening a scope:

| Advisory | Accepted because |
| --- | --- |
| `RUSTSEC-2024-0429` — `glib` 0.18.5, unsound in `glib::VariantStrIter` | reached only through the Linux WebView chain `gtk 0.18.2 ← tauri 2.12.0`; the advisory's fix needs `>=0.20.0` and `cargo update -p glib --precise 0.20.0` fails because gtk requires `^0.18`. Moving glib means moving gtk-rs, muda, tao and webkit2gtk — the frozen Tauri dependency architecture, which needs an ADR rather than a CI fix. P0 never calls GVariant APIs and glib is not compiled for Windows or macOS |
| `RUSTSEC-2024-0370` — `proc-macro-error` 1.0.4, unmaintained | a build-time proc macro arriving via `glib-macros 0.18.5`, again pinned by the gtk-rs 0.18 line the frozen Tauri requires. It runs during `cargo build` on the host, is not linked into the shipped binary, and is not in the graph for Windows or macOS |

Commit E re-measured this and recorded L12 as **REVISITED / CARRIED_FORWARD**: the compatible upgrade that
would clear them does not exist. `unused-ignored-advisory` stays at its `warn` default, so an entry that
stops matching is itself reported — the acceptance cannot quietly become stale.

Licence posture is unchanged and is **not** an engineering finding: the workspace is
`license = "Proprietary"` with no root `LICENSE`. Choosing a licence is an owner decision under `AGENTS.md`
§9, and no P5 prompt let an agent make it.

## 3. Privacy boundary, re-proved at the installed layer

The claim has two halves and F2 measured both.

- **Nothing leaves the machine.** No network capability, no upload path, no telemetry.
- **What a user hands to support carries no path.** The 1,122-byte diagnostics export was parsed outside the
  product and asserted against 11 absence classes — no firmware path, no project root, no database path, no
  bundle destination, no git remote, no username or home, no Release Notes body, no symbol name, no firmware
  byte, no MAP content, no environment dump — all clean, with 37 keys inside a 41-key allowlist.

The storage side is unchanged and still by design (L18): `artifacts.path` holds the location the user chose,
because that is the storage of record, and the display / IPC / redaction chain keeps it local. F2 confirmed
the user-visible half of that in the installed window: History shows leaf names only, and the build detail
says so in the product's own words.

## 4. Supportability — what a support conversation can actually start from

This is the part P5 was for: a stranger engineer, no dev team present.

**A support request can begin with one file.** Help → Export diagnostics produces a bounded JSON that names
the product, version, identifier, logical store file name, platform, Tauri and WebView2 versions, schema
version against supported version, store health, journal mode, five row counts, pre-migration snapshot names,
install channel, loaded release policy and up to eight stable error codes. F2 took that file from the
installed build, parsed it independently, and confirmed its counts matched a read-only query of the same
store at the same moment — so the file is coherent as well as private.

**Failure is inspectable rather than fatal.** The migration machinery refuses to upgrade a store it reports
unhealthy, takes a named snapshot before touching an older one, and stops with the store left at its old
version if the snapshot cannot be written. Gate runs are immutable, so a wrong verdict is superseded by a new
row rather than edited.

**Recovery from a bad install was exercised, not described.** Repair over the top of a running install
(§21) prompted before terminating the app, re-extracted a byte-identical payload, rewrote the uninstaller and
shortcuts, and left the store's logical content unchanged. Uninstall removed the program, both shortcuts and
the registration while preserving user data. Reinstall reopened the same store at v5 with no migration
re-run.

## 5. Supportability gaps worth naming, none of them S0/S1

| Gap | Class | Note |
| --- | --- | --- |
| Both native dialogs restore the OS last-used folder, which on this machine is a private project directory. A user who analysed private firmware and then exports diagnostics gets a dialog silently offering to drop the file into that same tree | **OBSERVATION**, ordinary Windows behaviour | Nothing leaves the machine and the payload carries no path either way, so the privacy position is unaffected. It is a documentation and expectation issue for support, and it is why §19 and §24 each cancelled one dialog before writing |
| The installer's running-app prompt reads "FirmwareSight is running! Click OK to kill it", with the buttons localised to the host (确定 / 取消) while the wizard body stays English | **OBSERVATION** | Behaviour is right — it asks rather than killing silently — and it worked on both the repair and the uninstall path. The wording and the mixed localisation are not |
| `NoModify` and `NoRepair` are set in the registry while the NSIS installer still presents a maintenance page | **OBSERVATION** | Windows "Apps & features" therefore offers neither button, so repair is reachable only by re-running the installer. The two surfaces disagree about whether repair is offered |
| The install directory after uninstall is not stable: one observed uninstall left it empty, three removed it entirely | **OBSERVATION**, unresolved | The user-facing wording must not promise either outcome; it says "normally removed, an empty directory may remain as installer residue" |
| WebView2 is a hard runtime dependency and this host already carries it | **KNOWN_LIMITATION**, F2-3 | "Runs without Rust/Cargo/Node/pnpm/Vite" is evidenced as *does not use them*, never as *would fail without them*. A machine with no WebView2 is still untested |
| Installed migration coverage is one path (v4 → v5) and the owner's real store is at v2 | **KNOWN_LIMITATION**, F2-2 | the chained v2 → v5 upgrade a returning user would receive has never been run by an installed build. `P5_MIGRATION_RECOVERY_REPORT.md` §4 |
| Every History list F2 walked held 1–3 rows | **bounded honestly** | the read APIs and the export counts are bounded, but a store with thousands of builds has never been walked in an installed window |

## 6. What a reviewer should not take from this file

Not a security audit, not a penetration test, and not a claim that the accepted advisories are safe — the
claim is that they are **documented, measured as unupgradable inside the frozen architecture, and visible to
`cargo deny` if they stop applying**. Not a distribution readiness statement: signing, notarization and
publishing are all explicitly not authorised, and `P5_RELEASE_READINESS.md` carries those states.
