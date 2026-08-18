#!/usr/bin/env bash
# Build Riddle for reMarkable 1 (ARMv7 hard-float).
set -euo pipefail
cd "$(dirname "$0")"
TARGET=armv7-unknown-linux-gnueabihf

if command -v arm-remarkable-linux-gnueabihf-gcc >/dev/null 2>&1; then
  export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-remarkable-linux-gnueabihf-gcc
elif command -v arm-linux-gnueabihf-gcc >/dev/null 2>&1; then
  export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
else
  echo "No ARMv7 cross linker found." >&2
  echo "Recommended: run this inside toltec/toolchain:4 or install an arm-linux-gnueabihf GCC toolchain." >&2
  exit 2
fi

rustup target add "$TARGET" >/dev/null 2>&1 || true
cargo build --release --target "$TARGET"
mkdir -p dist/rm1
install -m755 "target/$TARGET/release/riddle" dist/rm1/riddle
install -m755 scripts/riddle-rm1.sh dist/rm1/riddle-rm1.sh
install -m644 oracle.env.example dist/rm1/oracle.env.example
[ -f oracle.env ] && install -m600 oracle.env dist/rm1/oracle.env || true
printf '\nBuilt: dist/rm1/riddle\n'
file dist/rm1/riddle || true
