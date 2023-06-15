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
