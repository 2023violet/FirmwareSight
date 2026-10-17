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
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "assets" / "design-tokens.json"
TARGET = ROOT / "apps" / "desktop" / "ui" / "src" / "styles" / "tokens.css"

THEMES = ("light", "dark")
"""The palettes this file projects.

`ADR-0031` lifted the light-only rule for the Developer Preview. Light stays `:root` because every
accepted screenshot, every existing value and the exported artefacts describe it; dark is a sibling
block selected by `data-fs-theme` on the document element. Nothing else is themed: spacing, radius,
typography, layout, table density, motion, focus geometry and shadow are one set of numbers that a
colour change does not touch.
"""

DARK_SELECTOR = '[data-fs-theme="dark"]'

# (json path, css name suffix, unit for numeric values)
GROUPS = (
    ("color", "color", ""),
    ("spacing", "space", "px"),
    ("radius", "radius", "px"),
    ("border", "border", "px"),
    ("layout", "layout", "px"),
    ("layer", "layer", ""),
    ("table", "row", "px"),
    ("icon", "icon", "px"),
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
        elif rest[0] == "weight":
            # ADR-0034 D3: a numeric weight scale replaces the CSS `bold` keyword.
            name, unit = "--fs-font-weight-" + rest[1], ""
        elif rest[0] in ("ui", "mono"):
            name, unit = "--fs-font-" + rest[0], ""
        else:
            raise ValueError(f"unhandled typography token: {path}")
        rows.append((name, f"{value}{unit}"))
    return rows


def emitted_rows(tokens: dict, theme: str) -> list[tuple[str, str, str]]:
    """(group_css_prefix, css_name, raw_value) in a fixed, reproducible order, for one palette."""
    rows: list[tuple[str, str, str]] = []

    for section, css_prefix, _unit in GROUPS:
        group = tokens.get(section)
        if not isinstance(group, dict):
            continue
        for path, value in flatten(group, (section,)):
            if section == "color":
                if path[1] != theme:
                    continue
                name, text = color_name(path), str(value)
            elif section == "focus":
                if list(path[1:]) == ["ring", "color"]:
                    # Reference the accent role instead of repeating its hex, so the focus ring
                    # can never disagree with the accent - in either palette.
                    name = "--fs-focus-ring-color"
                    text = "{color.accent.primary}"
                else:
                    name = "--fs-focus-" + "-".join(p.replace("_", "-") for p in path[1:])
                    text = f"{value}px"
            elif section == "layer":
                # A stacking order is a bare integer: `z-index` takes no unit (ADR-0034 A1).
                name = "--fs-layer-" + "-".join(p.replace("_", "-") for p in path[1:])
                text = str(value)
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


def resolve(text: str, known: set[str], theme: str) -> str:
    """`{color.accent.primary}` -> `var(--fs-color-accent-primary)` of the palette being emitted."""
    if not text.startswith("{") or not text.endswith("}"):
        return text
    path = text[1:-1].split(".")
    if path[0] != "color":
        raise ValueError(f"unresolved token reference in {text}")
    if path[1] in THEMES:
        raise ValueError(
            f"a colour reference names a palette ({text}); drop the theme segment so it follows "
            "the theme in force instead of one of them"
        )
    name = color_name(("color", theme, *path[1:]))
    if name not in known:
        raise ValueError(f"unresolved token reference in {text}")
    return f"var({name})"


def render(tokens: dict, source_sha: str) -> str:
    rows = emitted_rows(tokens, "light")
    known = {name for _g, name, _v in rows}
    dark_rows = [
        (g, n, v) for g, n, v in emitted_rows(tokens, "dark") if n.startswith("--fs-color-")
    ]
    # The ring is emitted again inside the dark block rather than left to inherit its substitution
    # from `:root`, so the file does not depend on which element carries the attribute.
    dark_rows.append(("focus", "--fs-focus-ring-color", "{color.accent.primary}"))

    lines = [
        "/* GENERATED by scripts/generate_design_tokens.py - do not edit by hand.",
        " * Source: assets/design-tokens.json",
        f" * Source SHA-256: {source_sha}",
        " * Values are projected unchanged. A new number belongs in the token file first.",
        " *",
        " * Light is `:root`; dark is the same roles under `[data-fs-theme=\"dark\"]`, which the",
        " * application sets on the document element. Only colour is themed - see ADR-0031 D5.",
        " */",
        ":root {",
    ]
    labels = {css: label.title() for (_s, css, _u), label in zip(GROUPS, (
        "Color", "Spacing", "Radius", "Border", "Layout", "Layer", "Table density", "Icon", "Motion",
        "Focus", "Shadow",
    ))}
    labels["font"] = TYPOGRAPHY_LABEL

    current_group = None
    for group, name, value in rows:
        if group != current_group:
            if current_group is not None:
                lines.append("")
            lines.append(f"  /* {labels[group]} */")
            current_group = group
        lines.append(f"  {name}: {resolve(value, known, 'light')};")
    lines.append("}")
    lines.append("")
    lines.append(f"{DARK_SELECTOR} {{")
    current_group = None
    for group, name, value in dark_rows:
        if group != current_group:
            if current_group is not None:
                lines.append("")
            lines.append(f"  /* {labels[group]} */")
            current_group = group
        lines.append(f"  {name}: {resolve(value, known, 'dark')};")
    lines.append("}")
    lines.append("")
    return "\n".join(lines)


# --- ADR-0031 D6: the two palettes are measured, not admired --------------------------------

# (foreground role, background role, floor, what it is on screen)
#
# The floors are WCAG 2.2 AA: 4.5:1 for text (1.4.3) and 3:1 for a visual that identifies a
# component or its state (1.4.11 - the focus ring and the selection rule). Roles AA explicitly
# excepts are reported by `print_contrast` instead: a disabled label, a layout hairline, and a
# diff row's tint, which is one of three encodings the design requires together with a label and
# an icon and so carries no meaning on its own.
CONTRAST_ENFORCED = (
    ("text.primary", "bg.canvas", 4.5, "body text on the page"),
    ("text.primary", "bg.surface", 4.5, "body text on a panel"),
    ("text.primary", "bg.subtle", 4.5, "body text on a subtle row"),
    ("text.secondary", "bg.canvas", 4.5, "secondary text on the page"),
    ("text.secondary", "bg.surface", 4.5, "secondary text on a panel"),
    ("text.on-accent", "accent.primary", 4.5, "primary button label"),
    ("text.on-accent", "accent.hover", 4.5, "hovered button label"),
    ("text.on-accent", "accent.pressed", 4.5, "pressed button label"),
    ("status.pass", "bg.surface", 4.5, "PASS text and icon"),
    ("status.review", "bg.surface", 4.5, "REVIEW text and icon"),
    ("status.review.strong", "bg.surface", 4.5, "strong REVIEW text"),
    ("status.block", "bg.surface", 4.5, "BLOCK text and icon"),
    ("status.unknown", "bg.surface", 4.5, "UNKNOWN text and icon"),
    ("accent.primary", "bg.surface", 3.0, "focus ring and selection rule on a panel"),
    ("accent.primary", "bg.canvas", 3.0, "focus ring and selection rule on the page"),
    ("diff.added", "diff.added.bg", 4.5, "added figure on its own ground"),
    ("diff.removed", "diff.removed.bg", 4.5, "removed figure on its own ground"),
    ("diff.changed", "diff.changed.bg", 4.5, "changed figure on its own ground"),
)

CONTRAST_REPORTED = (
    ("text.disabled", "bg.surface", "disabled label (1.4.3 excepts inactive components)"),
    ("border.default", "bg.canvas", "hairline on the page (layout separator)"),
    ("border.default", "bg.surface", "hairline on a panel (layout separator)"),
    ("diff.added.bg", "bg.surface", "added row tint against the surface"),
    ("diff.removed.bg", "bg.surface", "removed row tint against the surface"),
    ("diff.changed.bg", "bg.surface", "changed row tint against the surface"),
    ("chart.blue.1", "bg.surface", "first chart step against a panel"),
    ("chart.blue.5", "bg.surface", "last chart step against a panel"),
)

# (theme, foreground, background, floor, why it is carried rather than fixed here)
#
# An exemption is a named, measured, printed thing, not a lowered floor. Each one is checked in
# both directions: the pair must still fail (a fix that landed makes the stale exemption an error),
# and the reason must still be true of the tree. Clearing one is its own change, with its own
# review, because the reason here is a frozen artefact and not a preference.
#
# Empty since tokens 0.4.0: ADR-0034 D3 moved light diff.removed to #C7372C (4.5592:1), which
# retired ADR-0031 D6's only exemption and regenerated the two report goldens that print it.
CONTRAST_EXEMPT: tuple[tuple[str, str, str, float, str], ...] = ()


def _channel(byte: float) -> float:
    return byte / 12.92 if byte <= 0.03928 else ((byte + 0.055) / 1.055) ** 2.4


def _luminance(hexstr: str) -> float:
    digits = hexstr.lstrip("#")
    r, g, b = (int(digits[i:i + 2], 16) / 255 for i in (0, 2, 4))
    return 0.2126 * _channel(r) + 0.7152 * _channel(g) + 0.0722 * _channel(b)


def contrast(first: str, second: str) -> float:
    """WCAG 2.2 relative-luminance ratio, rounded nowhere: the arithmetic is the evidence."""
    one, two = _luminance(first), _luminance(second)
    high, low = max(one, two), min(one, two)
    return (high + 0.05) / (low + 0.05)


def check_palettes(tokens: dict) -> list[str]:
    """Every problem that makes a palette unusable as an authority. Empty means ship-able."""
    problems: list[str] = []
    colors = tokens.get("color", {})

    if set(colors) != set(THEMES):
        problems.append(
            f"color holds {sorted(colors)}, expected exactly {list(THEMES)} - "
            "a palette in the file that nothing emits is a second place for a number to drift"
        )

    light_roles = set(colors.get("light", {}))
    for theme in THEMES:
        roles = set(colors.get(theme, {}))
        if roles != light_roles:
            missing = sorted(light_roles - roles)
            extra = sorted(roles - light_roles)
            problems.append(
                f"color.{theme} is not the same role set as color.light: "
                f"missing={missing} extra={extra}"
            )

    for theme in THEMES:
        palette = colors.get(theme, {})
        for role in ("bg.canvas", "bg.surface", "bg.subtle"):
            value = palette.get(role, "")
            if value and not value.startswith("#"):
                problems.append(f"color.{theme}.{role} is not a hex colour: {value!r}")
        for first, second, floor, label in CONTRAST_ENFORCED:
            try:
                ratio = contrast(palette[first], palette[second])
            except KeyError as error:
                problems.append(f"color.{theme} has no role {error} for the pair {label}")
                continue
            exempt = next(
                (
                    entry
                    for entry in CONTRAST_EXEMPT
                    if entry[0] == theme and entry[1] == first and entry[2] == second
                ),
                None,
            )
            if exempt is not None:
                if ratio >= floor:
                    problems.append(
                        f"{theme}: {label} now measures {ratio:.4f}:1 and passes, so the "
                        "exemption for it is stale - delete it rather than keep a hole with no "
                        "leak in it"
                    )
                continue
            if ratio < floor:
                problems.append(
                    f"ADR-0031 D6 contrast fails in {theme}: {label} is {first} on {second} "
                    f"= {ratio:.4f}:1, under the {floor}:1 floor"
                )

    for theme, first, second, _floor, _why in CONTRAST_EXEMPT:
        if (first, second) not in [(entry[0], entry[1]) for entry in CONTRAST_ENFORCED]:
            problems.append(
                f"the exemption for {theme} {first} on {second} names no enforced pair"
            )
    return problems


def print_contrast(tokens: dict) -> None:
    """The sample table, so the claim is reproducible from the token file rather than from a photo."""
    colors = tokens["color"]
    for theme in THEMES:
        palette = colors[theme]
        print(f"== {theme} ==")
        for first, second, floor, label in CONTRAST_ENFORCED:
            ratio = contrast(palette[first], palette[second])
            exempt = any(
                entry[0] == theme and entry[1] == first and entry[2] == second
                for entry in CONTRAST_EXEMPT
            )
            mark = "EXEMPT" if exempt else ("ok" if ratio >= floor else "FAIL")
            print(
                f"  {mark:>6} {ratio:7.4f} >= {floor:<4} {label}: "
                f"{first} {palette[first]} on {second} {palette[second]}"
            )
        for first, second, label in CONTRAST_REPORTED:
            print(
                f"        {contrast(palette[first], palette[second]):7.4f}        {label}: "
                f"{first} on {second}"
            )
    if CONTRAST_EXEMPT:
        print("-- carried exemptions, with the reason --")
        for theme, first, second, floor, why in CONTRAST_EXEMPT:
            print(f"  {theme}: {first} on {second} (floor {floor}:1) - {why}")


UI_STYLES = ROOT / "apps" / "desktop" / "ui" / "src"
# Anything but a token reference (or `auto`) is refused: a bare number, a negative one, or a `calc()`.
# Property names are case-insensitive in CSS, and a comment may mention a number freely (R7 review S3).
BARE_Z_INDEX = re.compile(r"z-index\s*:(?!\s*(?:var\(--fs-layer-|auto\b))", re.IGNORECASE)
CSS_COMMENT = re.compile(r"/\*.*?\*/", re.DOTALL)


def bare_stacking_orders() -> list[str]:
    """Stylesheets that stack with a bare number instead of a `--fs-layer-*` token (ADR-0034 A1).

    The layer scale exists so the order of overlays is one decision, not eight numbers that happen to agree.
    """
    return sorted(
        str(path.relative_to(ROOT)).replace("\\", "/")
        for path in UI_STYLES.rglob("*.css")
        if path != TARGET
        and BARE_Z_INDEX.search(CSS_COMMENT.sub("", path.read_text(encoding="utf-8")))
    )


def main(argv: list[str]) -> int:
    check = "--check" in argv[1:]
    raw = SOURCE.read_bytes()
    tokens = json.loads(raw.decode("utf-8"))

    problems = check_palettes(tokens)
    if problems:
        for line in problems:
            print(line, file=sys.stderr)
        return 1
    if "--contrast" in argv[1:]:
        print_contrast(tokens)
        return 0

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
        bare = bare_stacking_orders()
        if bare:
            print(
                "z-index must use a --fs-layer-* token (tokens 0.4.1, ADR-0034 A1): " + ", ".join(bare),
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
