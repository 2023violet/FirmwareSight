#!/usr/bin/env python3
"""Run the whole P0 verification gate, locally and in CI, from one definition.

CI and a developer machine must not disagree about what "green" means, so the workflow calls
this script instead of carrying its own copy of the command list.

Usage:
    python scripts/check.py                     # everything
    python scripts/check.py --only rust         # one group
    python scripts/check.py --list

Groups: rust, frontend, drift, deny, package, and core-smoke - the last being the headless subset
without the Tauri shell, which is what the macOS job in 05_ENGINEERING/06_CI_CD_BASELINE.md asks for.
It is a real local command, not a CI-only copy: run `python scripts/check.py --only core-smoke` on any
host. The default full gate does not repeat it, because the `rust` group already covers every one
of those packages plus the shell. The `package` group is likewise not in the default set: it builds a
release binary and a platform installer, which takes minutes rather than seconds and only means
something on the platform it runs on, so the three CI package jobs invoke it explicitly with
`--only package`. A machine without the Tauri CLI records that group as SKIPPED rather than as a
pass it never made.

Exit code is non-zero on the first failing step; every step's command line is printed before it
runs so a failure can be reproduced by hand.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UI = ROOT / "apps" / "desktop" / "ui"
TAURI = ROOT / "apps" / "desktop" / "src-tauri"
# Tauri's production codegen embeds this directory at compile time.
UI_MARKER = UI / "dist" / "index.html"
# The headless product: named so a platform that ships Core but not the shell can still be gated.
# A workspace member missing from this list is silently not covered by `--only core-smoke`, which is
# why ADR-0027's new crate is added here in the same commit that creates it.
CORE_PACKAGES = (
    "firmwaresight-core",
    "firmwaresight-artifact",
    "firmwaresight-project",
    "firmwaresight-storage",
    "firmwaresight-report",
    "fwsight",
)


def resolve(name: str) -> str:
    """Find the real executable behind a command name.

    Python's `subprocess` does not apply PATHEXT the way a shell does, so on Windows `cargo`
    has to be resolved to `cargo.exe` (or the npm `.cmd` shims) before it can be run.
    """
    for candidate in (name, f"{name}.exe", f"{name}.cmd", f"{name}.bat"):
        found = shutil.which(candidate)
        if found is not None:
            return found
    return name


def cargo() -> str:
    """Cargo's launcher filename differs on Windows even inside the same shell."""
    return resolve("cargo")


def pnpm_command() -> list[str]:
    """Corepack rather than an assumed global pnpm: the version pinned in package.json's
    `packageManager` field is the one that wrote the lockfile."""
    return [resolve("corepack"), "pnpm"]


class Gate:
    def __init__(self) -> None:
        self.results: list[tuple[str, str, int]] = []

    def run(self, group: str, name: str, argv: list[str], cwd: Path | None = None) -> bool:
        printable = " ".join(argv)
        print(f"\n=== [{group}] {name}\n$ {printable}", flush=True)
        completed = subprocess.run(argv, cwd=cwd or ROOT, check=False)
        self.results.append((group, name, completed.returncode))
        if completed.returncode != 0:
            print(f"FAILED: {name} (exit {completed.returncode})", file=sys.stderr, flush=True)
        return completed.returncode == 0

    def inline(self, group: str, name: str, examined: str, evaluate) -> bool:
        """A step whose check lives in this script instead of a subprocess.

        The printed line still says what was examined, so a failure can be reproduced by hand exactly
        the way a command failure can.
        """
        print(f"\n=== [{group}] {name}\n$ {examined}", flush=True)
        problems = evaluate()
        for line in problems:
            print(f"  {line}", file=sys.stderr, flush=True)
        self.results.append((group, name, 1 if problems else 0))
        if problems:
            print(f"FAILED: {name} ({len(problems)} problem(s))", file=sys.stderr, flush=True)
            return False
        return True


