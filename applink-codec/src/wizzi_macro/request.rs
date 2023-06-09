pub use crate::{permission::Dash7boardPermission, uid::Uid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wizzi_common::json;

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
    pub device_vars: HashMap<Uid, HashMap<String, usize>>,
    pub device_uids: Vec<String>,
    pub gateway_mode: GatewayMode,
}

impl Request {
    pub fn encode(&self) -> Result<String, json::EncodingError<Self>> {
        json::to_string(self)
    }
}
