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
what has *not* been proved yet. Real Windows install acceptance (§38, §64, §65) and the first-use test
(§66) belong to `P5_INSTALL_RECOVERY_REPORT.md` and are **not** claimed here: as of this commit no
installer has been run on a machine, because the CI package jobs that produce one land with this
commit. Nothing below says `SUPPORTED`, `signed`, `notarized`, `beta`, `RC` or `GA`.

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
| `desktop package` | `tauri build -- --locked` (cwd `apps/desktop/src-tauri`) | **SKIPPED**, with the pinned install command printed |
| `artifacts verified` | `python scripts/verify_package_artifacts.py` | **SKIPPED** — no package exists to verify |

Measured here, where the Tauri CLI is deliberately not installed: the group reports `4/4 steps passed`
with those two rows labelled `(skipped)`, so a local green never means "packaged" (§13's discipline: a
result that was not measured is not reported as one).

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
| Windows | the NSIS installer `.exe` (Authenticode, after the payload is built and before upload); optionally the contained `firmwaresight-desktop.exe` | a step between `tauri build` and artifact upload in `Package Windows` | an Authenticode certificate in an HSM or a managed vault, plus a timestamp authority URL | `WINDOWS_SIGNING_IDENTITY` / `WINDOWS_SIGNING_PASSWORD` — **not created** | `signtool verify /pa /v <installer.exe>`; `Get-AuthenticodeSignature` in PowerShell |
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
| A package has been built at all | the three package jobs' first run on `main` | pending — CI runs after this commit |
| A Windows installer installs, launches offline, uninstalls and reinstalls | `P5_INSTALL_RECOVERY_REPORT.md` (§38 A–L, §64, §65) | pending, and it is the next thing that must happen |
| macOS / Ubuntu packages build on their runners | `Package macOS` / `Package Ubuntu` | pending; until observed, `CI_BUILD_ONLY` is not even earned |
| Installed app needs no Rust, Cargo, Node, pnpm, Vite or a checkout (§65) | the real-install session, on the cleanest practical environment | pending |
| Uninstall behaviour on user data | measured, never asserted (§39 records it) | pending |
| Desktop About panel and Diagnostics as version surfaces | the commits that add them join `drift/version identity` | not built yet |
| The `.icns` derivation on a real macOS runner | `Package macOS` | assumed from tauri-bundler's source, unverified by a run |

Evidence for anything in this document that cites a command carries the command; anything cited from
upstream source names the file and lines; nothing here is recalled from a previous round.
