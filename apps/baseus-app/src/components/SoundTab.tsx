import { createSignal, onMount, For } from 'solid-js';
import type { SpatialMode, DynamicMode } from '../lib/tauri';
import { getEqModes } from '../lib/tauri';

interface Props {
  spatial: SpatialMode;
  dynamic: DynamicMode;
  eqId: number | null;
  onSpatial: (m: SpatialMode) => void;
  onDynamic: (m: DynamicMode) => void;
  onEq: (id: number) => void;
}

const SPATIAL: { id: SpatialMode; name: string }[] = [
  { id: 'normal', name: 'Normal' },
  { id: 'music', name: 'Music' },
  { id: 'cinema', name: 'Cinema' },
];

const DYNAMIC: { id: DynamicMode; name: string }[] = [
  { id: 'normal', name: 'Normal' },
  { id: 'bass_boost', name: 'Bass Boost' },
  { id: 'balance', name: 'Balance' },
];

// A rough visual curve per EQ preset, keyed by its id byte — purely decorative,
// the actual band data lives in the captured frames on the Rust side.
const EQ_BARS: Record<number, number[]> = {
  0x00: [55, 60, 65, 60, 55], // Baseus Classic
  0x01: [100, 85, 55, 40, 35], // Deep Bass
  0x03: [45, 55, 70, 60, 50], // Hi-Fi Live
  0x07: [60, 75, 55, 70, 60], // Jazz
  0x08: [70, 60, 50, 60, 80], // Classical
  0x09: [35, 45, 55, 80, 100], // Treble Boost
  0x0a: [50, 65, 80, 65, 50], // Acoustic
};

export default function SoundTab(props: Props) {
  const [eqModes, setEqModes] = createSignal<Array<[number, string]>>([]);
  onMount(async () => {
    setEqModes(await getEqModes().catch(() => []));
  });

  return (
    <div style={{ display: 'flex', 'flex-direction': 'column', gap: '20px' }}>
      <section>
        <div style={labelStyle}>Spatial Audio <Divider /></div>
        <Segmented
          options={SPATIAL}
          active={props.spatial}
          onPick={(id) => props.onSpatial(id as SpatialMode)}
        />
      </section>

      <section>
        <div style={labelStyle}>Dynamic Sound <Divider /></div>
        <Segmented
          options={DYNAMIC}
          active={props.dynamic}
          onPick={(id) => props.onDynamic(id as DynamicMode)}
        />
      </section>

      <section>
        <div style={labelStyle}>EQ Mode <Divider /></div>
        <div style={{ display: 'grid', 'grid-template-columns': '1fr 1fr', gap: '8px' }}>
          <For each={eqModes()}>
            {([id, name]) => {
              const active = () => props.eqId === id;
              return (
                <div
                  onClick={() => props.onEq(id)}
                  style={{
                    background: active() ? 'rgba(99,102,241,0.12)' : 'var(--surface-1)',
                    border: active()
                      ? '1px solid rgba(99,102,241,0.5)'
                      : '1px solid var(--border)',
                    'border-radius': '12px',
                    padding: '12px',
                    cursor: 'pointer',
                    transition: 'background 0.15s, border-color 0.15s',
                  }}
                >
                  <div
                    style={{
                      'font-size': '12px',
                      'font-weight': '600',
                      color: active() ? 'var(--accent-soft)' : 'var(--text-3)',
                      'margin-bottom': '8px',
                    }}
                  >
                    {name}
                  </div>
                  <div
                    style={{ display: 'flex', 'align-items': 'flex-end', gap: '3px', height: '28px' }}
                  >
                    <For each={EQ_BARS[id] ?? [50, 50, 50, 50, 50]}>
                      {(h) => (
                        <div
                          style={{
                            flex: '1',
                            height: `${h}%`,
                            background: active() ? 'var(--accent-soft)' : 'var(--text-muted)',
                            'border-radius': '2px',
                            transition: 'background 0.15s',
                          }}
                        />
                      )}
                    </For>
                  </div>
                </div>
              );
            }}
          </For>
        </div>
      </section>
    </div>
  );
}

function Segmented(props: {
  options: { id: string; name: string }[];
  active: string;
  onPick: (id: string) => void;
}) {
  return (
    <div
      style={{
        display: 'flex',
        gap: '6px',
        background: 'var(--surface-1)',
        border: '1px solid var(--border)',
        'border-radius': '12px',
        padding: '4px',
      }}
    >
      <For each={props.options}>
        {({ id, name }) => {
          const active = () => props.active === id;
          return (
            <button
              onClick={() => props.onPick(id)}
              style={{
                flex: '1',
                border: 'none',
                background: active() ? 'rgba(99,102,241,0.18)' : 'transparent',
                color: active() ? 'var(--accent-soft)' : 'var(--text-3)',
                'font-size': '12px',
                'font-weight': '600',
                padding: '9px 0',
                'border-radius': '9px',
                cursor: 'pointer',
                transition: 'background 0.15s, color 0.15s',
              }}
            >
              {name}
            </button>
          );
        }}
      </For>
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
