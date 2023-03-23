use crate::common::Uid;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Credentials {
    pub server: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "status")]
enum RawSiteDevices {
    #[serde(rename = "ok")]
    Ok { uids: Option<Vec<String>> },
    #[serde(rename = "error")]
    Err { msg: String },
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    Dash7board(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
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

    pub fn dash7board(username: String, password: String) -> Self {
        Self {
            server: "dash7board.wizzilab.com".to_string(),
            username,
            password,
        }
    }

    pub async fn get(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!(
            "https://{}:{}@{}/{}",
            self.username, self.password, self.server, path
        );
        println!("{url}");
        reqwest::get(&url).await
    }

    pub async fn get_site_devices(&self, id: usize) -> Result<Vec<Uid>, Error> {
        let raw = self
            .get(&format!("api/v1/sites/{id}/devices"))
            .await?
            .text()
            .await?;

        let resp: RawSiteDevices = serde_json::from_str(&raw)?;

        match resp {
            RawSiteDevices::Ok { uids } => {
                Ok(uids.unwrap().into_iter().map(|s| s.into()).collect())
            }
            RawSiteDevices::Err { msg } => Err(Error::Dash7board(msg)),
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
            conf.password.clone(),
        )
    }

    #[tokio::test]
    async fn get_site_devices() {
        let creds = creds().await;
        let devices = creds.get_site_devices(1).await.unwrap();
        assert_eq!(devices, vec![Uid::from("001BC50C70010EDE".to_string())]);
    }
}
