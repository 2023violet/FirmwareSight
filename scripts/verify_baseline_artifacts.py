"""Verify DIRECTORY_TREE.txt and SHA256SUMS against the tracked set, without the generator's help.

Shares no logic with scripts/generate_baseline_artifacts.py: it re-derives the tracked set from
`git ls-files` and hashes files straight off disk, so a bug shared with the generator cannot hide here.
The manifest cannot certify itself, which is exactly why this exists as a second, independent pass.

    python scripts/verify_baseline_artifacts.py

Exit code is non-zero when a hash mismatches, a listed file is absent, a tracked file is unlisted, a
stale or duplicate entry survives, or the artifacts are not LF-only.
"""

import hashlib
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def tracked_paths():
    out = subprocess.run(["git", "ls-files", "-z"], cwd=ROOT, capture_output=True, check=True).stdout
    return sorted(p.decode() for p in out.split(b"\0") if p)


def main():
    raw_sums = open(os.path.join(ROOT, "SHA256SUMS"), "rb").read()
    rows = []
    for line in raw_sums.decode("utf-8").splitlines():
        entry = re.fullmatch(r"([0-9a-f]{64})  (\S.*)", line)
        if entry is None:
            print(f"!! malformed entry: {line[:80]!r}")
            return 1
        rows.append((entry.group(1), entry.group(2)))
    listed = [path for _, path in rows]
    tracked = tracked_paths()

    mismatch = [p for d, p in rows if os.path.isfile(os.path.join(ROOT, p))
                and hashlib.sha256(open(os.path.join(ROOT, p), "rb").read()).hexdigest() != d]
    absent = [p for _, p in rows if not os.path.isfile(os.path.join(ROOT, p))]
    stale = sorted(set(listed) - set(tracked))
    unlisted = sorted(p for p in tracked if os.path.basename(p) != "SHA256SUMS" and p not in set(listed))
    duplicates = sorted({p for p in listed if listed.count(p) > 1})

    tree_raw = open(os.path.join(ROOT, "DIRECTORY_TREE.txt"), "rb").read()
    tree = tree_raw.decode("utf-8").splitlines()

    checks = [
        ("tracked files", len(tracked)),
        ("sum entries", len(rows)),
        ("hash mismatch", len(mismatch)),
        ("listed but absent", len(absent)),
        ("listed but untracked", len(stale)),
        ("tracked but unlisted", len(unlisted)),
        ("duplicate entries", len(duplicates)),
        ("SHA256SUMS hashes itself", int("SHA256SUMS" in listed)),
        ("sums sorted by path", int(listed == sorted(listed))),
        ("sums CRLF bytes", raw_sums.count(b"\r\n")),
        ("tree CRLF bytes", tree_raw.count(b"\r\n")),
        ("tree lines", len(tree)),
        ("tree names DIRECTORY_TREE.txt", int(any("DIRECTORY_TREE.txt" in line for line in tree))),
        ("tree names SHA256SUMS", int(any("SHA256SUMS" in line for line in tree))),
        ("host path in sums", int(any(re.search(r"[A-Za-z]:[\\/]|Users", p) for p in listed))),
    ]
    for name, value in checks:
        print(f"{name:<32} {value}")

    failed = bool(mismatch or absent or stale or unlisted or duplicates)
    failed = failed or raw_sums.count(b"\r\n") > 0 or tree_raw.count(b"\r\n") > 0
    failed = failed or "SHA256SUMS" in listed
    for label, items in (("mismatch", mismatch), ("absent", absent), ("stale", stale),
                         ("unlisted", unlisted), ("duplicate", duplicates)):
        for item in items[:5]:
            print(f"!! {label}: {item}")
    print("RESULT", "FAIL" if failed else "PASS")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
