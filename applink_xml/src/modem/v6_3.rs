use crate::*;
use serde::{Deserialize, Serialize};

//{"code"=>"30314243353043373030303031303031", "hwv"=>2118404, "fwid"=>144, "fwmaj"=>6, "fwmin"=>3, "fwp"=>300, "fwh"=>2144575652, "maxsize"=>76544}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModemRevision {
    pub code: String,
    pub hwv: u32,
    pub fwid: u8,
    pub fwmaj: u8,
    pub fwmin: u8,
    pub fwp: u16,
    pub fwh: u32,
    pub maxsize: u32,
}

impl_xml!(ModemRevision, 2, "modem_version");

//{"code"=>"30314243353043374646303030303146", "hwv"=>3346433, "fwid"=>131, "fwmaj"=>0, "fwmin"=>6, "fwp"=>115, "fwh"=>477821715, "maxsize"=>163840}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HostRevision {
    pub code: String,
    pub hwv: u32,
    pub fwid: u8,
    pub fwmaj: u8,
    pub fwmin: u8,
    pub fwp: u16,
    pub fwh: u32,
    pub maxsize: u32,
}

impl_xml!(HostRevision, 65, "host_version");

// {"last_assert"=>0, "last_assert_arg"=>0, "assert_count"=>0, "host_present"=>1, "rst_cause"=>80, "active_itf"=>1, "active_itf_fields"=>{"hst"=>1, "com"=>0, "dbg"=>0, "d7a"=>0, "lwan"=>0}}
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct WmDebug {
    pub last_assert: u32,
    pub last_assert_arg: u32,
    pub assert_count: u16,
    #[serde(deserialize_with = "de_boolean")]
    pub host_present: bool,
    #[serde(rename = "rst_cause")]
    #[serde(deserialize_with = "de_char")]
    pub boot_cause: char,
    pub active_itf: Option<u32>,
    pub active_itf_fields: Option<ActiveItf>,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct ActiveItf {
    #[serde(deserialize_with = "de_boolean")]
    pub hst: bool,
    #[serde(deserialize_with = "de_boolean")]
    pub com: bool,
    #[serde(deserialize_with = "de_boolean")]
    pub dbg: bool,
    #[serde(deserialize_with = "de_boolean")]
    pub d7a: bool,
    #[serde(deserialize_with = "de_boolean")]
    pub lwan: bool,
}

impl_xml!(WmDebug, 72, "wm_debug");
