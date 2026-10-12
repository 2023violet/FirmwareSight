#!/usr/bin/env python3
"""Replayable negative tests for `scripts/verify_bundle_portability.py` (`D-N09-1`).

The reader script is the one verifier in this repository that no gate step runs: it answers prompt §45's
question by hand, on a machine that holds nothing but the folder. These tests hold it to the same rule the
Rust tests hold the engine to — a promise a bundle broke has to come out as a FAIL with a non-zero exit, not
as a Python traceback, and a document this release never promised has to come out as a real SKIP rather than
as a pass.

Run them with:

    python -m unittest discover -s scripts/tests -v

Nothing here imports FirmwareSight code: the script under test is spawned exactly the way a stranger would
spawn it, and the subject is a *copy* of the committed P4 golden bundle with the two analyzed artifacts put
back in, so `golden/` is never written to.
"""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SCRIPT = REPO / "scripts" / "verify_bundle_portability.py"
GOLDEN_BUNDLE = REPO / "golden" / "reports" / "p4-release"
TARGET_ELF = REPO / "fixtures" / "elf" / "p2-diff" / "target" / "firmware.elf"
TARGET_MAP = REPO / "fixtures" / "elf" / "p2-diff" / "target" / "firmware.map"

MANIFEST = "release-manifest.json"
NOTES = "release-notes.md"
SUMS = "SHA256SUMS"


def run_reader(bundle: Path) -> subprocess.CompletedProcess[str]:
    """Read one folder the way the script is always read: a fresh process, no FirmwareSight in it."""
    return subprocess.run(
        [sys.executable, str(SCRIPT), str(bundle)],
        capture_output=True,
        text=True,
        check=False,
    )


def legal_bundle(dest: Path) -> Path:
    """The committed golden release, complete: the golden folder plus the two artifacts it indexes.

    `golden/reports/p4-release` keeps its JSON documents and not its 17 KiB of ELF and MAP, so a copy alone
    is not a bundle a reader can pass. The pair is the same fixture the golden was built from, and the
    manifest's own digests say so: this helper's first test asserts the copy passes whole.
    """
    shutil.copytree(GOLDEN_BUNDLE, dest)
    artifacts = dest / "artifacts"
    artifacts.mkdir()
    shutil.copy(TARGET_ELF, artifacts / "firmware.elf")
    shutil.copy(TARGET_MAP, artifacts / "firmware.map")
    return dest


def release_the_bundle_from_notes(bundle: Path) -> None:
    """Rewrite one release so it no longer promises notes, and no longer carries a baseline.

    This is the shape `NEXT-08`'s F03 made legal: a single-build release whose policy asked for no notes.
    The two indexes have to agree about it, so `SHA256SUMS` loses its rows first and the manifest's entry
    for that index is then re-hashed — otherwise the fixture would be a corrupt bundle, not a legal one.
    """
    for name in ("diff.json", NOTES):
        (bundle / name).unlink(missing_ok=True)
    rows = [
        line
        for line in (bundle / SUMS).read_text(encoding="utf-8").splitlines()
        if line.strip() and line.split("  ", 1)[1].removeprefix("*") not in ("diff.json", NOTES)
    ]
    (bundle / SUMS).write_text("\n".join(rows) + "\n", encoding="utf-8")
    manifest = json.loads((bundle / MANIFEST).read_text(encoding="utf-8"))
    manifest["release"]["notes"] = None
    for key in ("baseline_snapshot_id", "diff_schema_version"):
        manifest["extensions"][key] = None
    manifest["files"] = [
        entry for entry in manifest["files"] if entry["path"] not in ("diff.json", NOTES)
    ]
    digest = hashlib.sha256((bundle / SUMS).read_bytes())
    for entry in manifest["files"]:
        if entry["path"] == SUMS:
            entry["sha256"] = digest.hexdigest()
            entry["size"] = (bundle / SUMS).stat().st_size
    (bundle / MANIFEST).write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


