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
3. the JSON documents this release declares parse, declare the contract their urn names, and satisfy it.
   `diff.json` is declared only by a release that names a baseline: a single-build release ships no
   comparison, and its absence is legal, so it is reported as skipped rather than as a pass or a failure.
   A bundle that declares a baseline and does not carry the document fails;
4. the report is one self-contained HTML file with nothing to fetch;
5. no path on this machine and no wall-clock value appears in anything FirmwareSight wrote;
6. each of the nine questions section 61 lists is answered out of the bundle alone, including the two
   whose document may be legally absent — those are answered from what the manifest declares.

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
from typing import Any, Callable

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

# One of section 61's questions the bundle cannot answer, because a document it promised is not readable.
# Stated as a failure rather than left blank, so the tally and the exit code still describe the bundle.
UNANSWERABLE = "unanswerable: "


class Report:
    """A running record of what was checked, so the exit code and the printed table agree."""

    def __init__(self) -> None:
        self.lines: list[tuple[bool, str]] = []
        self.skips: list[str] = []

    def check(self, passed: bool, text: str) -> bool:
        self.lines.append((passed, text))
        print(f"{'ok  ' if passed else 'FAIL'}  {text}")
        return passed

    def skip(self, text: str) -> None:
        """A check this bundle does not ask for. Not a pass, not a failure, and not silent."""
        self.skips.append(text)
        print(f"SKIP  {text}")

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


def declared_documents(manifest: dict[str, Any]) -> tuple[set[str], set[str]]:
    """Which contract documents this release must carry, and which it may lawfully omit.

    The bundle decides, not the reader. `extensions.baseline_snapshot_id` and
    `extensions.diff_schema_version` are set together when a baseline was named and stay null when it
    was not, so a single-build release is not asked for a comparison it never promised — and a release
    that did name one is. `release.notes` is null for a policy that does not require notes.
    """
    extensions = manifest.get("extensions", {})
    baseline_named = (
        extensions.get("baseline_snapshot_id") is not None
        or extensions.get("diff_schema_version") is not None
    )
    required = set(CONTRACTS)
    optional: set[str] = set()
    if not baseline_named:
        required.discard("diff.json")
        optional.add("diff.json")
    if manifest.get("release", {}).get("notes") is None:
        optional.add(NOTES)
    return required, optional


def verify_contracts(
    bundle: Path, schemas: Path | None, report: Report, manifest: dict[str, Any]
) -> dict[str, Any]:
    """Read the contract documents this release declares, and hand back the ones that parsed.

    The documents returned are the same objects the nine answers are drawn from, so a bundle that a
    stranger cannot read fails once here and is then answered from what is actually in the folder,
    rather than crashing two functions further down.
    """
    required, optional = declared_documents(manifest)
    documents: dict[str, Any] = {}
    for name in CONTRACTS:
        if not (bundle / name).is_file():
            if name in required:
                report.check(
                    False,
                    f"{name} is there: this release declares a baseline, so it promised a comparison",
                )
            else:
                report.skip(
                    f"{name} is absent, and that is legal here: this release names no baseline, "
                    "so it promises no comparison"
                )
            continue
        if name in optional:
            report.check(
                False,
                f"{name} shipped although the manifest declares no baseline: the document and its own "
                "index disagree about what this release compared",
            )
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
        return documents
    try:
        from jsonschema import Draft202012Validator  # noqa: PLC0415 - optional on purpose
    except ImportError:
        report.check(False, "jsonschema is not installed, so the contracts could not be checked")
        return documents
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
    return documents


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
        if not (bundle / name).is_file():
            report.skip(f"{name} not scanned for host paths: the file is not in this bundle")
            continue
        text = (bundle / name).read_text(encoding="utf-8")
        report.check(not HOST_PATH.search(text), f"{name} names no path on this machine")

    # The one clock a bundle may carry is the moment a named person accepted a review, which is a recorded
    # human act rather than a generation time (`accepted_at`, written by SQLite when the decision landed).
    # Everything else must be reproducible from the same inputs, so every other date-shaped value is a
    # defect - and the report is allowed to print exactly the dates the review record holds, no more.
    review_dates = _acceptance_dates(bundle)
    for name in ("analysis.json", "diff.json", "gate-results.json", "release-manifest.json", SUMS):
        if not (bundle / name).is_file():
            report.skip(f"{name} not scanned for wall-clock values: the file is not in this bundle")
            continue
        text = (bundle / name).read_text(encoding="utf-8")
        report.check(not DATE_SHAPED.search(text), f"{name} carries no wall-clock value")
    accepted = (bundle / "accepted-reviews.json").read_text(encoding="utf-8")
    report.check(
        set(DATE_SHAPED.findall(accepted)) == review_dates,
        "accepted-reviews.json's only dates are the moments its reviews were accepted",
    )
    printed = set(DATE_SHAPED.findall((bundle / REPORT).read_text(encoding="utf-8")))
    report.check(
        printed <= review_dates,
        f"{REPORT} prints no date beyond the acceptance record ({sorted(printed) or 'none'})",
    )


