---
title: "P5 Packaging Report"
doc_id: "FS-P5-PACKAGING"
product: "FirmwareSight"
version: "1.0"
status: "IN_PROGRESS"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-03"
---

# P5 — packaging (prompt §8, §9, §10, §11, §12, §41, §42, §65)

**Scope of this document.** It records what packaging was enabled, what was measured to decide it, and
what has *not* been proved yet. §5b documents the first installer this repository has produced — built on
this host with the pinned CLI — and it is a built file, not an installed one: no installer has been run on
a machine yet. Real Windows install acceptance (§38, §64, §65) and the first-use test (§66) belong to
`P5_INSTALL_RECOVERY_REPORT.md` and are **not** claimed here, and no CI package job has produced an
artifact yet either. Nothing below says `SUPPORTED`, `signed`, `notarized`, `beta`, `RC` or `GA`.

## 1. Configuration that changed

`apps/desktop/src-tauri/tauri.conf.json`:

| Field | Before | After | Why |
| --- | --- | --- | --- |
| `bundle.active` | `false` | `true` | §8 "enable real packaging". |
| `bundle.targets` | `"all"` | `["nsis", "app", "dmg", "deb"]` | §8 asks for the canonical target per platform to be recorded; `"all"` would also ask for `msi`, `rpm` and `appimage`, which need toolchains this stage has no evidence to require. |
| `bundle.windows.nsis.installMode` | (absent) | `"currentUser"` | See §3. |
| `bundle.windows.nsis.startMenuFolder` | (absent) | `"FirmwareSight"` | A named folder rather than loose shortcuts in the Start Menu root (§39 records Start Menu behavior as required evidence). |
| `bundle.createUpdaterArtifacts` | (absent, so `false`) | explicit `false` | §12's boundary is easier to keep if it is written down; `tauri-utils` `Updater::default()` is `Bool(false)`, so this changes no behaviour. |
| `bundle.publisher` / `category` / `shortDescription` / `longDescription` | (absent) | `FirmwareSight` / `DeveloperTool` / two sentences | A `.deb` control file and a Linux desktop entry need them; leaving them unset would ship a package whose description is empty. They state what the product does locally and say no more. |
| `bundle.icon` | 5 files | unchanged, **no `.icns` added** | See §2. |

`license` is not duplicated in the config: `tauri-utils` falls back to `Cargo.toml`, which the workspace
already sets to `Proprietary`. `copyright` is left unset — naming a legal entity is the owner's call, and
`OPEN_SOURCE_LICENSE_DECISION` is still `PENDING_OWNER_CONFIRMATION` (§54).

## 2. Why there is still no committed `.icns`

`tauri-bundler`'s `macos/icon.rs::create_icns_file` first looks for an `.icns` in the icon list and,
finding none, **packs one from the listed images** — it opens each, treats `@2x` as density 2, resizes any
non-power-of-two side down, and writes the family, erroring only if no usable icon exists at all. The five
committed files are RGBA at 32, 128, 256 (`128x128@2x`) and 512 (`icon.png`) plus the `.ico`, which is
exactly the input that function accepts, so the macOS build derives its icon at build time.

Committing an `.icns` as well would add a binary asset to `drift/desktop icons`, whose check compares
*decoded pixels rather than bytes* because a CI run once reported three icons as stale on Linux while their
pixels matched (§7 of the P0 record). Reproducing that derivation in a generator, and then verifying it,
buys nothing the bundler does not already do. This is a decision to re-test the moment a macOS runner
disagrees: the `Package macOS` job is the first thing that will tell us.

## 3. The canonical target per platform, and the mechanism behind it

`tauri-bundler`'s `Settings::package_types` (source lines 1118-1149) intersects the configured target list
with the host's own list — macOS `[app, dmg]`, Linux `[deb, rpm, appimage]`, Windows `[msi, nsis]` — and
drops the rest. That is why one config can name all four targets and why `python scripts/check.py --only
package` needs no `--bundles`: the same command on each runner builds only what that runner can build, so
no job has to remember a platform flag. §8's requirement is met the same way for the cargo feature:
`tauri-cli`'s `interface/rust.rs::build_options` pushes `tauri/custom-protocol` into the release build
itself. Nobody has to type `--features custom-protocol` to obtain an official package.

