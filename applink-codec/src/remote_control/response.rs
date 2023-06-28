use serde::{Deserialize, Serialize};
use wizzi_common::json;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
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
    #[serde(untagged)]
    pub enum Value {
        Number(u32),
        Binary { hex: String },
    }

    #[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
    pub struct Message {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<Value>,
    }

    #[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
    #[serde(tag = "status")]
    #[serde(rename_all = "UPPERCASE")]
    pub enum RawMessage {
        #[serde(rename(serialize = "OK", deserialize = "OK"))]
        Ok(Message),
        #[serde(rename(serialize = "ERROR", deserialize = "ERR"))]
        Err { err_msg: String },
    }

    #[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
    pub struct Response {
        pub meta: super::Meta,
        pub msg: RawMessage,
    }

    #[test]
    fn write_response() {
        #![allow(clippy::unwrap_used)]
        let response = "{\"meta\":{\"uid\":\"001BC50C71006FD7\",\"guid\":\"001BC50C71004102\",\"gmuid\":\"001BC50C71004102\",\"rid\":\"0-1\"},\"msg\":{\"status\":\"OK\"}}";
        let raw: Response = serde_json::from_str(response).unwrap();
        assert_eq!(
            raw,
            Response {
                meta: super::Meta {
                    uid: Some("001BC50C71006FD7".to_string()),
                    guid: Some("001BC50C71004102".to_string()),
                    gmuid: Some("001BC50C71004102".to_string()),
                    rid: "0-1".to_string(),
                },
                msg: RawMessage::Ok(Message { value: None }),
            }
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(u32),
    Binary(Vec<u8>),
}

impl TryFrom<raw::Value> for Value {
    type Error = hex::FromHexError;

    fn try_from(raw: raw::Value) -> Result<Self, Self::Error> {
        match raw {
            raw::Value::Number(n) => Ok(Value::Number(n)),
            raw::Value::Binary { hex } => Ok(Value::Binary(hex::decode(hex)?)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub value: Option<Value>,
}

impl TryFrom<raw::Message> for Message {
    type Error = hex::FromHexError;

    fn try_from(raw: raw::Message) -> Result<Self, Self::Error> {
        let raw::Message { value } = raw;
        let value = match value {
            Some(value) => Some(value.try_into()?),
            None => None,
        };
        Ok(Self { value })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    pub meta: Meta,
    pub msg: Result<Message, String>,
}

impl TryFrom<raw::Response> for Response {
    type Error = hex::FromHexError;
    fn try_from(raw: raw::Response) -> Result<Self, Self::Error> {
        let raw::Response { meta, msg } = raw;
        let msg = match msg {
            raw::RawMessage::Ok(msg) => Ok(msg.try_into()?),
            raw::RawMessage::Err { err_msg } => Err(err_msg),
        };
        Ok(Self { meta, msg })
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Json(json::DecodingError),
    Hex(hex::FromHexError),
}

impl From<json::DecodingError> for Error {
    fn from(err: json::DecodingError) -> Self {
        Error::Json(err)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(err: hex::FromHexError) -> Self {
        Error::Hex(err)
    }
}

pub fn parse(raw: &str) -> Result<Response, Error> {
    let raw: raw::Response = json::from_str(raw)?;
    let response = raw.try_into()?;
    Ok(response)
}
