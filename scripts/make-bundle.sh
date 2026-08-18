#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
BIN=target/armv7-unknown-linux-gnueabihf/release/riddle
[ -f "$BIN" ] || { echo "build first: ./build-rm1.sh" >&2; exit 1; }
rm -rf dist/rm1
mkdir -p dist/rm1
install -m755 "$BIN" dist/rm1/riddle
install -m755 scripts/riddle-rm1.sh scripts/appload-launch.sh dist/rm1/
install -m644 oracle.env.example external.manifest.json icon.png settings.schema.json dist/rm1/
[ -f oracle.env ] && install -m600 oracle.env dist/rm1/ || true
