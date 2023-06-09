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
            pub const fn fid() -> u8 {
                $fid
            }
            pub const fn name() -> &'static str {
                $name
            }
        }
    };
}

pub fn de_boolean<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Bool(b) => b,
        serde_json::Value::Number(num) => {
            num.as_i64().ok_or(de::Error::custom("Invalid number"))? != 0
        }
        _ => return Err(de::Error::custom("Wrong type, expected boolean")),
    })
}

#[derive(Debug, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u64)]
pub enum DeviceType {
    UguardController = 0x01BC50C7FF00001F,
    UguardPeripheral = 0x01BC50C7FF000020,
    UguardTag = 0x01BC50C7FF000026,
    UguardSpot = 0x01BC50C7FF000028,
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
