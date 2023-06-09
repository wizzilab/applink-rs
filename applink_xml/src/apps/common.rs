use crate::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

// {"sys_op_mode"=>0, "sys_boot_cause"=>"41", "sys_assert_count"=>1, "sys_last_assert"=>2199215613, "sys_last_assert_arg"=>0}
#[derive(Debug, Deserialize, Serialize)]
pub struct WmSys {
    pub sys_op_mode: WmSysOpMode,
    #[serde(rename = "sys_boot_cause")]
    pub boot_cause: char,
    #[serde(rename = "sys_assert_count")]
    pub assert_count: u16,
    #[serde(rename = "sys_last_assert")]
    pub last_assert: u32,
    #[serde(rename = "sys_last_assert_arg")]
    pub last_assert_arg: u32,
}

#[derive(Debug, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum WmSysOpMode {
    Good,
    NoApp,
    NoLibex,
    NoFs,
    NoModem,
}

impl_xml!(WmSys, 255, "sys_status");
