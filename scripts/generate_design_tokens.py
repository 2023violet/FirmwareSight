#!/usr/bin/env python3
"""Generate apps/desktop/ui/src/styles/tokens.css from assets/design-tokens.json.

The token file is the authority (ADR-0018); this script only projects it into CSS custom
properties. It never rewrites a value, so changing a number in the UI requires changing the
frozen token file first.

Usage:
    python scripts/generate_design_tokens.py             # write the file
    python scripts/generate_design_tokens.py --check     # fail on drift, write nothing (CI)

Deterministic: the same input bytes produce the same output bytes. The only embedded digest is
the SHA-256 of the source token file, which is part of the point - it proves which bytes the
CSS came from.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "assets" / "design-tokens.json"
TARGET = ROOT / "apps" / "desktop" / "ui" / "src" / "styles" / "tokens.css"

THEME = "light"  # DESIGN.md 4: light is the only MVP theme.

# (json path, css name suffix, unit for numeric values)
GROUPS = (
    ("color", "color", ""),
    ("spacing", "space", "px"),
    ("radius", "radius", "px"),
    ("border", "border", "px"),
    ("layout", "layout", "px"),
    ("table", "row", "px"),
    ("motion", "motion", "ms"),
    ("focus", "focus", "px"),
    ("shadow", "shadow", ""),
)

# Typography is emitted separately because its leaf names carry meaning (`ui`, `mono`,
# `size.body`, `line_height.body`) that a single unit rule cannot express.
TYPOGRAPHY_LABEL = "Typography"


def flatten(node: dict, prefix: tuple[str, ...] = ()) -> list[tuple[tuple[str, ...], object]]:
    out: list[tuple[tuple[str, ...], object]] = []
    for key, value in node.items():
        path = (*prefix, key)
        if isinstance(value, dict):
            out.extend(flatten(value, path))
        else:
            out.append((path, value))
    return out


def color_name(path: tuple[str, ...]) -> str:
    # Color roles are stored as dotted keys under the theme, so `("color", "light",
    # "accent", "primary")` and `("color", "light", "accent.primary")` are the same role.
    key = ".".join(path[2:])
    return "--fs-color-" + key.replace("_", "-").replace(".", "-")


def typography_entries(pairs) -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []
    for path, value in pairs:
        # ("typography", ...) -> drop the section name, then special-case the two subtrees.
        rest = list(path[1:])
        if rest[0] == "size":
            name, unit = "--fs-font-size-" + rest[1], "px"
        elif rest[0] == "line_height":
            name, unit = "--fs-line-height-" + rest[1].replace("_", "-"), "px"
        elif rest[0] in ("ui", "mono"):
            name, unit = "--fs-font-" + rest[0], ""
        else:
            raise ValueError(f"unhandled typography token: {path}")
        rows.append((name, f"{value}{unit}"))
    return rows


def emitted_rows(tokens: dict) -> list[tuple[str, str, str]]:
    """(group_css_prefix, css_name, raw_value) in a fixed, reproducible order."""
    rows: list[tuple[str, str, str]] = []

    for section, css_prefix, _unit in GROUPS:
        group = tokens.get(section)
        if not isinstance(group, dict):
            continue
        for path, value in flatten(group, (section,)):
            if section == "color":
                if path[1] != THEME:
                    continue
                name, text = color_name(path), str(value)
            elif section == "focus":
                if list(path[1:]) == ["ring", "color"]:
                    # Reference the accent role instead of repeating its hex, so the focus ring
                    # can never disagree with the accent.
                    name = "--fs-focus-ring-color"
                    text = "{color.light.accent.primary}"
                else:
                    name = "--fs-focus-" + "-".join(p.replace("_", "-") for p in path[1:])
                    text = f"{value}px"
            elif section == "shadow":
                name = "--fs-shadow-" + str(path[1]).replace(".", "-")
                text = str(value)
            elif section == "table":
                # ("table", "row", "compact") -> --fs-row-compact; "row" is the group, not a name.
                name = "--fs-row-" + "-".join(p.replace("_", "-") for p in path[2:])
                text = f"{value}px"
            else:
                name = "--fs-" + css_prefix + "-" + "-".join(p.replace("_", "-") for p in path[1:])
                if isinstance(value, str):
                    # `motion.ease` is a curve, not a length; adding a unit here would emit
                    # invalid CSS.
                    text = value
                else:
                    text = f"{value}{'ms' if section == 'motion' else 'px'}"
            rows.append((css_prefix, name, text))

    for name, text in typography_entries(flatten(tokens["typography"], ("typography",))):
        rows.append(("font", name, text))
    return rows


def resolve(text: str, known: set[str]) -> str:
    """`{color.light.accent.primary}` -> `var(--fs-color-accent-primary)`."""
    if not text.startswith("{") or not text.endswith("}"):
        return text
    path = text[1:-1].split(".")
    name = color_name(("color", *path[1:])) if path[0] == "color" else None
    if name is None or name not in known:
        raise ValueError(f"unresolved token reference in {text}")
    return f"var({name})"


def render(tokens: dict, source_sha: str) -> str:
    rows = emitted_rows(tokens)
    known = {name for _g, name, _v in rows}

    lines = [
        "/* GENERATED by scripts/generate_design_tokens.py - do not edit by hand.",
        " * Source: assets/design-tokens.json",
        f" * Source SHA-256: {source_sha}",
        " * Values are projected unchanged. A new number belongs in the token file first.",
        " */",
        ":root {",
    ]
    labels = {css: label.title() for (_s, css, _u), label in zip(GROUPS, (
        "Color", "Spacing", "Radius", "Border", "Layout", "Table density", "Motion", "Focus",
        "Shadow",
    ))}
    labels["font"] = TYPOGRAPHY_LABEL

    current_group = None
    for group, name, value in rows:
        if group != current_group:
            if current_group is not None:
                lines.append("")
            lines.append(f"  /* {labels[group]} */")
            current_group = group
        lines.append(f"  {name}: {resolve(value, known)};")
    lines.append("}")
    lines.append("")
    return "\n".join(lines)


def main(argv: list[str]) -> int:
    check = "--check" in argv[1:]
    raw = SOURCE.read_bytes()
    tokens = json.loads(raw.decode("utf-8"))
    text = render(tokens, hashlib.sha256(raw).hexdigest())

    if check:
        current = TARGET.read_text(encoding="utf-8") if TARGET.exists() else ""
        if current != text:
            print(
                "tokens.css is out of date with assets/design-tokens.json.\n"
                "Run: python scripts/generate_design_tokens.py",
                file=sys.stderr,
            )
            return 1
        print("tokens.css matches the frozen token file.")
        return 0

    TARGET.parent.mkdir(parents=True, exist_ok=True)
    TARGET.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {TARGET.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
