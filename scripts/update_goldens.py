#!/usr/bin/env python3
"""Regenerate the committed goldens from the real `fwsight` binary - only with --confirm.

Goldens exist to catch an unintended change, so this script is deliberately awkward to use:
without `--confirm` it prints what *would* change and writes nothing. A failing test never
rewrites its own expectation (`05_ENGINEERING/02_TEST_STRATEGY.md`), and a one-command
"make it green" path would defeat the point of the golden.

The numbers are produced by the binary, not by this script: the CLI is run with `--json` and its
stdout is reformatted with the key order it already arrives in, and `--html` is read back from the
file the CLI wrote. Nothing here recomputes a fact, because a second implementation of the
projection is a second thing that can be wrong.

Usage:
    python scripts/update_goldens.py             # show the semantic diff, write nothing
    python scripts/update_goldens.py --confirm   # write the new goldens
"""

from __future__ import annotations

import argparse
import difflib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = (
    # (golden name, elf path, map path or None)
    ("p0-basic", "fixtures/elf/p0-basic/firmware.elf", None),
    (
        "p0-dual-region",
        "fixtures/elf/p0-dual-region/firmware.elf",
        "fixtures/elf/p0-dual-region/firmware.map",
    ),
)

# The P2 pair: two builds of one source tree, deliberately different in exactly the ways US-002
# needs to be demonstrated (a grown .text, a moved .rodata, one added and one removed section,
# unchanged .settings and .data, plus the two budgets staying exact on a MAP-backed layout).
DIFF_PAIR = {
    "old": "fixtures/elf/p2-diff/base/firmware.elf",
    "old_map": "fixtures/elf/p2-diff/base/firmware.map",
    "new": "fixtures/elf/p2-diff/target/firmware.elf",
    "new_map": "fixtures/elf/p2-diff/target/firmware.map",
}

# The P4 golden bundle (prompt section 46): one release subject built from the committed project fixture
# and the P2 pair, published by the real `release prepare`. Every document the bundle holds is recorded
# except the two artifact copies, whose bytes the fixture manifest already pins.
P4_PROJECT_FIXTURE = "fixtures/project/p4-release"
P4_BUNDLE_DOCS = (
    "SHA256SUMS",
    "accepted-reviews.json",
    "analysis.json",
    "diff.json",
    "gate-results.json",
    "release-manifest.json",
    "release-notes.md",
    "release-report.html",
)
# The commit is part of the release's identity, so it must be the same commit everywhere. Left to the
# machine's clock and identity, the manifest's `git_commit` would move on every run and take the release
# id, the digests and the golden with it.
P4_COMMIT = {
    "name": "Release Owner",
    "email": "release@example.invalid",
    "date": "2026-09-30T07:00:00+00:00",
    "message": "release candidate",
    "tag": "v1.2.3",
}


def cargo() -> str:
    for candidate in ("cargo", "cargo.exe", "cargo.cmd"):
        found = shutil.which(candidate)
        if found is not None:
            return found
    return "cargo"


def analyze(elf: str, map_path: str | None) -> dict[str, Any]:
    argv = [cargo(), "run", "-q", "-p", "fwsight", "--", "analyze", elf, "--json"]
    if map_path is not None:
        argv += ["--map", map_path]
    completed = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, encoding="utf-8", check=False)
    if completed.returncode != 0:
        raise SystemExit(
            f"analyze failed for {elf} (exit {completed.returncode}):\n{completed.stderr}"
        )
    return json.loads(completed.stdout)


def diff_document() -> dict[str, Any]:
    """Run `fwsight diff --json` over the P2 pair and return the emitted document."""
    argv = [
        cargo(),
        "run",
        "-q",
        "-p",
        "fwsight",
        "--",
        "diff",
        DIFF_PAIR["old"],
        DIFF_PAIR["new"],
        "--old-map",
        DIFF_PAIR["old_map"],
        "--new-map",
        DIFF_PAIR["new_map"],
        "--json",
    ]
    completed = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, encoding="utf-8", check=False)
    if completed.returncode != 0:
        raise SystemExit(
            f"diff failed (exit {completed.returncode}):\n{completed.stderr}"
        )
    return json.loads(completed.stdout)


def diff_html() -> str:
    """Run `fwsight diff --html FILE` and read back exactly what the binary wrote.

    The scratch file lives under target/, so the script never writes into the tracked tree and the
    golden is a copy of the shipped bytes rather than a re-encoding of them.
    """
    scratch = ROOT / "target" / "update_goldens-p2.html"
    scratch.parent.mkdir(parents=True, exist_ok=True)
    if scratch.exists():
        scratch.unlink()
    argv = [
        cargo(),
        "run",
        "-q",
        "-p",
        "fwsight",
        "--",
        "diff",
        DIFF_PAIR["old"],
        DIFF_PAIR["new"],
        "--old-map",
        DIFF_PAIR["old_map"],
        "--new-map",
        DIFF_PAIR["new_map"],
        "--html",
        str(scratch),
    ]
    completed = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, encoding="utf-8", check=False)
    if completed.returncode != 0:
        raise SystemExit(
            f"diff --html failed (exit {completed.returncode}):\n{completed.stderr}"
        )
    if not scratch.exists():
        raise SystemExit("diff --html exited 0 without writing the requested file")
    return scratch.read_text(encoding="utf-8")


