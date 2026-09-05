import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

type Palette = Record<string, string>;

/** CSS token -> key in caelestia's Material You palette. */
const TOKENS: Record<string, string> = {
  '--bg': 'surface',
  '--surface-1': 'surfaceContainerLow',
  '--surface-2': 'surfaceContainer',
  '--surface-3': 'surfaceContainerHigh',
  '--border': 'surfaceContainerHigh',
  '--border-strong': 'outlineVariant',
  '--text': 'onSurface',
  '--text-2': 'onSurfaceVariant',
  '--text-3': 'outline',
  '--text-muted': 'outlineVariant',
  '--accent': 'primary',
  '--accent-soft': 'secondary',
  '--accent-dim': 'inversePrimary',
  '--ok': 'success',
  '--danger': 'error',
  // --warn has no Material You counterpart; index.css keeps its literal.
};

function apply(palette: Palette) {
  const root = document.documentElement;
  for (const [token, key] of Object.entries(TOKENS)) {
    const value = palette[key];
    // Values arrive as bare hex ("131317"), and a missing key must fall through to
    // the built-in palette rather than blanking the token.
    if (value) root.style.setProperty(token, value.startsWith('#') ? value : `#${value}`);
  }
}

/**
 * Follow the desktop palette. No-op wherever caelestia is absent — the command
 * returns null and index.css keeps its defaults.
 */
export async function initColorScheme() {
  try {
    const palette = await invoke<Palette | null>('get_color_scheme');
    if (palette) apply(palette);
  } catch {
    /* command unavailable — keep the built-in palette */
  }
  await listen<Palette>('color-scheme', (e) => apply(e.payload));
}
