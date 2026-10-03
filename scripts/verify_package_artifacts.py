#!/usr/bin/env python3
"""Verify a platform's package set, then write its distribution artifacts and checksum index.

This is prompt section 41's "verify package version" and "verify frontend embedded" turned into a
program, plus the part of section 10 that has to be a file rather than a claim: a SHA256SUMS index
for *distribution* artifacts, kept separate from a firmware Release Bundle's own SHA256SUMS so the
two concepts cannot be confused for one another.

    python scripts/verify_package_artifacts.py                  # host platform, target/dist-package
    python scripts/verify_package_artifacts.py --print-prefix    # the section 42 name prefix only

Run it after `cargo tauri build` on the platform being packaged. It checks three things worth naming.

**The packaged version equals the workspace version.** `BASELINE.yaml` carried 0.6.0 while every
artifact said 0.1.0 until P5 unified them, so a package that takes its version from somewhere else
is exactly the defect that closure ended. The version is read from the package's own metadata - the
Windows resource table, the deb `Version` field, the .app's `CFBundleShortVersionString` - not from
the filename, because those three are what an operating system installs and a filename is only a
label on the way there.

**The frontend is inside the binary that shipped.** A release-compiled binary without Tauri's
`custom-protocol` feature is the failure this repository already documents at
`apps/desktop/src-tauri/Cargo.toml:9-13`: it starts, renders nothing offline, and works only if a
Vite server happens to be listening. Measured on this tree rather than assumed - with the feature
enabled the binary contains the built asset keys (`assets/index-*.js`, `assets/index-*.css`), and
without it contains neither. Both binaries contain the literal `localhost:5173`, because the dev URL
is part of the embedded config either way, so "no devUrl string in the binary" is a check that would
fail on a good build and pass on a broken one. The asset keys discriminate; the dev URL does not.

**The artifact set is complete and named the way section 42 says**: FirmwareSight, version,
platform, architecture, package kind - no wall-clock stamp, no username, no host path. `04_TECH/18`
additionally requires a build to record its toolchain, so the metadata file carries the rustc, node,
pnpm and tauri-cli versions, both lockfile digests, the commit and the runner image.

What this script does *not* claim: it does not open the NSIS container (there is no stdlib
decompressor for its payload), so on Windows the embedded check runs against the exact binary the
bundler packed and the metadata says so in as many words. The behavioral proof of "the installed app
runs with no dev server" is the real Windows install acceptance, not a byte scan.

Signing is not performed and not claimed. Every artifact here is unsigned, and the metadata records
that rather than leaving it to be inferred.

"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUNDLE = ROOT / "target" / "release" / "bundle"
# The frontend package. corepack reads `packageManager` from the nearest manifest, so a version
# recorded for the toolchain has to be asked *here*, not at the repository root.
UI = ROOT / "apps" / "desktop" / "ui"
DIST = UI / "dist"
CONFIG = ROOT / "apps" / "desktop" / "src-tauri" / "tauri.conf.json"

# The package kinds this project asks Tauri for, per host OS. It mirrors the filter in
# tauri-bundler's `Settings::package_types`, which intersects the configured targets with the
# platform's own list, so a target meant for another platform is never demanded of this host.
PLATFORM_KINDS = {
    "windows": ("nsis",),
    "linux": ("deb",),
    "darwin": ("app", "dmg"),
}
# tauri.conf.json does not set `app.mainBinaryName`, so the shipped executable keeps the crate's
# binary name. The embedded check has to open that exact file.
DESKTOP_BINARY = "firmwaresight-desktop"
CLI_BINARY = "fwsight"
KIND_SUFFIX = {"nsis": ".exe", "deb": ".deb", "dmg": ".dmg", "app": ".app", "cli": ""}


def resolve(name: str) -> str:
    """`shutil.which` does not apply PATHEXT, so a Windows command name needs the suffix tried."""
    for candidate in (name, f"{name}.exe", f"{name}.cmd", f"{name}.bat"):
        found = shutil.which(candidate)
        if found is not None:
            return found
    return name


def run(argv: list[str], cwd: Path | None = None) -> str:
    completed = subprocess.run(
        argv, cwd=cwd or ROOT, capture_output=True, text=True, check=False
    )
    if completed.returncode != 0:
        raise SystemExit(
            f"command failed (exit {completed.returncode}): {' '.join(argv)}\n"
            f"stdout: {completed.stdout.strip()}\nstderr: {completed.stderr.strip()}"
        )
    return (completed.stdout or "").strip()


def attempt(argv: list[str], cwd: Path | None = None) -> str:
    """A toolchain field is recorded as `unavailable` rather than failing a packaging run.

    A missing tool raises before it can return non-zero, so both shapes count as unavailable here;
    the required checks above deliberately do not swallow either one.

    `cwd` is not cosmetic: corepack reads the `packageManager` field from the nearest manifest, so
    asking it at the repository root - which has no `package.json` - answers with its own fallback
    rather than with the pnpm this project pins and that actually built the frontend.
    """
    try:
        return run(argv, cwd)
    except (SystemExit, OSError):
        return "unavailable"


def host_platform() -> str:
    return {"Windows": "windows", "Linux": "linux", "Darwin": "darwin"}[platform.system()]


def host_arch(plat: str) -> str:
    """The architecture token the platform reports for itself.

    Windows answers `AMD64` where every convention that matters writes `x86_64`, so that one name is
    translated. Linux answers `x86_64` and macOS answers `arm64` on the runner this project has, and
    neither is renamed: an artifact called `x86_64` because a script decided to spell it that way
    would be a claim about the machine rather than a record of it.
    """
    machine = platform.machine().lower()
    if plat == "windows" and machine in ("amd64", "x86_64"):
        return "x86_64"
    return machine


def workspace_version() -> str:
    import tomllib

    with (ROOT / "Cargo.toml").open("rb") as handle:
        return tomllib.load(handle)["workspace"]["package"]["version"]


def product_name() -> str:
    return json.loads(CONFIG.read_text(encoding="utf-8"))["productName"]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def sha256_tree(root: Path) -> str:
    """One digest over a directory bundle: every relative path and every byte, in sorted order.

    A `.app` is a directory, and a checksum index needs one line per artifact rather than one per
    file inside it. Sorting by relative path makes the digest independent of directory walk order.
    """
    files = sorted((p for p in root.rglob("*") if p.is_file()), key=lambda p: p.relative_to(root).as_posix())
    digest = hashlib.sha256()
    for path in files:
        digest.update(path.relative_to(root).as_posix().encode("utf-8"))
        digest.update(b"\0")
        with path.open("rb") as handle:
            for block in iter(lambda: handle.read(1 << 20), b""):
                digest.update(block)
    return digest.hexdigest()


def asset_keys() -> list[str]:
    """The frontend's own asset paths, read out of the index this build emitted."""
    if not (DIST / "index.html").is_file():
        raise SystemExit(
            f"{(DIST / 'index.html').relative_to(ROOT)} is missing. `cargo tauri build` produces it "
            "via tauri.conf.json's beforeBuildCommand, so run this after the build, not before it."
        )
    index = (DIST / "index.html").read_text(encoding="utf-8")
    keys = sorted(
        {ref.lstrip("/") for ref in re.findall(r'(?:src|href)="([^"]+)"', index) if ref.startswith("/assets/")}
    )
    if not keys:
        raise SystemExit(
            f"{(DIST / 'index.html').relative_to(ROOT)} references no /assets/ entry, so the "
            "embedded-frontend check has nothing to look for and would pass on an empty binary."
        )
    return keys


