use serde::{Deserialize, Serialize};

pub mod ble_uuids {
    /// BP1 Pro ANC — confirmed via nRF Connect on physical unit.
    pub const SERVICE: &str = "53527aa4-29f7-ae11-4e74-997334782568";
    pub const WRITE: &str = "ee684b1a-1e9b-ed3e-ee55-f894667e92ac";
    pub const NOTIFY: &str = "654b749c-e37f-ae1f-ebab-40ca133e3690";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatteryState {
    pub left_pct: u8,
    pub right_pct: u8,
    pub left_charging: bool,
    pub right_charging: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AncMode {
    // BP1 Pro ANC — verified on hardware.
    Off,
    Anc,
    Transparency,
}

impl AncMode {
    /// ANC modes supported by a given model (for UI filtering).
    pub fn supported_by(model: BaseusModel) -> &'static [AncMode] {
        match model {
            BaseusModel::Bp1ProAnc => &[AncMode::Off, AncMode::Anc, AncMode::Transparency],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EqPreset {
    Balanced = 0,
    BassBoost = 1,
    Voice = 2,
    Clear = 3,
}

impl EqPreset {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Balanced),
            1 => Some(Self::BassBoost),
            2 => Some(Self::Voice),
            3 => Some(Self::Clear),
            _ => None,
        }
    }

    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

/// Spatial Audio — opcode 0x43. Captured live from the vendor app on a BP1 Pro
/// (firmware 2.16.1): `BA 43 <mode>`. The upstream RE labelled 0x43 as an "EQ preset",
/// but on this device it is the Spatial Audio selector; the real EQ is [`EqMode`] (0x31).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialMode {
    Normal = 0,
    Music = 1,
    Cinema = 2,
}

impl SpatialMode {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Normal),
            1 => Some(Self::Music),
            2 => Some(Self::Cinema),
            _ => None,
        }
    }
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

/// Dynamic Sound — opcode 0x92. Captured as `BA 92 <mode> 03`; the trailing 0x03 is a
/// constant the app always sends.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DynamicMode {
    Normal = 0,
    BassBoost = 1,
    Balance = 2,
}

impl DynamicMode {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Normal),
            1 => Some(Self::BassBoost),
            2 => Some(Self::Balance),
            _ => None,
        }
    }
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

/// EQ mode — opcode 0x31. Each preset is a fixed frame carrying the full band curve,
/// captured verbatim from the vendor app; we replay the exact bytes rather than
/// synthesising the curve. `id` is the preset byte at offset 2 (frame[2]).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EqMode {
    BaseusClassic,
    DeepBass,
    HiFiLive,
    Jazz,
    Classical,
    TrebleBoost,
    Acoustic,
}

impl EqMode {
    pub const ALL: [EqMode; 7] = [
        Self::BaseusClassic,
        Self::DeepBass,
        Self::HiFiLive,
        Self::Jazz,
        Self::Classical,
        Self::TrebleBoost,
        Self::Acoustic,
    ];

    /// The preset id byte (frame[2]).
    pub fn id(self) -> u8 {
        match self {
            Self::BaseusClassic => 0x00,
            Self::DeepBass => 0x01,
            Self::HiFiLive => 0x03,
            Self::Jazz => 0x07,
            Self::Classical => 0x08,
            Self::TrebleBoost => 0x09,
            Self::Acoustic => 0x0a,
        }
    }

    pub fn from_id(id: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.id() == id)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BaseusClassic => "Baseus Classic",
            Self::DeepBass => "Deep Bass",
            Self::HiFiLive => "Hi-Fi Live",
            Self::Jazz => "Jazz",
            Self::Classical => "Classical",
            Self::TrebleBoost => "Treble Boost",
            Self::Acoustic => "Acoustic",
        }
    }

    /// The full 51-byte frame (magic + opcode + id + 6 bands), captured from hardware.
    pub fn frame(self) -> &'static [u8] {
        match self {
            Self::BaseusClassic => &EQ_BASEUS_CLASSIC,
            Self::DeepBass => &EQ_DEEP_BASS,
            Self::HiFiLive => &EQ_HIFI_LIVE,
            Self::Jazz => &EQ_JAZZ,
            Self::Classical => &EQ_CLASSICAL,
            Self::TrebleBoost => &EQ_TREBLE_BOOST,
            Self::Acoustic => &EQ_ACOUSTIC,
        }
    }
}

