use applink_client::amqp::{Client, Conf, QueueBindingConf, QueueConf};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Params {
    #[arg(long)]
    pub username: String,
    #[arg(long)]
    pub password: String,
}

#[tokio::main]
async fn main() {
    let params = Params::parse();

    let uri = format!(
        "amqps://{}:{}@roger.wizzilab.com:5671/wizzilab",
        params.username, params.password
    );
    let options = Conf {
        uri: uri.to_string(),
        amqp_options: lapin::ConnectionProperties::default(),
        queues: vec![QueueConf {
            name: "queue.assert_watcher".to_string(),
            consumer_name: "test".to_string(),
            bindings: vec![QueueBindingConf {
                exchange: "exchange.mqtt".to_string(),
                routing_key: ".applink.*.report.#".to_string(),
                options: lapin::options::QueueBindOptions::default(),
                arguments: amq_protocol_types::FieldTable::default(),
            }],
        }],
    };

    let client = Client::new(options, 1).await.unwrap();

    let mut rx = client.unsolicited().await;
    while let Some(msg) = rx.recv().await {
        println!("Received message: {:#?}", msg);
    }
}
