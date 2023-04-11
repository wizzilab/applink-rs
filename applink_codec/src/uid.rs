use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Uid {
    Dash7(wizzi_common::dash7::Uid),
    Vgw(String),
    Unknown(String),
}

pub enum UidDash7DecodeError {
    BadHex,
    BadSize,
}

impl Uid {
    pub const VGW_PREFIX: &'static str = "VGW-";

    pub fn parse_dash7(s: &str) -> Result<wizzi_common::dash7::Uid, UidDash7DecodeError> {
        let data: [u8; 8] = hex::decode(s)
            .map_err(|_| UidDash7DecodeError::BadHex)?
            .try_into()
            .map_err(|_| UidDash7DecodeError::BadSize)?;
        Ok(data)
    }
}

impl From<String> for Uid {
    #[allow(clippy::manual_strip)]
    fn from(uid: String) -> Self {
        if let Ok(uid) = Self::parse_dash7(&uid) {
            Self::Dash7(uid)
        } else if uid.starts_with(Self::VGW_PREFIX) {
            Self::Vgw(uid[Self::VGW_PREFIX.len()..].to_string())
        } else {
            Self::Unknown(uid)
        }
    }
}

impl From<wizzi_common::dash7::Uid> for Uid {
    fn from(uid: wizzi_common::dash7::Uid) -> Self {
        Self::Dash7(uid)
    }
}

impl std::convert::TryFrom<Uid> for wizzi_common::dash7::Uid {
    type Error = ();

    fn try_from(uid: Uid) -> Result<Self, Self::Error> {
        match uid {
            Uid::Dash7(uid) => Ok(uid),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dash7(uid) => write!(f, "{}", hex::encode_upper(uid)),
            Self::Vgw(uid) => write!(f, "{}{}", Self::VGW_PREFIX, uid),
            Self::Unknown(uid) => write!(f, "{}", uid),
        }
    }
}

impl std::fmt::Debug for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dash7(uid) => write!(f, "Dash7({})", hex::encode_upper(uid)),
            Self::Vgw(uid) => write!(f, "Vgw({})", uid),
            Self::Unknown(uid) => write!(f, "Unknown({})", uid),
        }
    }
}

impl<'de> Deserialize<'de> for Uid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from(s))
    }
}

impl Serialize for Uid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