def missing_assets(binary: Path, keys: list[str]) -> list[str]:
    data = binary.read_bytes()
    return [key for key in keys if data.count(key.encode()) == 0]


def find_package(kind: str) -> Path:
    directory = BUNDLE / {"nsis": "nsis", "deb": "deb", "dmg": "dmg", "app": "macos"}[kind]
    if not directory.is_dir():
        listing = (
            ", ".join(sorted(p.name for p in BUNDLE.iterdir()))
            if BUNDLE.is_dir()
            else f"{BUNDLE.relative_to(ROOT)} does not exist at all"
        )
        raise SystemExit(
            f"no {kind} package directory at {directory.relative_to(ROOT)}. The bundle tree found "
            f"here is: {listing}. Either `cargo tauri build` did not run on this host or it did not "
            f"produce a {kind} target."
        )
    hits = sorted(p for p in directory.iterdir() if p.name.endswith(KIND_SUFFIX[kind]))
    if len(hits) != 1:
        raise SystemExit(
            f"expected exactly one {kind} package in {directory.relative_to(ROOT)}, found "
            f"{[p.name for p in hits]}"
        )
    return hits[0]


def app_bundle() -> Path:
    return BUNDLE / "macos" / f"{product_name()}.app"


def tauri_cli_version() -> str:
    """Record the CLI that produced this package, whichever invocation form this host answers to.

    `cargo install tauri-cli` - the form the CI package jobs use - leaves a binary named
    `cargo-tauri` and is run as `cargo tauri`, while the npm package leaves a bare `tauri`. Asking
    only for the bare name wrote `unavailable` into the reproducibility fields of a run that had
    just built a package with that exact CLI. `check.py`'s `tauri_command()` discovers the same way.
    """
    for argv in (
        [resolve("cargo"), "tauri", "--version"],
        [resolve("tauri"), "--version"],
    ):
        text = attempt(argv)
        if text != "unavailable":
            return text
    return "unavailable"


