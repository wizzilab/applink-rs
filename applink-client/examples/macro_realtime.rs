use applink_client::{
    codec::wizzi_macro,
    mqtt::{Client, Conf},
};
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
    let conf = Conf {
        mqtt_options: options,
        subscription_topics: vec![(
            format!("/applink/{}/macro/response/#", params.company),
            rumqttc::QoS::AtMostOnce,
        )],
    };
    let mut client = Client::new(conf, params.company, 1).await.unwrap();

    println!("Send request");
    let mut response = client.real_time_wizzi_macro(request).await.unwrap();

    while let Some(response) = response.recv().await {
        match response.msg {
            wizzi_macro::Message::Status { status } => {
                println!("Status: {:?}", status);
            }
            wizzi_macro::Message::Log { progress } => {
                println!("Progress: {:.2}", progress);
            }
            wizzi_macro::Message::DstatusOk { uid } => {
                println!("Ok: {}", uid);
            }
            wizzi_macro::Message::DstatusError { uid, err } => {
                println!("Err: {}: {}", uid, err);
            }
        }
    }
}
