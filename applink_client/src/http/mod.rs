use crate::codec::uid::Uid;
use serde::{Deserialize, Serialize};
use wizzi_common::json;

#[derive(Debug, Clone, PartialEq)]
pub struct Credentials {
    pub server: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "status")]
enum RawSiteDevices {
    #[serde(rename = "ok")]
    Ok { uids: Vec<String> },
    #[serde(rename = "error")]
    Err { msg: String },
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DeviceInfos {
    pub uid: Uid,
    pub site_id: Option<usize>,
    pub vid: Option<usize>,
    pub key_ring_id: Option<usize>,
    pub key: Option<usize>,
    pub label: Option<String>,
    pub dc: Option<String>,
    pub mc: Option<String>,
    pub dfv: Option<String>,
    pub dhv: Option<String>,
    pub mfv: Option<String>,
    pub mhv: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "status")]
pub enum RawDevicesInfos {
    #[serde(rename = "ok")]
    Ok { devices: Vec<DeviceInfos> },
    #[serde(rename = "error")]
    Err { msg: String },
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "status")]
pub enum RawDeviceTags {
    #[serde(rename = "ok")]
    Ok { tags: Vec<String> },
    #[serde(rename = "error")]
    Err { msg: String },
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Json(json::DecodingError),
    Dash7board(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl From<json::DecodingError> for Error {
    fn from(e: json::DecodingError) -> Self {
        Error::Json(e)
    }
}

impl Credentials {
    pub fn new(server: String, username: String, password: String) -> Self {
        Self {
            server,
            username,
            password,
        }
    }

    pub fn try_default() -> Result<Self, std::env::VarError> {
        let server = std::env::var("DASH7BOARD_SERVER")?;
        let username = std::env::var("APPLINK_ID")?;
        let password = std::env::var("APPLINK_KEY")?;

        Ok(Self {
            server,
            username,
            password,
        })
    }

    pub fn dash7board(username: String, password: String) -> Self {
        Self {
            server: "dash7board.wizzilab.com".to_string(),
            username,
            password,
        }
    }

    pub async fn get(
        &self,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!(
            "https://{}:{}@{}/{}",
            self.username, self.password, self.server, path
        );
        let mut request = reqwest::Client::new().get(&url);
        if let Some(body) = body {
            request = request.json(&body);
        }
        request.send().await
    }

    pub async fn post<T: Serialize + Sized>(
        &self,
        path: &str,
        data: T,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!(
            "https://{}:{}@{}/{}",
            self.username, self.password, self.server, path
        );
        reqwest::Client::new().post(&url).json(&data).send().await
    }

    pub async fn get_site_devices(&self, id: usize) -> Result<Vec<Uid>, Error> {
        let raw = self
            .get(&format!("api/v1/sites/{id}/devices"), None)
            .await?
            .text()
            .await?;

        let resp: RawSiteDevices = json::from_str(raw)?;

        match resp {
            RawSiteDevices::Ok { uids } => Ok(uids.into_iter().map(|s| s.into()).collect()),
            RawSiteDevices::Err { msg } => Err(Error::Dash7board(msg)),
        }
    }

    pub async fn get_devices_infos<S: AsRef<str>>(
        &self,
        uids: &[S],
    ) -> Result<Vec<DeviceInfos>, Error> {
        let payload = serde_json::json!({ "uids": uids.iter().map(|uid| uid.as_ref().to_string()).collect::<Vec<_>>() });
        let raw = self
            .post("api/v1/devices/info", payload)
            .await?
            .text()
            .await?;

        let resp: RawDevicesInfos = json::from_str(raw)?;

        match resp {
            RawDevicesInfos::Ok { devices } => Ok(devices),
            RawDevicesInfos::Err { msg } => Err(Error::Dash7board(msg)),
        }
    }

    pub async fn get_device_tags<S: AsRef<str>>(&self, uid: &S) -> Result<Vec<String>, Error> {
        let data = serde_json::json!({"tags": []});
        let raw = self
            .post(&format!("api/v1/devices/{}/tags/add", uid.as_ref()), data)
            .await?
            .text()
            .await?;

        let resp: RawDeviceTags = json::from_str(raw)?;

        match resp {
            RawDeviceTags::Ok { tags } => Ok(tags),
            RawDeviceTags::Err { msg } => Err(Error::Dash7board(msg)),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::common::test::load_config;

    async fn creds() -> Credentials {
        let conf = load_config().await;
        Credentials::new(
            conf.http_server.clone(),
            conf.username.clone(),
            conf.password,
        )
    }

    #[tokio::test]
    async fn get_site_devices() {
        let creds = creds().await;
        let devices = creds.get_site_devices(1).await.unwrap();
        assert_eq!(devices, vec![Uid::from("001BC50C70010EDE".to_string())]);
    }

    #[tokio::test]
    async fn get_devices_infos() {
        let creds = creds().await;
        let uids: Vec<String> = vec![];
        let devices = creds.get_devices_infos(&uids).await.unwrap();
        assert_eq!(devices, vec![]);
    }

    #[tokio::test]
    async fn get_device_tags() {
        let creds = creds().await;
        let tags = creds
            .get_device_tags(&"001BC50C70010EDE".to_string())
            .await
            .unwrap();
        assert_eq!(tags, vec!["test".to_string()]);
    }
}
