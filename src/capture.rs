use anyhow::{anyhow, bail, Context, Result};
use std::ptr;
use x11rb::connection::Connection;
use x11rb::protocol::shm::ConnectionExt as _;
use x11rb::protocol::xproto::ImageFormat;
use x11rb::rust_connection::RustConnection;

use crate::x11::Rect;

pub struct ShmCapturer<'a> {
    conn: &'a RustConnection,
    root: u32,
    rect: Rect,
    seg_id: u32,
    shmid: i32,
    ptr: *mut u8,
    size: usize,
}

impl<'a> ShmCapturer<'a> {
    pub fn new(
        conn: &'a RustConnection,
        root: u32,
        rect: Rect,
        bits_per_pixel: u8,
    ) -> Result<Self> {
        if bits_per_pixel != 32 {
            bail!(
                "unsupported root pixel depth: {bits_per_pixel} bpp (only 32-bpp TrueColor is supported)"
            );
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
            return Err(anyhow!(
                "shmat failed: {}",
                std::io::Error::last_os_error()
            ));
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
            root,
            rect,
            seg_id,
            shmid,
            ptr,
            size,
        })
    }

    pub fn frame(&self) -> Result<&[u8]> {
        self.conn
            .shm_get_image(
                self.root,
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

impl<'a> Drop for ShmCapturer<'a> {
    fn drop(&mut self) {
        let _ = self.conn.shm_detach(self.seg_id);
        unsafe {
            libc::shmdt(self.ptr as *const libc::c_void);
            libc::shmctl(self.shmid, libc::IPC_RMID, ptr::null_mut());
        }
    }
}
