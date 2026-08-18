//! Raw reMarkable pen input. RM1 uses the Wacom I2C digitizer.

use std::io;
use std::os::fd::RawFd;
use crate::evdev;
use crate::fb::{SCREEN_H, SCREEN_W};

pub const MAX_PRESSURE: i32 = 4095;

const EV_SYN: u16 = 0;
const EV_KEY: u16 = 1;
const EV_ABS: u16 = 3;
const SYN_REPORT: u16 = 0;
const ABS_X: u16 = 0;
const ABS_Y: u16 = 1;
const ABS_PRESSURE: u16 = 24;
const BTN_TOOL_PEN: u16 = 320;
const BTN_TOOL_RUBBER: u16 = 321;
const BTN_TOUCH: u16 = 330;
const EVIOCGRAB: libc::c_ulong = 0x40044590;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool { Pen, Eraser }

#[derive(Debug, Clone, Copy)]
pub struct PenSample {
    pub x: i32,
    pub y: i32,
    pub pressure: i32,
    pub tool: Tool,
    pub touching: bool,
    pub proximity: bool,
}

pub struct PenDevice {
    fd: RawFd,
    raw_x: i32,
    raw_y: i32,
    pressure: i32,
    pressure_max: i32,
    x_min: i32,
    x_max: i32,
    y_min: i32,
    y_max: i32,
    tool: Tool,
    touching: bool,
    pen_in_range: bool,
    rubber_in_range: bool,
    proximity: bool,
    dirty: bool,
}

impl PenDevice {
    pub fn open() -> io::Result<Self> {
        let path = find_pen_device()?;
        let cpath = std::ffi::CString::new(path.clone()).unwrap();
        let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC) };
        if fd < 0 { return Err(io::Error::last_os_error()); }

        let x = evdev::abs_info(fd, ABS_X).unwrap_or(evdev::InputAbsInfo { minimum: 0, maximum: 20967, ..Default::default() });
        let y = evdev::abs_info(fd, ABS_Y).unwrap_or(evdev::InputAbsInfo { minimum: 0, maximum: 15725, ..Default::default() });
        let p = evdev::abs_info(fd, ABS_PRESSURE).unwrap_or(evdev::InputAbsInfo { minimum: 0, maximum: 4095, ..Default::default() });
        let grab = unsafe { libc::ioctl(fd, EVIOCGRAB, 1i32) };
        if grab != 0 {
            eprintln!("riddle: warning: pen EVIOCGRAB failed ({})", io::Error::last_os_error());
        }
        eprintln!("riddle: pen {path}: X {}..{}, Y {}..{}, pressure 0..{}", x.minimum, x.maximum, y.minimum, y.maximum, p.maximum);
        Ok(Self {
            fd, raw_x: x.minimum, raw_y: y.minimum, pressure: 0,
            pressure_max: p.maximum.max(1), x_min: x.minimum, x_max: x.maximum,
            y_min: y.minimum, y_max: y.maximum, tool: Tool::Pen, touching: false,
            pen_in_range: false, rubber_in_range: false, proximity: false, dirty: false,
        })
    }

    pub fn raw_fd(&self) -> RawFd { self.fd }

    fn screen_xy(&self) -> (i32, i32) {
        // RM1 Wacom placement is Rot270: portrait X comes from raw Y;
        // portrait Y is the inverse of raw X.
        let rw = (self.y_max - self.y_min).max(1);
        let rh = (self.x_max - self.x_min).max(1);
        let px = (self.raw_y - self.y_min).clamp(0, rw);
        let py = (self.x_max - self.raw_x).clamp(0, rh);
        (
            px * (SCREEN_W as i32 - 1) / rw,
            py * (SCREEN_H as i32 - 1) / rh,
        )
    }

    pub fn drain(&mut self) -> Vec<PenSample> {
        let mut out = Vec::new();
        let mut events = Vec::with_capacity(64);
        evdev::read_events(self.fd, &mut events);
        for ev in events {
            match (ev.type_, ev.code) {
                (EV_ABS, ABS_X) => { self.raw_x = ev.value; self.dirty = true; }
                (EV_ABS, ABS_Y) => { self.raw_y = ev.value; self.dirty = true; }
                (EV_ABS, ABS_PRESSURE) => { self.pressure = ev.value; self.dirty = true; }
                (EV_KEY, BTN_TOOL_PEN) => {
                    self.pen_in_range = ev.value != 0;
                    if self.pen_in_range { self.tool = Tool::Pen; }
                    self.proximity = self.pen_in_range || self.rubber_in_range; self.dirty = true;
                }
                (EV_KEY, BTN_TOOL_RUBBER) => {
                    self.rubber_in_range = ev.value != 0;
                    if self.rubber_in_range { self.tool = Tool::Eraser; }
                    self.proximity = self.pen_in_range || self.rubber_in_range; self.dirty = true;
                }
                (EV_KEY, BTN_TOUCH) => { self.touching = ev.value != 0; self.dirty = true; }
                (EV_SYN, SYN_REPORT) if self.dirty => {
                    self.dirty = false;
                    let (x, y) = self.screen_xy();
                    out.push(PenSample {
                        x, y,
                        pressure: self.pressure.clamp(0, self.pressure_max) * MAX_PRESSURE / self.pressure_max,
                        tool: self.tool, touching: self.touching, proximity: self.proximity,
                    });
                }
                _ => {}
            }
        }
        out
    }
}

impl Drop for PenDevice {
    fn drop(&mut self) { unsafe { libc::ioctl(self.fd, EVIOCGRAB, 0i32); libc::close(self.fd); } }
}

fn find_pen_device() -> io::Result<String> {
    for i in 0..32 {
        let name = evdev::event_name(i).to_lowercase();
        if name.contains("wacom") || name.contains("digitizer") || name.contains("marker") || name.contains("pen") {
            return Ok(format!("/dev/input/event{i}"));
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "no pen/digitizer input device found"))
}
