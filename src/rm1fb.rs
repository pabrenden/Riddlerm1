//! Native reMarkable 1 framebuffer backend.
//! Maps /dev/fb0 (RGB565) and refreshes rectangles with MXCFB_SEND_UPDATE.

use std::ffi::CString;
use std::io;
use std::os::fd::RawFd;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::fb::{SCREEN_H, SCREEN_W};
use crate::surface::{PixFmt, Surface};

const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
const MXCFB_SEND_UPDATE: libc::c_ulong = 0x4048_462e;

const UPDATE_MODE_PARTIAL: u32 = 0;
const UPDATE_MODE_FULL: u32 = 1;
const WAVEFORM_DU: u32 = 1;
const WAVEFORM_GC16: u32 = 2;
const WAVEFORM_GC16_FAST: u32 = 3;
const TEMP_DRAW: i32 = 0x18;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct MxcfbRect {
    top: u32,
    left: u32,
    width: u32,
    height: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct MxcfbAltBufferData {
    phys_addr: u32,
    width: u32,
    height: u32,
    alt_update_region: MxcfbRect,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct MxcfbUpdateData {
    update_region: MxcfbRect,
    waveform_mode: u32,
    update_mode: u32,
    update_marker: u32,
    temp: i32,
    flags: u32,
    dither_mode: i32,
    quant_bit: i32,
    alt_buffer_data: MxcfbAltBufferData,
}

// linux/fb.h. c_ulong is intentionally architecture-sized.
#[repr(C)]
#[derive(Clone, Copy)]
struct FbFixScreeninfo {
    id: [libc::c_char; 16],
    smem_start: libc::c_ulong,
    smem_len: u32,
    type_: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    line_length: u32,
    mmio_start: libc::c_ulong,
    mmio_len: u32,
    accel: u32,
    capabilities: u16,
    reserved: [u16; 2],
}

impl Default for FbFixScreeninfo {
    fn default() -> Self { unsafe { std::mem::zeroed() } }
}

pub struct Rm1Framebuffer {
    fd: RawFd,
    ptr: *mut u8,
    len: usize,
    pub stride: usize,
    marker: AtomicU32,
}

impl Rm1Framebuffer {
    pub fn open() -> io::Result<(Self, Surface)> {
        let path = CString::new("/dev/fb0").unwrap();
        let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };
        if fd < 0 { return Err(io::Error::last_os_error()); }

        let mut fix = FbFixScreeninfo::default();
        let rc = unsafe { libc::ioctl(fd, FBIOGET_FSCREENINFO, &mut fix) };
        let stride = if rc == 0 && fix.line_length >= (SCREEN_W * 2) as u32 {
            fix.line_length as usize
        } else {
            // RM1's visible width is 1404; its native framebuffer line is 1408 RGB565 pixels.
            1408 * 2
        };
        let len = stride * SCREEN_H;
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            let e = io::Error::last_os_error();
            unsafe { libc::close(fd); }
            return Err(e);
        }
        let ptr = ptr as *mut u8;
        let surface = Surface::new(ptr, len, SCREEN_W, SCREEN_H, stride, PixFmt::Rgb565);
        eprintln!("riddle: RM1 /dev/fb0 mapped: {}x{}, stride {}", SCREEN_W, SCREEN_H, stride);
        Ok((Self { fd, ptr, len, stride, marker: AtomicU32::new(1) }, surface))
    }

    fn refresh(&self, x: i32, y: i32, w: i32, h: i32, fast: bool, full: bool) -> io::Result<()> {
        if w <= 0 || h <= 0 { return Ok(()); }
        let x0 = x.max(0).min(SCREEN_W as i32 - 1) as u32;
        let y0 = y.max(0).min(SCREEN_H as i32 - 1) as u32;
        let x1 = (x.saturating_add(w)).max(0).min(SCREEN_W as i32) as u32;
        let y1 = (y.saturating_add(h)).max(0).min(SCREEN_H as i32) as u32;
        if x1 <= x0 || y1 <= y0 { return Ok(()); }

        // The EPDC/PxP works best on 8-pixel blocks. Expand the rectangle while clamping.
        let left = x0 & !7;
        let top = y0 & !7;
        let right = ((x1 + 7) & !7).min(SCREEN_W as u32);
        let bottom = ((y1 + 7) & !7).min(SCREEN_H as u32);
        let marker = self.marker.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        let mut data = MxcfbUpdateData {
            update_region: MxcfbRect { top, left, width: right - left, height: bottom - top },
            waveform_mode: if full { WAVEFORM_GC16 } else if fast { WAVEFORM_DU } else { WAVEFORM_GC16_FAST },
            update_mode: if full { UPDATE_MODE_FULL } else { UPDATE_MODE_PARTIAL },
            update_marker: marker,
            temp: TEMP_DRAW,
            ..Default::default()
        };
        let rc = unsafe { libc::ioctl(self.fd, MXCFB_SEND_UPDATE, &mut data) };
        if rc < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
    }

    pub fn update(&self, x: i32, y: i32, w: i32, h: i32, fast: bool) {
        if let Err(e) = self.refresh(x, y, w, h, fast, false) {
            eprintln!("riddle: framebuffer refresh failed: {e}");
        }
    }

    pub fn update_all(&self) { self.update(0, 0, SCREEN_W as i32, SCREEN_H as i32, false); }

    pub fn full_refresh(&self) {
        if let Err(e) = self.refresh(0, 0, SCREEN_W as i32, SCREEN_H as i32, false, true) {
            eprintln!("riddle: full refresh failed: {e}");
        }
    }
}

impl Drop for Rm1Framebuffer {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr as *mut libc::c_void, self.len);
            libc::close(self.fd);
        }
    }
}
