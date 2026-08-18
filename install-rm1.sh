#!/usr/bin/env bash
# Run this on your computer. Usage: ./install-rm1.sh [root@10.11.99.1]
set -euo pipefail
cd "$(dirname "$0")"
HOST=${1:-root@10.11.99.1}
DIR=/home/root/riddle
BIN=dist/rm1/riddle
[ -f "$BIN" ] || { echo "Missing $BIN. Run ./build-rm1.sh first." >&2; exit 1; }
ssh "$HOST" "mkdir -p '$DIR'"
scp "$BIN" scripts/riddle-rm1.sh oracle.env.example "$HOST:$DIR/"
if [ -f oracle.env ]; then scp oracle.env "$HOST:$DIR/oracle.env"; fi
ssh "$HOST" "chmod +x '$DIR/riddle' '$DIR/riddle-rm1.sh'; echo 'Installed in $DIR'; echo 'Start with: $DIR/riddle-rm1.sh'"