def ensure_frontend_assets(gate: Gate) -> bool:
    """The clippy step passes `--all-features`, which enables the desktop crate's
    `custom-protocol` feature; that is the configuration that ships, and its codegen embeds
    `apps/desktop/ui/dist/`. With the directory absent the build fails inside the macro:
    `The `frontendDist` configuration is set to "../ui/dist" but this path doesn't exist`. `dist/`
    is gitignored, so a fresh clone needs the UI built before that lint can run. Without
    `custom-protocol` the same commands pass with no `dist/` at all, because tauri's dev-mode
    codegen embeds nothing.
    """
    if UI_MARKER.exists():
        return True
    print(
        "\napps/desktop/ui/dist/index.html is missing, so `clippy --all-features` cannot compile the\n"
        "desktop crate in its shipping configuration. Building the UI first.",
        flush=True,
    )
    pnpm = pnpm_command()
    if not gate.run("rust", "frontend assets (install)", pnpm + ["install", "--frozen-lockfile"], cwd=UI):
        return False
    if not gate.run("rust", "frontend assets (build)", pnpm + ["build"], cwd=UI):
        return False
    return UI_MARKER.exists()


def rust_steps(gate: Gate) -> None:
    if not ensure_frontend_assets(gate):
        return
    exe = cargo()
    steps = (
        ("fmt", [exe, "fmt", "--all", "--", "--check"]),
        ("clippy", [exe, "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"]),
        ("test", [exe, "test", "--workspace"]),
    )
    for name, argv in steps:
        if not gate.run("rust", name, argv):
            return


def core_smoke_steps(gate: Gate) -> None:
    """The headless product on its own: fixture parsing, memory accounting, storage, report and CLI
    contracts, with no Tauri shell in the selection.

    `firmwaresight-desktop` is excluded by package name rather than by a feature flag, so this
    group needs no WebView system libraries, no built frontend and no `dist/` marker. That is what
    lets a third platform carry a required smoke without the shell's compile coverage moving with
    it - the shell stays covered by the `rust` group on the two platforms that build it.
    """
    exe = cargo()
    select: list[str] = []
    for package in CORE_PACKAGES:
        select += ["-p", package]

    steps = (
        ("fmt", [exe, "fmt", "--all", "--", "--check"]),
        ("clippy", [exe, "clippy", *select, "--all-targets", "--", "-D", "warnings"]),
        ("test", [exe, "test", *select]),
    )
    for name, argv in steps:
        if not gate.run("core-smoke", name, argv):
            return


def untracked_fixture_paths(root: Path) -> list[str]:
    """Manifest paths that git does not track.

    `p0_acceptance.rs` reads `fixtures/manifest.json` and hashes whatever it finds in the working
    tree, so a half-committed pair passes on the machine that generated it and fails on a clean
    checkout. That is how `**/target/` in `.gitignore` ate `fixtures/elf/p2-diff/target/`: every local
    run was green and CI panicked on the missing half. The manifest is the statement that a fixture
    set is complete, so the check is that statement against the index.
    """
    manifest = json.loads((root / "fixtures" / "manifest.json").read_text(encoding="utf-8"))
    listed = subprocess.run(
        [resolve("git"), "ls-files", "-z"], cwd=root, capture_output=True, check=False
    )
    if listed.returncode != 0:
        return [f"`git ls-files` exited {listed.returncode}; the index cannot be read"]
    tracked = set(listed.stdout.decode("utf-8", "replace").split("\0")) - {""}
    return [
        f"{entry['path']} is recorded in fixtures/manifest.json but is not tracked by git"
        for entry in manifest["files"]
        if entry["path"] not in tracked
    ]


