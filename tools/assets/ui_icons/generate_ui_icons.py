# 将 Ant Design outlined / 自绘 SVG 栅格化为 UI 白图标 PNG
"""把 tools/assets/ui_icons/svg/*.svg 渲成 assets/ui/icons/*.png。

来源：
- edit / delete / close：Ant Design Icons（MIT）outlined
- crosshair：自绘准心（风格对齐 outlined）
"""

from __future__ import annotations

import ctypes.util
import sys
from pathlib import Path

_TOOLS = Path(__file__).resolve().parents[1]  # tools/assets
sys.path.insert(0, str(_TOOLS))
from common.paths import ASSETS, REPO_ROOT  # noqa: E402

SVG_DIR = Path(__file__).resolve().parent / "svg"
OUT_DIR = ASSETS / "ui" / "icons"
SIZE = 128

# Homebrew / 常见 libcairo 位置（cairocffi 默认找不到时用）
_CAIRO_CANDIDATES = (
    "/opt/homebrew/lib/libcairo.2.dylib",
    "/usr/local/lib/libcairo.2.dylib",
    "/usr/lib/x86_64-linux-gnu/libcairo.so.2",
)


def _patch_cairo_find_library() -> None:
    orig = ctypes.util.find_library

    def find_library(name: str):
        if "cairo" in (name or ""):
            for path in _CAIRO_CANDIDATES:
                if Path(path).exists():
                    return path
        return orig(name)

    ctypes.util.find_library = find_library  # type: ignore[assignment]


def main() -> None:
    _patch_cairo_find_library()
    import cairosvg

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    svgs = sorted(SVG_DIR.glob("*.svg"))
    if not svgs:
        raise SystemExit(f"no svg in {SVG_DIR}")

    for svg in svgs:
        out = OUT_DIR / f"{svg.stem}.png"
        cairosvg.svg2png(
            url=str(svg),
            write_to=str(out),
            output_width=SIZE,
            output_height=SIZE,
            background_color="rgba(0,0,0,0)",
        )
        print(f"wrote {out.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    main()