// Frames captured verbatim over RFCOMM from the BP1 Pro vendor app (firmware 2.16.1).
const EQ_BASEUS_CLASSIC: [u8; 51] = [
    0xba, 0x31, 0x00, 0xbe, 0x00, 0x3e, 0x00, 0x06, 0x00, 0x01, 0x00, 0xec, 0x13, 0x1e, 0x00, 0x28,
    0x00, 0x01, 0x00, 0xf0, 0x0a, 0x64, 0x00, 0x14, 0x00, 0x01, 0x00, 0x28, 0x23, 0x46, 0x00, 0x3c,
    0x00, 0x01, 0x00, 0x82, 0x00, 0x5a, 0x00, 0x05, 0x00, 0x00, 0x00, 0x28, 0x23, 0xaa, 0x00, 0x07,
    0x00, 0x02, 0x00,
];
const EQ_DEEP_BASS: [u8; 51] = [
    0xba, 0x31, 0x01, 0xec, 0x13, 0x32, 0x00, 0x28, 0x00, 0x01, 0x00, 0xf0, 0x0a, 0x64, 0x00, 0x14,
    0x00, 0x01, 0x00, 0x28, 0x23, 0x46, 0x00, 0x3c, 0x00, 0x01, 0x00, 0x82, 0x00, 0x5a, 0x00, 0x05,
    0x00, 0x00, 0x00, 0x28, 0x23, 0xaa, 0x00, 0x07, 0x00, 0x02, 0x00, 0x4c, 0x1d, 0xa0, 0x00, 0x28,
    0x00, 0x01, 0x00,
];
const EQ_HIFI_LIVE: [u8; 51] = [
    0xba, 0x31, 0x03, 0x64, 0x00, 0x0a, 0x00, 0x06, 0x00, 0x01, 0x00, 0x7c, 0x15, 0x1e, 0x00, 0x0a,
    0x00, 0x01, 0x00, 0xf0, 0x0a, 0x64, 0x00, 0x14, 0x00, 0x01, 0x00, 0x34, 0x21, 0x14, 0x00, 0x3c,
    0x00, 0x01, 0x00, 0x82, 0x00, 0x5a, 0x00, 0x05, 0x00, 0x00, 0x00, 0x28, 0x23, 0xaa, 0x00, 0x07,
    0x00, 0x02, 0x00,
];
const EQ_JAZZ: [u8; 51] = [
    0xba, 0x31, 0x07, 0xbe, 0x00, 0x5a, 0x00, 0x06, 0x00, 0x01, 0x00, 0xec, 0x13, 0x32, 0x00, 0x28,
    0x00, 0x01, 0x00, 0xf0, 0x0a, 0x8c, 0x00, 0x07, 0x00, 0x01, 0x00, 0x28, 0x23, 0x46, 0x00, 0x3c,
    0x00, 0x01, 0x00, 0x82, 0x00, 0x5a, 0x00, 0x05, 0x00, 0x00, 0x00, 0x28, 0x23, 0xaa, 0x00, 0x07,
    0x00, 0x02, 0x00,
];
const EQ_CLASSICAL: [u8; 51] = [
    0xba, 0x31, 0x08, 0xbe, 0x00, 0x5a, 0x00, 0x06, 0x00, 0x01, 0x00, 0xec, 0x13, 0x32, 0x00, 0x28,
    0x00, 0x01, 0x00, 0xb8, 0x0b, 0x82, 0x00, 0x07, 0x00, 0x01, 0x00, 0x28, 0x23, 0x46, 0x00, 0x3c,
    0x00, 0x01, 0x00, 0x82, 0x00, 0x5a, 0x00, 0x05, 0x00, 0x00, 0x00, 0x28, 0x23, 0xaa, 0x00, 0x07,
    0x00, 0x02, 0x00,
];
const EQ_TREBLE_BOOST: [u8; 51] = [
    0xba, 0x31, 0x09, 0xbe, 0x00, 0x3e, 0x00, 0x06, 0x00, 0x01, 0x00, 0xec, 0x13, 0x32, 0x00, 0x28,
    0x00, 0x01, 0x00, 0xf0, 0x0a, 0x64, 0x00, 0x14, 0x00, 0x01, 0x00, 0x28, 0x23, 0x46, 0x00, 0x3c,
    0x00, 0x01, 0x00, 0x82, 0x00, 0x5a, 0x00, 0x05, 0x00, 0x00, 0x00, 0xb8, 0x0b, 0xaa, 0x00, 0x07,
    0x00, 0x02, 0x00,
];
const EQ_ACOUSTIC: [u8; 51] = [
    0xba, 0x31, 0x0a, 0xbe, 0x00, 0x3e, 0x00, 0x06, 0x00, 0x01, 0x00, 0xec, 0x13, 0x32, 0x00, 0x28,
    0x00, 0x01, 0x00, 0xf0, 0x0a, 0x8c, 0x00, 0x0a, 0x00, 0x01, 0x00, 0x28, 0x23, 0x46, 0x00, 0x3c,
    0x00, 0x01, 0x00, 0x82, 0x00, 0x3c, 0x00, 0x05, 0x00, 0x00, 0x00, 0x28, 0x23, 0xaa, 0x00, 0x07,
    0x00, 0x02, 0x00,
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WearState {
    pub left_in_ear: bool,
    pub right_in_ear: bool,
}

/// Which earbud a gesture belongs to. Wire byte in `BA 8D <side> …`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GestureSide {
    Left = 0,
    Right = 1,
}