def version_identity_reasons(root: Path) -> list[str]:
    """Prompt section 67: one version identity across every declared surface, checked not asserted.

    Until P5 the repository carried two answers to "which version is this": `BASELINE.yaml` said 0.6.0
    while `Cargo.toml`, `tauri.conf.json` and the UI package all said 0.1.0, and no command could see
    the split. Section 67 makes a split a closure blocker, so it needs a check that runs on every
    commit rather than a paragraph that gets re-read at closure.

    Two surfaces are deliberately not here. The package filename and the installer's own metadata
    only exist after a build, so `scripts/verify_package_artifacts.py` compares them against
    `[workspace.package] version` inside each CI package job; the Desktop About panel and
    Diagnostics do not exist yet and will be added to this list by the commits that create them.
    """
    import tomllib

    def read(path: Path) -> str:
        return path.read_text(encoding="utf-8")

    workspace = tomllib.loads(read(root / "Cargo.toml"))["workspace"]["package"]["version"]
    # The document baseline is the surface that used to be the only place 0.6.0 appeared, which is
    # precisely the split section 67 forbids, so it joins the equality rather than commenting on it.
    # A missing key reads as `MISSING`, which fails the equality instead of raising a traceback.
    baseline = re.search(r"^  baseline_version:\s*(\S+)", read(root / "BASELINE.yaml"), re.MULTILINE)
    surfaces = {
        "Cargo.toml [workspace.package] version": workspace,
        "tauri.conf.json version": json.loads(
            read(root / "apps" / "desktop" / "src-tauri" / "tauri.conf.json")
        )["version"],
        "apps/desktop/ui/package.json version": json.loads(
            read(root / "apps" / "desktop" / "ui" / "package.json")
        )["version"],
        "BASELINE.yaml product.baseline_version": baseline.group(1) if baseline else "MISSING",
    }

    problems = [
        f"{name} is {value!r} but [workspace.package] version is {workspace!r}"
        for name, value in surfaces.items()
        if value != workspace
    ]

    # Every crate must inherit rather than restate. A crate with its own literal version string is a
    # surface this check cannot see, because it is not in any of the four places above.
    for manifest in sorted((root / "crates").glob("*/Cargo.toml")) + [
        root / "apps" / "cli" / "Cargo.toml",
        root / "apps" / "desktop" / "src-tauri" / "Cargo.toml",
    ]:
        body = read(manifest)
        if not re.search(r"^version\.workspace\s*=\s*true\s*$", body, re.MULTILINE):
            problems.append(
                f"{manifest.relative_to(root)} does not set `version.workspace = true`, so it can "
                "carry a version this check never reads"
            )
    return problems


def drift_steps(gate: Gate) -> None:
    py = sys.executable
    checks = (
        ("design tokens", [py, "scripts/generate_design_tokens.py", "--check"]),
        ("desktop icons", [py, "scripts/gen_desktop_icons.py", "--check"]),
    )
    for name, argv in checks:
        if not gate.run("drift", name, argv):
            return

    # ts-rs writes the bindings during `cargo test`, so regeneration is the check: any diff after
    # running it means the committed `.ts` files no longer match the Rust DTOs.
    if not gate.run("drift", "ipc bindings", [cargo(), "test", "-p", "firmwaresight-desktop"]):
        return
    if not gate.run(
        "drift",
        "ipc bindings unchanged",
        [resolve("git"), "diff", "--exit-code", "--", "apps/desktop/ui/src/ipc/generated"],
    ):
        return
    # A fixture the repository does not carry is not a fixture: it reproduces nothing and its hash
    # proves nothing. This is the only step that looks at the index rather than the working tree.
    if not gate.inline(
        "drift",
        "fixtures tracked",
        "every path recorded in fixtures/manifest.json appears in `git ls-files`",
        lambda: untracked_fixture_paths(ROOT),
    ):
        return
    # One version across four declared surfaces, plus the rule that no crate restates it.
    if not gate.inline(
        "drift",
        "version identity",
        "Cargo.toml, tauri.conf.json, ui/package.json and BASELINE.yaml agree, and every crate "
        "inherits `version.workspace = true`",
        lambda: version_identity_reasons(ROOT),
    ):
        return
    gate.run("drift", "goldens unchanged", [resolve("git"), "diff", "--exit-code", "--", "golden"])


def frontend_steps(gate: Gate) -> None:
    pnpm = pnpm_command()
    for name in ("install", "typecheck", "lint", "test", "build"):
        argv = pnpm + (["install", "--frozen-lockfile"] if name == "install" else [name])
        if not gate.run("frontend", name, argv, cwd=UI):
            return


