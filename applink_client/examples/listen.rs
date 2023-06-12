use applink_client::mqtt::Client;
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
    pub client_id: String,
}

#[tokio::main]
async fn main() {
    let params = Params::parse();

    println!("Start mqtt client");
    let mut options = rumqttc::MqttOptions::new(params.client_id, "roger.wizzilab.com", 8883);
    options.set_credentials(params.username, params.password);
    options.set_transport(rumqttc::Transport::tls_with_default_config());
    let mut client = Client::new(options.into(), params.company, 1)
        .await
        .unwrap();

    let mut rx = client.unsolicited().await;
    while let Some(msg) = rx.recv().await {
        println!("Received message: {:#?}", msg);
    }
}
