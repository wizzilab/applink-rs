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

    #[test]
    fn write_response() {
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
                msg: RawMessage::Ok(super::Message { value: None }),
            }
        );
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
    let raw: raw::Response = serde_json::from_slice(raw)?;
    Ok(raw.into())
}
