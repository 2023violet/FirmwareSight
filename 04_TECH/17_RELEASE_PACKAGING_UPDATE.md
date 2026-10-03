---
title: "Build, Packaging, Signing and Update Baseline"
doc_id: "FS-TECH-018"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-03"
---

# Build / Package / Sign / Update

## 1. Product lifecycle

FirmwareSight itself follows:

```text
Build
 ↓
Test
 ↓
Bundle
 ↓
OS Code Sign
 ↓
Installer
 ↓
Release
 ↓
Update Metadata
 ↓
Signed Auto Update (when enabled)
```

## 2. MVP distribution targets

### Windows — Tier 1
- x86_64 MSVC
- NSIS primary installer
- MSI may be produced for enterprise evaluation
- Authenticode signing before stable commercial release

### macOS — Tier 2 initially
- arm64
- x86_64
- Developer ID signing + notarization before stable distribution

### Linux — Tier 2
- x86_64
- AppImage
- `.deb`
- document WebKitGTK dependency expectations

## 3. CLI

Release same version as desktop.
Artifacts:
- Windows zip
- macOS/Linux tar.gz
- SHA-256

Package managers (Scoop/Homebrew/etc.) are post-MVP distribution channels.

## 4. Auto updater

Architecture is planned from v0.2.0, but updater is **disabled in MVP baseline**.

Enable only when:
- release hosting selected；
- update signing key generated；
- private key storage procedure exists；
- rollback/recovery tested；
- OS signing story documented；
- offline/enterprise disable switch exists。

Tauri updater signatures are separate from OS code signing.

## 5. Update key

Private updater key:
- never committed；
- CI secret/HSM/secure vault；
- documented recovery；
- ownership assigned。

Losing updater private key can strand installed clients, so this is a high-risk operational asset.

## 6. Versioning

SemVer:
`MAJOR.MINOR.PATCH`

Tag:
`vX.Y.Z`

Release pipeline must validate:
- Cargo package versions；
- Tauri app version；
- frontend package metadata；
- Git tag
are consistent.

## 7. Release provenance

FirmwareSight release publishes:
- installers/packages；
- CLI archives；
- SHA256SUMS；
- changelog；
- source commit；
- dependency/license report；
- update metadata only when updater enabled。

## P5 packaging (2026-10-03)

P5 turned `bundle.active` on. This section records what was chosen, what was measured, and what is
deliberately still not claimed. It changes no earlier clause of this document: §2's target list is
what was built, §3's CLI archive rule is what the companion artifact follows, and §4/§5's updater
boundary is untouched - `createUpdaterArtifacts` is now explicitly `false` in `tauri.conf.json`
rather than merely absent.

### Canonical target per platform, and why it is that one

| Platform | Target | Why, and what was checked first |
| --- | --- | --- |
| Windows | NSIS | §2 already names NSIS as the primary installer. `tauri-bundler`'s `Settings::package_types` intersects the configured targets with the host's list (`WindowsMsi, Nsis`), so a config that also names `deb` and `dmg` builds NSIS here and nothing else. MSI was not enabled: it pulls the WiX toolset for an "enterprise evaluation" the stage has no evidence to ask for. |
| macOS | `.app` + `.dmg` | §2's Tier 2 list. Both are configured because the `.dmg` is only a container for the `.app`, and the embedded-frontend check needs the `.app`'s bytes. |
| Ubuntu | `.deb` | §2 lists `.deb` and AppImage. AppImage needs `linuxdeploy` fetched at build time and FUSE at run time, neither of which this stage has any machine to test, so §8's "do not force a target the current runner/toolchain cannot support" excludes it for now rather than adding a download to CI to produce a file nobody has opened. |

`bundle.icon` keeps the five committed files. No `.icns` is committed, because
`tauri-bundler`'s `macos/icon.rs::create_icns_file` packs one from the listed PNGs when the list does
not contain an `.icns` - and the committed PNGs are RGBA at power-of-two sizes, which is what its
`add_icon_to_family` accepts. A committed `.icns` would have added a binary asset to the drift gate's
pixel comparison to reproduce something the bundler already does.

### Windows install mode

`bundle.windows.nsis.installMode: "currentUser"`. `tauri-utils` 2.10.0 (`config.rs:825-844`) makes
that the default, documents it as installing "in a directory that doesn't require Administrator
access" with metadata under `HKCU`, and offers `perMachine` and `both` - the latter of which demands
elevation even when the user then chooses per-user. P5's real-install acceptance runs on the owner's
machine, so the mode that cannot prompt for administrator rights is both the safer install and the
one that does not write to `HKLM`. It is written explicitly rather than left to a default so that a
later change to it is a diff a reviewer sees. `startMenuFolder` is set to `FirmwareSight` so the
shortcut lands in a named folder rather than loose in the Start Menu root.

### The frontend-embedding check, and the check that would have lied

Section 8 requires the production package not to depend on `localhost:5173`, and section 41 requires a
package job to verify the frontend is embedded. Measured on this tree, a release binary built
*without* `custom-protocol` and one built *with* it both contain the literal string
`localhost:5173`: the dev URL is part of the config Tauri embeds either way. Grepping for its absence
would therefore have failed every good build and passed every broken one.

