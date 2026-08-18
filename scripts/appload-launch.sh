#!/bin/sh
# RM1 AppLoad entry point. Detach from xochitl before the real launcher stops it.
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
systemctl is-active --quiet riddle-rm1 && exit 0
systemd-run --unit=riddle-rm1 --collect \
  --property="ExecStopPost=-/bin/systemctl start xochitl" \
  /bin/sh "$HERE/riddle-rm1.sh" >/dev/null 2>&1 \
  || systemd-run --unit=riddle-rm1 --collect /bin/sh "$HERE/riddle-rm1.sh" >/dev/null 2>&1
exit 0
