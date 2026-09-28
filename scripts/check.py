#!/usr/bin/env python3
"""Run the whole P0 verification gate, locally and in CI, from one definition.

CI and a developer machine must not disagree about what "green" means, so the workflow calls
this script instead of carrying its own copy of the command list.

Usage:
    python scripts/check.py                     # everything
    python scripts/check.py --only rust         # one group
    python scripts/check.py --list

Groups: rust, frontend, drift, deny, and core-smoke - the last being the headless subset without
the Tauri shell, which is what the macOS job in 05_ENGINEERING/06_CI_CD_BASELINE.md asks for. It is
a real local command, not a CI-only copy: run `python scripts/check.py --only core-smoke` on any
host. The default full gate does not repeat it, because the `rust` group already covers every one
of those packages plus the shell.

Exit code is non-zero on the first failing step; every step's command line is printed before it
runs so a failure can be reproduced by hand.
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UI = ROOT / "apps" / "desktop" / "ui"
# Tauri's production codegen embeds this directory at compile time.
UI_MARKER = UI / "dist" / "index.html"
# The headless product: named so a platform that ships Core but not the shell can still be gated.
CORE_PACKAGES = (
    "firmwaresight-core",
    "firmwaresight-artifact",
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
    gate.run("drift", "goldens unchanged", [resolve("git"), "diff", "--exit-code", "--", "golden"])


def frontend_steps(gate: Gate) -> None:
    pnpm = pnpm_command()
    for name in ("install", "typecheck", "lint", "test", "build"):
        argv = pnpm + (["install", "--frozen-lockfile"] if name == "install" else [name])
        if not gate.run("frontend", name, argv, cwd=UI):
            return


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
        choices=("rust", "frontend", "drift", "deny", "core-smoke"),
        help="run a single group instead of the whole gate",
    )
    parser.add_argument("--list", action="store_true", help="print the groups and exit")
    args = parser.parse_args(argv[1:])

    if args.list:
        print("rust\nfrontend\ndrift\ndeny\ncore-smoke")
        return 0

    gate = Gate()
    # `core-smoke` is deliberately not in the default set: every package it selects is already
    # inside `cargo test --workspace`, so running both would re-lint the same code twice.
    groups = [args.only] if args.only else ["rust", "frontend", "drift", "deny"]
    dispatch = {
        "rust": rust_steps,
        "frontend": frontend_steps,
        "drift": drift_steps,
        "deny": deny_steps,
        "core-smoke": core_smoke_steps,
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
