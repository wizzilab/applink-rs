use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    pub uid: String,
    pub rid: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "status")]
#[serde(rename_all = "UPPERCASE")]
pub enum Message {
    Ok,
    Err { err_msg: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub meta: Meta,
    pub msg: Message,
}
