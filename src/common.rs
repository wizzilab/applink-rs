use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Dash7boardPermission {
    Operator,
    Admin,
    Root,
}

#[cfg(test)]
pub(crate) mod test {
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Clone)]
    pub(crate) struct TestConfig {
        pub(crate) http_server: String,
        pub(crate) roger_server: String,
        pub(crate) device: String,
        pub(crate) gateway: String,
        pub(crate) site_id: usize,
        pub(crate) username: String,
        pub(crate) password: String,
        pub(crate) client_id: String,
        pub(crate) company: String,
    }

    pub(crate) async fn load_config() -> TestConfig {
        let conf_path = format!("{}/test_credentials.json", env!("CARGO_MANIFEST_DIR"));
        if !std::path::Path::new(&conf_path).exists() {
            panic!("Please create a test_credentials.json file in the root of the project containing the applink username and password.");
        }
        let conf_s = tokio::fs::read_to_string(conf_path).await.unwrap();

        serde_json::from_str(&conf_s).unwrap()
    }
}
