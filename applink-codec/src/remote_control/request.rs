pub use crate::permission::Dash7boardPermission;
use serde::{Deserialize, Serialize, Serializer};
use std::convert::{TryFrom, TryInto};
use wizzi_common::json;

pub mod raw {
    use super::{Dash7boardPermission, GatewayModemUid};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "lowercase")]
    pub enum Action {
        #[serde(rename(serialize = "R"))]
        Read,
        #[serde(rename(serialize = "W"))]
        Write,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Request {
        pub action: Action,
        pub user_type: Dash7boardPermission,
        pub gmuid: GatewayModemUid,
        pub uid: String,
        pub fid: u8,
        pub field_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<Vec<u8>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<serde_json::Number>,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Data {
    Integer(i64),
    Float(f64),
    Raw(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    Read,
    Write(Data),
}

#[derive(Deserialize, Debug, Clone)]
pub enum GatewayModemUid {
    Auto,
    Uid(String),
}
impl Serialize for GatewayModemUid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            GatewayModemUid::Auto => serializer.serialize_str("auto"),
            GatewayModemUid::Uid(s) => serializer.serialize_str(s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub action: Action,
    pub user_type: Dash7boardPermission,
    pub gmuid: GatewayModemUid,
    pub uid: String,
    pub fid: u8,
    pub field_name: String,
}

impl TryFrom<Request> for raw::Request {
    type Error = f64;
    fn try_from(cmd: Request) -> Result<Self, Self::Error> {
        let (data, value) = match &cmd.action {
            Action::Read => (None, None),
            Action::Write(Data::Integer(i)) => (None, Some(serde_json::Number::from(*i))),
            Action::Write(Data::Float(f)) => {
                (None, Some(serde_json::Number::from_f64(*f).ok_or(*f)?))
            }
            Action::Write(Data::Raw(r)) => (Some(r.clone()), None),
        };
        Ok(Self {
            action: match cmd.action {
                Action::Read => raw::Action::Read,
                Action::Write(_) => raw::Action::Write,
            },
            user_type: cmd.user_type,
            gmuid: cmd.gmuid,
            uid: cmd.uid,
            fid: cmd.fid,
            field_name: cmd.field_name,
            data,
            value,
        })
    }
}

#[derive(Clone, Debug)]
pub enum BadRequest {
    BadValue(f64),
    BadJson(json::EncodingError<raw::Request>),
}

impl Request {
    pub fn encode(self) -> Result<String, BadRequest> {
        let raw: raw::Request = self.try_into().map_err(BadRequest::BadValue)?;
        json::to_string(&raw).map_err(BadRequest::BadJson)
    }
}
