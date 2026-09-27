#!/usr/bin/env python3
"""Generate the P0 desktop icon set under apps/desktop/src-tauri/icons/.

tauri-build fails on Windows without `icons/icon.ico`, so the shell needs a real icon file
before it can compile at all. This is a placeholder mark, not a frozen brand asset: it is drawn
from `assets/design-tokens.json` (accent + surface) with flat fills only, so it cannot smuggle
in a gradient, glow or purple that DESIGN.md 9 forbids. A brand-approved mark replaces it when
02_BRAND freezes one.

Usage:
    python scripts/gen_desktop_icons.py            # write the set
    python scripts/gen_desktop_icons.py --check    # verify it exists and is current (CI)

Deterministic: fixed geometry, no text rendering, no timestamps in the PNG bytes.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
TOKENS = ROOT / "assets" / "design-tokens.json"
OUT = ROOT / "apps" / "desktop" / "src-tauri" / "icons"

SIZES = (16, 24, 32, 48, 64, 128, 256)


def palette() -> tuple[str, str]:
    tokens = json.loads(TOKENS.read_text(encoding="utf-8"))
    color = tokens["color"]["light"]
    return color["accent.primary"], color["bg.surface"]


def mark(size: int, accent: str, surface: str) -> Image.Image:
    """A flat baseline with three rising bars: an instrument reading, not a logo claim."""
    img = Image.new("RGBA", (size, size), accent)
    draw = ImageDraw.Draw(img)

    def px(fraction: float) -> int:
        return round(size * fraction)

    unit = max(1, px(0.055))
    baseline_y = size - px(0.22)
    bars = ((0.24, 0.30), (0.45, 0.50), (0.66, 0.20))

    for left, height in bars:
        top = baseline_y - px(height)
        draw.rectangle(
            [px(left), top, px(left) + unit * 2, baseline_y],
            fill=surface,
        )
    draw.rectangle(
        [px(0.18), baseline_y, size - px(0.18), baseline_y + unit],
        fill=surface,
    )
    return img


def build() -> dict[str, bytes]:
    accent, surface = palette()
    files: dict[str, bytes] = {}

    renders: dict[int, Image.Image] = {size: mark(size, accent, surface) for size in SIZES}
    renders[512] = mark(512, accent, surface)

    for size in SIZES:
        if size in (32, 128):
            files[f"{size}x{size}.png"] = to_png(renders[size])
    files["128x128@2x.png"] = to_png(renders[256])
    files["icon.png"] = to_png(renders[512])
    files["icon.ico"] = to_ico([renders[size] for size in (16, 24, 32, 48, 64, 128, 256)])

    return files


def to_png(image: Image.Image) -> bytes:
    from io import BytesIO

    buffer = BytesIO()
    image.save(buffer, format="PNG", optimize=True)
    return buffer.getvalue()


def to_ico(images: list[Image.Image]) -> bytes:
    from io import BytesIO

    buffer = BytesIO()
    images[-1].save(
        buffer,
        format="ICO",
        bitmap_format="rgba",
        sizes=tuple((image.width, image.height) for image in images),
        append_images=images[1:],
    )
    return buffer.getvalue()


def main(argv: list[str]) -> int:
    check = "--check" in argv[1:]
    files = build()

    missing = [name for name, data in files.items() if not (OUT / name).exists()]
    stale = [
        name
        for name, data in files.items()
        if (OUT / name).exists() and (OUT / name).read_bytes() != data
    ]
    if check:
        if missing or stale:
            print(
                "desktop icons are missing or stale:\n  "
                + "\n  ".join(missing + stale)
                + "\nRun: python scripts/gen_desktop_icons.py",
                file=sys.stderr,
            )
            return 1
        print("desktop icons are current.")
        return 0

    OUT.mkdir(parents=True, exist_ok=True)
    for name, data in files.items():
        (OUT / name).write_bytes(data)
        digest = hashlib.sha256(data).hexdigest()[:12]
        print(f"wrote {OUT.relative_to(ROOT) / name} ({len(data)} bytes, sha256 {digest}...)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