def release_subject() -> Path:
    """A throwaway release subject: the P4 project fixture, committed and tagged with pinned facts.

    FirmwareSight's own repository is never the subject (prompt section 47); it is dirty or clean for
    reasons that have nothing to do with a release. The two artifact sides are copied in as well, so the
    subject's source paths point at this scratch folder and a portability check can remove them.
    """
    if shutil.which("git") is None:
        raise SystemExit("git is not on PATH; the P4 bundle golden needs a real workspace to cut from")
    scratch = ROOT / "target" / "update_goldens-p4"
    if scratch.exists():
        shutil.rmtree(scratch)
    project = scratch / "project"
    (project / "docs").mkdir(parents=True)
    fixture = ROOT / P4_PROJECT_FIXTURE
    for relative in ("firmwaresight.toml", "docs/RELEASE_NOTES.md"):
        source = fixture / relative
        # The fixture's bytes are its identity: the notes' digest reaches the manifest, so a checkout
        # that turned LF into CRLF would silently change the release.
        if b"\r" in source.read_bytes():
            raise SystemExit(f"{relative} carries a CR byte, so its hash is not reproducible")
        shutil.copyfile(source, project / relative)

    environment = dict(
        os.environ,
        GIT_AUTHOR_DATE=P4_COMMIT["date"],
        GIT_COMMITTER_DATE=P4_COMMIT["date"],
    )
    for arguments in (
        ["init", "-q"],
        ["add", "-A"],
        ["-c", f"user.name={P4_COMMIT['name']}", "-c", f"user.email={P4_COMMIT['email']}",
         "commit", "-q", "-m", P4_COMMIT["message"]],
        ["tag", P4_COMMIT["tag"]],
    ):
        completed = subprocess.run(
            ["git", *arguments], cwd=project, env=environment, capture_output=True, text=True, check=False
        )
        if completed.returncode != 0:
            raise SystemExit(f"`git {' '.join(arguments)}` failed: {completed.stderr}")
    return project


def p4_bundle_documents() -> dict[str, str]:
    """Every document the shipped `release prepare` writes, read back as the bytes it wrote."""
    project = release_subject()
    scratch = project.parent
    # The leaf name is what the bundle ships the artifact under (§5), so the two sides keep their own
    # directory rather than being flattened into one folder with a renamed file.
    staged: dict[str, str] = {}
    for role, relative in (
        ("artifact", DIFF_PAIR["new"]),
        ("artifact_map", DIFF_PAIR["new_map"]),
        ("baseline", DIFF_PAIR["old"]),
        ("baseline_map", DIFF_PAIR["old_map"]),
    ):
        source = ROOT / relative
        folder = scratch / "source" / ("target" if role.startswith("artifact") else "base")
        folder.mkdir(parents=True, exist_ok=True)
        copy = folder / source.name
        shutil.copyfile(source, copy)
        staged[role] = str(copy)
    out = scratch / "out"
    argv = [
        cargo(), "run", "-q", "-p", "fwsight", "--", "release", "prepare",
        "--project", str(project),
        "--artifact", staged["artifact"],
        "--map", staged["artifact_map"],
        "--baseline", staged["baseline"],
        "--baseline-map", staged["baseline_map"],
        "--out", str(out),
    ]
    completed = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, check=False)
    if completed.returncode != 0:
        raise SystemExit(
            f"release prepare failed (exit {completed.returncode}):\n{completed.stdout}\n{completed.stderr}"
        )
    bundles = [entry for entry in out.iterdir() if entry.is_dir()]
    if len(bundles) != 1:
        raise SystemExit(f"one release writes one bundle directory, found {len(bundles)}")
    bundle = bundles[0]
    return {name: (bundle / name).read_text(encoding="utf-8") for name in P4_BUNDLE_DOCS}


def pretty(document: dict[str, Any]) -> str:
    # Keys are written in sorted order, which is what the two committed P0 CLI goldens already use
    # and what `render::semantically_equal` canonicalizes to. Field declaration order is still what
    # serde emits, and the report crate's tests assert that; a golden records content, not the
    # incidental ordering of one serializer.
    return json.dumps(document, indent=2, ensure_ascii=False, sort_keys=True) + "\n"


