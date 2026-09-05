import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export interface BatteryState {
  left_pct: number;
  right_pct: number;
  left_charging: boolean;
  right_charging: boolean;
}

export interface CaseState {
  case_pct: number;
  case_charging: boolean;
}

export interface WearState {
  left_in_ear: boolean;
  right_in_ear: boolean;
}

export type AncMode = 'off' | 'anc' | 'transparency';

export type EqPreset = 'balanced' | 'bass_boost' | 'voice' | 'clear';

export type SpatialMode = 'normal' | 'music' | 'cinema';
export type DynamicMode = 'normal' | 'bass_boost' | 'balance';
export type EqModeId = number; // preset id byte (0x00,0x01,0x03,0x07,0x08,0x09,0x0a)

export interface ModelInfo {
  name: string;
}

export type DeviceEvent =
  | { type: 'battery_update'; data: BatteryState }
  | { type: 'case_update'; data: CaseState }
  | { type: 'anc_mode_update'; data: AncMode }
  | { type: 'game_mode_update'; data: boolean }
  | { type: 'wear_update'; data: WearState }
  | { type: 'eq_preset_update'; data: EqPreset }
  | { type: 'spatial_mode_update'; data: SpatialMode }
  | { type: 'dynamic_mode_update'; data: DynamicMode }
  | { type: 'eq_mode_update'; data: string }
  | { type: 'connected' }
  | { type: 'disconnected' };

export type ConnectionState = 'connecting' | 'connected' | 'disconnected';

export function onDeviceEvent(cb: (e: DeviceEvent) => void): Promise<UnlistenFn> {
  return listen<DeviceEvent>('device-event', (event) => cb(event.payload));
}

export function onConnectionState(cb: (s: ConnectionState) => void): Promise<UnlistenFn> {
  return listen<ConnectionState>('connection-state', (event) => cb(event.payload));
}

export function onModelInfo(cb: (info: ModelInfo) => void): Promise<UnlistenFn> {
  return listen<ModelInfo>('model-info', (event) => cb(event.payload));
}

export interface Settings {
  launch_at_login: boolean;
  low_battery_alerts: boolean;
  show_session_timer: boolean;
}

export function setAncMode(mode: AncMode, level?: number): Promise<void> {
  return invoke('set_anc_mode', { mode, level });
}

export function setGameMode(enabled: boolean): Promise<void> {
  return invoke('set_game_mode', { enabled });
}

export function findEarbud(side: 'left' | 'right'): Promise<void> {
  return invoke('find_earbud', { side });
}

export function getSettings(): Promise<Settings> {
  return invoke('get_settings');
}

export function setSettings(settings: Settings): Promise<void> {
  return invoke('set_settings', { settings });
}

export function setEqPreset(preset: EqPreset): Promise<void> {
  const map: Record<EqPreset, number> = { balanced: 0, bass_boost: 1, voice: 2, clear: 3 };
  return invoke('set_eq_preset', { preset: map[preset] });
}

const SPATIAL_MAP: Record<SpatialMode, number> = { normal: 0, music: 1, cinema: 2 };
export function setSpatialMode(mode: SpatialMode): Promise<void> {
  return invoke('set_spatial_mode', { mode: SPATIAL_MAP[mode] });
}

const DYNAMIC_MAP: Record<DynamicMode, number> = { normal: 0, bass_boost: 1, balance: 2 };
export function setDynamicMode(mode: DynamicMode): Promise<void> {
  return invoke('set_dynamic_mode', { mode: DYNAMIC_MAP[mode] });
}

export function setEqMode(id: EqModeId): Promise<void> {
  return invoke('set_eq_mode', { id });
}

export function getEqModes(): Promise<Array<[number, string]>> {
  return invoke('get_eq_modes');
}

export function setGesture(side: number, key: number, func: number): Promise<void> {
  return invoke('set_gesture', { side, key, function: func });
}

// Returns [keys, functions], each a list of [byte, label].
export function getGestureOptions(): Promise<
  [Array<[number, string]>, Array<[number, string]>]
> {
  return invoke('get_gesture_options');
}

export function getSupportedAncModes(modelName: string): Promise<AncMode[]> {
  return invoke('get_supported_anc_modes', { modelName });
}

export function onUpdateAvailable(cb: (version: string) => void): Promise<UnlistenFn> {
  return listen<string>('update-available', (event) => cb(event.payload));
}

export function checkForUpdate(): Promise<string | null> {
  return invoke('check_for_update');
}

export function installUpdate(): Promise<void> {
  return invoke('install_update');
}
