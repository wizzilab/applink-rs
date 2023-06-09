pub mod apps;
pub mod modem;

use serde::{de, Deserialize, Deserializer};

pub trait ApplinkXML {
    fn fid() -> u8;
    fn name() -> String;
}

#[macro_export]
macro_rules! impl_xml {
    ($xml:ident, $fid:literal, $name:literal) => {
        /// Implement the File trait for $file
        impl ApplinkXML for $xml {
            fn fid() -> u8 {
                $fid
            }
            fn name() -> String {
                $name.to_owned()
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
