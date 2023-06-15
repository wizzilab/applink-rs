pub mod apps;
pub mod modem;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;

#[macro_export]
macro_rules! impl_xml {
    ($xml:ident, $fid:literal, $name:literal) => {
        /// Implement the File trait for $file
        impl $xml {
            pub const fn fid() -> &'static u8 {
                &$fid
            }
            pub const fn name() -> &'static str {
                $name
            }
            pub const fn file() -> (&'static u8, &'static str) {
                (&$fid, $name)
            }
        }
    };
}

pub fn de_boolean<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Bool(b) => b,
        serde_json::Value::Number(n) => n.as_i64().ok_or(de::Error::custom("Invalid number"))? != 0,
        _ => return Err(de::Error::custom("Wrong type, expected boolean")),
    })
}

pub fn de_character<'de, D: Deserializer<'de>>(deserializer: D) -> Result<char, D::Error> {
    Ok(match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Number(n) => {
            // Extract as u64
            let n = n.as_u64().ok_or(de::Error::custom("Invalid number"))?;

            // Convert to char
            char::from_u32(n as u32).ok_or(de::Error::custom("Invalid character"))?
        }
        serde_json::Value::Object(s) => {
            // Extract "hex" key
            let s = s.get("hex").ok_or(de::Error::custom("No hex key"))?;

            // Convert to slice
            let s = s
                .as_str()
                .ok_or(de::Error::custom("Failed converting to slice"))?;

            // Decode hexadecimal slice
            let s = hex::decode(s).map_err(|_| de::Error::custom("Failed to decode hex"))?;

            // Extract first byte
            let s = *s.first().ok_or(de::Error::custom("Empty"))?;

            // Convert to char
            char::from_u32(s as u32).ok_or(de::Error::custom("Invalid character"))?
        }
        _ => return Err(de::Error::custom("Wrong type, expected char")),
    })
}

pub fn de_string_u64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    Ok(match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Object(s) => {
            let mut raw: [u8; 8] = [0; 8];

            // Extract "utf8" key
            let s = s.get("utf8").ok_or(de::Error::custom("No utf8 key"))?;

            // Convert to slice
            let s = s
                .as_str()
                .ok_or(de::Error::custom("Failed converting to slice"))?;

            // Decode hexadecimal slice
            hex::decode_to_slice(s, &mut raw)
                .map_err(|_| de::Error::custom("Failed to decode hex"))?;

            // To u64
            u64::from_be_bytes(raw)
        }
        _ => return Err(de::Error::custom("Wrong type, expected string")),
    })
}

#[derive(Debug, Clone)]
pub enum XMLError {
    ParseError((String, u32)),
}

impl fmt::Display for XMLError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        match self {
            Self::ParseError((file, line)) => write!(f, "{}:{}", file, line),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u64)]
pub enum DeviceType {
    D7AMote = 0x01BC50C700001001,
    D7AFileSystem = 0x01BC50C700001000,
    GatewayHost = 0x01BC50C710000000,
    GatewaySecondaryModem = 0x01BC50C710000001,
    WBeacon = 0x01BC50C70000003A,
    Wisense2 = 0x01BC50C7FF000000,
    Wisp = 0x01BC50C7FF000009,
    WispLight = 0x01BC50C7FF000023,
    Wult = 0x01BC50C700000032,
    WoltUWBTag = 0x01BC50C7FF00001C,
    WoltUWBAnchor = 0x01BC50C7FF00001D,
    WoltMeter = 0x01BC50C7FF000022,
    UguardController = 0x01BC50C7FF00001F,
    UguardPeripheral = 0x01BC50C7FF000020,
    UguardTag = 0x01BC50C7FF000026,
    UguardSpot = 0x01BC50C7FF000028,
    MotionConnect = 0x01BC50C7FF00002A,
    AirConnect = 0x01BC50C7FF00002B,
    BLEBeaconEddystone = 0x01BC50C7FF00002D,
    BLEBeaconiBeacon = 0x01BC50C7FF00002E,
    AiforsiteAnchor = 0x01BC50C7FF000017,
    LucyTrot = 0x01BC50C7FF007307,

    // Non WizziLab
    WFITag = 0x5A75160477F10000,
    OS200 = 0x0A3EF31F00000200,
    OS300 = 0x0A3EF31F00000300,
    OS110 = 0x0A3EF31F00000400,
}

impl DeviceType {
    // Returns the device app name
    // for searching for strbin files
    pub fn app(&self) -> Option<String> {
        match self {
            Self::D7AMote | Self::D7AFileSystem => Some("wm".to_owned()),
            Self::GatewaySecondaryModem => Some("gw".to_owned()),
            Self::Wult => Some("wult".to_owned()),
            Self::WoltUWBTag => Some("wolt_uwb_tag".to_owned()),
            Self::WoltUWBAnchor => Some("wolt_uwb_anchor".to_owned()),
            Self::WoltMeter => Some("wolt_uwb_tag".to_owned()), // XXX No dedicated app yet
            Self::UguardController => Some("uguard_controller".to_owned()),
            Self::UguardPeripheral => Some("uguard_peripheral".to_owned()),
            Self::UguardTag => Some("uguard_tag".to_owned()),
            Self::UguardSpot => Some("uguard_spot".to_owned()),
            Self::AirConnect => Some("air_connect".to_owned()),
            Self::MotionConnect => Some("motion_connect".to_owned()),
            Self::WBeacon => Some("wbeacon".to_owned()),
            Self::Wisense2 => Some("ws".to_owned()),
            Self::Wisp => Some("wisp".to_owned()),
            Self::WispLight => Some("wisp_light".to_owned()),
            Self::WFITag => Some("wfi_tag".to_owned()),
            Self::AiforsiteAnchor => Some("aiforsite_anchor".to_owned()),
            Self::OS200 => Some("os200".to_owned()),
            Self::OS300 => Some("os300".to_owned()),
            Self::OS110 => Some("os110".to_owned()),
            Self::LucyTrot => Some("lucy_trot".to_owned()),
            Self::BLEBeaconiBeacon | Self::BLEBeaconEddystone | Self::GatewayHost => None,
        }
    }
}
