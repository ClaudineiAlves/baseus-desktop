//! Reads the Material You palette caelestia derives from the wallpaper, so the app
//! follows the desktop instead of shipping a frozen copy of it.
//!
//! Local customisation, not part of upstream: the file is caelestia-specific and
//! absent everywhere else, in which case the frontend keeps its built-in palette.

use std::{collections::HashMap, path::PathBuf, time::Duration};

use tauri::{AppHandle, Emitter};

const POLL: Duration = Duration::from_secs(2);

pub type Palette = HashMap<String, String>;

fn scheme_path() -> Option<PathBuf> {
    let state = match std::env::var_os("XDG_STATE_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => dirs_next::home_dir()?.join(".local/state"),
    };
    Some(state.join("caelestia/scheme.json"))
}

/// Parse the palette, returning `None` when caelestia is not installed or the file is
/// mid-write — a wallpaper change rewrites it, and a torn read is expected, not an error.
fn read() -> Option<Palette> {
    let raw = std::fs::read_to_string(scheme_path()?).ok()?;
    let doc: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let colours = doc.get("colours").or_else(|| doc.get("colors"))?;
    let map = colours
        .as_object()?
        .iter()
        .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
        .collect::<Palette>();
    (!map.is_empty()).then_some(map)
}

#[tauri::command]
pub fn get_color_scheme() -> Option<Palette> {
    read()
}

/// Re-emit the palette whenever the file's mtime moves. Polling rather than inotify:
/// caelestia replaces the file atomically, which turns a file watch into a stream of
/// create/remove events on a path that briefly does not exist.
pub fn watch(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last = None;
        loop {
            let stamp = scheme_path()
                .and_then(|p| std::fs::metadata(p).ok())
                .and_then(|m| m.modified().ok());
            if stamp != last {
                if last.is_some() {
                    if let Some(palette) = read() {
                        tracing::debug!("colour scheme changed, re-theming");
                        let _ = app.emit("color-scheme", palette);
                    }
                }
                last = stamp;
            }
            tokio::time::sleep(POLL).await;
        }
    });
}
