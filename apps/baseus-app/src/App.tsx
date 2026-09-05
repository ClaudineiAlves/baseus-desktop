import { createMemo, createSignal, onCleanup, onMount, Show } from 'solid-js';
import Sidebar, { type Tab } from './components/Sidebar';
import HomeTab from './components/HomeTab';
import AncTab from './components/AncTab';
import SoundTab from './components/SoundTab';
import SettingsTab from './components/SettingsTab';
import { initColorScheme } from './lib/scheme';
import type { SpatialMode, DynamicMode } from './lib/tauri';
import {
  onDeviceEvent,
  onConnectionState,
  onModelInfo,
  onUpdateAvailable,
  setAncMode,
  setSpatialMode,
  setDynamicMode,
  setEqMode,
  setGameMode,
  type AncMode,
  type ModelInfo,
  type WearState,
} from './lib/tauri';
import { pushLeft, pushRight, pushCase, left, right, caseData } from './stores/batteryHistory';
import { loadSettings, getSettingsStore } from './stores/settings';
import { startTimer, stopTimer, useElapsed } from './lib/timer';

type ConnStatus = 'connected' | 'connecting' | 'disconnected';

const BP1_ANC_MODES: AncMode[] = ['off', 'anc', 'transparency'];

export default function App() {
  const [status, setStatus] = createSignal<ConnStatus>('connecting');
  const [modelInfo, setModelInfo] = createSignal<ModelInfo | null>(null);
  const [ancMode, setAncModeSignal] = createSignal<AncMode>('off');
  const [ancLoading, setAncLoading] = createSignal<AncMode | null>(null);
  const [ancLevel, setAncLevel] = createSignal(7);
  const [gameMode, setGameModeSignal] = createSignal(false);
  const [activeTab, setActiveTab] = createSignal<Tab>('home');
  const [leftCharging, setLeftCharging] = createSignal(false);
  const [rightCharging, setRightCharging] = createSignal(false);
  const [caseCharging, setCaseCharging] = createSignal(false);
  const [wear, setWear] = createSignal<WearState | null>(null);
  const [spatial, setSpatialSignal] = createSignal<SpatialMode>('normal');
  const [dynamic, setDynamicSignal] = createSignal<DynamicMode>('normal');
  const [eqId, setEqIdSignal] = createSignal<number | null>(null);
  const [updateVersion, setUpdateVersion] = createSignal<string | null>(null);

  const connectedModelName = createMemo(() => modelInfo()?.name ?? 'Bass BP1 Pro ANC');
  const supportedAncModes = createMemo<AncMode[]>(() => BP1_ANC_MODES);

  onMount(async () => {
    const unlisteners: Array<() => void> = [];
    onCleanup(() => unlisteners.forEach((fn) => fn()));

    // Before anything paints, so the window never flashes the built-in palette.
    await initColorScheme();

    // Scale the fixed-px design to the window. The floor of 1 keeps it honest on a
    // small window; the ceiling stops a maximised window from looking like a mockup.
    const rescale = () => {
      const scale = Math.min(Math.max(window.innerWidth / 640, 1), 1.7);
      document.documentElement.style.setProperty('--ui-scale', String(scale));
    };
    rescale();
    window.addEventListener('resize', rescale);
    unlisteners.push(() => window.removeEventListener('resize', rescale));
    await loadSettings();

    onDeviceEvent((e) => {
      if (e.type === 'battery_update') {
        pushLeft(e.data.left_pct);
        pushRight(e.data.right_pct);
        setLeftCharging(e.data.left_charging);
        setRightCharging(e.data.right_charging);
      } else if (e.type === 'case_update') {
        pushCase(e.data.case_pct);
        setCaseCharging(e.data.case_charging);
      } else if (e.type === 'anc_mode_update') {
        setAncModeSignal(e.data);
        setAncLoading(null);
      } else if (e.type === 'game_mode_update') {
        setGameModeSignal(e.data);
      } else if (e.type === 'wear_update') {
        setWear(e.data);
      } else if (e.type === 'spatial_mode_update') {
        setSpatialSignal(e.data);
      } else if (e.type === 'dynamic_mode_update') {
        setDynamicSignal(e.data);
      }
    }).then((fn) => unlisteners.push(fn));

    onConnectionState((s) => {
      setStatus(s);
      if (s === 'connected') startTimer();
      else stopTimer();
    }).then((fn) => unlisteners.push(fn));

    onModelInfo((info) => {
      setModelInfo(info);
      setAncModeSignal('off');
    }).then((fn) => unlisteners.push(fn));

    onUpdateAvailable((version) => setUpdateVersion(version))
      .then((fn) => unlisteners.push(fn));
  });

  async function handleAnc(mode: AncMode) {
    if (ancMode() === mode) return;
    setAncLoading(mode);
    const byte = Math.round(((ancLevel() - 1) / 9) * (0xff - 0x10) + 0x10);
    try {
      await setAncMode(mode, mode === 'off' ? undefined : byte);
    } catch {
      setAncLoading(null);
    }
  }

  async function handleGameMode(on: boolean) {
    setGameModeSignal(on);
    await setGameMode(on).catch(() => setGameModeSignal(!on));
  }

  async function handleSpatial(m: SpatialMode) {
    const prev = spatial();
    setSpatialSignal(m);
    await setSpatialMode(m).catch(() => setSpatialSignal(prev));
  }

  async function handleDynamic(m: DynamicMode) {
    const prev = dynamic();
    setDynamicSignal(m);
    await setDynamicMode(m).catch(() => setDynamicSignal(prev));
  }

  async function handleEqMode(id: number) {
    const prev = eqId();
    setEqIdSignal(id);
    await setEqMode(id).catch(() => setEqIdSignal(prev));
  }

  function handleLevel(v: number) {
    setAncLevel(v);
  }

  async function handleLevelCommit(v: number) {
    setAncLevel(v);
    const mode = ancMode();
    if (mode !== 'off') {
      const byte = Math.round(((v - 1) / 9) * (0xff - 0x10) + 0x10);
      await setAncMode(mode, byte).catch(() => {});
    }
  }

  const statusColor = () =>
    status() === 'connected' ? 'var(--ok)' : status() === 'connecting' ? 'var(--warn)' : 'var(--text-muted)';

  const statusText = () =>
    status() === 'connected' ? 'Connected' : status() === 'connecting' ? 'Connecting…' : 'Disconnected';

  return (
    <div
      style={{
        width: '100%',
        height: '100vh',
        background: 'var(--bg)',
        color: 'var(--text)',
        'font-family': "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
        'box-sizing': 'border-box',
        display: 'flex',
        'flex-direction': 'column',
      }}
    >
      {/* Title bar */}
      <div
        class="titlebar-glow"
        style={{
          display: 'flex',
          'align-items': 'center',
          gap: '6px',
          padding: '12px 16px 10px',
          'border-bottom': '1px solid var(--border)',
          'flex-shrink': '0',
        }}
      >
        <div
          style={{
            flex: '1',
            'text-align': 'center',
            'font-size': '12px',
            'font-weight': '600',
            color: 'var(--text-2)',
          }}
        >
          {connectedModelName()}
        </div>
        <div style={{ display: 'flex', 'align-items': 'center', gap: '5px', 'font-size': '11px', color: statusColor(), 'font-weight': '500' }}>
          <div class={status() === 'connected' ? 'status-dot' : undefined} style={{ width: '6px', height: '6px', background: statusColor(), 'border-radius': '50%' }} />
          {statusText()}
        </div>
      </div>

      {/* Body: sidebar + content */}
      <div style={{ display: 'flex', flex: '1' }}>
        <Sidebar active={activeTab()} onSwitch={setActiveTab} updateAvailable={updateVersion() !== null} />

        <div style={{ flex: '1', padding: '16px', 'overflow-y': 'auto', display: 'flex', 'flex-direction': 'column' }}>
          <div style={{ 'max-width': '560px', margin: 'auto', width: '100%' }}>
          <Show when={activeTab() === 'home'}>
            <div class="panel-in">
              <HomeTab
                leftPct={left()[left().length - 1]?.pct ?? 0}
                rightPct={right()[right().length - 1]?.pct ?? 0}
                casePct={caseData()[caseData().length - 1]?.pct ?? 0}
                leftCharging={leftCharging()}
                rightCharging={rightCharging()}
                caseCharging={caseCharging()}
                leftInEar={wear()?.left_in_ear ?? false}
                rightInEar={wear()?.right_in_ear ?? false}
                wearKnown={wear() !== null}
                leftHistory={left().map((r) => r.pct)}
                rightHistory={right().map((r) => r.pct)}
                elapsed={useElapsed()()}
                showTimer={getSettingsStore().show_session_timer}
              />
            </div>
          </Show>

          <Show when={activeTab() === 'anc'}>
            <div class="panel-in">
              <AncTab
                mode={ancMode()}
                loading={ancLoading()}
                level={ancLevel()}
                supportedModes={supportedAncModes()}
                gameMode={gameMode()}
                showGameMode={true}
                onMode={handleAnc}
                onLevel={handleLevel}
                onLevelCommit={handleLevelCommit}
                onGameMode={handleGameMode}
              />
            </div>
          </Show>

          <Show when={activeTab() === 'sound'}>
            <div class="panel-in">
              <SoundTab
                spatial={spatial()}
                dynamic={dynamic()}
                eqId={eqId()}
                onSpatial={handleSpatial}
                onDynamic={handleDynamic}
                onEq={handleEqMode}
              />
            </div>
          </Show>

          <Show when={activeTab() === 'settings'}>
            <div class="panel-in">
              <SettingsTab initialUpdateVersion={updateVersion()} onUpdateInstalled={() => setUpdateVersion(null)} />
            </div>
          </Show>
          </div>
        </div>
      </div>
    </div>
  );
}
