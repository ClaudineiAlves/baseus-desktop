# BP1 Pro — Spatial / Dynamic / EQ / Gesture (live capture)

Captured on 2026-09-05 from the vendor Android app (`com.baseus.intelligent`
v2.16.1) by injecting frida-gadget into a repackaged APK and hooking the SDK's
command builder (`ClassicBtManager.b` / `RequestParam.a`), on a real BP1 Pro.
Each opcode was then replayed from Linux and **acked by the earbuds** (RX with
payload `01`).

All frames are RFCOMM, magic `0xBA` (app→device).

## Spatial Audio — opcode `0x43`

`BA 43 <mode>` — mode `00` Normal, `01` Music, `02` Cinema.

> The upstream RE labelled `0x43` as "EQ preset" (Balanced/Bass/Voice/Clear).
> On this firmware `0x43` is the **Spatial Audio** selector; the real EQ is `0x31`.

## Dynamic Sound — opcode `0x92`

`BA 92 <mode> 03` — mode `00` Normal, `01` Bass Boost, `02` Balance. Trailing
`0x03` is constant. Device notifies state back on `0x91`.

## EQ mode — opcode `0x31`

`BA 31 <id> <6 bands × 8 bytes>` — 51-byte frame carrying the full band curve.
Presets (id byte at offset 2):

| id | preset |
|----|--------|
| 00 | Baseus Classic |
| 01 | Deep Bass |
| 03 | Hi-Fi Live |
| 07 | Jazz |
| 08 | Classical |
| 09 | Treble Boost |
| 0a | Acoustic |

The full frames are embedded verbatim in
`crates/baseus-protocol/src/types.rs` (`EqMode::frame`).

## Gesture — opcode `0x8d` (captured, not yet implemented)

`BA 8d <side> <key> <function>` — side `00` left / `01` right; `key` a per-side
gesture id (e.g. `03` = double tap); `function` from the SDK `KeyFunction` enum
(0 None, 1 Recall, 2 Assistant, 3 Prev, 4 Next, 5 Vol+, 6 Vol-, 7 Play/Pause,
8 Game mode, 9 ANC mode). Query is `BA 8c <side> ff`.

## Notes

- `0x92` was previously guessed to be "gesture-related" — that was wrong, it is
  Dynamic Sound.
- Not captured: Dual-Device Connection, custom EQ band editing.
