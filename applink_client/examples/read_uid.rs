use applink_client::{codec::remote_control, mqtt::Client};
use clap::Parser;

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
    let request = remote_control::Request {
        action: remote_control::Action::Read,
        user_type: remote_control::Dash7boardPermission::Admin,
        gmuid: remote_control::GatewayModemUid::Auto,
        uid: params.uid,
        fid: 0,
        field_name: "uid".to_string(),
    };

    let response = client.remote_control(request).await.unwrap();
    println!("{:#?}", response);
}
