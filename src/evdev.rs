//! Small evdev helpers that work on both 32-bit RM1 and 64-bit hosts.
//! Never hard-code Linux `input_event` to 24 bytes: on ARMv7 it is 16 bytes.

use std::io;
use std::os::fd::RawFd;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InputEvent {
    pub time: libc::timeval,
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InputAbsInfo {
    pub value: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub fuzz: i32,
    pub flat: i32,
    pub resolution: i32,
}

// Linux _IOC encoding (asm-generic/ioctl.h).
const IOC_NRBITS: u32 = 8;
const IOC_TYPEBITS: u32 = 8;
const IOC_SIZEBITS: u32 = 14;
const IOC_NRSHIFT: u32 = 0;
const IOC_TYPESHIFT: u32 = IOC_NRSHIFT + IOC_NRBITS;
const IOC_SIZESHIFT: u32 = IOC_TYPESHIFT + IOC_TYPEBITS;
const IOC_DIRSHIFT: u32 = IOC_SIZESHIFT + IOC_SIZEBITS;
const IOC_READ: u32 = 2;

fn ior(ty: u8, nr: u8, size: usize) -> libc::c_ulong {
    ((IOC_READ << IOC_DIRSHIFT)
        | ((ty as u32) << IOC_TYPESHIFT)
        | ((nr as u32) << IOC_NRSHIFT)
        | ((size as u32) << IOC_SIZESHIFT)) as libc::c_ulong
}

/// EVIOCGABS(abs_code)
pub fn abs_info(fd: RawFd, code: u16) -> io::Result<InputAbsInfo> {
    let mut info = InputAbsInfo::default();
    let request = ior(b'E', (0x40u16 + code) as u8, std::mem::size_of::<InputAbsInfo>());
    let rc = unsafe { libc::ioctl(fd, request, &mut info) };
    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(info)
    }
}

/// Drain whole input_event records from a non-blocking evdev fd.
pub fn read_events(fd: RawFd, out: &mut Vec<InputEvent>) {
    const N: usize = 64;
    let mut events: [InputEvent; N] = unsafe { std::mem::zeroed() };
    loop {
        let bytes = unsafe {
            libc::read(
                fd,
                events.as_mut_ptr() as *mut libc::c_void,
                std::mem::size_of_val(&events),
            )
        };
        if bytes <= 0 {
            break;
        }
        let count = bytes as usize / std::mem::size_of::<InputEvent>();
        out.extend_from_slice(&events[..count]);
        if count < N {
            break;
        }
    }
}

pub fn event_name(i: usize) -> String {
    std::fs::read_to_string(format!("/sys/class/input/event{i}/device/name"))
        .unwrap_or_default()
        .trim()
        .to_string()
}