def packaged_binary(kind: str, package: Path, workdir: Path) -> tuple[Path, str]:
    """The executable to scan, and an honest label for what relation it bears to the package.

    Unpacking where an unpacker exists is the point: scanning `target/release/<binary>` alone would
    only prove that some binary on this machine embeds the frontend, not that the shipped one does.
    """
    if kind == "deb":
        unpacked = workdir / "deb"
        run(["dpkg-deb", "-x", str(package), str(unpacked)])
        candidates = sorted(
            p for p in unpacked.rglob(DESKTOP_BINARY) if p.is_file() and os.access(p, os.X_OK)
        )
        if len(candidates) != 1:
            raise SystemExit(
                f"expected one {DESKTOP_BINARY} inside {package.name}, found "
                f"{[str(p.relative_to(unpacked)) for p in candidates]}"
            )
        return candidates[0], f"extracted from {package.name} with dpkg-deb -x"
    if kind in ("app", "dmg"):
        # The `.app` directory carries the product name, but the executable inside it does not:
        # tauri-cli builds the bundle binary list from the Cargo bin name
        # (`interface/rust.rs:945` `BundleBinary::with_path(bin.file_name(), …)`) and
        # tauri-bundler copies each binary into `Contents/MacOS` under that same name
        # (`bundle/macos/app.rs:174` `dest_dir.join(bin.name())`). `bundle.mainBinaryName` would
        # rename it (`desktop.rs:336 rename_app`) and this config does not set it. Reading whichever
        # single file is there, instead of a name this script guessed, is what keeps a packaging
        # change from being reported as a missing binary.
        bundle = package if kind == "app" else app_bundle()
        bin_dir = bundle / "Contents" / "MacOS"
        if not bin_dir.is_dir():
            raise SystemExit(f"no executable directory at {bin_dir.relative_to(ROOT)}")
        candidates = sorted(p.name for p in bin_dir.iterdir() if p.is_file())
        if len(candidates) != 1:
            raise SystemExit(
                f"expected exactly one executable in {bin_dir.relative_to(ROOT)}, found {candidates}"
            )
        relation = (
            f"read from inside {package.name}"
            if kind == "app"
            else f"read from the {bundle.name} built in this same run, which the dmg wraps"
        )
        return bin_dir / candidates[0], relation
    if kind == "nsis":
        binary = ROOT / "target" / "release" / f"{DESKTOP_BINARY}.exe"
        if not binary.is_file():
            raise SystemExit(f"the binary the bundler packed was not found at {binary}")
        return (
            binary,
            f"read from target/release/{binary.name}, the exact file this installer bundles; the NSIS "
            "container itself is not opened here, and the real install acceptance is what proves the "
            "installed app runs offline",
        )
    raise SystemExit(f"no binary reader for package kind {kind!r}")


def ps_version_resource(path: Path) -> list[str]:
    return [
        "powershell",
        "-NoProfile",
        "-Command",
        f"(Get-Item -LiteralPath '{path}').VersionInfo.ProductVersion",
    ]


def windows_version_resource(path: Path) -> str:
    """The version an operating system reads out of a PE file's resource table."""
    return run(ps_version_resource(path))


def observed_version_resources(kind: str, package: Path) -> dict[str, str]:
    """What the distributable file itself reports, recorded rather than asserted.

    On Windows the file a user launches is the one inside the installer, and that is where the
    version check runs. Whether the NSIS stub carries its own VERSIONINFO entry is the bundler
    template's business, so it is recorded here as an observation instead of becoming a failure this
    repository would have to explain away.
    """
    if kind != "nsis":
        return {}
    return {"installer_version_resource": attempt(ps_version_resource(package)) or "(none reported)"}


