use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(tag = "mode")]
#[serde(rename_all = "lowercase")]
pub enum MqttBridgeTlsConf {
    Ca { capath: String },
}

#[derive(Serialize)]
pub struct MqttBridgeConf {
    pub address: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub active: bool,
    pub tls: Option<MqttBridgeTlsConf>,
}

#[derive(Serialize)]
pub struct ConfUpdate {
    pub mqtt_bridge: MqttBridgeConf,
}

#[derive(Serialize)]
#[serde(tag = "action")]
#[serde(rename_all = "lowercase")]
pub enum GatewayControlCommand {
    Ping {
        uid: String,
    },
    Led {
        uid: String,
        name: String,
        pattern: String,
        period: Option<f32>,
    },
    ConfUpdate {
        uid: String,
        conf: ConfUpdate,
    },
}
