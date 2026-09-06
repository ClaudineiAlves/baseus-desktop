import { createSignal, onMount, For } from 'solid-js';
import { getGestureOptions, setGesture, type GestureOption } from '../lib/tauri';

// Module-level so the picked assignments survive switching tabs (the tab remounts).
// It reflects what we last sent, not a read-back from the earbud.
export const [chosen, setChosen] = createSignal<Record<string, number>>({});

// Apply a full gesture map read back from the earbud (side + [key,func] pairs).
export function applyGestureConfig(side: number, assignments: [number, number][]) {
  const next = { ...chosen() };
  for (const [key, func] of assignments) next[`${side}:${key}`] = func;
  setChosen(next);
}

const SIDES = [
  { id: 0, name: 'Left' },
  { id: 1, name: 'Right' },
];

export default function GestureTab() {
  const [gestures, setGestures] = createSignal<GestureOption[]>([]);

  onMount(async () => {
    setGestures(await getGestureOptions().catch(() => []));
  });

  async function pick(side: number, key: number, func: number) {
    setChosen({ ...chosen(), [`${side}:${key}`]: func });
    await setGesture(side, key, func).catch(() => {});
  }

  return (
    <div style={{ display: 'flex', 'flex-direction': 'column', gap: '18px' }}>
      <div style={hintStyle}>
        Tap types and functions confirmed on this earbud. One Tap only offers None and
        Play / Pause.
      </div>
      <For each={SIDES}>
        {(side) => (
          <section>
            <div style={labelStyle}>
              {side.name} Earbud <Divider />
            </div>
            <div style={{ display: 'flex', 'flex-direction': 'column', gap: '10px' }}>
              <For each={gestures()}>
                {(g) => (
                  <div>
                    <div
                      style={{
                        'font-size': '12px',
                        'font-weight': '600',
                        color: 'var(--text-2)',
                        'margin-bottom': '6px',
                      }}
                    >
                      {g.label}
                    </div>
                    <div style={{ display: 'flex', 'flex-wrap': 'wrap', gap: '6px' }}>
                      <For each={g.functions}>
                        {([fByte, fName]) => {
                          const active = () => chosen()[`${side.id}:${g.key}`] === fByte;
                          return (
                            <button
                              onClick={() => pick(side.id, g.key, fByte)}
                              style={{
                                border: active()
                                  ? '1px solid rgba(99,102,241,0.5)'
                                  : '1px solid var(--border)',
                                background: active()
                                  ? 'rgba(99,102,241,0.14)'
                                  : 'var(--surface-1)',
                                color: active() ? 'var(--accent-soft)' : 'var(--text-3)',
                                'font-size': '11px',
                                'font-weight': '600',
                                padding: '6px 10px',
                                'border-radius': '8px',
                                cursor: 'pointer',
                                transition: 'background 0.15s, border-color 0.15s, color 0.15s',
                              }}
                            >
                              {fName}
                            </button>
                          );
                        }}
                      </For>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </section>
        )}
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
  'margin-bottom': '10px',
};

const hintStyle = {
  'font-size': '10px',
  color: 'var(--text-3)',
  'line-height': '1.5',
};
