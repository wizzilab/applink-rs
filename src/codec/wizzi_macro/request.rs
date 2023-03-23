pub use crate::common::Dash7boardPermission;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum GatewayMode {
    Best,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Request {
    pub site_id: usize,
    pub user_type: Dash7boardPermission,
    pub name: String,
    // TODO What are the different value types supported?
    pub shared_vars: HashMap<String, usize>,
    pub device_uids: Vec<String>,
    pub gateway_mode: GatewayMode,
}

impl Request {
    pub fn encode(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
