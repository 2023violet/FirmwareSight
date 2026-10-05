"""Regenerate DIRECTORY_TREE.txt and SHA256SUMS from the tracked file set.

Both artifacts describe the baseline, so both derive their path set from the Git stage-0 index and
nothing else: a gitignored path cannot enter them, and a path that was never staged is not part of the
baseline until it is staged. The tree lists itself; the manifest does not hash itself.

    python scripts/generate_baseline_artifacts.py tree   > DIRECTORY_TREE.txt
    python scripts/generate_baseline_artifacts.py sums   > SHA256SUMS

WHAT BYTES ARE HASHED (ADR-0029). `sums` hashes the canonical Git stage-0 index blob content, never the
raw bytes currently sitting in the working directory. A checkout is a rendering of the repository, not the
repository: with `core.autocrlf=true` and `.gitattributes` `text eol=lf`, 13 tracked files here hold CRLF
on disk while their blobs are LF, so working-copy digests described the host that wrote the manifest and
moved between heads (19, 19, 14, 14, 17, 13) without anyone deciding anything. Index blobs are the same
bytes on Windows, Linux and macOS, in a clean worktree, and in a `git archive` extraction. `sha256sum -c`
against a working tree is therefore no longer a valid check of this file; the canonical external proof is
`python scripts/verify_baseline_artifacts.py`, or `git archive <commit>` extracted outside the repository
followed by `sha256sum -c SHA256SUMS` over the extracted tree.

THE ORDER THAT MUST BE FOLLOWED, AND WHY IT IS ENFORCED.

    1  edit intended source/docs
    2  confirm no unrelated changes             git status --porcelain
    3  stage all intended non-baseline changes   git add <paths>
    4  if the tracked path set changed:          generate_baseline_artifacts.py tree > DIRECTORY_TREE.txt
    5  stage the tree                            git add DIRECTORY_TREE.txt
    6  generate the manifest from index blobs    generate_baseline_artifacts.py sums > SHA256SUMS
    7  stage the manifest                        git add SHA256SUMS
    8  run the independent verifier              python scripts/verify_baseline_artifacts.py
    9  run the full gate                         python scripts/check.py
    10 inspect the staged diff                   git diff --cached
    11 commit

`sums` refuses to write when Git reports a semantic unstaged change to any tracked path, so a forgotten
`git add` cannot silently produce a manifest for old index content — the mistake this file has made twice.
The only path exempted from that guard is `SHA256SUMS` itself, which the redirection truncates before this
script runs. The guard asks Git (`git diff`), so a worktree file that differs from its blob only in line
terminators, and which the clean filter maps back to the same blob, is a clean file and is not refused;
the tool never inspects raw EOL bytes to call a file dirty. `DIRECTORY_TREE.txt` is deliberately *not*
exempted: step 5 stages it first, and if it has not been, the manifest would certify the tree's old blob.

Ordering follows the layout the baseline has always used: at every level directories come first, then
files, each group sorted case-insensitively. Paths are repository-relative with `/` and LF-terminated, so
no host path and no CRLF reaches either artifact.
"""

import hashlib
import os
import posixpath
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROOT_LABEL = "FirmwareSight_Project_Baseline_v0.6.0/"
NUL = b"\0"
MANIFEST = "SHA256SUMS"


def emit(text):
    """Write LF bytes: on Windows, text-mode stdout would translate them to CRLF.

    `surrogateescape` because paths arrive as bytes decoded that way; re-encoding them any stricter would
    turn a non-ASCII tracked filename into a crash instead of into the line the manifest has to carry.
    """
    sys.stdout.buffer.write(text.encode("utf-8", "surrogateescape"))


def index_entries():
    """The stage-0 index as (path bytes, blob OID bytes), rejecting an unmerged index.

    Paths stay bytes: the baseline contains non-ASCII names and one with a space, and any quoting layer
    would either drop them or hand back a name that is not the path.
    """
    raw = subprocess.run(["git", "ls-files", "-s", "-z"], cwd=ROOT,
                         capture_output=True, check=True).stdout
    entries, unmerged = [], []
    for chunk in raw.split(NUL):
        if not chunk:
            continue
        meta, _, path = chunk.partition(b"\t")
        _mode, oid, stage = meta.split(b" ")
        if stage != b"0":
            unmerged.append((path.decode("utf-8", "replace"), stage.decode()))
            continue
        entries.append((path, oid))
    if unmerged:
        raise SystemExit(
            "refusing to derive the baseline from an unmerged index; resolve the conflict first: "
            + ", ".join(f"{p} (stage {s})" for p, s in unmerged[:10])
        )
    return entries


