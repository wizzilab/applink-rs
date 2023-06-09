use crate::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

//{"uguard_app_status_mode"=>2, "uguard_app_status_errors"=>0, "uguard_app_status_errors_fields"=>{"libex"=>0, "motion_axl"=>0, "motion_mag"=>0, "sensor"=>0, "ext_i2c"=>0, "ext_spi"=>0}, "uguard_app_status_vbat"=>4079}
#[derive(Debug, Deserialize, Serialize)]
pub struct AppStatus {
    #[serde(rename = "uguard_app_status_mode")]
    pub mode: AppMode,
    #[serde(rename = "uguard_app_status_errors")]
    pub errors: u8,
    #[serde(rename = "uguard_app_status_errors_fields")]
    pub errors_fields: AppStatusError,
    #[serde(rename = "uguard_app_status_vbat")]
    pub vbat: u16,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[serde(try_from = "serde_json::Value")]
pub enum AppMode {
    Shelf,
    Maintenance,
    Active,
    Test,
}

impl TryFrom<serde_json::Value> for AppMode {
    type Error = XMLError;
    fn try_from(from: serde_json::Value) -> Result<Self, Self::Error> {
        Ok(match from {
            serde_json::Value::Number(n) => {
                let n = n
                    .as_u64()
                    .ok_or(XMLError::ParseError((file!().to_owned(), line!())))?
                    as u8;
                AppMode::try_from(n)
                    .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?
            }
            _ => return Err(XMLError::ParseError((file!().to_owned(), line!()))),
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppStatusError {
    #[serde(deserialize_with = "de_boolean")]
    pub libex: bool,
    #[serde(deserialize_with = "de_boolean")]
    pub motion_axl: bool,
    #[serde(deserialize_with = "de_boolean")]
    pub motion_mag: bool,
    #[serde(deserialize_with = "de_boolean")]
    pub sensor: bool,
    #[serde(deserialize_with = "de_boolean")]
    pub ext_i2c: bool,
    #[serde(deserialize_with = "de_boolean")]
    pub ext_spi: bool,
}

impl_xml!(AppStatus, 172, "uguard_app_status");