def packaged_version(kind: str, package: Path) -> tuple[str, str]:
    """The version the operating system will see, plus where it was read from."""
    if kind == "nsis":
        binary = ROOT / "target" / "release" / f"{DESKTOP_BINARY}.exe"
        text = windows_version_resource(binary)
        return text, f"{binary.name}'s Windows version resource, which is the file this installer bundles"
    if kind == "deb":
        return run(["dpkg-deb", "-f", str(package), "Version"]), f"{package.name} control field Version"
    if kind == "app":
        info = package / "Contents" / "Info.plist"
        return (
            run(["plutil", "-extract", "CFBundleShortVersionString", "raw", str(info)]),
            f"{package.name}/Contents/Info.plist CFBundleShortVersionString",
        )
    if kind == "dmg":
        app = app_bundle()
        return packaged_version("app", app)[0], f"the {app.name} this dmg wraps"
    raise SystemExit(f"no version reader for package kind {kind!r}")


def distribution_name(kind: str, version: str, plat: str, arch: str, source: Path) -> str:
    """Section 42's shape: FirmwareSight, version, platform, architecture, package kind.

    A package keeps its container's extension. The CLI companion is archived, because
    `04_TECH/17` 3 fixes that form - zip on Windows, tar.gz elsewhere - so its kind token says both
    what it is (`cli`) and what is inside it (`fwsight`) rather than pretending to be an executable.
    """
    if kind == "cli":
        suffix = ".zip" if plat == "windows" else ".tar.gz"
        return f"FirmwareSight-{version}-{plat}-{arch}-cli-fwsight{suffix}"
    return f"FirmwareSight-{version}-{plat}-{arch}-{kind}{KIND_SUFFIX[kind] or source.suffix}"


def archive_cli(source: Path, out: Path, name: str) -> Path:
    """Wrap the CLI companion in the archive form `04_TECH/17` 3 fixes: zip on Windows, tar.gz else.

    The companion is a single executable, so the archive carries it at the top level with its
    ordinary command name inside - a stranger extracts it and runs `./fwsight`, not
    `./FirmwareSight-0.6.0-linux-x86_64-cli-fwsight`.
    """
    archive = out / name
    if archive.exists():
        archive.unlink()
    if source.suffix == ".exe":
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as bundle:
            bundle.write(source, arcname=source.name)
    else:
        with tarfile.open(archive, "w:gz") as bundle:
            bundle.add(source, arcname=source.name)
    return archive


