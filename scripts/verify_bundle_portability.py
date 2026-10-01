#!/usr/bin/env python3
"""Read a FirmwareSight release bundle the way a stranger would: with no FirmwareSight involved.

prompt section 45 and section 61 ask whether another person, on another machine, with the project
deleted and the app closed, can still review a release. The Rust tests answer that with
`firmwaresight-project`'s own verifier, which is still our code reading our bytes. This script answers
it without any of that: only Python's standard library plus `jsonschema`, which is a third party's
implementation of the same contracts and would agree or disagree on its own.

What it proves about one bundle directory:

1. every file `SHA256SUMS` names is there and hashes to what it says, and the index covers exactly the
   set it claims to cover (prompt section 20);
2. every file `release-manifest.json` names does the same, including `SHA256SUMS` itself;
3. the five JSON documents parse, declare the contract their urn names, and satisfy it;
4. the report is one self-contained HTML file with nothing to fetch;
5. no path on this machine and no wall-clock value appears in anything FirmwareSight wrote;
6. each of the nine questions section 61 lists is answered out of the bundle alone.

Usage:
    python scripts/verify_bundle_portability.py <bundle-directory> [--schemas <dir>]

The schema check is the one place this script can cheat: `schemas/` is the repository's authored copy,
which a person holding only the bundle would not have. Pass --schemas only when you mean to consult it;
without it the documents' own `schema` urns are still cross-checked against each other and against the
manifest, and the report says the contract validation was skipped rather than pretending it ran.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

CONTRACTS = {
    "analysis.json": "urn:firmwaresight:schema:analysis:1",
    "diff.json": "urn:firmwaresight:schema:diff:1",
    "gate-results.json": "urn:firmwaresight:schema:gate-results:1",
    "accepted-reviews.json": "urn:firmwaresight:schema:accepted-reviews:1",
    "release-manifest.json": "urn:firmwaresight:schema:release-manifest:1",
}
REPORT = "release-report.html"
NOTES = "release-notes.md"
SUMS = "SHA256SUMS"

# What FirmwareSight wrote. The shipped artifact and the release owner's notes are excluded: they are
# copied byte for byte, and an image carries whatever its compiler embedded in it.
COMPOSED = [*CONTRACTS, REPORT, SUMS]

HOST_PATH = re.compile(r"(?:[A-Za-z]:[\\/]|/home/|/Users/|/tmp/)")
DATE_SHAPED = re.compile(r"20\d\d-\d\d-\d\d")


class Report:
    """A running record of what was checked, so the exit code and the printed table agree."""

    def __init__(self) -> None:
        self.lines: list[tuple[bool, str]] = []

    def check(self, passed: bool, text: str) -> bool:
        self.lines.append((passed, text))
        print(f"{'ok  ' if passed else 'FAIL'}  {text}")
        return passed

    @property
    def failed(self) -> int:
        return sum(1 for passed, _ in self.lines if not passed)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(65536), b""):
            digest.update(block)
    return digest.hexdigest()


def read_sums(bundle: Path) -> dict[str, str]:
    entries: dict[str, str] = {}
    for line in (bundle / SUMS).read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        digest, _, name = line.partition("  ")
        entries[name.removeprefix("*")] = digest
    return entries


def verify_sums(bundle: Path, report: Report) -> dict[str, str]:
    entries = read_sums(bundle)
    report.check(
        SUMS not in entries and "release-manifest.json" not in entries,
        f"{SUMS} lists neither itself nor the manifest (section 20)",
    )
    for name, digest in sorted(entries.items()):
        file = bundle / name
        report.check(file.is_file() and sha256(file) == digest, f"{SUMS} describes {name}")
    return entries


def verify_manifest(bundle: Path, sums: dict[str, str], report: Report) -> dict[str, Any]:
    manifest = json.loads((bundle / "release-manifest.json").read_text(encoding="utf-8"))
    listed = {entry["path"]: entry for entry in manifest["files"]}
    report.check("release-manifest.json" not in listed, "the manifest does not hash itself (section 20)")
    report.check(SUMS in listed, "the manifest covers SHA256SUMS, which the sums index cannot")
    for name, entry in sorted(listed.items()):
        file = bundle / name
        present = file.is_file()
        report.check(
            present and sha256(file) == entry["sha256"] and file.stat().st_size == entry["size"],
            f"release-manifest.json describes {name}",
        )
    report.check(
        set(sums) <= set(listed) and set(listed) - set(sums) == {SUMS},
        "the two indexes differ by the sums file alone, so neither hides a file from the other",
    )
    return manifest


def verify_contracts(bundle: Path, schemas: Path | None, report: Report) -> None:
    documents: dict[str, Any] = {}
    for name in CONTRACTS:
        try:
            documents[name] = json.loads((bundle / name).read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            report.check(False, f"{name} parses: {error}")
            continue
    for name, document in documents.items():
        declares_a_version = (
            "schema" in document or "schemaVersion" in document or "schema_version" in document
        )
        report.check(declares_a_version, f"{name} states which contract it was written under")
        if isinstance(document.get("schema"), str):
            report.check(
                document["schema"] == CONTRACTS[name],
                f"{name} declares {document['schema']}",
            )
    if schemas is None:
        print("----  contract validation skipped: no --schemas given")
        return
    try:
        from jsonschema import Draft202012Validator  # noqa: PLC0415 - optional on purpose
    except ImportError:
        report.check(False, "jsonschema is not installed, so the contracts could not be checked")
        return
    for name, document in documents.items():
        path = schemas / f"{name.replace('.json', '')}.schema.json"
        if not path.is_file():
            report.check(False, f"{path.name} is not there to validate against")
            continue
        schema = json.loads(path.read_text(encoding="utf-8"))
        failures = sorted(
            Draft202012Validator(schema).iter_errors(document), key=lambda e: list(e.absolute_path)
        )
        first = "" if not failures else f" - {failures[0].json_path}: {failures[0].message}"
        report.check(not failures, f"{name} satisfies {path.name}{first}")


def verify_report_is_self_contained(bundle: Path, report: Report) -> str:
    html = (bundle / REPORT).read_text(encoding="utf-8")
    lowered = html.lower()
    report.check(html.startswith("<!doctype html>"), f"{REPORT} is a complete HTML document")
    report.check("<style>" in lowered, f"{REPORT} embeds its CSS")
    for external in ("<script", "http://", "https://", "<link", "@import", "cdn."):
        report.check(external not in lowered, f"{REPORT} reaches for nothing ({external} absent)")
    return html


def verify_no_host_leaks(bundle: Path, report: Report) -> None:
    for name in COMPOSED:
        text = (bundle / name).read_text(encoding="utf-8")
        report.check(not HOST_PATH.search(text), f"{name} names no path on this machine")
        report.check(not DATE_SHAPED.search(text), f"{name} carries no wall-clock value")


def answer_the_nine_questions(bundle: Path, manifest: dict[str, Any], report: Report) -> None:
    gate = json.loads((bundle / "gate-results.json").read_text(encoding="utf-8"))
    reviews = json.loads((bundle / "accepted-reviews.json").read_text(encoding="utf-8"))
    analysis = json.loads((bundle / "analysis.json").read_text(encoding="utf-8"))
    diff = json.loads((bundle / "diff.json").read_text(encoding="utf-8"))
    answers = {
        "what release is this": f"{manifest['release']['id']} version {manifest['release']['version']}",
        "what firmware artifact ships": ", ".join(
            entry["path"] for entry in manifest["files"] if entry["path"].startswith("artifacts/")
        ),
        "what are its hashes": f"{len(manifest['files'])} digests in the manifest, {len(read_sums(bundle))} in {SUMS}",
        "what did analysis say": (
            f"{len(analysis['sections'])} sections, nonvolatile "
            f"{analysis['memory']['nonvolatileImageFootprint']['bytes']} B "
            f"({analysis['memory']['nonvolatileImageFootprint']['state']})"
        ),
        "what changed from baseline": (
            f"{diff['counts']['sectionsAdded']} added, {diff['counts']['sectionsRemoved']} removed, "
            f"{diff['counts']['sectionsChanged']} changed"
        ),
        "what did Gate say": (
            f"{len(gate['findings'])} rules, overall "
            f"{gate['extensions']['overall_effective_severity']}"
        ),
        "which Reviews were accepted": f"{len(reviews['acceptances'])} recorded",
        "what were Release Notes": f"{(bundle / NOTES).stat().st_size} bytes, shipped verbatim",
        "how do I verify files": manifest["extensions"]["integrity_model"]["rule"],
    }
    print("----  what a reader can answer from the bundle alone")
    for question, answer in answers.items():
        report.check(bool(answer), f"{question}: {answer}")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("bundle", type=Path, help="a release bundle directory")
    parser.add_argument(
        "--schemas",
        type=Path,
        default=None,
        help="validate the five documents against this repository's contracts",
    )
    args = parser.parse_args(argv[1:])

    bundle: Path = args.bundle
    if not bundle.is_dir():
        print(f"{bundle} is not a directory", file=sys.stderr)
        return 2
    report = Report()
    sums = verify_sums(bundle, report)
    manifest = verify_manifest(bundle, sums, report)
    verify_contracts(bundle, args.schemas, report)
    verify_report_is_self_contained(bundle, report)
    verify_no_host_leaks(bundle, report)
    answer_the_nine_questions(bundle, manifest, report)

    print(
        f"\n{len(report.lines) - report.failed} of {len(report.lines)} checks passed"
        + ("" if report.failed == 0 else f", {report.failed} FAILED")
    )
    return 1 if report.failed else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
