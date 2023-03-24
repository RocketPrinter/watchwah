use anyhow::Result;
use procfs::process::Process;
use std::iter::FilterMap;
use std::path::PathBuf;
use tracing::error;
use x11rb::atom_manager;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{get_property, query_tree, AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

#[derive(Debug)]
struct WindowInfo {
    window: Window,
    name: String,
    pid: u32,
    path: Option<PathBuf>,
}

atom_manager! {
    Atoms: AtomCollectionCookie {
        _NET_WM_NAME,
        _NET_WM_VISIBLE_NAME,
        _NET_WM_PID,
        WM_NAME,
        _NET_ACTIVE_WINDOW,

        STRING,
        UTF8_STRING,
        WINDOW,
    }
}

pub fn proof_of_concept() -> Result<()> {
    let (conn, _) = x11rb::connect(None)?;
    let atoms = Atoms::new(&conn)?.reply()?;


    let windows = query_all_windows(&conn)
        .filter_map(|window| match query_props(&conn, &atoms, window) {
            Ok(w) => Some(w),
            Err(e) => {
                error!("Query props error: {e}");
                None
            }
        })
        .collect::<Vec<WindowInfo>>();
    dbg!(&windows);

    let windows = query_all_active_windows(&conn, &atoms);
    dbg!(&windows);

    Ok(())
}

fn query_all_active_windows(conn: &RustConnection, atoms: &Atoms) -> Vec<Result<Window>> {
    conn.setup().roots.iter().map(|screen| -> Result<Window> {
        let window = get_property(conn, false, screen.root, atoms._NET_ACTIVE_WINDOW, atoms.WINDOW, 0, 4)?.reply()?.value;
        let window = u32::from_ne_bytes(window.try_into().unwrap_or_default());
        Ok(window)
    }).collect::<Vec<Result<Window>>>()
}

fn query_all_windows(conn: &RustConnection) -> impl Iterator<Item = Window> + '_ {
    conn
        .setup()
        .roots
        .iter()
        // find all windows
        .map(|screen| -> Result<Vec<Window>> {
            Ok(query_tree(conn, screen.root)?.reply()?.children)
        })
        .filter_map(|r| match r {
            Ok(w) => Some(w),
            Err(e) => {
                error!("Query tree error: {e}");
                None
            }
        })
        .flatten()
}

fn query_props(conn: &RustConnection, atoms: &Atoms, window: Window) -> Result<WindowInfo> {
    let name = get_property(conn, false, window, atoms._NET_WM_NAME, atoms.UTF8_STRING, 0, 512,)?
    .reply()?
    .value;

    let mut name = String::from_utf8(name)?;

    if name.is_empty() {
        // we query for WM_NAME if the name is empty cause cause why not
        let other_name = get_property(conn, false, window, atoms.WM_NAME, atoms.STRING, 0, 512)?
            .reply()?
            .value;

        name = String::from_utf8(other_name)?;
    }

    let pid = get_property(conn, false, window, atoms._NET_WM_PID, AtomEnum::CARDINAL, 0, 1,)?
    .reply()?
    .value;

    let pid = u32::from_ne_bytes(pid.try_into().unwrap_or_default());

    let path = if pid != 0 {
        Some(Process::new(pid as i32)?.exe()?)
    } else {
        None
    };

    Ok(WindowInfo {
        window,
        name,
        pid,
        path,
    })
}
