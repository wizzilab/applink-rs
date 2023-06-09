pub mod apps;
pub mod modem;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{de, Deserialize, Deserializer, Serialize};

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
