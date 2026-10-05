"""Verify DIRECTORY_TREE.txt and SHA256SUMS against the tracked index, without the generator's help.

Shares no code with scripts/generate_baseline_artifacts.py: it re-parses the manifest, re-derives the
tracked set from `git ls-files -s -z`, and resolves content through a DIFFERENT Git path — `git cat-file
--batch` fed `:<path>`, which asks the index for the file, where the generator asks for the blob OID. A
mistake in either parser, or in how either frames the batch call, shows up here as a disagreement rather
than as agreement. The manifest cannot certify itself, which is exactly why this exists as a second,
independent pass.

    python scripts/verify_baseline_artifacts.py

WHAT THE DIGESTS ARE OF (ADR-0029). Root `SHA256SUMS` records the SHA-256 of canonical Git stage-0 index
blob content, not of the bytes currently present in the working directory. Raw disk bytes are NOT the
root-baseline authority: a checkout is a rendering, and with `core.autocrlf=true` plus `.gitattributes`
`text eol=lf` some tracked files are stored LF and written CRLF. So `sha256sum -c SHA256SUMS` run against
a working tree can legitimately report failures on those files, and that is not corruption — this
verifier is the canonical check, and `git archive <commit>` extracted outside the repository followed by
`sha256sum -c SHA256SUMS` is the independent external proof, because an archive carries canonical blob
content.

Exit code is non-zero when a digest does not match the index blob, a listed path has no stage-0 entry, a
tracked file is unlisted, a stale or duplicate entry survives, the index is unmerged, the manifest hashes
itself, either artifact is not LF-only, or the working copies of the two artifacts are not the blobs they
claim to be. Nothing here repairs anything: it reports and fails.
"""

import hashlib
import os
import posixpath
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MANIFEST = "SHA256SUMS"
TREE = "DIRECTORY_TREE.txt"
NUL = b"\0"


def index_paths():
    """Tracked paths and their stage, parsed independently of the generator.

    Bytes stay bytes until the very last moment; two tracked names are non-ASCII and one contains a space,
    so any quoting layer would either drop them or return a name that is not a path.
    """
    raw = subprocess.run(["git", "ls-files", "-s", "-z"], cwd=ROOT,
                         capture_output=True, check=True).stdout
    paths, unmerged = [], []
    for chunk in raw.split(NUL):
        if not chunk:
            continue
        meta, _, path = chunk.partition(b"\t")
        _mode, _oid, stage = meta.split(b" ")
        if stage != b"0":
            unmerged.append(path.decode("utf-8", "replace"))
            continue
        paths.append(path)
    return paths, unmerged


def index_blob_contents(paths):
    """Canonical content of each path's stage-0 index entry, fetched as `:<path>` through one batch call.

    Deliberately not the generator's OID lookup: this asks Git to resolve the index entry by name.
    """
    feed = b"\n".join(b":" + p for p in paths) + b"\n"
    out = subprocess.run(["git", "cat-file", "--batch"], cwd=ROOT, input=feed,
                         capture_output=True, check=True).stdout
    contents, missing = {}, []
    cursor = 0
    for path in paths:
        end = out.index(b"\n", cursor)
        header = out[cursor:end].decode("utf-8", "replace")
        parts = header.split(" ")
        if len(parts) == 3 and parts[1] == "blob":
            size = int(parts[2])
            body = out[end + 1:end + 1 + size]
            cursor = end + 2 + size
            contents[path] = body
        else:
            missing.append(path.decode("utf-8", "replace"))
            cursor = end + 1
    return contents, missing


def read_blob_bytes(name):
    """The committed form of one baseline artifact, so a stale working copy cannot be read as the answer."""
    out = subprocess.run(["git", "cat-file", "--batch"], cwd=ROOT, input=(":" + name + "\n").encode(),
                         capture_output=True, check=True).stdout
    end = out.index(b"\n")
    parts = out[:end].decode("utf-8", "replace").split(" ")
    if len(parts) != 3 or parts[1] != "blob":
        raise SystemExit(f"!! {name} has no stage-0 index entry; stage it before verifying")
    return out[end + 1:end + 1 + int(parts[2])]