| Platform | Canonical target | Rejected alternative, and the reason |
| --- | --- | --- |
| Windows | NSIS | MSI: needs the WiX toolset and serves the "enterprise evaluation" option in `04_TECH/17` §2 that no evidence in this stage asks for. |
| macOS | `.app` + `.dmg` | `.app` alone is not distributable; `.dmg` alone leaves the embedded-frontend check without a readable binary. Both are configured. |
| Ubuntu | `.deb` | AppImage: fetches `linuxdeploy` during the build and expects FUSE at run time, on the platform this round may only ever call `CI_BUILD_ONLY`. `rpm` is not in the baseline's Linux list. |

`installMode: currentUser` is `NSISInstallerMode`'s own default (`tauri-utils` 2.10.0 `config.rs:825-844`),
documented there as installing without Administrator access and writing metadata under `HKCU`. It is set
explicitly so that a change to it is a reviewable diff. `perMachine` would require elevation; `both`
prompts for administrator rights even when the user then chooses per-user, which §39's safety list and the
owner's decision to install on their own machine both argue against.

## 4. The check that would have passed a broken build

§8 requires the production package not to depend on `localhost:5173`. The obvious automated reading —
"assert the dev URL is absent from the shipped binary" — was measured here and **rejected**.

On this host, two binaries of the same crate, same day, same `dist/`:

| Binary | Build | `assets/index-i5lbKrS9.js` | `assets/index-DyI80Tci.css` | `localhost:5173` |
| --- | --- | --- | --- | --- |
| `target/release/firmwaresight-desktop.exe` (15,001,088 B) | `--release --features custom-protocol` | present | present | **present** |
| `target/debug/firmwaresight-desktop.exe` (22,581,760 B) | debug, no `custom-protocol` | **absent** | **absent** | **present** |

The dev URL is part of the config Tauri embeds either way, so its presence proves nothing and its absence
would fail every good build. What does discriminate is the built asset table: the keys `tauri-codegen`
compiles in are the paths `dist/index.html` references. `scripts/verify_package_artifacts.py` therefore
reads those keys out of the built frontend and requires each one in the binary that shipped — extracted
from the `.deb` with `dpkg-deb -x`, read from inside the `.app` — and the negative control is the table
above: point the same check at the dev-mode binary and it exits 1 naming both missing keys.

One limit stated rather than papered over: **the NSIS payload is not opened.** NSIS compresses its content
and this repository has no stdlib way to decompress it, so on Windows the check runs against
`target/release/firmwaresight-desktop.exe` — the exact file the bundler packs — and `artifact-metadata.json`
records that in words. The claim "the installed app runs with no dev server" is §64's real-install
acceptance, which is a stronger form of evidence than a byte scan; it is pending, and this document does
not pre-claim it.

## 5. What a package job produces

`python scripts/check.py --only package`, run by each of the three CI package jobs and by any developer on
any host:

| Step | Command | On a machine without the Tauri CLI |
| --- | --- | --- |
| `frontend deps` | `corepack pnpm install --frozen-lockfile` | runs |
| `cli companion` | `cargo build --release --locked -p fwsight` | runs |
| `desktop package` | `cargo tauri build -- --locked` (cwd `apps/desktop/src-tauri`) | **SKIPPED**, with the pinned install command printed |
| `artifacts verified` | `python scripts/verify_package_artifacts.py` | **SKIPPED** — no package exists to verify |

The CLI is found by probing both forms with `--version` and using whichever answered, because the install
method decides the filename: `cargo install tauri-cli` leaves `cargo-tauri`, which is run as
`cargo tauri <args>` — cargo forwards the subcommand token and the CLI strips it
(`crates/tauri-cli/src/main.rs` at tag `tauri-cli-v2.12.1`) — while the npm package leaves a bare `tauri`.
Checking only the bare name is what made the three package jobs of Run `37133706214` install the CLI,
report `SKIPPED: the Tauri CLI is not installed on this machine`, and go on printing `4/4 steps passed`.

Two rules close that class of lie, both measured here:

