use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Number(u32),
    Binary { hex: String },
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
pub struct Message {
    pub value: Value,
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
pub struct Meta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gmuid: Option<String>,
    pub rid: String,
}

pub mod raw {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
    #[serde(tag = "status")]
    #[serde(rename_all = "UPPERCASE")]
    pub enum RawMessage {
        #[serde(rename(serialize = "OK", deserialize = "OK"))]
        Ok(super::Message),
        #[serde(rename(serialize = "ERROR", deserialize = "ERR"))]
        Err { err_msg: String },
    }

    #[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
    pub struct Response {
        pub meta: super::Meta,
        pub msg: RawMessage,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    pub meta: Meta,
    pub msg: Result<Message, String>,
}

impl From<raw::Response> for Response {
    fn from(raw: raw::Response) -> Self {
        let raw::Response { meta, msg } = raw;
        let msg = match msg {
            raw::RawMessage::Ok(msg) => Ok(msg),
            raw::RawMessage::Err { err_msg } => Err(err_msg),
        };
        Self { meta, msg }
    }
}

pub fn parse(raw: &[u8]) -> Result<Response, serde_json::Error> {
    println!("raw: {:?}", std::str::from_utf8(raw));
    let raw: raw::Response = serde_json::from_slice(raw)?;
    Ok(raw.into())
}