def _acceptance_dates(bundle: Path) -> set[str]:
    document = json.loads((bundle / "accepted-reviews.json").read_text(encoding="utf-8"))
    return {
        match
        for entry in document.get("acceptances", [])
        for match in DATE_SHAPED.findall(str(entry.get("accepted_at", "")))
    }


def answer_the_nine_questions(
    bundle: Path, manifest: dict[str, Any], documents: dict[str, Any], report: Report
) -> None:
    required, _optional = declared_documents(manifest)

    def read(name: str, ask: Callable[[dict[str, Any]], str]) -> str:
        """One question's answer out of a document that may not be there to read."""
        document = documents.get(name)
        if document is None:
            return f"{UNANSWERABLE}{name} is not readable in this bundle"
        return ask(document)

    # Two of these answers may come from the manifest rather than from a document, and that is not a
    # weaker answer: "this release named no baseline" and "this release shipped no notes" are facts the
    # bundle states about itself, which is what a stranger holding only the folder needs to be told.
    if "diff.json" in required:
        changed = read(
            "diff.json",
            lambda d: (
                f"{d['counts']['sectionsAdded']} added, {d['counts']['sectionsRemoved']} removed, "
                f"{d['counts']['sectionsChanged']} changed"
            ),
        )
    else:
        changed = (
            "nothing: this release names no baseline, so it ships no comparison "
            "(extensions.baseline_snapshot_id is null)"
        )
    notes = manifest.get("release", {}).get("notes")
    if notes is None:
        notes_answer = (
            "none: release.notes is null, so this release shipped no notes file "
            "(its policy does not require one)"
        )
    else:
        notes_answer = f"{(bundle / NOTES).stat().st_size} bytes, shipped verbatim"
    answers = {
        "what release is this": f"{manifest['release']['id']} version {manifest['release']['version']}",
        "what firmware artifact ships": ", ".join(
            entry["path"] for entry in manifest["files"] if entry["path"].startswith("artifacts/")
        ),
        "what are its hashes": f"{len(manifest['files'])} digests in the manifest, {len(read_sums(bundle))} in {SUMS}",
        "what did analysis say": read(
            "analysis.json",
            lambda d: (
                f"{len(d['sections'])} sections, nonvolatile "
                f"{d['memory']['nonvolatileImageFootprint']['bytes']} B "
                f"({d['memory']['nonvolatileImageFootprint']['state']})"
            ),
        ),
        "what changed from baseline": changed,
        "what did Gate say": read(
            "gate-results.json",
            lambda d: f"{len(d['findings'])} rules, overall {d['extensions']['overall_effective_severity']}",
        ),
        "which Reviews were accepted": read(
            "accepted-reviews.json", lambda d: f"{len(d['acceptances'])} recorded"
        ),
        "what were Release Notes": notes_answer,
        "how do I verify files": manifest["extensions"]["integrity_model"]["rule"],
    }
    print("----  what a reader can answer from the bundle alone")
    for question, answer in answers.items():
        report.check(bool(answer) and not answer.startswith(UNANSWERABLE), f"{question}: {answer}")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("bundle", type=Path, help="a release bundle directory")
    parser.add_argument(
        "--schemas",
        type=Path,
        default=None,
        help="validate the declared JSON documents against this repository's contracts",
    )
    args = parser.parse_args(argv[1:])

    bundle: Path = args.bundle
    if not bundle.is_dir():
        print(f"{bundle} is not a directory", file=sys.stderr)
        return 2
    report = Report()
    sums = verify_sums(bundle, report)
    manifest = verify_manifest(bundle, sums, report)
    documents = verify_contracts(bundle, args.schemas, report, manifest)
    verify_report_is_self_contained(bundle, report)
    verify_no_host_leaks(bundle, report)
    answer_the_nine_questions(bundle, manifest, documents, report)

    skipped = f", {len(report.skips)} skipped as not applicable" if report.skips else ""
    print(
        f"\n{len(report.lines) - report.failed} of {len(report.lines)} checks passed"
        + ("" if report.failed == 0 else f", {report.failed} FAILED")
        + skipped
    )
    return 1 if report.failed else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
