#!/usr/bin/env bash
# 从 assets/app_icon.png 生成 Android mipmap /（可选）macOS .icns
# 用法:
#   ./scripts/generate_app_icons.sh           # 只写 Android res/mipmap-*
#   ./scripts/generate_app_icons.sh --icns OUT.icns
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/assets/app_icon.png"
ANDROID_RES="$ROOT/mobile/android/app/src/main/res"

ICNS_OUT=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --icns)
      ICNS_OUT="${2:?missing path after --icns}"
      shift 2
      ;;
    -h | --help)
      echo "Usage: $0 [--icns OUT.icns]"
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 2
      ;;
  esac
done

if [[ ! -f "$SRC" ]]; then
  echo "missing $SRC" >&2
  echo "bake: ./scripts/bake_factory_icons.sh --only conveyor --size 512 --output app_icon.png" >&2
  echo "then: mv assets/factory_blocks/conveyor/app_icon.png assets/app_icon.png" >&2
  exit 1
fi

python3 - "$SRC" "$ANDROID_RES" <<'PY'
from pathlib import Path
import sys
from PIL import Image

src = Path(sys.argv[1])
res = Path(sys.argv[2])
img = Image.open(src).convert("RGBA")

densities = {
    "mipmap-mdpi": 48,
    "mipmap-hdpi": 72,
    "mipmap-xhdpi": 96,
    "mipmap-xxhdpi": 144,
    "mipmap-xxxhdpi": 192,
}
for folder, size in densities.items():
    out_dir = res / folder
    out_dir.mkdir(parents=True, exist_ok=True)
    scaled = img.resize((size, size), Image.Resampling.LANCZOS)
    scaled.save(out_dir / "ic_launcher.png", optimize=True)
    scaled.save(out_dir / "ic_launcher_round.png", optimize=True)
    print(f"wrote {out_dir}/ic_launcher{{,_round}}.png ({size}x{size})")
PY

if [[ -n "$ICNS_OUT" ]]; then
  if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "icns generation requires macOS iconutil" >&2
    exit 1
  fi
  TMP="$(mktemp -d)"
  ICONSET="$TMP/AppIcon.iconset"
  mkdir -p "$ICONSET"
  for s in 16 32 128 256 512; do
    sips -z "$s" "$s" "$SRC" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
    s2=$((s * 2))
    sips -z "$s2" "$s2" "$SRC" --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
  done
  mkdir -p "$(dirname "$ICNS_OUT")"
  iconutil -c icns "$ICONSET" -o "$ICNS_OUT"
  rm -rf "$TMP"
  echo "wrote $ICNS_OUT"
fi