class ReleaseNotesAreCheckedAgainstWhatTheBundlePromises(unittest.TestCase):
    """`D-N09-1`: the notes document is conditional, and so is the refusal its absence earns."""

    def setUp(self) -> None:
        self._temp = tempfile.TemporaryDirectory(prefix="fws-portability-")
        self.tmp = Path(self._temp.name)

    def tearDown(self) -> None:
        self._temp.cleanup()

    def bundle(self, label: str) -> Path:
        return legal_bundle(self.tmp / label)

    def test_a_release_that_shipped_its_notes_still_passes_whole(self):
        """The PASS this round must not disturb: 59 checks, exit 0, notes answered from the file."""
        result = run_reader(self.bundle("shipped"))
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertNotIn("Traceback", result.stderr)
        self.assertIn(f"ok    what were Release Notes: {151} bytes, shipped verbatim", result.stdout)
        self.assertIn("59 of 59 checks passed", result.stdout)

    def test_notes_a_release_promised_but_did_not_ship_fail_instead_of_crashing(self):
        """The defect: a promised notes file that is not there has to be a verdict, not a traceback."""
        bundle = self.bundle("promised")
        (bundle / NOTES).unlink()
        result = run_reader(bundle)
        combined = result.stdout + result.stderr

        self.assertNotIn("Traceback", combined, "an unhandled exception is not a review result")
        self.assertNotEqual(result.returncode, 0, f"a broken promise was accepted:\n{combined}")
        self.assertIn(f"{NOTES}", result.stdout)
        self.assertIn(
            "FAIL  what were Release Notes: unanswerable: ",
            result.stdout,
            f"the refusal never reached the report:\n{result.stdout}",
        )
        self.assertIn("checks passed", result.stdout, "the script stopped before printing a tally")

    def test_the_indexes_are_still_read_when_a_promised_document_is_gone(self):
        """One missing file cannot switch the rest of the review off."""
        bundle = self.bundle("indexes")
        (bundle / NOTES).unlink()
        result = run_reader(bundle)

        self.assertEqual(result.returncode, 1, f"exit code does not describe the report:\n{result.stdout}")
        self.assertNotIn("Traceback", result.stderr)
        self.assertIn("checks passed", result.stdout, "the script stopped before printing a tally")
        self.assertIn(f"ok    {SUMS} describes analysis.json", result.stdout)
        self.assertIn("ok    the manifest does not hash itself (section 20)", result.stdout)
        self.assertIn(f"FAIL  release-manifest.json describes {NOTES}", result.stdout)

    def test_notes_a_release_never_promised_are_skipped_and_not_marked_passed(self):
        """Legal absence is reported as legal absence: a SKIP, and no invented missing-file error."""
        bundle = self.bundle("unpromised")
        release_the_bundle_from_notes(bundle)
        result = run_reader(bundle)

        self.assertNotIn("Traceback", result.stderr)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn(
            f"SKIP  {NOTES} is absent, and that is legal here",
            result.stdout,
            f"a legal absence was reported as a pass:\n{result.stdout}",
        )
        self.assertIn("ok    what were Release Notes: none:", result.stdout)
        self.assertFalse(
            any(
                line.startswith("FAIL") and NOTES in line
                for line in result.stdout.splitlines()
            ),
            f"a document this release never promised was refused:\n{result.stdout}",
        )
        self.assertIn("skipped as not applicable", result.stdout)

    def test_notes_that_changed_after_the_index_was_written_are_refused(self):
        """A tampered promised document is a FAIL about the document, never a legal absence."""
        bundle = self.bundle("tampered")
        (bundle / NOTES).write_text(
            (bundle / NOTES).read_text(encoding="utf-8") + "\nwritten after the bundle closed\n",
            encoding="utf-8",
        )
        result = run_reader(bundle)

        self.assertNotIn("Traceback", result.stderr)
        self.assertEqual(result.returncode, 1)
        self.assertIn(f"FAIL  {SUMS} describes {NOTES}", result.stdout)

    def test_an_index_that_still_holds_notes_does_not_call_its_absence_legal(self):
        """The manifest releasing a document does not erase the sums index that promised it."""
        bundle = self.bundle("stale-index")
        manifest = json.loads((bundle / MANIFEST).read_text(encoding="utf-8"))
        manifest["release"]["notes"] = None
        (bundle / MANIFEST).write_text(
            json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
        (bundle / NOTES).unlink()
        result = run_reader(bundle)

        self.assertNotIn("Traceback", result.stderr)
        self.assertEqual(result.returncode, 1)
        self.assertIn(
            f"FAIL  release-manifest.json describes {NOTES}",
            result.stdout,
            f"an index that still promises the file excused its absence:\n{result.stdout}",
        )


if __name__ == "__main__":
    unittest.main()
