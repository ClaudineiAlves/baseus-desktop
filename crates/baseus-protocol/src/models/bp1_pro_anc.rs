use crate::{
    models::DecodeError,
    types::{
        AncMode, BatteryState, CaseState, DeviceEvent, DynamicMode, EqMode, GestureConfig,
        SpatialMode,
    },
    Frame,
};

pub struct Bp1ProAnc;

impl Bp1ProAnc {
    /// Decode a GATT notification frame from the BP1 Pro ANC.
    ///
    /// Wire format confirmed via nRF Connect on BASS BP1 PRO (4A:01:CE:BA:C8:03):
    ///   AA 02 L% L_chg R% R_chg   → battery report (left + right buds)
    ///   AA 23 [00|01]              → game mode state (issue #3, community-verified)
    ///   AA 30 …                    → ANC off
    ///   AA 32 …                    → Transparency mode
    ///   AA 33 …                    → ANC active
    ///   AA 80 …                    → Case/connection event (partially decoded)
    pub fn decode_frame(frame: &Frame) -> Result<DeviceEvent, DecodeError> {
        match frame.cmd {
            0x02 => Self::decode_battery(&frame.payload),
            // Game/low-latency mode state: AA 23 01 = on, AA 23 00 = off (issue #3).
            // The companion AA 24 01 is a flat "command received" ack carrying no
            // state — it intentionally falls through to UnknownOpcode below.
            0x23 => Ok(DeviceEvent::GameModeUpdate(
                *frame.payload.first().unwrap_or(&0) != 0,
            )),
            0x27 => Self::decode_case(&frame.payload),
            // EQ mode state (query response). Live capture: AA 30 <preset_id>. The old
            // RE read 0x30 as an ANC keepalive; on this firmware it carries the EQ preset.
            0x30 => {
                let id = *frame.payload.first().unwrap_or(&0);
                EqMode::from_id(id)
                    .map(DeviceEvent::EqModeUpdate)
                    .ok_or(DecodeError::UnknownOpcode(0x30))
            }
            // ANC state (query response): AA 33 <mode> <level>. mode 0=off, 1=anc,
            // 2=transparency; level is the strength byte. The old RE hardcoded Anc and
            // dropped the level, so the UI never reflected the saved strength.
            0x32 | 0x33 => {
                let mode = match frame.payload.first().copied().unwrap_or(1) {
                    0 => AncMode::Off,
                    2 => AncMode::Transparency,
                    _ => AncMode::Anc,
                };
                let level = frame.payload.get(1).copied().unwrap_or(0);
                Ok(DeviceEvent::AncStateUpdate { mode, level })
            }
            // Gesture map for one earbud (AA 8C query response):
            // 8C <side> [<key> <func>]... — four key/func pairs.
            0x8C => {
                let side = *frame.payload.first().unwrap_or(&0);
                let assignments = frame.payload[1..]
                    .chunks(2)
                    .filter(|c| c.len() == 2)
                    .map(|c| (c[0], c[1]))
                    .collect();
                Ok(DeviceEvent::GestureConfigUpdate(GestureConfig {
                    side,
                    assignments,
                }))
            }
            // Spatial Audio state (query response): AA 42 <mode>. Set is 0x43; the old RE
            // mislabelled this pair as "EQ preset".
            0x42 => {
                let m = *frame.payload.first().unwrap_or(&0);
                SpatialMode::from_byte(m)
                    .map(DeviceEvent::SpatialModeUpdate)
                    .ok_or(DecodeError::UnknownOpcode(0x42))
            }
            // Dynamic Sound state (query response): AA 91 <mode> [03].
            0x91 => {
                let m = *frame.payload.first().unwrap_or(&0);
                DynamicMode::from_byte(m)
                    .map(DeviceEvent::DynamicModeUpdate)
                    .ok_or(DecodeError::UnknownOpcode(0x91))
            }
            other => Err(DecodeError::UnknownOpcode(other)),
        }
    }

    /// Resolve an `AA 34` ANC ack into an ANC state.
    ///
    /// Firmware variants differ here (issue #3): some units echo the mode in the
    /// payload (`00` = off, non-zero = active), while others ack every ANC command
    /// with a flat `AA 34 01` — even for Off. A zero payload therefore always means
    /// Off, but a non-zero payload only confirms whatever mode was last commanded.
    pub fn resolve_anc_ack(payload: &[u8], last_commanded: Option<AncMode>) -> AncMode {
        if payload.first().copied().unwrap_or(0) == 0 {
            AncMode::Off
        } else {
            last_commanded.unwrap_or(AncMode::Anc)
        }
    }

