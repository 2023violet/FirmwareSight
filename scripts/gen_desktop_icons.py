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

`--check` compares decoded pixels, not file bytes. A PNG or ICO is a container whose contents
depend on the encoder that wrote it: zlib version, filter heuristics and `optimize` all change the
bytes while the picture stays the same. Measured on this mark, across `optimize=True`,
`optimize=False` and `compress_level=1`, every size produced a different SHA-256 while the decoded
RGBA content produced exactly one value per size. Asserting byte equality therefore asks every
platform to reproduce one specific encoder's internal decisions, which is why CI Run #1 reported
`128x128.png`, `128x128@2x.png` and `icon.ico` as stale on Linux while the pixels were identical -
and why the two files it did not flag were only accidentally in agreement.

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


def read_frames(data: bytes) -> dict[tuple[int, int], bytes] | None:
    """Decode an image file into raw RGBA frames keyed by pixel size.

    Returns None when the bytes are not a readable image. An ICO is a container of frames, so it
    decodes to several entries; every other format here decodes to one.
    """
    from io import BytesIO

    try:
        image = Image.open(BytesIO(data))
    except Exception as exc:  # unreadable asset is a drift failure, not a traceback
        print(f"  cannot decode an icon file ({exc})", file=sys.stderr)
        return None

    frames: dict[tuple[int, int], bytes] = {}
    ico = getattr(image, "ico", None)
    if ico is not None:
        for size in sorted(ico.sizes()):
            frames[size] = ico.getimage(size).convert("RGBA").tobytes()
    else:
        frames[(image.width, image.height)] = image.convert("RGBA").tobytes()
    return frames


def drift_reasons(committed: bytes, generated: bytes, required: set[tuple[int, int]]) -> list[str]:
    """Semantic differences between a committed asset and the one this build renders."""
    left = read_frames(committed)
    right = read_frames(generated)
    if left is None or right is None:
        return ["undecodable"]

    reasons: list[str] = []
    missing = required - set(right)
    if missing:
        reasons.append("generated set is missing frame(s): " + frame_names(missing))
    for size in sorted(set(right) - set(left)):
        reasons.append(f"missing frame {frame_names({size})}")
    for size in sorted(set(left) - set(right)):
        reasons.append(f"unexpected frame {frame_names({size})}")
    for size in sorted(set(left) & set(right)):
        if left[size] != right[size]:
            reasons.append(f"pixel content differs at {size[0]}x{size[1]}")
    return reasons


def frame_names(sizes: set[tuple[int, int]]) -> str:
    return ", ".join(f"{w}x{h}" for w, h in sorted(sizes)) or "none"


def png_size(data: bytes) -> tuple[int, int]:
    from io import BytesIO

    with Image.open(BytesIO(data)) as image:
        return image.size


def main(argv: list[str]) -> int:
    check = "--check" in argv[1:]
    files = build()
    required_frames = {(size, size) for size in SIZES}

    if check:
        problems: list[str] = []
        for name, data in sorted(files.items()):
            path = OUT / name
            if not path.exists():
                problems.append(f"{name}: missing")
                continue
            committed = path.read_bytes()
            # An ICO must carry every frame the shell asks for; a PNG is identified by its size.
            required = required_frames if name.endswith(".ico") else {png_size(data)}
            reasons = drift_reasons(committed, data, required)
            if reasons:
                problems.append(f"{name}: " + "; ".join(reasons))
        if problems:
            print(
                "desktop icons are missing or drifted from this build:\n  "
                + "\n  ".join(problems)
                + "\nRun: python scripts/gen_desktop_icons.py",
                file=sys.stderr,
            )
            return 1
        print("desktop icons are current (pixel-identical to this build).")
        return 0

    OUT.mkdir(parents=True, exist_ok=True)
    for name, data in files.items():
        (OUT / name).write_bytes(data)
        digest = hashlib.sha256(data).hexdigest()[:12]
        print(f"wrote {OUT.relative_to(ROOT) / name} ({len(data)} bytes, sha256 {digest}...)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