impl GestureSide {
    pub fn to_byte(self) -> u8 {
        self as u8
    }
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Left),
            1 => Some(Self::Right),
            _ => None,
        }
    }
}

/// The tap/press a gesture responds to. Wire byte in `BA 8D <side> <key> <func>`,
/// all four confirmed live from the vendor app (not the SDK KeyType enum, which
/// numbers them differently).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GestureKey {
    TripleTap = 0x00,
    LongPress = 0x01,
    TapHold = 0x02,
    DoubleTap = 0x03,
}

impl GestureKey {
    pub const ALL: [GestureKey; 4] = [
        Self::DoubleTap,
        Self::TripleTap,
        Self::LongPress,
        Self::TapHold,
    ];
    pub fn to_byte(self) -> u8 {
        self as u8
    }
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(Self::TripleTap),
            0x01 => Some(Self::LongPress),
            0x02 => Some(Self::TapHold),
            0x03 => Some(Self::DoubleTap),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::DoubleTap => "Double Tap",
            Self::TripleTap => "Triple Tap",
            Self::LongPress => "Long Press",
            Self::TapHold => "Tap & Hold",
        }
    }
}

/// What a gesture does. Wire bytes confirmed live from the vendor app (this is the
/// app's own gesture-function table, not the SDK KeyFunction enum). Only the values
/// verified on hardware are listed; the app offers a few more (Play/Pause, Volume
/// Down, Assistant) whose bytes were not captured.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GestureFunction {
    None = 0x00,
    Next = 0x01,
    Previous = 0x02,
    AncMode = 0x06,
    VolumeUp = 0x0b,
}

impl GestureFunction {
    pub const ALL: [GestureFunction; 5] = [
        Self::None,
        Self::Next,
        Self::Previous,
        Self::VolumeUp,
        Self::AncMode,
    ];
    pub fn to_byte(self) -> u8 {
        self as u8
    }
    pub fn from_byte(b: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.to_byte() == b)
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Next => "Next",
            Self::Previous => "Previous",
            Self::AncMode => "ANC Mode",
            Self::VolumeUp => "Volume Up",
        }
    }
}

/// Events emitted from the device to the app (via Tauri `device-event`).
/// Serialised as `{ "type": "<variant>", "data": <payload> }` for TypeScript.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum DeviceEvent {
    BatteryUpdate(BatteryState),
    CaseUpdate(CaseState),
    AncModeUpdate(AncMode),
    /// Game/low-latency mode — independent toggle, not a mutually-exclusive ANC state.
    GameModeUpdate(bool),
    WearUpdate(WearState),
    EqPresetUpdate(EqPreset),
    SpatialModeUpdate(SpatialMode),
    DynamicModeUpdate(DynamicMode),
    EqModeUpdate(EqMode),
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseState {
    pub case_pct: u8,
    pub case_charging: bool,
}

/// Registry of supported Baseus models.
///
/// Only hardware-verified models live here. The enum is intentionally kept as a
/// registry (rather than flattened to BP1-only) so future owner-contributed,
/// verified models can be added without reworking the dispatch structure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BaseusModel {
    Bp1ProAnc,
}

impl BaseusModel {
    pub fn all() -> &'static [BaseusModel] {
        &[BaseusModel::Bp1ProAnc]
    }

    pub fn display_name(self) -> &'static str {
        match self {
            BaseusModel::Bp1ProAnc => "Bass BP1 Pro ANC",
        }
    }

    /// BLE advertising name(s) used to identify this device during scan.
    /// Includes short-form aliases for devices that omit the model suffix.
    ///
    /// Treat these as a hint, not as the primary key: the name travels in the scan
    /// response, so BlueZ reports it only intermittently and `local_name` is often
    /// `None` for a device that is plainly there. Match on `service_uuid` first.
    pub fn advertising_names(self) -> &'static [&'static str] {
        match self {
            BaseusModel::Bp1ProAnc => &["Bass BP1 Pro"],
        }
    }

    /// GATT service UUID advertised by this model. Unlike the advertising name,
    /// this is always present in the advertisement payload, which makes it the only
    /// reliable way to spot the device on BlueZ (see `advertising_names`).
    pub fn service_uuid(self) -> &'static str {
        match self {
            BaseusModel::Bp1ProAnc => ble_uuids::SERVICE,
        }
    }

    /// GATT (notify_uuid, write_uuid) for BLE control.
    /// Confirmed via nRF Connect on a physical unit.
    pub fn gatt_uuids(self) -> (&'static str, &'static str) {
        match self {
            BaseusModel::Bp1ProAnc => (ble_uuids::NOTIFY, ble_uuids::WRITE),
        }
    }

    /// Look up the model from a BLE advertising name seen during a scan.
    /// Matching is case-insensitive to handle firmware variations.
    pub fn from_advertising_name(name: &str) -> Option<Self> {
        Self::all()
            .iter()
            .find(|m| {
                m.advertising_names()
                    .iter()
                    .any(|n| n.eq_ignore_ascii_case(name))
            })
            .copied()
    }
}
