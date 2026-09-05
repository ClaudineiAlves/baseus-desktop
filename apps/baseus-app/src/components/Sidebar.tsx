import { createEffect, onMount } from 'solid-js';

export type Tab = 'home' | 'anc' | 'sound' | 'gesture' | 'settings';

interface Props {
  active: Tab;
  onSwitch: (tab: Tab) => void;
  updateAvailable?: boolean;
}

const NAV: Array<{ tab: Tab; icon: string; label: string }> = [
  { tab: 'home', icon: '⊙', label: 'Battery' },
  { tab: 'anc',  icon: '◎', label: 'Noise Control' },
  { tab: 'sound', icon: '≋', label: 'Sound' },
  { tab: 'gesture', icon: '☝', label: 'Gestures' },
];

export default function Sidebar(props: Props) {
  const refs: Partial<Record<Tab, HTMLButtonElement>> = {};
  let rail: HTMLDivElement | undefined;
  let marker: HTMLDivElement | undefined;

  function placeMarker() {
    const btn = refs[props.active];
    if (!btn || !rail || !marker) return;
    const top = btn.offsetTop + (btn.offsetHeight - 18) / 2;
    marker.style.top = `${top}px`;
  }

  onMount(placeMarker);
  createEffect(() => {
    // re-run whenever the active tab changes
    props.active;
    placeMarker();
  });

  const btnStyle = (tab: Tab) => ({
    width: '36px',
    height: '36px',
    'border-radius': '9px',
    border: 'none',
    background: props.active === tab ? 'rgba(99,102,241,0.18)' : 'transparent',
    color: props.active === tab ? 'var(--accent-soft)' : 'var(--text-muted)',
    'font-size': '18px',
    cursor: 'pointer',
    display: 'flex',
    'align-items': 'center',
    'justify-content': 'center',
    position: 'relative' as const,
  });

  return (
    <div
      ref={rail}
      style={{
        position: 'relative',
        width: '52px',
        background: 'var(--bg)',
        'border-right': '1px solid var(--border)',
        display: 'flex',
        'flex-direction': 'column',
        'align-items': 'center',
        padding: '10px 0',
        gap: '2px',
        'flex-shrink': '0',
      }}
    >
      {/* Single sliding active indicator */}
      <div
        ref={marker}
        class="rail-marker"
        style={{
          position: 'absolute',
          left: '0',
          top: '0',
          width: '3px',
          height: '18px',
          background: 'var(--accent)',
          'border-radius': '0 3px 3px 0',
        }}
      />

      {NAV.map(({ tab, icon, label }) => (
        <button
          ref={(el) => (refs[tab] = el)}
          class="rail-btn"
          title={label}
          onClick={() => props.onSwitch(tab)}
          style={btnStyle(tab)}
        >
          {icon}
        </button>
      ))}

      <div style={{ flex: '1' }} />

      <button
        ref={(el) => (refs.settings = el)}
        class="rail-btn"
        title="Settings"
        onClick={() => props.onSwitch('settings')}
        style={btnStyle('settings')}
      >
        ⚙
        {props.updateAvailable && (
          <div
            style={{
              position: 'absolute',
              top: '4px',
              right: '4px',
              width: '7px',
              height: '7px',
              'border-radius': '50%',
              background: 'var(--ok)',
              border: '1.5px solid var(--bg)',
            }}
          />
        )}
      </button>
    </div>
  );
}
