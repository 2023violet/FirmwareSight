"""Generate THIRD_PARTY_LICENSES.md: every third-party component the released binaries are built from on the
four shipping targets of `deny.toml` `[graph]`, with the licence texts those components ship. Procedural macros
are listed too: they run at build time and are not linked, but listing them over-states rather than omits.

Sources, both read from the local build inputs (nothing is downloaded):
- Rust: `cargo tree` over the workspace's normal (non-dev, non-build) dependency edges for each shipping target,
  then each package's own LICENSE / LICENCE / COPYING / NOTICE / UNLICENSE files from its source directory
  (`cargo metadata`).
- Frontend: `pnpm licenses list --prod --json` in `apps/desktop/ui`, then each package's licence files.

Identical licence texts are written once and referenced by every package that ships them, so the file stays
readable. Deterministic: the same lockfiles produce the same bytes.

    python scripts/gen_third_party_licenses.py           # write THIRD_PARTY_LICENSES.md
    python scripts/gen_third_party_licenses.py --check   # fail if the file is out of date
"""

from __future__ import annotations

import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UI = ROOT / "apps" / "desktop" / "ui"
OUT = ROOT / "THIRD_PARTY_LICENSES.md"
LICENSE_FILE = re.compile(r"^(licen[cs]e|copying|notice|unlicense)([-._].*)?$", re.IGNORECASE)


def run(cmd: list[str], cwd: Path) -> str:
    exe = shutil.which(cmd[0]) or cmd[0]
    result = subprocess.run([exe, *cmd[1:]], cwd=cwd, capture_output=True, text=True, encoding="utf-8", check=False)
    if result.returncode != 0:
        sys.exit(f"{' '.join(cmd)} failed:\n{result.stderr}")
    return result.stdout


def licence_files(directory: Path) -> list[tuple[str, str]]:
    found = []
    if directory.is_dir():
        for path in sorted(directory.iterdir(), key=lambda p: p.name.lower()):
            if path.is_file() and LICENSE_FILE.match(path.name):
                text = path.read_text(encoding="utf-8", errors="replace").replace("\r\n", "\n").strip()
                if text:
                    found.append((path.name, text))
    return found


def rust_packages() -> list[dict]:
    meta = json.loads(run(["cargo", "metadata", "--format-version", "1", "--locked"], ROOT))
    workspace = set(meta["workspace_members"])
    by_id = {p["id"]: p for p in meta["packages"]}
    # deny.toml uses multi-line inline tables that Python's tomllib does not read, so only the `[graph]`
    # targets array is taken from it, verbatim.
    deny = (ROOT / "deny.toml").read_text(encoding="utf-8")
    graph = re.search(r"^\[graph\]\s*$(.*?)^\[", deny, re.MULTILINE | re.DOTALL)
    array = re.search(r"^targets\s*=\s*\[(.*?)\]", graph.group(1), re.MULTILINE | re.DOTALL) if graph else None
    targets = re.findall(r'"([^"]+)"', array.group(1)) if array else []
    if not targets:
        sys.exit("deny.toml [graph] declares no targets")
    shipped = set()
    for target in targets:
        tree = run(
            ["cargo", "tree", "--workspace", "--locked", "-e", "normal", "--target", target, "--prefix", "none",
             "--no-dedupe", "-f", "{p}"],
            ROOT,
        )
        for line in tree.splitlines():
            parts = line.split()
            if len(parts) >= 2 and parts[1].startswith("v"):
                shipped.add((parts[0], parts[1][1:]))
    out = []
    for pid, package in by_id.items():
        if pid in workspace or (package["name"], package["version"]) not in shipped:
            continue
        out.append({
            "ecosystem": "Rust crate",
            "name": package["name"],
            "version": package["version"],
            "license": package.get("license") or "(see licence file)",
            "source": package.get("repository") or f"https://crates.io/crates/{package['name']}",
            "files": licence_files(Path(package["manifest_path"]).parent),
        })
    return out


def npm_packages() -> list[dict]:
    if not (UI / "node_modules").is_dir():
        sys.exit("apps/desktop/ui/node_modules is missing; run `pnpm install --frozen-lockfile` there first")
    listing = json.loads(run(["pnpm", "licenses", "list", "--prod", "--json"], UI))
    out = []
    for license_name, entries in listing.items():
        for entry in entries:
            versions = entry.get("versions") or [entry.get("version", "")]
            paths = entry.get("paths") or [entry.get("path", "")]
            for version, path in zip(versions, paths):
                out.append({
                    "ecosystem": "npm package",
                    "name": entry["name"],
                    "version": version,
                    "license": license_name,
                    "source": entry.get("homepage") or f"https://www.npmjs.com/package/{entry['name']}",
                    "files": licence_files(Path(path)) if path else [],
                })
    return out


def render(packages: list[dict]) -> str:
    packages.sort(key=lambda p: (p["ecosystem"], p["name"].lower(), p["version"]))
    texts: dict[str, str] = {}
    lines = [
        "# Third-party licences",
        "",
        "FirmwareSight is licensed under the Apache License 2.0 (see `LICENSE`). Its release binaries are built from",
        "the third-party components below (on the shipping targets of `deny.toml`), each under its own licence.",
        "Procedural macros run at build time and are listed although they are not linked. Generated by",
        "`scripts/gen_third_party_licenses.py` from `Cargo.lock` and `apps/desktop/ui/pnpm-lock.yaml`; do not edit.",
        "",
        f"Components: {sum(1 for p in packages if p['ecosystem'] == 'Rust crate')} Rust crates, "
        f"{sum(1 for p in packages if p['ecosystem'] == 'npm package')} npm packages.",
        "",
        "## Components",
        "",
        "| Component | Version | Licence | Source | Licence text |",
        "|---|---|---|---|---|",
    ]
    for package in packages:
        refs = []
        for name, text in package["files"]:
            key = hashlib.sha256(text.encode("utf-8")).hexdigest()[:12]
            texts.setdefault(key, text)
            refs.append(f"[{name}](#text-{key})")
        cell = ", ".join(refs) if refs else "not shipped in the package; licence expression applies"
        lines.append(
            f"| {package['ecosystem']} `{package['name']}` | {package['version']} | {package['license']} | "
            f"{package['source']} | {cell} |"
        )
    lines += ["", "## Licence texts", ""]
    for key in sorted(texts):
        lines += [f'<a id="text-{key}"></a>', "", f"### text-{key}", "", "```text", texts[key], "```", ""]
    return "\n".join(lines).rstrip("\n") + "\n"


def main(argv: list[str]) -> int:
    text = render(rust_packages() + npm_packages())
    if "--check" in argv[1:]:
        current = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
        if current != text:
            print("THIRD_PARTY_LICENSES.md is out of date. Run: python scripts/gen_third_party_licenses.py", file=sys.stderr)
            return 1
        print("THIRD_PARTY_LICENSES.md matches the dependency graph.")
        return 0
    OUT.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {OUT.relative_to(ROOT)} ({len(text) // 1024} KiB)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
