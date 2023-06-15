use crate::codec::{remote_control, report, wizzi_macro};
use futures_lite::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[derive(Debug)]
pub enum Command {
    Publish { routing_key: String, data: Vec<u8> },
}

#[derive(Debug, Clone)]
pub enum ApplinkReportBadFormat {
    MissingSite {
        company: String,
        routing_key: String,
        data: Vec<u8>,
    },
    MissingDevice {
        company: String,
        routing_key: String,
        data: Vec<u8>,
    },
    BadUtf8 {
        company: String,
        site: String,
        device: String,
        data: Vec<u8>,
    },
    BadReport {
        company: String,
        site: String,
        device: String,
        data: Vec<u8>,
        error: report::ReportParseError,
    },
}

#[derive(Debug, Clone)]
pub enum ApplinkBadFormat {
    Utf8 {
        routing_key: String,
        data: Vec<u8>,
    },
    MissingCompany {
        routing_key: String,
        data: Vec<u8>,
    },
    Report(ApplinkReportBadFormat),
    RemoteControl(remote_control::response::Error),
    Macro(wizzi_macro::Error),
    UnknownSubtopic {
        routing_key: String,
        sub_topic: String,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub enum GatewayBadFormat {
    MissingUid {
        routing_key: String,
        data: Vec<u8>,
    },
    MissingModemUid {
        routing_key: String,
        data: Vec<u8>,
    },
    UnknownSubtopic {
        routing_key: String,
        sub_topic: String,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub enum BadFormat {
    Applink(ApplinkBadFormat),
    Gateway(GatewayBadFormat),
    UnknownRoutingKey { routing_key: String, data: Vec<u8> },
}

#[derive(Debug, Clone)]
pub enum ApplinkPayload {
    Report(report::Report),
    RemoteControl(remote_control::response::Response),
    Macro(wizzi_macro::Response),
    // TODO
    // GatewayControl(Respone),
    // Location(Location),
}

#[derive(Debug, Clone)]
pub enum GatewayPayload {
    MdReport { modem: String, payload: Vec<u8> },
    // TODO
    // Pmd(Report),
    // Control(Response)
    // Cup(Response)
}

#[derive(Debug, Clone)]
pub enum Unsolicited {
    Connect,
    Disconnect,
    Applink {
        company: String,
        site: String,
        device: String,
        payload: ApplinkPayload,
    },
    Gateway {
        uid: String,
        payload: GatewayPayload,
    },
    BadFormat(BadFormat),
}

#[derive(Debug, Clone)]
pub struct AmqpUnsolicted {
    pub routing_key: String,
    pub data: Vec<u8>,
}

struct AmqpConnection {
    connection: lapin::Connection,
    channel: lapin::Channel,
    queues: HashMap<String, lapin::Queue>,
    consumers: HashMap<String, lapin::Consumer>,
}

struct ClientBackend {
    connection: AmqpConnection,
    conf: Conf,
    _command_rx: mpsc::Receiver<Command>,
    unsolicited_tx: mpsc::Sender<Unsolicited>,
}

pub struct QueueBindingConf {
    pub exchange: String,
    pub routing_key: String,
    pub options: lapin::options::QueueBindOptions,
    pub arguments: amq_protocol_types::FieldTable,
}

pub struct QueueConf {
    pub name: String,
    pub consumer_name: String,
    pub bindings: Vec<QueueBindingConf>,
}

pub struct Conf {
    pub uri: String,
    pub amqp_options: lapin::ConnectionProperties,
    pub queues: Vec<QueueConf>,
}

impl ClientBackend {
    async fn connect(conf: &Conf) -> Result<AmqpConnection, lapin::Error> {
        let connection = lapin::Connection::connect(&conf.uri, conf.amqp_options.clone()).await?;
        let channel = connection.create_channel().await?;
        let mut queues = HashMap::new();
        let mut consumers = HashMap::new();
        for queue_conf in &conf.queues {
            let queue = channel
                .queue_declare(
                    &queue_conf.name,
                    lapin::options::QueueDeclareOptions::default(),
                    amq_protocol_types::FieldTable::default(),
                )
                .await?;
            for binding in &queue_conf.bindings {
                channel
                    .queue_bind(
                        &queue_conf.name,
                        &binding.exchange,
                        &binding.routing_key,
                        binding.options,
                        binding.arguments.clone(),
                    )
                    .await?;
            }
            queues.insert(queue_conf.name.clone(), queue);
            let consumer = channel
                .basic_consume(
                    &queue_conf.name,
                    &queue_conf.consumer_name,
                    lapin::options::BasicConsumeOptions::default(),
                    amq_protocol_types::FieldTable::default(),
                )
                .await?;
            consumers.insert(queue_conf.name.clone(), consumer);
        }
        Ok(AmqpConnection {
            connection,
            channel,
            queues,
            consumers,
        })
    }

    async fn start(
        conf: Conf,
        internal_queue_size: usize,
    ) -> Result<(mpsc::Sender<Command>, mpsc::Receiver<Unsolicited>), lapin::Error> {
        let (command_tx, command_rx) = mpsc::channel(internal_queue_size);
        let (unsolicited_tx, unsolicited_rx) = mpsc::channel(internal_queue_size);
        let connection = Self::connect(&conf).await?;
        let client = Self {
            connection,
            conf,
            _command_rx: command_rx,
            unsolicited_tx,
        };
        tokio::spawn(client.run());
        Ok((command_tx, unsolicited_rx))
    }

    fn parse_gateway_unsolicited(
        routing_key_parts: &[&str],
        data: &[u8],
    ) -> Result<Unsolicited, GatewayBadFormat> {
        let uid = match routing_key_parts.first() {
            Some(uid) => uid.to_string(),
            None => {
                return Err(GatewayBadFormat::MissingUid {
                    routing_key: routing_key_parts.join("."),
                    data: data.to_vec(),
                })
            }
        };

        match routing_key_parts.get(1) {
            Some(&"md") => {
                let modem = match routing_key_parts.get(2) {
                    Some(modem) => modem.to_string(),
                    None => {
                        return Err(GatewayBadFormat::MissingModemUid {
                            routing_key: routing_key_parts.join("."),
                            data: data.to_vec(),
                        })
                    }
                };
                let payload = GatewayPayload::MdReport {
                    modem,
                    payload: data.to_vec(),
                };
                Ok(Unsolicited::Gateway { uid, payload })
            }
            sub_topic => Err(GatewayBadFormat::UnknownSubtopic {
                routing_key: routing_key_parts.join("."),
                sub_topic: sub_topic.map(|s| s.to_string()).unwrap_or("".to_string()),
                data: data.to_vec(),
            }),
        }
    }

    fn parse_applink_report(
        company: &str,
        routing_key_parts: &[&str],
        data: &[u8],
    ) -> Result<Unsolicited, ApplinkReportBadFormat> {
        let site = match routing_key_parts.first() {
            Some(site) => site.to_string(),
            None => {
                return Err(ApplinkReportBadFormat::MissingSite {
                    company: company.to_string(),
                    routing_key: routing_key_parts.join("."),
                    data: data.to_vec(),
                })
            }
        };
        let device = match routing_key_parts.get(1) {
            Some(device) => device.to_string(),
            None => {
                return Err(ApplinkReportBadFormat::MissingDevice {
                    company: company.to_string(),
                    routing_key: routing_key_parts.join("."),
                    data: data.to_vec(),
                })
            }
        };
        let data_s = match std::str::from_utf8(data) {
            Ok(data) => data,
            Err(_) => {
                return Err(ApplinkReportBadFormat::BadUtf8 {
                    company: company.to_string(),
                    site,
                    device,
                    data: data.to_vec(),
                })
            }
        };
        let report = match report::parse(data_s) {
            Ok(payload) => payload,
            Err(error) => {
                return Err(ApplinkReportBadFormat::BadReport {
                    company: company.to_string(),
                    site,
                    device,
                    data: data.to_vec(),
                    error,
                })
            }
        };

        let payload = ApplinkPayload::Report(report);
        Ok(Unsolicited::Applink {
            company: company.to_string(),
            site,
            device,
            payload,
        })
    }

    fn parse_applink_unsolicited(
        routing_key_parts: &[&str],
        data: &[u8],
    ) -> Result<Unsolicited, ApplinkBadFormat> {
        let company = match routing_key_parts.first() {
            Some(company) => company.to_string(),
            None => {
                return Err(ApplinkBadFormat::MissingCompany {
                    routing_key: routing_key_parts.join("."),
                    data: data.to_vec(),
                })
            }
        };

        match routing_key_parts.get(1) {
            #[allow(clippy::indexing_slicing)]
            Some(&"report") => Self::parse_applink_report(&company, &routing_key_parts[2..], data)
                .map_err(ApplinkBadFormat::Report),
            sub_topic => Err(ApplinkBadFormat::UnknownSubtopic {
                routing_key: routing_key_parts.join("."),
                sub_topic: sub_topic.map(|s| s.to_string()).unwrap_or("".to_string()),
                data: data.to_vec(),
            }),
        }
    }

    fn parse_unsolicited(routing_key: &str, data: &[u8]) -> Unsolicited {
        let routing_key_parts = routing_key.split('.').collect::<Vec<_>>();

        match routing_key_parts.first() {
            Some(&"") => match routing_key_parts.get(1) {
                #[allow(clippy::indexing_slicing)]
                Some(&"applink") => {
                    match Self::parse_applink_unsolicited(&routing_key_parts[2..], data) {
                        Ok(payload) => payload,
                        Err(error) => Unsolicited::BadFormat(BadFormat::Applink(error)),
                    }
                }
                #[allow(clippy::indexing_slicing)]
                Some(&"gw") => match Self::parse_gateway_unsolicited(&routing_key_parts[2..], data)
                {
                    Ok(payload) => payload,
                    Err(error) => Unsolicited::BadFormat(BadFormat::Gateway(error)),
                },
                _ => Unsolicited::BadFormat(BadFormat::UnknownRoutingKey {
                    routing_key: routing_key.to_string(),
                    data: data.to_vec(),
                }),
            },
            _ => Unsolicited::BadFormat(BadFormat::UnknownRoutingKey {
                routing_key: routing_key.to_string(),
                data: data.to_vec(),
            }),
        }
    }

    async fn run(self) {
        let Self {
            connection,
            _command_rx,
            unsolicited_tx,
            conf,
            ..
        } = self;

        let AmqpConnection {
            connection: mut _connection,
            channel: mut _channel,
            mut queues,
            mut consumers,
        } = connection;
        'main: loop {
            let (amqp_recv_tx, mut amqp_recv_rx) = mpsc::channel(queues.len());

            // Start monitoring the consumers
            for (_queue_name, mut consumer) in consumers {
                let amqp_recv_tx = amqp_recv_tx.clone();
                tokio::spawn(async move {
                    while let Some(delivery) = consumer.next().await {
                        let delivery = match delivery {
                            Ok(delivery) => delivery,
                            Err(err) => {
                                log::error!("AMQP consumer error: {}", err);
                                let _ = amqp_recv_tx.send(Err(err)).await;
                                break;
                            }
                        };
                        match delivery
                            .ack(lapin::options::BasicAckOptions::default())
                            .await
                        {
                            Ok(()) => {}
                            Err(err) => {
                                log::error!("AMQP consumer error: {}", err);
                                let _ = amqp_recv_tx.send(Err(err)).await;
                                break;
                            }
                        }
                        let data = delivery.data;
                        let routing_key = delivery.routing_key;
                        if amqp_recv_tx
                            .send(Ok(AmqpUnsolicted {
                                routing_key: routing_key.to_string(),
                                data,
                            }))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                });
            }

            // Process incoming packets
            while let Some(Ok(AmqpUnsolicted { routing_key, data })) = amqp_recv_rx.recv().await {
                let unsolicited = Self::parse_unsolicited(&routing_key, &data);
                if unsolicited_tx.send(unsolicited).await.is_err() {
                    break 'main;
                }
            }

            // Notify disconnection
            if unsolicited_tx.send(Unsolicited::Disconnect).await.is_err() {
                break 'main;
            }

            // Reconnect
            let AmqpConnection {
                connection: new_connection,
                channel: new_channel,
                queues: new_queues,
                consumers: new_consumers,
            } = loop {
                match Self::connect(&conf).await {
                    Ok(connection) => break connection,
                    Err(err) => {
                        log::error!("failed to connect to AMQP: {}", err);
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            };
            _connection = new_connection;
            _channel = new_channel;
            queues = new_queues;
            consumers = new_consumers;

            // Notify reconnection
            if unsolicited_tx.send(Unsolicited::Connect).await.is_err() {
                break 'main;
            }
        }
    }
}

pub struct Client {
    _command_tx: mpsc::Sender<Command>,
    listeners: Arc<Mutex<Vec<mpsc::Sender<Unsolicited>>>>,
}

impl Client {
    pub async fn new(conf: Conf, internal_queue_size: usize) -> Result<Self, lapin::Error> {
        let (_command_tx, unsolicited_rx) = ClientBackend::start(conf, internal_queue_size).await?;

        // Start the listener dispatcher
        let listeners: Arc<Mutex<Vec<mpsc::Sender<Unsolicited>>>> = Arc::new(Mutex::new(vec![]));
        tokio::spawn(wizzi_common_tokio::forward_to_multiple(
            unsolicited_rx,
            Arc::downgrade(&listeners),
        ));

        Ok(Self {
            _command_tx,
            listeners,
        })
    }

    pub async fn unsolicited(&self) -> mpsc::Receiver<Unsolicited> {
        let (tx, rx) = mpsc::channel(1);
        self.listeners.lock().await.push(tx);
        rx
    }
}