def place(kind: str, source: Path, out: Path, version: str, plat: str, arch: str) -> dict[str, object]:
    """Copy one artifact into the distribution directory under its section 42 name."""
    name = distribution_name(kind, version, plat, arch, source)
    if kind == "cli":
        stored = archive_cli(source, out, name)
        return {
            "kind": kind,
            "name": name,
            "built_name": source.name,
            "sha256": sha256(stored),
            "bytes": stored.stat().st_size,
            "digest_scope": f"archive of {source.name}, written by this script",
        }
    target = out / name
    if source.is_dir():
        if target.exists():
            shutil.rmtree(target)
        shutil.copytree(source, target)
        digest = sha256_tree(target)
    else:
        shutil.copy2(source, target)
        digest = sha256(target)
    return {
        "kind": kind,
        "name": name,
        "built_name": source.name,
        "sha256": digest,
        "bytes": target.stat().st_size if target.is_file() else None,
        "digest_scope": "directory bundle, every file, sorted by relative path" if source.is_dir() else "file",
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Verify packaged artifacts and write their checksums.")
    parser.add_argument("--out", type=Path, default=ROOT / "target" / "dist-package")
    parser.add_argument(
        "--print-prefix",
        action="store_true",
        help="print the section 42 distribution name prefix for this host and exit",
    )
    options = parser.parse_args(argv[1:])

    plat = host_platform()
    arch = host_arch(plat)
    version = workspace_version()
    out = (options.out if options.out.is_absolute() else ROOT / options.out).resolve()

    if options.print_prefix:
        # The CI job names its uploaded artifact set with this line, so the section 42 shape is
        # written in exactly one place: a rename in the workflow cannot drift from the tool that
        # names the files inside the archive.
        print(f"FirmwareSight-{version}-{plat}-{arch}")
        return 0

    keys = asset_keys()
    out = options.out if options.out.is_absolute() else ROOT / options.out
    out.mkdir(parents=True, exist_ok=True)

    print(f"platform {plat} ({arch}) - version {version}")
    print(f"asset keys to prove embedded: {', '.join(keys)}")

    problems: list[str] = []
    entries: list[dict[str, object]] = []
    workdir = Path(tempfile.mkdtemp(prefix="firmwaresight-package-"))
    try:
        for kind in PLATFORM_KINDS[plat]:
            package = find_package(kind)
            observed, version_from = packaged_version(kind, package)
            if observed != version:
                problems.append(
                    f"{package.name}: packaged version {observed!r} (read from {version_from}) does not "
                    f"match [workspace.package] version {version!r}"
                )
            else:
                print(f"  {package.name}: version {observed} from {version_from}")

            binary, how = packaged_binary(kind, package, workdir)
            absent = missing_assets(binary, keys)
            if absent:
                problems.append(
                    f"{binary.name} ({how}) does not embed the built frontend; missing asset key(s): "
                    + ", ".join(absent)
                    + ". That is the `custom-protocol` failure described in the desktop crate's comment."
                )
            else:
                print(f"  {binary.name}: frontend embedded ({how})")

            entry = place(kind, package, out, version, plat, arch)
            entries.append({**entry, **observed_version_resources(kind, package)})

        cli = ROOT / "target" / "release" / (f"{CLI_BINARY}.exe" if plat == "windows" else CLI_BINARY)
        if not cli.is_file():
            problems.append(
                f"the CLI companion binary is missing at {cli.relative_to(ROOT)}. Prompt section 9 makes "
                "it part of every platform's artifact set, so an artifact set without it is incomplete."
            )
        else:
            entry = place("cli", cli, out, version, plat, arch)
            entries.append(entry)
            print(f"  CLI companion: {cli.name} -> {entry['name']}")
    finally:
        shutil.rmtree(workdir, ignore_errors=True)

    checksums = out / "SHA256SUMS.txt"
    if problems:
        # A failing verification must not leave behind an index that looks like it passed. The
        # per-artifact digests still go to stdout, because the point of a red run is to be readable.
        for entry in entries:
            print(f"  digested anyway: {entry['sha256']}  {entry['name']}", file=sys.stderr)
        for line in problems:
            print(f"  PROBLEM: {line}", file=sys.stderr)
        return 1

    checksums.write_text(
        "\n".join(f"{entry['sha256']}  {entry['name']}" for entry in sorted(entries, key=lambda e: e["name"]))
        + "\n",
        encoding="utf-8",
        newline="\n",
    )

    metadata = {
        "product": product_name(),
        "version": version,
        "platform": plat,
        "architecture": arch,
        "artifacts": entries,
        "checksum_index": checksums.name,
        "checksum_index_is": (
            "distribution artifacts built by CI. It is NOT the SHA256SUMS inside a firmware Release "
            "Bundle; prompt section 10 keeps the two concepts apart on purpose."
        ),
        "signing": "unsigned. No certificate, no signature, no notarization - the boundary and the "
        "strategy live in P5_VALIDATION/P5_PACKAGING_REPORT.md.",
        "updater": "not enabled; bundle.createUpdaterArtifacts is false in tauri.conf.json",
        "toolchain": {
            "rustc": attempt(["rustc", "--version"]),
            "cargo": attempt([resolve("cargo"), "--version"]),
            "node": attempt(["node", "--version"]),
            "pnpm": attempt([resolve("corepack"), "pnpm", "--version"], cwd=UI),
            "tauri_cli": tauri_cli_version(),
            "cargo_lock_sha256": sha256(ROOT / "Cargo.lock"),
            "pnpm_lock_sha256": sha256(ROOT / "apps" / "desktop" / "ui" / "pnpm-lock.yaml"),
            "git_commit": attempt([resolve("git"), "rev-parse", "HEAD"]),
            "runner_image": os.environ.get("ImageOS") or "not a GitHub runner",
            "runner_os": os.environ.get("RUNNER_OS") or platform.system(),
            "runner_arch": os.environ.get("RUNNER_ARCH") or platform.machine(),
        },
    }
    (out / "artifact-metadata.json").write_text(
        json.dumps(metadata, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n"
    )
    print(f"wrote {len(entries)} artifact(s), {checksums.name} and artifact-metadata.json to {out}")

    for line in problems:
        print(f"  PROBLEM: {line}", file=sys.stderr)
    return 1 if problems else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