def package_steps(gate: Gate) -> None:
    """Build what a stranger would install, then check what the build actually produced.

    This is the group the three CI package jobs run. It is separate from `rust` and `frontend`
    because a package can only be built for the platform standing in front of it, and because an
    installer is a minutes-long artifact rather than a lint. It is still the same command a
    developer runs, which is the promise the header comment of
    `.github/workflows/p0-check.yml` makes.
    """
    exe = cargo()
    # `tauri build` runs tauri.conf.json's beforeBuildCommand (`pnpm build`), and that needs the
    # dependencies the lockfile names. `--frozen-lockfile` is the exact-lockfile rule prompt
    # section 41 asks of a package job, applied where the package actually builds.
    if not gate.run(
        "package", "frontend deps", pnpm_command() + ["install", "--frozen-lockfile"], cwd=UI
    ):
        return
    if not gate.run("package", "cli companion", [exe, "build", "--release", "--locked", "-p", "fwsight"]):
        return

    tauri = resolve("tauri")
    try:
        probe = subprocess.run([tauri, "--version"], cwd=TAURI, capture_output=True, check=False)
    except OSError:
        probe = subprocess.CompletedProcess([tauri], returncode=1)
    if probe.returncode != 0:
        print(
            "\n=== [package] desktop package\n"
            "SKIPPED: the Tauri CLI is not installed on this machine, so no package was built and\n"
            "         nothing here verifies one. CI installs the pinned version, so the package,\n"
            "         its checksums and its verification come from the CI run rather than from\n"
            "         here. Locally:\n"
            "             cargo install tauri-cli@2.12.1 --locked",
            flush=True,
        )
        gate.results.append(("package", "desktop package (skipped)", 0))
        gate.results.append(("package", "artifacts verified (skipped)", 0))
        return

    # No `--bundles`: tauri.conf.json names the canonical targets and tauri-bundler intersects that
    # list with what the host can build, so this one command is the whole documented packaging path.
    # That is what prompt section 8 means by a user not having to remember `--features
    # custom-protocol` - the CLI adds the feature itself (tauri-cli's `build_options`), and the
    # verify step below is what proves the effect rather than trusting the intent.
    if not gate.run("package", "desktop package", [tauri, "build", "--", "--locked"], cwd=TAURI):
        return
    gate.run("package", "artifacts verified", [sys.executable, "scripts/verify_package_artifacts.py"])


def deny_steps(gate: Gate) -> None:
    # cargo-deny is an optional local tool. A machine without it must not report a license
    # failure it never checked, so the step is recorded as SKIPPED rather than passed.
    probe = subprocess.run([cargo(), "deny", "--version"], cwd=ROOT, capture_output=True, check=False)
    if probe.returncode != 0:
        print(
            "\n=== [deny] cargo-deny\n"
            "SKIPPED: `cargo deny` is not installed on this machine. CI installs it, so the\n"
            "         license and ban result comes from the CI run rather than from here.",
            flush=True,
        )
        gate.results.append(("deny", "cargo-deny (skipped)", 0))
        return
    gate.run("deny", "cargo-deny", [cargo(), "deny", "check", "licenses", "bans", "sources", "advisories"])


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--only",
        choices=("rust", "frontend", "drift", "deny", "core-smoke", "package"),
        help="run a single group instead of the whole gate",
    )
    parser.add_argument("--list", action="store_true", help="print the groups and exit")
    args = parser.parse_args(argv[1:])

    if args.list:
        print("rust\nfrontend\ndrift\ndeny\ncore-smoke\npackage")
        return 0

    gate = Gate()
    # `core-smoke` and `package` are deliberately not in the default set: every package `core-smoke`
    # selects is already inside `cargo test --workspace`, and `package` builds a release installer
    # for whichever host runs it, so neither belongs in the per-commit cycle on every machine.
    groups = [args.only] if args.only else ["rust", "frontend", "drift", "deny"]
    dispatch = {
        "rust": rust_steps,
        "frontend": frontend_steps,
        "drift": drift_steps,
        "deny": deny_steps,
        "core-smoke": core_smoke_steps,
        "package": package_steps,
    }
    for group in groups:
        dispatch[group](gate)

    print("\n=== summary ===")
    for group, name, code in gate.results:
        mark = "PASS" if code == 0 else f"FAIL({code})"
        print(f"{mark:<10} {group}/{name}")
    failed = [r for r in gate.results if r[2] != 0]
    print(f"\n{len(gate.results) - len(failed)}/{len(gate.results)} steps passed")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
