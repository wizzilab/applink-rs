use applink::{codec::wizzi_macro, mqtt::Client};
use clap::Parser;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Params {
    #[arg(long)]
    pub company: String,
    #[arg(long)]
    pub username: String,
    #[arg(long)]
    pub password: String,
    #[arg(long)]
    pub uid: String,
    #[arg(long)]
    pub site_id: usize,
    #[arg(long)]
    pub macro_name: String,
}

#[tokio::main]
async fn main() {
    let params = Params::parse();
    let client_id = format!("{}:0", params.username);

    println!("Start mqtt client");
    let mut options = rumqttc::MqttOptions::new(client_id, "roger.wizzilab.com", 8883);
    options.set_credentials(params.username, params.password);
    options.set_transport(rumqttc::Transport::tls_with_default_config());
    let mut client = Client::new(options, params.company, 1).await.unwrap();

    println!("Send request");
    let request = wizzi_macro::Request {
        site_id: params.site_id,
        user_type: applink::common::Dash7boardPermission::Admin,
        name: params.macro_name,
        shared_vars: HashMap::new(),
        device_vars: HashMap::new(),
        device_uids: vec![params.uid],
        gateway_mode: wizzi_macro::GatewayMode::Best,
    };

    let response = client.wizzi_macro(request).await.unwrap();
    println!("{:#?}", response);
}