- **a skip is not a pass.** The summary now prints `SKIP` rows and counts them separately: the same tree
  that used to say `4/4 steps passed` says `2/4 steps passed, 2 skipped` (`scripts/check.py` records the
  exit code `None` for a step that ran nothing).
- **CI may not skip.** Every tool the gate needs is installed by the job itself, so a skip in CI means the
  job did not check what it claims to have checked. `CI` set in the environment turns any `SKIP` row into
  exit 1 and names it on stderr. Measured on this host with the CLI absent: `python scripts/check.py
  --only package` → exit 0 with the two SKIP rows; `CI=true python scripts/check.py --only package` →
  exit 1 and `SKIPPED IN CI: package/desktop package`, `SKIPPED IN CI: package/artifacts verified`.

The `deny` group's cargo-deny skip moved to the same machinery, so it reports `SKIP` and fails CI for the
same reason. It is not a live CI condition: `Dependency policy` installs `cargo-deny@0.20.2` one step
earlier, and this host has it (`cargo-deny.exe` on `PATH`), so the group still runs it for real here.

The third proof is the one that shows the fix is not just a nicer message: with a stand-in `cargo-tauri.exe`
placed on `PATH` — a Rust binary that prints its argv and exits 0, never presented as the real CLI — the
same command found it and invoked the build it used to skip:

```
[package] Tauri CLI found: cargo tauri - tauri-cli 2.12.1 (stand-in, not the real CLI)
=== [package] desktop package
$ C:\Users\16429\.cargo\bin\cargo.EXE tauri build -- --locked
STAND-IN cargo-tauri argv=["…\cargo-tauri.exe", "tauri", "build", "--", "--locked"]
FAIL(1)    package/artifacts verified
```

That output is evidence about argument forwarding and about the group's honesty, and nothing else: the
stand-in built no package, so `verify_package_artifacts.py` failed it —
`no nsis package directory at target\release\bundle\nsis` — and the group exited 1 rather than reporting a
packaging success it had not had. It also confirms the verifier refuses a build that claims to have
succeeded: a `desktop package` step that exits 0 without producing a bundle tree cannot green the group.
The shim was a 139,776-byte Rust binary compiled in a temp directory, and the temp directory is gone; the
pinned CLI described next is what replaced it on this host.

### 5a. The directory the CLI has to be run from

The stand-in could not have found this. With the real CLI installed, the same command from
`apps/desktop/src-tauri` failed one step later, in the config's own hook:

```
$ cargo tauri build -- --locked
     Running beforeBuildCommand `pnpm build`
[ERR_PNPM_NO_IMPORTER_MANIFEST_FOUND] No package.json (or package.yaml, or package.json5) was found
  in "D:\study\Software\FirmwareSight".
   Error beforeBuildCommand `pnpm build` failed with exit code 1
FAILED: desktop package (exit 1)
```

The cause is in the CLI's path resolution (`helpers/app_paths.rs:153-174` at 2.12.1): the frontend
directory is resolved from the **process cwd**, not from the shell's directory. `resolve_frontend_dir()`
looks for a `package.json` at the cwd, then walks *down* three levels for one; finding none inside
`src-tauri`, `resolve_dirs()` falls back to `tauri.parent()` (`app_paths.rs:148`) = `apps/desktop`, and
`run_hook()` executes the hook there (`helpers/mod.rs:80,94`). This repository keeps `ui/` and
`src-tauri/` as siblings, so the hook ran in a directory that is not a pnpm package, and pnpm walked up
to the workspace root and reported it.

Running the build from **`apps/desktop`** makes the CLI's own lookup answer `apps/desktop/ui`, which is
where the frontend lives, and the hook then builds it. `scripts/check.py` now invokes the package step with
that cwd, with the reason in a comment beside it. The alternative the CLI offers —
`build.beforeBuildCommand` as `{script, cwd}` (`tauri-utils-2.10.0/src/config.rs:3741 HookCommand`) — was
rejected because its `cwd` is also resolved against wherever the person typing the command happened to
stand, which is the same trap in a config file. A developer typing `cargo tauri build` by hand gets the
same instruction: run it from `apps/desktop`.