def main():
    tracked, unmerged = index_paths()
    raw_sums = read_blob_bytes(MANIFEST)
    rows = []
    for line in raw_sums.decode("utf-8", "surrogateescape").splitlines():
        entry = re.fullmatch(r"([0-9a-f]{64})  (\S.*)", line)
        if entry is None:
            print(f"!! malformed entry: {line[:80]!r}")
            return 1
        rows.append((entry.group(1), entry.group(2)))
    listed = [path for _, path in rows]
    listed_bytes = [p.encode("utf-8", "surrogateescape") for p in listed]

    contents, unresolved = index_blob_contents(listed_bytes)
    expected = dict(zip(listed_bytes, [d for d, _ in rows]))
    mismatch = sorted(p.decode("utf-8", "replace") for p in listed_bytes
                      if p in contents and hashlib.sha256(contents[p]).hexdigest() != expected[p])
    unindexed = sorted({p.decode("utf-8", "replace") for p in listed_bytes if p not in contents})
    stale = sorted(p.decode("utf-8", "replace") for p in set(listed_bytes) - set(tracked))
    present = {p.decode("utf-8", "replace") for p in tracked}
    unlisted = sorted(p for p in present
                      if posixpath.basename(p) != MANIFEST and p not in set(listed))
    duplicates = sorted({p for p in listed if listed.count(p) > 1})
    on_disk_absent = [p for p in listed if not os.path.isfile(os.path.join(ROOT, p))]

    tree_raw = read_blob_bytes(TREE)
    tree = tree_raw.decode("utf-8", "surrogateescape").splitlines()
    worktree_drift = [name for name, blob in ((MANIFEST, raw_sums), (TREE, tree_raw))
                      if open(os.path.join(ROOT, name), "rb").read() != blob]

    checks = [
        ("tracked files", len(tracked)),
        ("sum entries", len(rows)),
        ("index blob mismatch", len(mismatch)),
        ("listed but unindexed", len(unindexed)),
        ("listed but untracked", len(stale)),
        ("tracked but unlisted", len(unlisted)),
        ("duplicate entries", len(duplicates)),
        ("unmerged index entries", len(unmerged)),
        ("SHA256SUMS hashes itself", int(MANIFEST in listed)),
        ("sums sorted by path", int(listed == sorted(listed))),
        ("sums CRLF bytes", raw_sums.count(b"\r\n")),
        ("tree CRLF bytes", tree_raw.count(b"\r\n")),
        ("tree lines", len(tree)),
        ("tree names DIRECTORY_TREE.txt", int(any(TREE in line for line in tree))),
        ("tree names SHA256SUMS", int(any(MANIFEST in line for line in tree))),
        ("host path in sums", int(any(re.search(r"[A-Za-z]:[\\/]|Users", p) for p in listed))),
        ("artifact worktree != blob", len(worktree_drift)),
        ("listed but absent on disk", len(on_disk_absent)),
    ]
    for name, value in checks:
        print(f"{name:<32} {value}")

    print("-- ADR-0029: digests above are of canonical Git stage-0 index blob bytes. Raw "
          "working-directory bytes are not the root-baseline authority, so the count on "
          "'listed but absent on disk' is reported and not failed; a checkout missing such a file is a "
          "checkout problem, not a baseline one. --")

    failed = bool(mismatch or unindexed or stale or unlisted or duplicates or unmerged)
    failed = failed or raw_sums.count(b"\r\n") > 0 or tree_raw.count(b"\r\n") > 0
    failed = failed or MANIFEST in listed or bool(worktree_drift) or bool(unresolved)
    for label, items in (("index blob mismatch", mismatch), ("unindexed", unindexed),
                         ("stale", stale), ("unlisted", unlisted), ("duplicate", duplicates),
                         ("unmerged", unmerged), ("worktree differs from blob", worktree_drift)):
        for item in items[:5]:
            print(f"!! {label}: {item}")
    print("RESULT", "FAIL" if failed else "PASS")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
