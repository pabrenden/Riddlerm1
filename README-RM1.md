# Riddle for reMarkable 1

This fork ports the original Paper Pro Riddle app to reMarkable 1 and is aimed at stock 3.27.x.

## What changed

- 1404 x 1872 RM1 geometry, with native framebuffer stride detected through FBIOGET_FSCREENINFO.
- Native `/dev/fb0` RGB565 mapping and RM1 `MXCFB_SEND_UPDATE` partial/full refreshes. No Quill or `libqsgepaper.so` is required.
- ARMv7 hard-float build target (`armv7-unknown-linux-gnueabihf`).
- Linux input events are ABI-correct on 32-bit ARM by using a real `struct input_event` layout rather than fixed 24-byte records.
- Wacom axis ranges, touch ranges and pressure range are queried dynamically with `EVIOCGABS`.
- RM1 Wacom portrait transform: Rot270. RM1 multitouch portrait transform: Rot180.
- Input device discovery recognizes Wacom/digitizer, cyttsp/touch and gpio/power-key names instead of Paper Pro-only names.
- Full-screen launcher stops xochitl before taking over `/dev/fb0`, and restarts xochitl on exit.

## Build

Use an ARMv7 reMarkable/Toltec toolchain. With a compatible cross compiler in PATH:

```sh
./build-rm1.sh
```

The output is `dist/rm1/riddle`.

Toltec's toolchain is suitable for ARMv7 reMarkable builds. A 4.x toolchain matches modern 3.x reMarkable releases.

## Configure the oracle

Copy:

```sh
cp oracle.env.example oracle.env
```

and put your API endpoint/key/model in `oracle.env`.

## Install over USB SSH

```sh
./install-rm1.sh root@10.11.99.1
```

For Wi-Fi SSH, replace the host with the tablet's IP.

## Run on the tablet

```sh
/home/root/riddle/riddle-rm1.sh
```

Exit with the five-finger gesture or Ctrl-C over SSH. The launcher restores xochitl when Riddle exits.

Emergency recovery if the app is killed unexpectedly:

```sh
systemctl start xochitl
```

## Diagnostics on the tablet

```sh
cat /sys/devices/soc0/machine
uname -m
for f in /sys/class/input/event*/device/name; do echo "$f: $(cat "$f")"; done
fbset 2>/dev/null || true
```

Expected machine/architecture for RM1 is `reMarkable 1.0` (or the old prototype identifier) and `armv7l`.
