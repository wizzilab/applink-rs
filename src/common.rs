#[derive(Clone, PartialEq, Eq)]
pub enum Uid {
    Dash7(u64),
    Vgw(String),
    Unknown(String),
}

pub enum UidDash7DecodeError {
    BadHex,
    BadSize,
}

impl Uid {
    pub const VGW_PREFIX: &'static str = "VGW-";

    pub fn parse_dash7(s: &str) -> Result<u64, UidDash7DecodeError> {
        let data: [u8; 8] = hex::decode(s)
            .map_err(|_| UidDash7DecodeError::BadHex)?
            .try_into()
            .map_err(|_| UidDash7DecodeError::BadSize)?;
        Ok(u64::from_be_bytes(data))
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

impl std::fmt::Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dash7(uid) => write!(f, "{:016X}", uid),
            Self::Vgw(uid) => write!(f, "{}{}", Self::VGW_PREFIX, uid),
            Self::Unknown(uid) => write!(f, "{}", uid),
        }
    }
}

impl std::fmt::Debug for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dash7(uid) => write!(f, "Dash7({:016X})", uid),
            Self::Vgw(uid) => write!(f, "Vgw({})", uid),
            Self::Unknown(uid) => write!(f, "Unknown({})", uid),
        }
    }
}
