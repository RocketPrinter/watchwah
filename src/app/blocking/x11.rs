use crate::app::blocking;
use crate::app::blocking::WindowInfo;
use crate::app::SState;
use anyhow::{bail, Result};
use procfs::process::Process;
use std::iter::FilterMap;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use chrono::{Timelike, Utc};
use tokio::task::spawn_blocking;
use tracing::{error, info, instrument};
use x11rb::atom_manager;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{get_property, query_tree, AtomEnum, ConnectionExt, Window, get_geometry};
use x11rb::rust_connection::RustConnection;

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

#[instrument(name = "x11 blocker", skip_all)]
pub async fn blocker(state: SState) {
    info!("Starting");

    spawn_blocking(move || {
        if let Err(e) = blocker_loop(state) {
            error!("{e}");
        }
    })
    .await
    .unwrap();

    info!("Stopping")
}

fn blocker_loop(state: SState) -> Result<()> {
    let (conn, _) = x11rb::connect(None)?;
    let atoms = Atoms::new(&conn)?.reply()?;

    loop {
        thread::sleep(Duration::from_millis(100));

        let windows = query_active_windows(&conn, &atoms)?
            .iter()
            .map(|w| query_props(&conn, &atoms, *w))
            .collect::<Result<Vec<WindowInfo>>>()?;

        // we lock the mutex *after* doing potentially long operations to avoid blocking the ui too much
        let mut state = state.lock().unwrap();
        if !blocking::should_block(&state) {
            break;
        }

        // for debug menu
        let utc = Utc::now();
        for wi in &windows {
            let name = if !wi.name.is_empty() {wi.name.clone()} else { format!("{:?}",wi) };
            state.last_detected_windows.insert(name, utc);
        }
        state.last_detected_windows.retain(|_, time| (utc - *time).num_seconds() < 5);
    }

    Ok(())
}

fn query_active_windows(conn: &RustConnection, atoms: &Atoms) -> Result<Vec<Window>> {
    let mut vec = vec![];
    for screen in conn.setup().roots.iter() {
        let window = get_property(
            conn,
            false,
            screen.root,
            atoms._NET_ACTIVE_WINDOW,
            atoms.WINDOW,
            0,
            4,
        )?
        .reply()?
        .value;
        let window = u32::from_ne_bytes(window.try_into().unwrap_or_default());
        if window != 0 {
            vec.push(window);
        }
    }
    Ok(vec)
}

fn query_props(conn: &RustConnection, atoms: &Atoms, window: Window) -> Result<WindowInfo> {
    let name = get_property(
        conn,
        false,
        window,
        atoms._NET_WM_NAME,
        atoms.UTF8_STRING,
        0,
        512,
    )?
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

    let pid = get_property(
        conn,
        false,
        window,
        atoms._NET_WM_PID,
        AtomEnum::CARDINAL,
        0,
        1,
    )?
    .reply()?
    .value;

    let pid = u32::from_ne_bytes(pid.try_into().unwrap_or_default());

    let path = if pid != 0 {
        Some(Process::new(pid as i32)?.exe()?)
    } else {
        None
    };

    let reply = get_geometry(conn, window)?.reply()?;

    Ok(WindowInfo {
        window,
        name,
        pid,
        path,
    })
}
