import type { AncMode } from '../lib/tauri';

interface Props {
  mode: AncMode;
  loading: AncMode | null;
  level: number; // 1–10
  supportedModes: AncMode[];
  gameMode: boolean;
  showGameMode: boolean;
  onMode: (mode: AncMode) => void;
  onLevel: (level: number) => void;
  onLevelCommit: (level: number) => void;
  onGameMode: (on: boolean) => void;
}

const MODE_META: Record<AncMode, { icon: string; name: string; desc: string }> = {
  off:          { icon: '🔇', name: 'Off',                       desc: 'Passthrough — no processing' },
  anc:          { icon: '🎧', name: 'Active Noise Cancellation', desc: 'Blocks ambient sound' },
  transparency: { icon: '🌬️', name: 'Transparency',              desc: 'Lets ambient sound in' },
};

// Level slider only applies to ANC/Transparency modes.
const LEVEL_MODES: AncMode[] = ['anc', 'transparency'];

function prefersReducedMotion() {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export default function AncTab(props: Props) {
  let gameRef: HTMLButtonElement | undefined;

  function sliderInput(e: Event) {
    props.onLevel(Number((e.target as HTMLInputElement).value));
  }
  function sliderCommit(e: Event) {
    props.onLevelCommit(Number((e.target as HTMLInputElement).value));
  }

  function handleGameClick() {
    props.onGameMode(!props.gameMode);
    if (gameRef && !prefersReducedMotion()) {
      gameRef.animate(
        [{ transform: 'scale(1)' }, { transform: 'scale(0.98)' }, { transform: 'scale(1)' }],
        { duration: 240, easing: 'ease-out' },
      );
    }
  }

  const showStrength = () => LEVEL_MODES.includes(props.mode);

  return (
    <div>
      <div style={labelStyle}>Noise Control <Divider /></div>

      <div style={{ display: 'flex', 'flex-direction': 'column', gap: '8px', 'margin-bottom': '16px' }}>
        {props.supportedModes.map((mode) => {
          const meta = MODE_META[mode] ?? { icon: '?', name: mode, desc: '' };
          const isActive = () => props.mode === mode;
          const isLoading = () => props.loading === mode;
          return (
            <button
              class="lift press mode-glow"
              classList={{ on: isActive() }}
              onClick={() => props.onMode(mode)}
              style={{
                background: isActive() ? 'rgba(99,102,241,0.08)' : 'var(--surface-1)',
                border: `1px solid ${isActive() ? 'rgba(99,102,241,0.4)' : 'var(--border)'}`,
                'border-radius': '12px',
                padding: '14px 16px',
                display: 'flex',
                'align-items': 'center',
                gap: '12px',
                cursor: 'pointer',
                transition: 'border-color 0.12s, background 0.12s, transform 0.16s, box-shadow 0.16s',
                animation: isLoading() ? 'pulse 0.8s ease-in-out infinite' : 'none',
                width: '100%',
                'text-align': 'left',
              }}
            >
              <span style={{ 'font-size': '20px', width: '28px', 'text-align': 'center' }}>{meta.icon}</span>
              <div style={{ flex: '1' }}>
                <div style={{ 'font-size': '13px', 'font-weight': '600', color: isActive() ? 'var(--accent-soft)' : 'var(--text-2)', transition: 'color 0.16s' }}>
                  {meta.name}
                </div>
                <div style={{ 'font-size': '10px', color: 'var(--text-muted)', 'margin-top': '2px' }}>{meta.desc}</div>
              </div>
              <div
                class="anc-check"
                classList={{ on: isActive() }}
                style={{
                  width: '16px', height: '16px', 'border-radius': '50%',
                  background: 'var(--accent)', display: 'flex',
                  'align-items': 'center', 'justify-content': 'center',
                  'font-size': '9px', color: 'var(--text)', 'flex-shrink': '0',
                }}
              >
                ✓
              </div>
            </button>
          );
        })}
      </div>

      {/* Level slider — collapses smoothly when the mode has no strength control */}
      <div class="strength" classList={{ shown: showStrength(), hidden: !showStrength() }}>
        <div style={labelStyle}>Strength <Divider /></div>
        <div
          style={{
            background: 'var(--surface-1)',
            border: '1px solid var(--border)',
            'border-radius': '12px',
            padding: '14px 16px',
          }}
        >
          <div style={{ display: 'flex', 'justify-content': 'space-between', 'margin-bottom': '12px' }}>
            <span style={{ 'font-size': '12px', color: 'var(--text-3)' }}>Level</span>
            <span class="num-mono" style={{ 'font-size': '12px', 'font-weight': '700', color: 'var(--accent-soft)' }}>
              {props.level} / 10
            </span>
          </div>
          <input
            type="range" min="1" max="10" value={props.level}
            onInput={sliderInput}
            onChange={sliderCommit}
            style={{
              width: '100%', height: '4px',
              'accent-color': 'var(--accent)',
              cursor: 'pointer',
            }}
          />
          <div style={{ display: 'flex', 'justify-content': 'space-between', 'margin-top': '6px', 'font-size': '9px', color: 'var(--text-muted)' }}>
            <span>Low</span><span>High</span>
          </div>
        </div>
      </div>

      {/* Game / low-latency mode — independent toggle, not an ANC state */}
      {props.showGameMode && (
        <>
          <div style={{ ...labelStyle, 'margin-top': '16px' }}>Game Mode <Divider /></div>
          <div class="game-wrap" classList={{ on: props.gameMode }}>
            <button ref={gameRef} class="game-card" onClick={handleGameClick}>
              <span class="game-ic">🎮</span>
              <div style={{ flex: '1' }}>
                <div class="game-title">Low Latency</div>
                <div class="game-desc">Reduces audio delay for gaming</div>
              </div>
              <div class="game-switch">
                <span class="game-state">{props.gameMode ? 'ON' : 'OFF'}</span>
                <span class="game-knob" />
              </div>
            </button>
          </div>
        </>
      )}
    </div>
  );
}

function Divider() {
  return <div style={{ flex: '1', height: '1px', background: 'var(--border)' }} />;
}

const labelStyle = {
  'font-size': '9px',
  'font-weight': '700',
  color: 'var(--text-muted)',
  'letter-spacing': '0.12em',
  'text-transform': 'uppercase' as const,
  display: 'flex',
  'align-items': 'center',
  gap: '8px',
  'margin-bottom': '8px',
};
