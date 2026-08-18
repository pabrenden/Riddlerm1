#!/bin/sh
# Full-screen launcher for reMarkable 1.
set -u
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

restore_xochitl() {
  systemctl start xochitl >/dev/null 2>&1 || true
}
trap restore_xochitl EXIT INT TERM HUP

if [ -f "$HERE/oracle.env" ]; then
  set -a
  . "$HERE/oracle.env"
  set +a
fi

# Riddle owns /dev/fb0 and the pen/touch devices while open.
systemctl stop xochitl
sleep 1
cd "$HERE"
HOME=/home/root exec "$HERE/riddle"
