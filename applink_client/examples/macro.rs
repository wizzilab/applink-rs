use applink_client::{codec::wizzi_macro, mqtt::Client};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Params {
    #[arg(short, long)]
    pub company: String,
    #[arg(short, long)]
    pub username: String,
    #[arg(short, long)]
    pub password: String,
    /// Example files are found in applink_client/examples/macro/
    #[arg(short, long)]
    pub macro_file: String,
}

#[tokio::main]
async fn main() {
    let params = Params::parse();
    let client_id = format!("{}:0", params.username);

    println!("Read config");
    let config_json = std::fs::read_to_string(params.macro_file).unwrap();
    let request: wizzi_macro::Request = serde_json::from_str(&config_json).unwrap();

    println!("Start mqtt client");
    let mut options = rumqttc::MqttOptions::new(client_id, "roger.wizzilab.com", 8883);
    options.set_credentials(params.username, params.password);
    options.set_transport(rumqttc::Transport::tls_with_default_config());
    let mut client = Client::new(options, params.company, 1).await.unwrap();

    println!("Send request");
    let response = client.wizzi_macro(request).await.unwrap();
    println!("{:#?}", response);
}
