"""Regenerate DIRECTORY_TREE.txt and SHA256SUMS from the tracked file set.

Both artifacts describe the baseline, so both derive from `git ls-files` and nothing else: a gitignored
path cannot enter them. The tree lists itself; the manifest does not hash itself.

    python scripts/generate_baseline_artifacts.py tree   > DIRECTORY_TREE.txt
    python scripts/generate_baseline_artifacts.py sums   > SHA256SUMS

Ordering follows the layout the baseline has always used: at every level directories come first, then
files, each group sorted case-insensitively. Paths are repository-relative with `/` and LF-terminated, so
no host path and no CRLF reaches either artifact. Run this before every baseline closeout commit, tree
first and sums last, and then verify with scripts/verify_baseline_artifacts.py.
"""

import hashlib
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROOT_LABEL = "FirmwareSight_Project_Baseline_v0.6.0/"


def emit(text):
    """Write LF bytes: on Windows, text-mode stdout would translate them to CRLF."""
    sys.stdout.buffer.write(text.encode("utf-8"))


def tracked():
    out = subprocess.run(["git", "ls-files", "-z"], cwd=ROOT, capture_output=True, check=True).stdout
    return sorted(p.decode() for p in out.split(b"\0") if p)


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
    paths = tracked()
    if mode == "tree":
        lines = [ROOT_LABEL]
        render(build_tree(paths), "", lines)
        emit("\n".join(lines) + "\n")
    elif mode == "sums":
        rows = []
        for path in paths:
            if os.path.basename(path) == "SHA256SUMS":
                continue
            digest = hashlib.sha256(open(os.path.join(ROOT, path), "rb").read()).hexdigest()
            rows.append(f"{digest}  {path}")
        emit("\n".join(sorted(rows, key=lambda r: r.split("  ", 1)[1])) + "\n")
    else:
        raise SystemExit(f"unknown mode {mode}")


if __name__ == "__main__":
    main()
