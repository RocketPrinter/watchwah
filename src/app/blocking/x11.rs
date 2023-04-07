use std::collections::hash_map::Entry;
use std::collections::HashMap;
use crate::app::blocking;
use crate::app::SState;
use anyhow::{bail, Result};
use procfs::process::Process;
use std::iter::FilterMap;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use chrono::{DateTime, Timelike, Utc};
use eframe::epaint::ahash::HashSet;
use notify_rust::{Notification, NotificationHandle, Timeout, Urgency};
use tokio::task::spawn_blocking;
use tracing::{error, info, instrument};
use x11rb::atom_manager;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{get_property, query_tree, AtomEnum, ConnectionExt, Window, get_geometry, Screen, translate_coordinates, GetGeometryReply, TranslateCoordinatesReply};
use x11rb::rust_connection::RustConnection;
use crate::app::blocking::should_block_windows;

#[derive(Debug)]
struct WindowInfo {
    window: Window,
    name: String,
    pid: u32,
    path: Option<String>,
    pos: WindowPosition,

    blocked: bool,
}

#[derive(Debug)]
struct WindowPosition {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
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

type NotificationMap = HashMap<Window, (DateTime<Utc>, NotificationHandle)>;

#[instrument(name = "x11 blocker", skip_all)]
pub async fn blocker(state: SState) {
    info!("Starting");

    spawn_blocking(move || {
        let mut sent_notifications: NotificationMap = HashMap::new();

        if let Err(e) = blocker_loop(state, &mut sent_notifications) {
            error!("{e}");
        }

        // release left notifications
        for (_,handle) in sent_notifications.into_values() {
            handle.close();
        }
    })
    .await
    .unwrap();

    info!("Stopping")
}

// todo: lots and lots of caching
fn blocker_loop(state: SState, sent_notifications: &mut NotificationMap) -> Result<()> {
    let (conn, _) = x11rb::connect(None)?;
    let atoms = Atoms::new(&conn)?.reply()?;

    loop {
        thread::sleep(Duration::from_millis(100));

        // we query the active windows and data about them
        let mut windows = query_active_windows(&conn, &atoms)?
            .into_iter()
            .map(|(window, root)| query_props(&conn, &atoms, window, root))
            .collect::<Result<Vec<WindowInfo>>>()?;

        // we lock the mutex *after* doing potentially long operations to avoid blocking the ui too much
        let mut state = state.lock().unwrap();
        if !blocking::should_enable_blocker(&state) {
            break;
        }

        // we check if we should block any of the windows
        for wi in windows.iter_mut() {
            wi.blocked = should_block_windows(&state, &wi.name, wi.path.as_deref());
        }

        // for debug menu
        let now = Utc::now();
        for wi in &windows {
            let name = if !wi.name.is_empty() {wi.name.clone()} else { format!("{:?}",wi) };
            let pos = format!("x:{}, y:{}, w:{}, h:{}", wi.pos.x, wi.pos.y, wi.pos.width, wi.pos.height);
            let mut extra = vec![pos];
            if let Some(path) = &wi.path {
                extra.push(path.clone());
            }
            state.detected_windows.insert(name, (now, wi.blocked, Some(extra)));
        }
        state.detected_windows.retain(|_, (time,_ ,_)| (now - *time).num_seconds() < 5);

        // we release the lock cause we don't need it anymore
        drop(state);

        // todo: in the future we should render an overlay that blocks windows, but for now we are just gonna send a notification
        // close the notification for a window which hasn't been blocked in 2 seconds
        let to_remove = sent_notifications.iter()
            .filter_map(|(window, (time, _))| {
                if (now - *time).num_seconds() > 2 {
                    Some(*window)
                } else {
                    None
                }
            }).collect::<Vec<Window>>();
        for window in to_remove {
            sent_notifications.remove(&window).unwrap().1.close();
        }
        // refresh the time for notifications with a blocked window or send a new notification
        for wi in windows {
            if !wi.blocked { continue }
            match sent_notifications.entry(wi.window) {
                Entry::Occupied(mut entry) => entry.get_mut().0 = now,
                Entry::Vacant(entry) => {
                    let handle = Notification::new()
                        .summary("Stop using that window!")
                        .body(&wi.name)
                        .urgency(Urgency::Critical)
                        .show()?;
                    entry.insert((now,handle));
                }
            }
        }
    }

    Ok(())
}

fn query_active_windows(conn: &RustConnection, atoms: &Atoms) -> Result<Vec<(Window, Window)>> {
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
            vec.push((window,screen.root));
        }
    }
    Ok(vec)
}

fn query_props(conn: &RustConnection, atoms: &Atoms, window: Window, root: Window) -> Result<WindowInfo> {
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
        Some(Process::new(pid as i32)?.exe()?.to_string_lossy().to_string())
    } else {
        None
    };

    let GetGeometryReply {x, y, width, height, ..} = get_geometry(conn, window)?.reply()?;
    let TranslateCoordinatesReply { dst_x: x, dst_y: y, ..} = translate_coordinates(conn, window, root, x, y)?.reply()?;

    Ok(WindowInfo {
        window,
        name,
        pid,
        path,
        pos: WindowPosition {
            x,y,width,height
        },
        blocked: false,
    })
}