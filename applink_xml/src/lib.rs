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
    UguardController = 0x01BC50C7FF00001F,
    UguardPeripheral = 0x01BC50C7FF000020,
    UguardTag = 0x01BC50C7FF000026,
    UguardSpot = 0x01BC50C7FF000028,
}

impl DeviceType {
    // Returns the device binary name
    // to know where to search for the strbin file
    pub fn bin(&self) -> String {
        match self {
            Self::D7AMote | Self::D7AFileSystem => "wm".to_owned(),
            Self::UguardController => "uguard_controller".to_owned(),
            Self::UguardPeripheral => "uguard_peripheral".to_owned(),
            Self::UguardTag => "uguard_tag".to_owned(),
            Self::UguardSpot => "uguard_spot".to_owned(),
        }
    }
}
