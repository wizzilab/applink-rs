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
    pub fn new(server: &str, username: &str, password: &str) -> Self {
        Self {
            server: server.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub fn dash7board(username: &str, password: &str) -> Self {
        Self {
            server: "dash7board.wizzilab.com".to_string(),
            username: username.to_string(),
            password: password.to_string(),
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
macro_rules! tokio_with_creds {
    ([$creds:ident] $block:block) => {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let conf_path = format!("{}/test_credentials.json", env!("CARGO_MANIFEST_DIR"));
            let conf = tokio::fs::read_to_string(conf_path).await.unwrap();
            let conf: std::collections::HashMap<String, String> =
                serde_json::from_str(&conf).unwrap();
            let username = conf.get("username").unwrap();
            let password = conf.get("password").unwrap();
            let $creds = Credentials::dash7board(username, password);
            $block
        });
    };
}

#[test]
fn get_site_devices() {
    tokio_with_creds!(
        [creds] {
        let devices = creds.get_site_devices(1).await.unwrap();
        assert_eq!(devices, vec![Uid::from("001BC50C70010EDE".to_string())]);
    });
}
