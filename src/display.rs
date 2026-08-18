//! Display backends. The RM1 build uses the native framebuffer directly.
//! QTFB is retained only as a compatibility path when QTFB_KEY is present.

use crate::surface::{PixFmt, Surface};
use std::io;

pub enum Display {
    Qtfb(crate::qtfb::QtfbClient),
    Rm1(crate::rm1fb::Rm1Framebuffer),
}

impl Display {
    pub fn open() -> io::Result<(Self, Surface)> {
        if let Ok(key) = std::env::var("QTFB_KEY") {
            let key: i32 = key.parse().map_err(io::Error::other)?;
            let mut client = crate::qtfb::QtfbClient::connect(
                key,
                crate::qtfb::FBFMT_RMPP_RGB565,
                crate::fb::SCREEN_W as i32,
                crate::fb::SCREEN_H as i32,
                2,
            )?;
            let _ = client.set_refresh_mode(crate::qtfb::REFRESH_MODE_UFAST);
            let buf = client.framebuffer();
            let (ptr, len) = (buf.as_mut_ptr(), buf.len());
            let surface = Surface::new(
                ptr,
                len,
                crate::fb::SCREEN_W,
                crate::fb::SCREEN_H,
                crate::fb::SCREEN_W * 2,
                PixFmt::Rgb565,
            );
            return Ok((Display::Qtfb(client), surface));
        }

        let (fb, surface) = crate::rm1fb::Rm1Framebuffer::open()?;
        Ok((Display::Rm1(fb), surface))
    }

    pub fn update(&self, x: i32, y: i32, w: i32, h: i32, fast: bool) {
        match self {
            Display::Qtfb(c) => { let _ = c.update_partial(x, y, w, h); }
            Display::Rm1(fb) => fb.update(x, y, w, h, fast),
        }
    }

    pub fn update_all(&self, _w: usize, _h: usize) {
        match self {
            Display::Qtfb(c) => { let _ = c.update_all(); }
            Display::Rm1(fb) => fb.update_all(),
        }
    }

    pub fn full_refresh(&self, _w: usize, _h: usize) {
        match self {
            Display::Qtfb(c) => { let _ = c.request_full_refresh(); }
            Display::Rm1(fb) => fb.full_refresh(),
        }
    }

    pub fn pump(&self) -> io::Result<Vec<crate::qtfb::InputEvent>> {
        match self {
            Display::Qtfb(c) => c.drain_events(),
            Display::Rm1(_) => Ok(Vec::new()),
        }
    }

    pub fn terminate(&self) {
        if let Display::Qtfb(c) = self { c.terminate(); }
    }
}