def flatten(node: Any, prefix: str = "") -> dict[str, Any]:
    """A dotted view of a document, so a diff can name the leaf that moved."""
    if isinstance(node, dict):
        out: dict[str, Any] = {}
        for key, value in node.items():
            out.update(flatten(value, f"{prefix}.{key}" if prefix else key))
        return out
    if isinstance(node, list):
        return {prefix: json.dumps(node, sort_keys=True)}
    return {prefix: node}


def describe(old: dict[str, Any], new: dict[str, Any]) -> list[str]:
    before, after = flatten(old), flatten(new)
    lines: list[str] = []
    for key in sorted(set(before) - set(after)):
        lines.append(f"- {key} (removed)")
    for key in sorted(set(after) - set(before)):
        lines.append(f"+ {key} = {after[key]}")
    for key in sorted(set(before) & set(after)):
        if before[key] != after[key]:
            lines.append(f"~ {key}: {before[key]} -> {after[key]}")
    return lines


def describe_text(old: str, new: str) -> list[str]:
    """A line view of a non-JSON golden, so an HTML change is reviewable as a diff."""
    return [
        line
        for line in difflib.unified_diff(
            old.splitlines(), new.splitlines(), lineterm="", n=0
        )
        if line.startswith(("+", "-")) and not line.startswith(("+++", "---"))
    ]


def matches(committed: str, regenerated: str, suffix: str) -> bool:
    """Would this golden actually change?

    A JSON golden is compared by content, not by bytes. The committed P0 goldens are stored in sorted
    key order while serde emits declaration order, and rewriting them for ordering alone would be the
    blind bulk regeneration this script exists to prevent. A non-JSON golden is compared exactly,
    because byte stability is the property under test.
    """
    if suffix == ".json":
        try:
            return json.loads(committed) == json.loads(regenerated)
        except json.JSONDecodeError:
            return False
    return committed == regenerated


def previous_json(path: Path) -> dict[str, Any]:
    """The committed document, or an empty one when the golden does not exist yet."""
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Regenerate the committed goldens.")
    parser.add_argument(
        "--confirm",
        action="store_true",
        help="actually write the files; without it nothing changes",
    )
    args = parser.parse_args(argv[1:])

    targets: list[tuple[Path, str, list[str]]] = []
    for name, elf, map_path in FIXTURES:
        document = analyze(elf, map_path)
        for path, payload in (
            (ROOT / "golden" / "cli" / f"{name}-analyze.json", document),
            (ROOT / "golden" / "core" / f"{name}-memory.json", document["memory"]),
        ):
            targets.append((path, pretty(payload), describe(previous_json(path), payload)))

    # The P2 pair: the whole portable diff document and the whole HTML it renders into. Both are
    # full outputs rather than excerpts, so a change anywhere in either one has to be reviewed.
    diff_json_path = ROOT / "golden" / "core" / "p2-diff.json"
    document = diff_document()
    targets.append(
        (diff_json_path, pretty(document), describe(previous_json(diff_json_path), document))
    )
    html_path = ROOT / "golden" / "reports" / "p2-diff.html"
    html = diff_html()
    targets.append(
        (
            html_path,
            html,
            describe_text(html_path.read_text(encoding="utf-8"), html)
            if html_path.exists()
            else ["(new file)"],
        )
    )

    # The P4 bundle: each document exactly as the shipped binary wrote it, because a digest was taken
    # over those bytes. Nothing here is re-indented or re-ordered - a reformatted golden would no
    # longer be the file `SHA256SUMS` records.
    for name, text in p4_bundle_documents().items():
        path = ROOT / "golden" / "reports" / "p4-release" / name
        if not path.exists():
            targets.append((path, text, ["(new file)"]))
        elif path.suffix == ".json":
            targets.append(
                (path, text, describe(json.loads(path.read_text(encoding="utf-8")), json.loads(text)))
            )
        else:
            targets.append((path, text, describe_text(path.read_text(encoding="utf-8"), text)))

    total = 0
    for path, text, diff in targets:
        previous = path.read_text(encoding="utf-8") if path.exists() else None
        # A bundle document is compared byte for byte; the older goldens are compared by content,
        # because their sorted-key layout is a convenience and not the property under test.
        exact = path.is_relative_to(ROOT / "golden" / "reports" / "p4-release")
        if previous is not None and (previous == text if exact else matches(previous, text, path.suffix)):
            print(f"unchanged  {path.relative_to(ROOT)}")
            continue
        total += 1
        print(f"changed    {path.relative_to(ROOT)}")
        for line in diff[:20]:
            print(f"           {line}")
        if len(diff) > 20:
            print(f"           ... {len(diff) - 20} more")
        if args.confirm:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8", newline="\n")

    if not args.confirm:
        if total:
            print(
                f"\n{total} golden(s) would change. Review the diff above, then re-run with "
                "--confirm and record the reason in the commit message.",
                file=sys.stderr,
            )
            return 1
        print("\nall goldens match current output.")
        return 0

    print(f"\nwrote {total} changed golden(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
