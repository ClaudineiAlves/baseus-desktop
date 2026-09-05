use crate::device::{CommandSender, DeviceCommand, Side};
use crate::settings::{self, Settings};
use baseus_protocol::types::{
    AncMode, BaseusModel, DynamicMode, EqMode, EqPreset, GestureFunction, GestureKey, GestureSide,
    SpatialMode,
};
use tauri::{AppHandle, Runtime, State};
use tauri_plugin_autostart::ManagerExt;
#[cfg(not(target_os = "linux"))]
use tauri_plugin_updater::UpdaterExt;

#[tauri::command]
pub fn set_anc_mode(
    mode: String,
    level: Option<u8>,
    cmd_tx: State<CommandSender>,
) -> Result<(), String> {
    let anc_mode = match mode.as_str() {
        "off" => AncMode::Off,
        "anc" => AncMode::Anc,
        "transparency" => AncMode::Transparency,
        other => return Err(format!("unknown mode: {other}")),
    };
    let byte = level.unwrap_or(0x68);
    cmd_tx
        .send(DeviceCommand::SetAncMode(anc_mode, byte))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_eq_preset(preset: u8, cmd_tx: State<CommandSender>) -> Result<(), String> {
    let eq = EqPreset::from_byte(preset).ok_or_else(|| format!("unknown EQ preset: {preset}"))?;
    cmd_tx
        .send(DeviceCommand::SetEqPreset(eq))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_spatial_mode(mode: u8, cmd_tx: State<CommandSender>) -> Result<(), String> {
    let m = SpatialMode::from_byte(mode).ok_or_else(|| format!("unknown spatial mode: {mode}"))?;
    cmd_tx
        .send(DeviceCommand::SetSpatialMode(m))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_dynamic_mode(mode: u8, cmd_tx: State<CommandSender>) -> Result<(), String> {
    let m = DynamicMode::from_byte(mode).ok_or_else(|| format!("unknown dynamic mode: {mode}"))?;
    cmd_tx
        .send(DeviceCommand::SetDynamicMode(m))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_eq_mode(id: u8, cmd_tx: State<CommandSender>) -> Result<(), String> {
    let m = EqMode::from_id(id).ok_or_else(|| format!("unknown EQ mode id: {id}"))?;
    cmd_tx
        .send(DeviceCommand::SetEqMode(m))
        .map_err(|e| e.to_string())
}

/// The EQ-mode presets (id + label) for the frontend to render.
#[tauri::command]
pub fn get_eq_modes() -> Vec<(u8, String)> {
    EqMode::ALL
        .iter()
        .map(|m| (m.id(), m.label().to_string()))
        .collect()
}

#[tauri::command]
pub fn set_gesture(
    side: u8,
    key: u8,
    function: u8,
    cmd_tx: State<CommandSender>,
) -> Result<(), String> {
    let side = GestureSide::from_byte(side).ok_or_else(|| format!("bad gesture side: {side}"))?;
    let key = GestureKey::from_byte(key).ok_or_else(|| format!("bad gesture key: {key}"))?;
    let func = GestureFunction::from_byte(function)
        .ok_or_else(|| format!("bad gesture function: {function}"))?;
    cmd_tx
        .send(DeviceCommand::SetGesture(side, key, func))
        .map_err(|e| e.to_string())
}

/// A (wire byte, label) pair for a frontend picker.
type LabeledByte = (u8, String);

/// Gesture keys (tap types) and functions for the frontend to render.
#[tauri::command]
pub fn get_gesture_options() -> (Vec<LabeledByte>, Vec<LabeledByte>) {
    let keys = GestureKey::ALL
        .iter()
        .map(|k| (k.to_byte(), k.label().to_string()))
        .collect();
    let funcs = GestureFunction::ALL
        .iter()
        .map(|f| (f.to_byte(), f.label().to_string()))
        .collect();
    (keys, funcs)
}

#[tauri::command]
pub fn set_game_mode(enabled: bool, cmd_tx: State<CommandSender>) -> Result<(), String> {
    cmd_tx
        .send(DeviceCommand::SetGameMode(enabled))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn find_earbud(side: String, cmd_tx: State<CommandSender>) -> Result<(), String> {
    let s = match side.as_str() {
        "left" => Side::Left,
        "right" => Side::Right,
        other => return Err(format!("unknown side: {other}")),
    };
    cmd_tx
        .send(DeviceCommand::FindEarbud(s))
        .map_err(|e| e.to_string())
}

/// Return the ANC modes supported by a given model name.
/// The frontend calls this after receiving a `model-info` event to know which modes to show.
#[tauri::command]
pub fn get_supported_anc_modes(model_name: String) -> Vec<String> {
    let model = BaseusModel::all()
        .iter()
        .find(|m| m.display_name() == model_name)
        .copied();

    let Some(m) = model else {
        // Fallback to BP1 defaults for unknown models.
        return vec![
            "off".to_string(),
            "anc".to_string(),
            "transparency".to_string(),
        ];
    };

    AncMode::supported_by(m)
        .iter()
        .map(|mode| {
            serde_json::to_value(mode)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default()
        })
        .collect()
}

/// Silent background check — returns version string if an update is available, None otherwise.
#[cfg(not(target_os = "linux"))]
pub(crate) async fn check_update_silent(app: &AppHandle) -> Option<String> {
    app.updater().ok()?.check().await.ok()?.map(|u| u.version)
}

/// Linux builds carry no updater plugin — updates come from the distro package,
/// so the in-app check reports "up to date" and the install is a no-op.
#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn check_for_update(_app: AppHandle) -> Result<Option<String>, String> {
    Ok(None)
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<Option<String>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    Ok(updater
        .check()
        .await
        .map_err(|e| e.to_string())?
        .map(|u| u.version))
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn install_update(_app: AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    if let Some(update) = updater.check().await.map_err(|e| e.to_string())? {
        update
            .download_and_install(|_, _| {}, || {})
            .await
            .map_err(|e| e.to_string())?;
        app.restart();
    }
    Ok(())
}

#[tauri::command]
pub fn get_settings() -> Settings {
    settings::load()
}

#[tauri::command]
pub fn set_settings<R: Runtime>(app: AppHandle<R>, settings: Settings) -> Result<(), String> {
    settings::save(&settings)?;
    if settings.launch_at_login {
        app.autolaunch().enable().map_err(|e| e.to_string())
    } else {
        app.autolaunch().disable().map_err(|e| e.to_string())
    }
}
