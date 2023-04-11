use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use wizzi_common::json;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Meta {
    pub rid: String,
}

pub mod raw {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum Status {
        Start,
        End,
        Err,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum Dstatus {
        Ok,
        Error,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    #[serde(tag = "type")]
    #[serde(rename_all = "UPPERCASE")]
    pub enum Message {
        Status {
            status: Status,
            err: Option<String>,
        },
        Log {
            progress: f64,
        },
        Dstatus {
            uid: String,
            dstatus: Dstatus,
            err: Option<String>,
        },
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    pub struct Response {
        pub meta: Meta,
        pub msg: Message,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    Start,
    End,
    Err { err: String },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Status { status: Status },
    Log { progress: f64 },
    DstatusOk { uid: String },
    DstatusError { uid: String, err: String },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Response {
    pub meta: Meta,
    pub msg: Message,
}

impl TryFrom<raw::Response> for Response {
    type Error = raw::Response;
    fn try_from(raw: raw::Response) -> Result<Self, Self::Error> {
        let msg = match raw.msg.clone() {
            raw::Message::Status { status, err } => Message::Status {
                status: match status {
                    raw::Status::Start => Status::Start,
                    raw::Status::End => Status::End,
                    raw::Status::Err => Status::Err {
                        err: err.unwrap_or("Missing error message".to_string()),
                    },
                },
            },
            raw::Message::Log { progress } => Message::Log { progress },
            raw::Message::Dstatus { uid, dstatus, err } => match dstatus {
                raw::Dstatus::Ok => Message::DstatusOk { uid },
                raw::Dstatus::Error => Message::DstatusError {
                    uid,
                    err: err.ok_or(raw.clone())?,
                },
            },
        };
        Ok(Self {
            meta: raw.meta,
            msg,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Json(json::Error),
    BadRaw(raw::Response),
}

impl Response {
    pub fn parse(data: &str) -> Result<Self, Error> {
        let raw: raw::Response = json::from_str(data).map_err(Error::Json)?;
        Self::try_from(raw).map_err(Error::BadRaw)
    }
}