def blob_digests(entries):
    """SHA-256 of each index blob's canonical content, fetched through one batched Git call.

    One `git cat-file --batch` process for the whole set rather than ~700 heavyweight subprocesses.
    """
    feed = b"\n".join(oid for _path, oid in entries) + b"\n"
    out = subprocess.run(["git", "cat-file", "--batch"], cwd=ROOT, input=feed,
                         capture_output=True, check=True).stdout
    digests = {}
    cursor = 0
    for path, oid in entries:
        end = out.index(b"\n", cursor)
        header = out[cursor:end].decode("utf-8", "replace")
        parts = header.split(" ")
        name = path.decode("utf-8", "replace")
        if len(parts) != 3 or parts[1] != "blob":
            raise SystemExit(f"git did not return a blob for {name!r}: {header!r}")
        if parts[0].encode() != oid:
            raise SystemExit(f"git returned {parts[0]} where index blob {oid.decode()} was asked for")
        size = int(parts[2])
        body = out[end + 1:end + 1 + size]
        if len(body) != size:
            raise SystemExit(f"truncated blob for {name!r}: {len(body)} of {size} bytes")
        cursor = end + 2 + size
        digests[path] = hashlib.sha256(body).hexdigest()
    return digests


def unstaged_semantic_changes():
    """Tracked paths Git itself reports as modified in the worktree but not staged, minus the manifest.

    Asking `git diff` rather than reading bytes is the point: a checkout that differs from its blob only in
    line terminators is clean to Git, and a clean file is not a change waiting to be certified.
    """
    raw = subprocess.run(["git", "diff", "--name-only", "-z"], cwd=ROOT,
                         capture_output=True, check=True).stdout
    return [p.decode("utf-8", "surrogateescape") for p in raw.split(NUL) if p and p != MANIFEST.encode()]


def build_tree(paths):
    """Nested dict of directories, with files as the empty-dict leaves keyed by name."""
    root = {}
    for path in paths:
        node = root
        for part in path.split("/"):
            node = node.setdefault(part, {})
    return root


def render(node, prefix, lines):
    entries = sorted(node, key=str.lower)
    dirs = [e for e in entries if node[e]]
    files = [e for e in entries if not node[e]]
    ordered = dirs + files
    for i, name in enumerate(ordered):
        last = i == len(ordered) - 1
        branch = "└── " if last else "├── "
        child_prefix = "    " if last else "│   "
        if node[name]:
            lines.append(f"{prefix}{branch}{name}/")
            render(node[name], prefix + child_prefix, lines)
        else:
            lines.append(f"{prefix}{branch}{name}")


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "tree"
    entries = index_entries()
    if mode == "tree":
        lines = [ROOT_LABEL]
        render(build_tree(sorted(p.decode("utf-8", "surrogateescape") for p, _ in entries)), "", lines)
        emit("\n".join(lines) + "\n")
    elif mode == "sums":
        dirty = unstaged_semantic_changes()
        if dirty:
            raise SystemExit(
                "refusing to write SHA256SUMS: "
                f"{len(dirty)} tracked path(s) have unstaged changes that are not in the index, so the "
                "manifest would certify content that is not about to be committed. Stage the intended "
                "changes (step 3) first. " + ", ".join(sorted(dirty)[:10])
                + ("" if len(dirty) <= 10 else f" … (+{len(dirty) - 10} more)")
            )
        digests = blob_digests(entries)
        rows = []
        for path, _oid in entries:
            name = path.decode("utf-8", "surrogateescape")
            if posixpath.basename(name) == MANIFEST:
                continue
            rows.append(f"{digests[path]}  {name}")
        emit("\n".join(sorted(rows, key=lambda r: r.split("  ", 1)[1])) + "\n")
    else:
        raise SystemExit(f"unknown mode {mode}")


if __name__ == "__main__":
    main()
