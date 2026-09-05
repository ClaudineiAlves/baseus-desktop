import { createSignal, onMount, For } from 'solid-js';
import { getGestureOptions, setGesture } from '../lib/tauri';

type Opt = [number, string];
type Side = { id: number; name: string };

const SIDES: Side[] = [
  { id: 0, name: 'Left' },
  { id: 1, name: 'Right' },
];

// One assignment cell keeps its own selected function per (side, key); the earbud
// stores it, so this is the last value we sent rather than a read-back.
export default function GestureTab() {
  const [keys, setKeys] = createSignal<Opt[]>([]);
  const [funcs, setFuncs] = createSignal<Opt[]>([]);
  const [chosen, setChosen] = createSignal<Record<string, number>>({});

  onMount(async () => {
    const [k, f] = await getGestureOptions().catch(() => [[], []] as [Opt[], Opt[]]);
    setKeys(k);
    setFuncs(f);
  });

  async function pick(side: number, key: number, func: number) {
    setChosen({ ...chosen(), [`${side}:${key}`]: func });
    await setGesture(side, key, func).catch(() => {});
  }

  return (
    <div style={{ display: 'flex', 'flex-direction': 'column', gap: '18px' }}>
      <div style={hintStyle}>
        Double Tap is confirmed on hardware; the other gestures follow the vendor SDK's
        key table.
      </div>
      <For each={SIDES}>
        {(side) => (
          <section>
            <div style={labelStyle}>
              {side.name} Earbud <Divider />
            </div>
            <div style={{ display: 'flex', 'flex-direction': 'column', gap: '10px' }}>
              <For each={keys()}>
                {([keyByte, keyName]) => (
                  <div>
                    <div
                      style={{
                        'font-size': '12px',
                        'font-weight': '600',
                        color: 'var(--text-2)',
                        'margin-bottom': '6px',
                      }}
                    >
                      {keyName}
                    </div>
                    <div style={{ display: 'flex', 'flex-wrap': 'wrap', gap: '6px' }}>
                      <For each={funcs()}>
                        {([fByte, fName]) => {
                          const active = () => chosen()[`${side.id}:${keyByte}`] === fByte;
                          return (
                            <button
                              onClick={() => pick(side.id, keyByte, fByte)}
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
