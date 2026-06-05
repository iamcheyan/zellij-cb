#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

TARGET="wasm32-wasip1"
OUT_DIR="target/$TARGET/release"
WASM="zellij-cb.wasm"
DEST="$HOME/.config/zellij/plugins/$WASM"

# Check target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
  echo "Installing target $TARGET..."
  rustup target add "$TARGET"
fi

echo "Building $WASM..."
cargo build --target "$TARGET" --release

cp "$OUT_DIR/$WASM" "$DEST"
echo "Installed: $DEST"