    fn decode_battery(payload: &[u8]) -> Result<DeviceEvent, DecodeError> {
        // Confirmed live: AA 02 64 00 64 01 = left 100%, right 100% (both in ear).
        // Frame structure: [left_pct, 0x00, right_pct, 0x01]
        // Bytes 1 and 3 are fixed bud-ID markers (0x00=left, 0x01=right), NOT charging flags.
        // Charging state is not present in this frame; set false until a charging frame is found.
        if payload.len() < 4 {
            return Err(DecodeError::PayloadTooShort {
                opcode: 0x02,
                need: 4,
                got: payload.len(),
            });
        }
        Ok(DeviceEvent::BatteryUpdate(BatteryState {
            left_pct: payload[0],
            left_charging: false,
            right_pct: payload[2],
            right_charging: false,
        }))
    }

    fn decode_case(payload: &[u8]) -> Result<DeviceEvent, DecodeError> {
        // Confirmed live: AA 27 32 00 = case 50%, not charging.
        // payload[0] = case_pct, payload[1] = charging flag (0x00=no, 0x01=yes).
        if payload.len() < 2 {
            return Err(DecodeError::PayloadTooShort {
                opcode: 0x27,
                need: 2,
                got: payload.len(),
            });
        }
        Ok(DeviceEvent::CaseUpdate(CaseState {
            case_pct: payload[0],
            case_charging: payload[1] != 0,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Frame;

    fn decode(raw: &[u8]) -> Result<DeviceEvent, DecodeError> {
        Bp1ProAnc::decode_frame(&Frame::decode(raw).unwrap())
    }

    #[test]
    fn battery_frame_decodes_correctly() {
        // Golden: AA 02 64 00 64 01 — captured live, both buds in ear at 100%.
        // Bytes [1] and [3] are bud-ID markers (0x00=left, 0x01=right), not charging flags.
        let ev = decode(&[0xAA, 0x02, 0x64, 0x00, 0x64, 0x01]).unwrap();
        assert_eq!(
            ev,
            DeviceEvent::BatteryUpdate(BatteryState {
                left_pct: 100,
                left_charging: false,
                right_pct: 100,
                right_charging: false,
            })
        );
    }

    #[test]
    fn eq_state_baseus_classic_from_0x30_00() {
        // AA 30 00 was read as an ANC keepalive by the old RE; it is actually the EQ
        // mode query response, preset id 0 = Baseus Classic.
        use crate::types::EqMode;
        assert_eq!(
            decode(&[0xAA, 0x30, 0x00]).unwrap(),
            DeviceEvent::EqModeUpdate(EqMode::BaseusClassic)
        );
    }

    #[test]
    fn anc_transparency_decodes_correctly() {
        // AA 32/33 with mode 2 = transparency.
        assert_eq!(
            decode(&[0xAA, 0x33, 0x02, 0xFF]).unwrap(),
            DeviceEvent::AncStateUpdate {
                mode: AncMode::Transparency,
                level: 0xFF
            }
        );
    }

    #[test]
    fn anc_state_decodes_mode_and_level() {
        // AA 33 <mode> <level> — query response carries the strength level.
        assert_eq!(
            decode(&[0xAA, 0x33, 0x01, 0x68]).unwrap(),
            DeviceEvent::AncStateUpdate {
                mode: AncMode::Anc,
                level: 0x68
            }
        );
        assert_eq!(
            decode(&[0xAA, 0x33, 0x00, 0x00]).unwrap(),
            DeviceEvent::AncStateUpdate {
                mode: AncMode::Off,
                level: 0x00
            }
        );
    }

    #[test]
    fn gesture_config_decodes() {
        use crate::types::GestureConfig;
        // AA 8C <side> [<key> <func>]*4
        assert_eq!(
            decode(&[0xAA, 0x8C, 0x00, 0x00, 0x04, 0x03, 0x01]).unwrap(),
            DeviceEvent::GestureConfigUpdate(GestureConfig {
                side: 0,
                assignments: vec![(0x00, 0x04), (0x03, 0x01)],
            })
        );
    }

    #[test]
    fn battery_too_short_is_error() {
        let frame = Frame {
            cmd: 0x02,
            payload: vec![0x64, 0x00, 0x5A],
        };
        assert!(matches!(
            Bp1ProAnc::decode_frame(&frame),
            Err(DecodeError::PayloadTooShort {
                opcode: 0x02,
                need: 4,
                got: 3
            })
        ));
    }

    #[test]
    fn case_frame_decodes_correctly() {
        // Golden: AA 27 32 00 — captured live, case at 50%, not charging.
        let ev = decode(&[0xAA, 0x27, 0x32, 0x00]).unwrap();
        assert_eq!(
            ev,
            DeviceEvent::CaseUpdate(CaseState {
                case_pct: 50,
                case_charging: false
            })
        );
    }

    #[test]
    fn unknown_opcode_is_error() {
        let frame = Frame {
            cmd: 0x99,
            payload: vec![],
        };
        assert!(matches!(
            Bp1ProAnc::decode_frame(&frame),
            Err(DecodeError::UnknownOpcode(0x99))
        ));
    }

    #[test]
    fn spatial_state_decodes() {
        // AA 42 <mode> — query response for Spatial Audio.
        use crate::types::SpatialMode;
        assert_eq!(
            decode(&[0xAA, 0x42, 0x00]).unwrap(),
            DeviceEvent::SpatialModeUpdate(SpatialMode::Normal)
        );
        assert_eq!(
            decode(&[0xAA, 0x42, 0x02]).unwrap(),
            DeviceEvent::SpatialModeUpdate(SpatialMode::Cinema)
        );
    }

    #[test]
    fn dynamic_state_decodes() {
        // AA 91 <mode> [03] — query response for Dynamic Sound.
        use crate::types::DynamicMode;
        assert_eq!(
            decode(&[0xAA, 0x91, 0x01, 0x03]).unwrap(),
            DeviceEvent::DynamicModeUpdate(DynamicMode::BassBoost)
        );
    }

    #[test]
    fn eq_state_decodes() {
        // AA 30 <preset_id> — query response for EQ mode.
        use crate::types::EqMode;
        assert_eq!(
            decode(&[0xAA, 0x30, 0x07]).unwrap(),
            DeviceEvent::EqModeUpdate(EqMode::Jazz)
        );
        assert_eq!(
            decode(&[0xAA, 0x30, 0x00]).unwrap(),
            DeviceEvent::EqModeUpdate(EqMode::BaseusClassic)
        );
    }

    #[test]
    fn game_mode_state_on_decodes() {
        // Issue #3: AA 23 01 — state confirmation after BA 24 01 (game mode on)
        let ev = decode(&[0xAA, 0x23, 0x01]).unwrap();
        assert_eq!(ev, DeviceEvent::GameModeUpdate(true));
    }

    #[test]
    fn game_mode_state_off_decodes() {
        // Issue #3: AA 23 00 — state confirmation after BA 24 00 (game mode off)
        let ev = decode(&[0xAA, 0x23, 0x00]).unwrap();
        assert_eq!(ev, DeviceEvent::GameModeUpdate(false));
    }

    #[test]
    fn game_mode_generic_ack_is_ignored() {
        // Issue #3: AA 24 01 is a flat "command received" ack (payload is 01 even
        // for game-mode-off) — it carries no state and must never decode as one.
        assert!(matches!(
            decode(&[0xAA, 0x24, 0x01]),
            Err(DecodeError::UnknownOpcode(0x24))
        ));
    }

    #[test]
    fn anc_ack_zero_payload_resolves_off() {
        assert_eq!(
            Bp1ProAnc::resolve_anc_ack(&[0x00], Some(AncMode::Anc)),
            AncMode::Off
        );
    }

    #[test]
    fn anc_ack_flat_nonzero_after_off_command_resolves_off() {
        // Issue #3: some firmware acks every ANC command with AA 34 01 — even Off.
        // A non-zero payload must not override a just-commanded Off.
        assert_eq!(
            Bp1ProAnc::resolve_anc_ack(&[0x01], Some(AncMode::Off)),
            AncMode::Off
        );
    }

    #[test]
    fn anc_ack_nonzero_confirms_last_commanded_mode() {
        assert_eq!(
            Bp1ProAnc::resolve_anc_ack(&[0x01], Some(AncMode::Transparency)),
            AncMode::Transparency
        );
    }

    #[test]
    fn anc_ack_nonzero_with_no_history_defaults_to_anc() {
        assert_eq!(Bp1ProAnc::resolve_anc_ack(&[0x01], None), AncMode::Anc);
    }
}