What does discriminate is the built asset table. `apps/desktop/ui/dist/index.html` references
`assets/index-i5lbKrS9.js` and `assets/index-DyI80Tci.css`; the release binary compiled with
`custom-protocol` contains both keys, and the debug binary of the same crate contains neither. That is
the assertion `scripts/verify_package_artifacts.py` makes, and it reads the binary that was packaged -
extracted from the `.deb` with `dpkg-deb -x`, read from inside the `.app` - rather than whichever
binary happens to be in `target/`. On Windows the NSIS payload is compressed and this repository has
no stdlib way to open it, so the check runs against the exact file the bundler packed into the
installer and says so in the metadata; the behavioral proof that an installed app needs no dev server
is section 64's real-install acceptance, which is a stronger claim than a byte scan ever was.

The packaging path itself needs no remembered flag: `tauri-cli`'s `interface/rust.rs::build_options`
pushes `tauri/custom-protocol` for a release build, which is how §8's requirement is met.

### What a package job produces

`python scripts/check.py --only package` - the same command CI runs - produces the CLI companion
release binary, then the desktop package, then `target/dist-package/`. The desktop package step runs
`cargo tauri build -- --locked`, not `tauri build`: `cargo install tauri-cli` leaves a binary named
`cargo-tauri`, so the bare name only exists for the npm distribution. The group discovers which form this
machine answers to by probing both with `--version`, and a machine that answers neither reports `SKIP` —
and because a job whose purpose is an artifact must not go green without one, `check.py` exits non-zero on
any `SKIP` when `CI` is set. Run `37133706214` is the case the rule exists for: three jobs installed the
CLI, skipped the build, printed `4/4 steps passed`, and were caught only by their own empty artifact
upload.

The step also runs **from `apps/desktop`**, not from `apps/desktop/src-tauri`. The CLI resolves the
frontend directory from the process cwd (`tauri-cli-2.12.1/src/helpers/app_paths.rs:153-174`) and falls
back to the shell directory's parent when it finds no `package.json` there
(`resolve_dirs()`, same file, line 148); this repository keeps `ui/` and `src-tauri/` as siblings, so a cwd
inside the shell runs the config's `pnpm build` in a directory that is not a pnpm package. Measured, from
`src-tauri`: `[ERR_PNPM_NO_IMPORTER_MANIFEST_FOUND] No package.json … was found in "<workspace root>"`.
From `apps/desktop` the lookup answers `apps/desktop/ui`, the hook builds it, and the bundler produces the
installer. A hand-run build follows the same directory rule.

- `FirmwareSight-<version>-<platform>-<arch>-<kind>` for each package, and `-cli-fwsight` for the CLI
  companion, per §42's shape: product, version, platform, architecture, kind - no wall-clock stamp,
  no username, no host path;
- `SHA256SUMS.txt`, the **distribution** checksum index;
- `artifact-metadata.json`: the digests, the byte counts, where each version string was read from,
  the signing and updater status in words, and the reproducibility fields §18 of
  `04_TECH/18_CI_SUPPLY_CHAIN.md` asks a build to record.

The distribution `SHA256SUMS.txt` and a firmware Release Bundle's `SHA256SUMS` are different documents
about different things, and prompt §10 requires them to stay conceptually separate: the metadata file
says so in a field rather than relying on the filename. `04_TECH/17` §3's CLI archive convention
(Windows zip, macOS/Linux tar.gz) is carried by `06_DELIVERY`'s P5 record as an open item: this stage
publishes no release, so the companion ships inside the run's artifact set as a bare binary, and the
archive form is a release-channel decision rather than a packaging-mechanics one.

### Version identity is now a gate step, not a paragraph

`drift/version identity` compares `[workspace.package] version`, `tauri.conf.json`, the UI
`package.json` and `BASELINE.yaml product.baseline_version`, and requires every workspace crate to
inherit `version.workspace = true` so no manifest can hold a version string the check never reads.
Before P5 the repository's answer to "which version is this" was 0.6.0 in the governance documents and
0.1.0 in every artifact, and nothing failed. Both mutation directions are proven to bind: splitting
the UI package version reddens the step, and replacing one crate's `version.workspace = true` with a
literal names that crate. The package filename and the installer metadata are checked by the package
jobs against the same workspace version; the Desktop About panel and Diagnostics join this step in the
commits that create those surfaces.

### The Tauri CLI is a CI tool, not a dependency

`cargo install tauri-cli@2.12.1 --locked` in each package job, the same mechanism this workflow already
uses for `cargo-deny@0.20.2`. Justified per `AGENTS.md` §4: the requirement it answers is §8's
packaging path; the standard library and the existing toolchain cannot bundle an installer; its licence
is the Tauri set (MIT / Apache-2.0); it is maintenance-borne upstream and pinned exactly; it adds
nothing to the product binary because it never enters `Cargo.lock` - the workspace does not depend on
it, and `cargo-deny` still sees the whole graph. It is not in the trusted core. The npm-side
alternative (`@tauri-apps/cli` as a devDependency) was rejected because it would have widened the
frontend's audited lockfile for a tool the frontend does not use at runtime.

### What is not claimed

No GitHub Release, no tag, no published installer, no signing, no notarization, no updater. macOS and
Ubuntu packages are `CI_BUILD_ONLY`: their existence is proof a build runs on that runner, not proof
the product works there - the runtime evidence for Tier 2 remains the headless core smoke. The
`.app` carries `LSMinimumSystemVersion` at Tauri's own default (`10.13`), which is the bundler's
default and not a FirmwareSight support claim; the `.deb` declares no runtime dependencies, so the
WebKitGTK expectation §2 tells us to document is documented here rather than encoded in a package
nobody has installed. `P5_VALIDATION/P5_KNOWN_LIMITATIONS.md` carries each of these as a numbered
disposition.