### 5b. The first package this repository has produced

From this host, Windows 10.0.19045 x86_64, with `cargo install tauri-cli@2.12.1 --locked` actually present,
`python scripts/check.py --only package` ran all four steps for real and returned `4/4 steps passed`, exit 0:

```
[package] Tauri CLI found: cargo tauri - tauri-cli 2.12.1
     Running makensis to produce …\target\release\bundle\nsis\FirmwareSight_0.6.0_x64-setup.exe
    Finished 1 bundle at:
        …\target\release\bundle\nsis\FirmwareSight_0.6.0_x64-setup.exe (3.64 MiB)
```

The build log is `tauri_build_from_approot.log` in the evidence root, and it records the two things §41 asks
a package job to prove. The bundler patched and packed
`D:\study\Software\FirmwareSight\target\release\firmwaresight-desktop.exe` — the assumption that the NSIS
installer carries that exact file is now evidence rather than an assumption — and it downloaded NSIS 3.11
and `nsis_tauri_utils` 0.5.3 from the pinned releases with hash validation. The produced distribution set:

| Artifact | Bytes | SHA-256 |
| --- | --- | --- |
| `FirmwareSight-0.6.0-windows-x86_64-nsis.exe` | 3,811,140 | `c5c8cf231aa90bb2686b199335797b99c6a4ecd2972d863d267ed0bf091cf328` |
| `FirmwareSight-0.6.0-windows-x86_64-cli-fwsight.zip` | 1,668,706 | `761c5a4d21bfa7ba81bc874fd4c3ef2e4682e44bf8b32d1f71953a436e76db07` |

These two lines are this build's artifact set, and §5e explains why they are the only ones that can be:
rebuilding the same tree here gives different digests, and the runner's set (§5c) differs again.

`sha256sum -c SHA256SUMS.txt` reports both `OK`; the zip lists `['fwsight.exe']` at the top level; and the
version the verifier read is the installer's own payload — `0.6.0` out of
`firmwaresight-desktop.exe`'s Windows version resource, with the NSIS stub's own `0.6.0` recorded as an
observation beside it — not a filename.

Producing this set corrected two recorded fields that the stub run had written wrong, both in
`artifact-metadata.json`'s toolchain block:

| Field | Before | After | Why it was wrong |
| --- | --- | --- | --- |
| `tauri_cli` | `unavailable` | `tauri-cli 2.12.1` | the field probed the bare `tauri` name, the same filename assumption that reddened Run `37133706214` |
| `pnpm` | `12.6.0` | `12.7.0` | corepack reads `packageManager` from the nearest manifest, and the repository root has no `package.json`, so it answered with its own fallback instead of the pinned version `apps/desktop/ui/package.json:6` declares |

Both are now read from a directory where the tool actually has a manifest, and the values above are what a
run of `scripts/verify_package_artifacts.py` on this tree records.

### 5c. The three CI package jobs, and what their artifacts said back

Run `37138881977` at head `1055242` is the first 10-of-10 green run, and the first that attached a package
for every platform (§41's evidence, §42's names). Each set was downloaded with
`gh run download 37138881977` and read back here rather than described from the log:

