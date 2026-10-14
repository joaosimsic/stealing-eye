use anyhow::{anyhow, bail, Context, Result};
use std::ptr;
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::composite::{self, ConnectionExt as _, Redirect};
use x11rb::protocol::shm::ConnectionExt as _;
use x11rb::protocol::xproto::{ConnectionExt as _, ImageFormat, Pixmap, Window};
use x11rb::rust_connection::RustConnection;

use crate::x11::Rect;

enum Unredirect {
    Window(Window),
}

struct CompositeCleanup {
    pixmap: Pixmap,
    unredirect: Unredirect,
}

pub struct ShmCapturer<'a> {
    conn: &'a RustConnection,
    drawable: u32,
    rect: Rect,
    seg_id: u32,
    shmid: i32,
    ptr: *mut u8,
    size: usize,
    composite: Option<CompositeCleanup>,
}

impl<'a> ShmCapturer<'a> {
    /// Capture from the raw root front buffer (tears on GL/page-flipped apps).
    pub fn root(conn: &'a RustConnection, root: u32, rect: Rect, bpp: u8) -> Result<Self> {
        Self::build(conn, root, rect, bpp, None)
    }

    /// Capture a window from its own composite pixmap (stable, occlusion-free).
    pub fn composite_window(conn: &'a RustConnection, win: Window, bpp: u8) -> Result<(Self, u8)> {
        let pixmap = composite_setup(conn, win, true)?;
        let geom = conn
            .get_geometry(pixmap)?
            .reply()
            .context("get_geometry on window pixmap")?;
        let rect = Rect {
            x: 0,
            y: 0,
            w: geom.width,
            h: geom.height,
        };
        let capturer = Self::build(
            conn,
            pixmap,
            rect,
            bpp,
            Some(CompositeCleanup {
                pixmap,
                unredirect: Unredirect::Window(win),
            }),
        )?;
        Ok((capturer, geom.depth))
    }

    fn build(
        conn: &'a RustConnection,
        drawable: u32,
        rect: Rect,
        bits_per_pixel: u8,
        composite: Option<CompositeCleanup>,
    ) -> Result<Self> {
        if bits_per_pixel != 32 {
            bail!(
                "unsupported pixel depth: {bits_per_pixel} bpp (only 32-bpp TrueColor is supported)"
            );
        }
        if rect.w == 0 || rect.h == 0 {
            bail!("capture region is empty after clamping to the screen");
        }

        let bpl = ((rect.w as usize) * bits_per_pixel as usize).div_ceil(32) * 4;
        let size = bpl * rect.h as usize;

        let shmid = unsafe { libc::shmget(libc::IPC_PRIVATE, size, libc::IPC_CREAT | 0o600) };
        if shmid < 0 {
            return Err(anyhow!(
                "shmget failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        let ptr = unsafe { libc::shmat(shmid, ptr::null(), 0) } as *mut u8;
        if ptr as isize == -1 {
            unsafe { libc::shmctl(shmid, libc::IPC_RMID, ptr::null_mut()) };
            return Err(anyhow!("shmat failed: {}", std::io::Error::last_os_error()));
        }

        let seg_id = conn.generate_id().context("generating shm seg id")?;
        if let Err(e) = conn
            .shm_attach(seg_id, shmid as u32, false)
            .context("calling shm_attach")?
            .check()
        {
            unsafe {
                libc::shmdt(ptr as *const libc::c_void);
                libc::shmctl(shmid, libc::IPC_RMID, ptr::null_mut());
            }
            return Err(anyhow!("shm_attach rejected: {e}"));
        }

        Ok(ShmCapturer {
            conn,
            drawable,
            rect,
            seg_id,
            shmid,
            ptr,
            size,
            composite,
        })
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn frame(&self) -> Result<&[u8]> {
        self.conn
            .shm_get_image(
                self.drawable,
                self.rect.x,
                self.rect.y,
                self.rect.w,
                self.rect.h,
                !0u32,
                ImageFormat::Z_PIXMAP.into(),
                self.seg_id,
                0,
            )
            .context("sending shm_get_image")?
            .reply()
            .context("getting shm_get_image reply")?;

        Ok(unsafe { std::slice::from_raw_parts(self.ptr, self.size) })
    }
}

impl Drop for ShmCapturer<'_> {
    fn drop(&mut self) {
        if let Some(c) = self.composite.take() {
            let _ = self.conn.free_pixmap(c.pixmap);
            match c.unredirect {
                Unredirect::Window(win) => {
                    let _ = self
                        .conn
                        .composite_unredirect_window(win, Redirect::AUTOMATIC);
                }
            }
            let _ = self.conn.flush();
        }
        let _ = self.conn.shm_detach(self.seg_id);
        unsafe {
            libc::shmdt(self.ptr as *const libc::c_void);
            libc::shmctl(self.shmid, libc::IPC_RMID, ptr::null_mut());
        }
    }
}

pub fn composite_available(conn: &RustConnection) -> bool {
    conn.extension_information(composite::X11_EXTENSION_NAME)
        .ok()
        .flatten()
        .is_some()
}

fn composite_setup(conn: &RustConnection, window: Window, single: bool) -> Result<Pixmap> {
    conn.composite_query_version(0, 4)
        .context("composite_query_version")?
        .reply()
        .context("composite_query_version reply")?;

    if single {
        conn.composite_redirect_window(window, Redirect::AUTOMATIC)
            .context("composite_redirect_window")?
            .check()
            .context("composite_redirect_window rejected")?;
    } else {
        conn.composite_redirect_subwindows(window, Redirect::AUTOMATIC)
            .context("composite_redirect_subwindows")?
            .check()
            .context("composite_redirect_subwindows rejected")?;
    }

    let pixmap = conn.generate_id().context("generating pixmap id")?;
    conn.composite_name_window_pixmap(window, pixmap)
        .context("composite_name_window_pixmap")?
        .check()
        .context("composite_name_window_pixmap rejected")?;
    conn.flush().context("flush after composite setup")?;
    Ok(pixmap)
}
