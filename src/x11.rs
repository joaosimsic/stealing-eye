use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::shm::X11_EXTENSION_NAME as SHM_NAME;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt as _, Window};
use x11rb::rust_connection::RustConnection;

use crate::cli::Cli;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub w: u16,
    pub h: u16,
}

pub struct X11 {
    pub conn: RustConnection,
    pub root: Window,
    pub root_width: u16,
    pub root_height: u16,
    pub bits_per_pixel: u8,
    pub shm_available: bool,
}

impl X11 {
    pub fn connect() -> Result<Self> {
        let (conn, screen_num) = x11rb::connect(None).context("connecting to the X server")?;

        let (root, root_width, root_height, root_depth) = {
            let screen = conn
                .setup()
                .roots
                .get(screen_num)
                .ok_or_else(|| anyhow!("X server reported no screen {screen_num}"))?;
            (
                screen.root,
                screen.width_in_pixels,
                screen.height_in_pixels,
                screen.root_depth,
            )
        };

        let bits_per_pixel = conn
            .setup()
            .pixmap_formats
            .iter()
            .find(|f| f.depth == root_depth)
            .map(|f| f.bits_per_pixel)
            .unwrap_or(32);

        let shm_available = conn
            .extension_information(SHM_NAME)?
            .is_some();

        Ok(X11 {
            conn,
            root,
            root_width,
            root_height,
            bits_per_pixel,
            shm_available,
        })
    }

    pub fn resolve_region(&self, cli: &Cli) -> Result<Rect> {
        if let Some(spec) = &cli.region {
            let mut rect = parse_region(spec).context("parsing --region")?;
            self.clamp(&mut rect);
            return Ok(rect);
        }

        if let Some(id) = &cli.window {
            let win = parse_window_id(id).context("parsing --window")?;
            return self.window_rect(win);
        }

        if let Some(pat) = &cli.window_name {
            let win = self.find_window_by_name(pat)?;
            return self.window_rect(win);
        }

        Ok(Rect {
            x: 0,
            y: 0,
            w: self.root_width,
            h: self.root_height,
        })
    }

    fn clamp(&self, rect: &mut Rect) {
        rect.w = rect.w.min(self.root_width.saturating_sub(rect.x.max(0) as u16));
        rect.h = rect.h.min(self.root_height.saturating_sub(rect.y.max(0) as u16));
    }

    fn window_rect(&self, win: Window) -> Result<Rect> {
        let geom = self
            .conn
            .get_geometry(win)?
            .reply()
            .context("get_geometry on target window")?;
        let abs = self
            .conn
            .translate_coordinates(win, self.root, 0, 0)?
            .reply()
            .context("translate_coordinates to root")?;
        Ok(Rect {
            x: abs.dst_x,
            y: abs.dst_y,
            w: geom.width,
            h: geom.height,
        })
    }

    fn find_window_by_name(&self, pattern: &str) -> Result<Window> {
        let re = Regex::new(pattern).context("compiling --window-name regex")?;
        let windows = self.client_list()?;
        for win in windows {
            if let Some(name) = self.window_name(win)? {
                if re.is_match(&name) {
                    return Ok(win);
                }
            }
        }
        bail!("no window matched --window-name '{pattern}'")
    }

    fn client_list(&self) -> Result<Vec<Window>> {
        let atom = self.intern("_NET_CLIENT_LIST")?;
        let reply = self
            .conn
            .get_property(false, self.root, atom, AtomEnum::WINDOW, 0, u32::MAX)?
            .reply()
            .context("reading _NET_CLIENT_LIST")?;
        let wins: Vec<Window> = reply
            .value32()
            .map(|it| it.collect())
            .unwrap_or_default();
        if wins.is_empty() {
            bail!("_NET_CLIENT_LIST is empty (no EWMH-compliant window manager?)");
        }
        Ok(wins)
    }

    fn window_name(&self, win: Window) -> Result<Option<String>> {
        let net_name = self.intern("_NET_WM_NAME")?;
        let reply = self
            .conn
            .get_property(false, win, net_name, AtomEnum::ANY, 0, 1024)?
            .reply()
            .context("reading _NET_WM_NAME")?;
        if !reply.value.is_empty() {
            return Ok(Some(String::from_utf8_lossy(&reply.value).into_owned()));
        }

        let reply = self
            .conn
            .get_property(false, win, AtomEnum::WM_NAME, AtomEnum::ANY, 0, 1024)?
            .reply()
            .context("reading WM_NAME")?;
        if reply.value.is_empty() {
            Ok(None)
        } else {
            Ok(Some(String::from_utf8_lossy(&reply.value).into_owned()))
        }
    }

    fn intern(&self, name: &str) -> Result<Atom> {
        Ok(self.conn.intern_atom(false, name.as_bytes())?.reply()?.atom)
    }

    pub fn list_windows(&self) -> Result<()> {
        for win in self.client_list()? {
            let name = self.window_name(win)?.unwrap_or_else(|| "<unnamed>".into());
            let rect = self.window_rect(win).ok();
            match rect {
                Some(r) => println!(
                    "0x{win:x}\t{}x{}+{}+{}\t{name}",
                    r.w, r.h, r.x, r.y
                ),
                None => println!("0x{win:x}\t<geometry unavailable>\t{name}"),
            }
        }
        Ok(())
    }
}

pub fn parse_region(spec: &str) -> Result<Rect> {
    let parts: Vec<&str> = spec.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        bail!("expected X,Y,W,H, got '{spec}'");
    }
    let x: i32 = parts[0].parse().context("X")?;
    let y: i32 = parts[1].parse().context("Y")?;
    let w: u32 = parts[2].parse().context("W")?;
    let h: u32 = parts[3].parse().context("H")?;
    if w == 0 || h == 0 {
        bail!("width and height must be non-zero");
    }
    Ok(Rect {
        x: x as i16,
        y: y as i16,
        w: w as u16,
        h: h as u16,
    })
}

pub fn parse_window_id(spec: &str) -> Result<Window> {
    let spec = spec.trim();
    let id = if let Some(hex) = spec.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).context("hex window id")?
    } else {
        spec.parse().context("decimal window id")?
    };
    if id == 0 {
        bail!("window id 0 is not valid");
    }
    Ok(id)
}