| Artifact set (the run's attachment name) | Contents | Largest file |
| --- | --- | --- |
| `FirmwareSight-0.6.0-windows-x86_64` | `…-nsis.exe` 3,812,717 B, `…-cli-fwsight.zip` 1,668,377 B, `SHA256SUMS.txt`, `artifact-metadata.json` | the installer |
| `FirmwareSight-0.6.0-linux-x86_64` | `…-deb.deb` 5,365,916 B, `…-cli-fwsight.tar.gz` 1,834,782 B, index, metadata | the `.deb` |
| `FirmwareSight-0.6.0-darwin-arm64` | `…-app.app/` (directory), `…-dmg.dmg` 4,864,786 B, `…-cli-fwsight.tar.gz` 1,635,429 B, index, metadata | the `.dmg` |

`sha256sum -c` over the Windows and Linux indexes returns `OK` on every line. Each set's
`artifact-metadata.json` records `tauri_cli: tauri-cli 2.12.1`, `pnpm: 12.7.0`,
`rustc 1.98.1 (48a229cea 2026-09-01)`, `git_commit 1055242`, and the runner it came from
(`win25-vs2026` / `ubuntu24` / `macos26`).

What the runners proved that this host could not (§9's pending rows, now closed):

- **macOS derived an `.icns` from the committed PNGs and shipped.** `Bundling FirmwareSight.app` and
  `Bundling FirmwareSight_0.6.0_aarch64.dmg` both completed; no `.icns` is in the repository, and the
  bundler's own `tauri-icns` crate compiled on the runner.
- **The `.app`'s executable is the Cargo bin name, exactly as §9 predicted from source.** The verifier's
  line reads `firmwaresight-desktop: frontend embedded (read from inside FirmwareSight.app)` — it found the
  file by taking whatever single entry `Contents/MacOS` holds, and the name it read back was
  `firmwaresight-desktop`, not `FirmwareSight`. Had the reader kept its old guess, this job would have
  failed on a good build.
- **The `.deb` was really unpacked and the binary inside it scanned**:
  `firmwaresight-desktop: frontend embedded (extracted from FirmwareSight_0.6.0_amd64.deb with dpkg-deb -x)`.
- **The Linux job built the shell against the shared prerequisite action**, so the apt list that used to be
  copied between jobs is now exercised from one place.
- **§42's names and the bundler's names are different tokens, and the metadata keeps both.** The runner's
  `.deb` is `FirmwareSight_0.6.0_amd64.deb` and its `.dmg` is `FirmwareSight_0.6.0_aarch64.dmg`, while the
  distribution names carry the architecture the *host reported* (`x86_64`, `arm64`). Each entry records
  `built_name` beside `name`, so nothing is lost between the two conventions.

### 5d. The macOS index line the runner exposed, and what it cost

The downloaded darwin set was the only one whose index did not verify:

```
$ sha256sum -c SHA256SUMS.txt
sha256sum: FirmwareSight-0.6.0-darwin-arm64-app.app: Is a directory
FirmwareSight-0.6.0-darwin-arm64-app.app: FAILED open or read
FirmwareSight-0.6.0-darwin-arm64-cli-fwsight.tar.gz: OK
FirmwareSight-0.6.0-darwin-arm64-dmg.dmg: OK
sha256sum: WARNING: 1 listed file could not be read
```

The `.app` is a directory, and the index gave it one line carrying an aggregate tree digest. That digest is
well defined — §5's `sha256_tree()` sorts the bundle's relative paths and folds every byte in — but the
standard tool named by §41 cannot read a directory, so a correct macOS artifact set came home reporting a
`FAILED` line. This is the mirror image of the `localhost:5173` mistake in §4: an assertion written to be
readable that a good build cannot satisfy.

The index now keeps both properties. A directory bundle contributes **one line per file**, with the path
written relative to the index (`…-app.app/Contents/MacOS/firmwaresight-desktop`), so `sha256sum -c` verifies
every byte of the bundle; the aggregate tree digest stays in `artifact-metadata.json` as the artifact's
single identity, and `bytes` becomes the sum of the bundle's files instead of `null`. Proved on a synthetic
three-file bundle on this host: the index lists the three members sorted by path, `sha256sum -c` exits 0,
changing one byte in `Contents/Info.plist` makes it exit 1 and name that file `FAILED`, and the aggregate
tree digest moves with the same byte. A re-run of the Windows and Linux sets is unchanged: two lines, both
`OK`.

**The runner has not yet said this back.** The fix is local-evidence only until a package job produces a
darwin set whose index verifies line for line, and this document will not call it closed until that run is
read from `gh run download`, not from the job's summary.

`verify_package_artifacts.py` then writes, into `target/dist-package/`:

- one file per artifact named per §42 — `FirmwareSight-<version>-<platform>-<arch>-<kind>`, e.g.
  `FirmwareSight-0.6.0-windows-x86_64-nsis.exe` — no wall-clock stamp, no username, no host path;
- the CLI companion as `…-cli-fwsight.zip` on Windows and `…-cli-fwsight.tar.gz` elsewhere, per
  `04_TECH/17` §3, containing the binary under its ordinary name (`fwsight.exe` at the top level — verified
  by listing the archive);
- `SHA256SUMS.txt`, the **distribution** checksum index, verified readable by `sha256sum -c`;
- `artifact-metadata.json`: digests, byte counts, `digest_scope` for each artifact (a `.app` is a directory
  and is digested over its sorted file list rather than pretending to be one blob), where each version
  string was read from, the signing and updater status in words, and the reproducibility fields
  `04_TECH/18` §"Build reproducibility" asks a build to record — rustc, cargo, node, pnpm and tauri-cli
  versions, `Cargo.lock` and `pnpm-lock.yaml` digests, the commit, and the runner image.

Fail-closed behaviour is tested: with no `target/release/bundle/nsis` directory the tool exits with the
path it looked for and what it found instead, and it writes **no** `SHA256SUMS.txt` when any verification
problem is pending — a red run must not leave behind an index that looks like a pass.

Packaged-version verification reads the value an operating system uses, not a filename: the Windows
version resource (`ProductVersion 0.6.0`, `ProductName FirmwareSight` on the binary the installer bundles),
the deb `Version` control field via `dpkg-deb -f`, and the `.app`'s `CFBundleShortVersionString` via
`plutil`. Each is compared to `[workspace.package] version`. One measured asymmetry is recorded rather than
hidden: `fwsight.exe` carries **no** Windows version resource at all, so the CLI companion's version is
proved by running it — `fwsight 0.6.0`.

### 5e. What a digest can compare, measured on the payload

Two builds of the same tree on this host produced installers the bundler itself reported as 3.64 MiB and
3.63 MiB, which is unsurprising for a container. What would matter for `04_TECH/18` "Build reproducibility"
is the payload, so two consecutive ones were kept and compared byte by byte:
`target/release/firmwaresight-desktop.exe`, 15,001,088 bytes both times, **20 bytes different** — and those
20 are all linker identity, not code:

| Offset | Bytes | What lives there | Measured |
| --- | --- | --- | --- |
| `256` | 4 | the PE COFF `TimeDateStamp` | `0x6ac14091` = 2026-10-03T17:51:13Z, and `0x6ac14191` = 17:55:29Z — 256 s apart, i.e. the two link times |
| `12598421`, `12598449`, `12598477` | 4 each | the same `TimeDateStamp` copied into three debug-directory entries, at the 28-byte `IMAGE_DEBUG_DIRECTORY` stride | one byte differs in each, the low-order byte of that second |
| `12600616…12600631` | 16 | the RSDS CodeView **GUID**, 4 bytes after the `RSDS` magic at `12600612` | `53b528c9b1f36844943429366aa95e1c` against `1bad884cde63df40a882d01d0a54da21` — the linker randomises it per link |

Same length, and nothing outside those 20 bytes differs: the embedded frontend, the version resource and the
instruction stream are identical between builds. So a package digest is **not** an equality key across
builds, on this host or across hosts, and no sentence in this repository may claim that an installer or a
payload binary reproduces byte for byte. The reproducibility record stays what §18 actually asks for —
toolchain versions, both lockfile digests, the commit, the runner image — and `payload_sha256` exists so
that a difference can be counted and attributed, as it was here, rather than assumed to be the container's
fault.

This is why §5d's index change matters more than it looks: when byte-level identity of a bundle cannot be
claimed, what a stranger can still check is the per-file index, file by file, of the set they were given.

## 6. Distribution checksums are not bundle checksums

§10 requires the concepts kept apart, so they are apart in three places, not one:

| Index | Written by | About |
| --- | --- | --- |
| `target/dist-package/SHA256SUMS.txt` | `scripts/verify_package_artifacts.py`, in CI | the installable files this stage builds |
| a firmware Release Bundle's `SHA256SUMS` | `firmwaresight-report` / the CLI, in the product | the analysed firmware and its evidence, per ADR-0020's bundle layout |
| the repository's root `SHA256SUMS` | `scripts/generate_baseline_artifacts.py sums` | the tracked source tree of this project |

`artifact-metadata.json` carries a `checksum_index_is` field naming that distinction, so a reader of the
artifact set cannot mistake it for a bundle manifest.

## 7. Signing: `READY_NOT_EXECUTED`

§11 asks for readiness, not a signed release. Nothing here signs, and nothing here pretends: no
certificate exists, no key is committed, no CI secret is referenced by name in any job, and the package
jobs' only credential is the default `contents: read` permission the workflow already carries.

**Strategy, per platform — prepared, not executed.**

| Platform | What would be signed | Where in the pipeline | Required credential | CI secret name (placeholder only) | Verification command |
| --- | --- | --- | --- | --- | --- |
| Windows | the NSIS installer `.exe` (Authenticode, after the payload is built and before upload); optionally the contained `firmwaresight-desktop.exe` | a step between `cargo tauri build` and artifact upload in `Package Windows` | an Authenticode certificate in an HSM or a managed vault, plus a timestamp authority URL | `WINDOWS_SIGNING_IDENTITY` / `WINDOWS_SIGNING_PASSWORD` — **not created** | `signtool verify /pa /v <installer.exe>`; `Get-AuthenticodeSignature` in PowerShell |
| macOS | the `.app` bundle (`codesign --force --deep --options runtime`), then the notarization ticket stapled to the `.app` and to the `.dmg` | between the `.app` build and the `.dmg` step in `Package macOS` | Apple Developer ID application certificate, its private key in a keychain, plus App Store Connect API key for notarytool | `MACOS_CERTIFICATE` / `MACOS_CERTIFICATE_PWD` / `APPLE_ID` / `APPLE_APP_SPECIFIC_PASSWORD` / `APPLE_TEAM_ID` — **not created** | `codesign --verify --deep --strict --verbose=2 <app>`; `spctl -a -vv <app>`; `stapler validate <dmg>` |
| Linux | none. Integrity is the distribution `SHA256SUMS.txt`; a deb may optionally be signed with `debsig-verify` against a maintainer keyring | n/a for this stage | a Debian maintainer GPG key (a policy decision, not a build step) | — | `sha256sum -c SHA256SUMS.txt` (this is what P5 can prove today) |

**Failure behaviour, stated before it is needed.** Signing is a *release* step: if a credential is absent
or a signing step fails, the build must fail closed and publish nothing, rather than uploading an
unsigned artifact under a name that implies otherwise. Under P5 the opposite is also true and is the
current state — the package jobs upload unsigned artifacts to the run and the metadata says `unsigned`,
because an unsigned artifact in a *staging* run with a stated status is evidence, while an unsigned
artifact presented as a release would be a false claim.

**What is not permitted here.** No self-signed certificate standing in for a production one, no committed
private key, no signing call reading secrets from source, no notarization retry loop that hides a
rejection, and no sentence in any P5 document that says signed, notarized, trusted or gated. §11 is
explicit that this does not block technical P5 closure but does block any later `RC` or `GA` claim, so the
line is kept in the documents, not in a build flag.

## 8. Updates: `UPDATE_READY_MANUAL`

§12 forbids an automatic network updater without separate authority, and this commit adds none: no
updater plugin, no endpoint, no manifest, no background service, and `createUpdaterArtifacts: false`. The
product's network boundary is unchanged and remains verifiable where it always was: `deny.toml` bans
`reqwest`, `hyper`, `sqlx`, `wgpu`, `axum` and `tonic` outright (`[bans] deny`, lines 76-84) with
`skip = []`, and its graph is walked only over the four shipping targets (lines 23-28) — which is the
mechanism that keeps tauri 2.12.0's Android/iOS-only `reqwest` edge, still present in `Cargo.lock` at line
2777, from being mistaken for a dependency the desktop can reach. A desktop build that actually pulled an
HTTP client would fail the `deny` job rather than needing this sentence to notice.

Manual upgrade semantics for P5, written where a user will find them and repeated in
`P5_INSTALL_RECOVERY_REPORT.md` once measured:

- **Upgrade = install the newer package over the older one.** Same install mode, same per-user location;
  NSIS `currentUser` replaces the application files and leaves `%APPDATA%\com.firmwaresight.desktop\`
  alone, so the store is not part of the upgrade path.
- **The store migrates, it is never reset.** `firmwaresight-storage` applies additive numbered migrations
  in a per-migration transaction and refuses a database from a future version rather than touching it
  (measured by `a_database_from_the_future_is_refused_and_left_alone`). A 0.6.0 install therefore upgrades
  a v5 store in place; the §21 matrix over real stores is Commit D's work, and the rule from
  `04_TECH/15`'s P5 section still bounds it — no build carrying migration `0005` may be installed over a
  user's real store before the backup capability exists.
- **Version comparison is informational.** The version a user sees is the artifact version
  (`About`/`fwsight --version`), and the release identity a Gate compares is content-derived
  (`release-<digest>`), so "same version, different bytes" is possible and is what the Gate exists to
  catch. That distinction is L22's question and stays with the Architect.
- **Future updater integration point, documented not built:** Tauri's updater plugin plus minisign
  signatures plus a hosted static manifest. Its preconditions are the six conditions in `04_TECH/17` §4
  and the key rules in its §5, none of which is satisfied. Enabling it stays an ADR-level move
  (`AGENTS.md` §2 forbids changing the updater model silently), and `AUTO_UPDATE_SUPPORTED` is not written
  anywhere in this repository.

## 9. What is still missing from this report

| Item | Where it gets proved | Status |
| --- | --- | --- |
| A package has been built at all | the three package jobs on `main` | **closed for all three platforms.** Windows here in §5b, and Windows / Ubuntu / macOS on the runner in §5c |
| A macOS and an Ubuntu package build on their runners | `Package macOS` / `Package Ubuntu` | **closed by §5c** — `.app` + `.dmg` and `.deb` produced and verified. This is `CI_BUILD_ONLY` in the owner's words: proof a package builds, not proof anyone ran one |
| The `.icns` derivation on a real macOS runner | `Package macOS` | **closed by §5c** — `tauri-icns` compiled, `Bundling FirmwareSight.app` completed, and no `.icns` is committed (§2's decision holds) |
| The executable name inside `FirmwareSight.app/Contents/MacOS` | `Package macOS` | **closed by §5c** — the reader found `firmwaresight-desktop`, the Cargo bin name the source said it would be |
| `dpkg-deb -x` extraction and the version field read | `Package Ubuntu` | **closed by §5c** — the embedded-frontend check ran on the binary extracted from the `.deb` |
| The `apps/desktop` working directory on Linux and macOS | `Package Ubuntu` / `Package macOS` | **closed by §5c** — all three jobs ran the same `check.py --only package` from that directory and each produced a package |
| A checksum index a stranger can verify, for a `.app` | `Package macOS` | **open.** §5d: the runner's darwin index carries one line naming a directory, which `sha256sum -c` reports `FAILED` for. Fixed here and proved on a synthetic bundle; it needs the next run's darwin artifact set read back from `gh run download` before this row closes |
| A Windows installer installs, launches offline, uninstalls and reinstalls | `P5_INSTALL_RECOVERY_REPORT.md` (§38 A–L, §64, §65) | pending, and it is the next thing that must happen. The installer exists on both this host and the runner, so this is no longer blocked on packaging |
| Installed app needs no Rust, Cargo, Node, pnpm, Vite or a checkout (§65) | the real-install session, on the cleanest practical environment | pending |
| Uninstall behaviour on user data | measured, never asserted (§39 records it) | pending |
| Desktop About panel and Diagnostics as version surfaces | the commits that add them join `drift/version identity` | not built yet |
| Are the produced packages byte-reproducible? | `04_TECH/18`'s reproducibility fields, if a claim is ever made over an installer | **measured and attributed — and the answer is no.** Two builds of the same tree on this host produced installers of 3,811,140 / 3,809,059 / 3,808,294 bytes, and the runner's was 3,812,717; the payload inside differs too, in exactly **20 of 15,001,088 bytes** (§5e): the PE `TimeDateStamp` and the 16-byte RSDS CodeView GUID, which the linker sets per link. No digest in this document is an equality key across builds, and nothing here may claim an installer reproduces byte for byte |


Evidence for anything in this document that cites a command carries the command; anything cited from
upstream source names the file and lines; nothing here is recalled from a previous round.
