#!/usr/bin/env bash
# 用普通 Free 存档可视化编辑启动界面橱窗，并显式发布方块数据。
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ACTION="${1:-}"
SCENE_ID="${2:-}"

usage() {
  echo "Usage:"
  echo "  scripts/showcase_author.sh edit <transport|conversion|welding|pusher|lifter>"
  echo "  scripts/showcase_author.sh publish <transport|conversion|welding|pusher|lifter>"
}

if [[ "$ACTION" != "edit" && "$ACTION" != "publish" ]]; then
  usage
  exit 2
fi

ASSET_DIR="$ROOT_DIR/assets/showcases/$SCENE_ID"
if [[ -z "$SCENE_ID" || ! -f "$ASSET_DIR/meta.json" || ! -f "$ASSET_DIR/blocks.bin" ]]; then
  echo "error: unknown showcase '$SCENE_ID'" >&2
  usage
  exit 2
fi

SAVE_ID="showcase_author_$SCENE_ID"
SAVE_DIR="$ROOT_DIR/saves/$SAVE_ID"

if [[ "$ACTION" == "edit" ]]; then
  if [[ ! -d "$SAVE_DIR" ]]; then
    mkdir -p "$SAVE_DIR"
    cp "$ASSET_DIR/meta.json" "$SAVE_DIR/meta.json"
    cp "$ASSET_DIR/blocks.bin" "$SAVE_DIR/blocks.bin"
    echo "Created editable Free save: $SAVE_ID"
  else
    echo "Reusing editable Free save without overwriting it: $SAVE_ID"
  fi
  cd "$ROOT_DIR"
  exec cargo run -- --debug-http --load-save="$SAVE_ID"
fi

if [[ ! -f "$SAVE_DIR/blocks.bin" ]]; then
  echo "error: editable save '$SAVE_ID' does not exist; run edit first" >&2
  exit 1
fi
cp "$SAVE_DIR/blocks.bin" "$ASSET_DIR/blocks.bin"
echo "Published $SAVE_DIR/blocks.bin"
echo "       -> $ASSET_DIR/blocks.bin"
echo "Restart the game to preview the newly embedded showcase."
