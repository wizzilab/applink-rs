use crate::codec::{remote_control, report};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::PollSender;

#[derive(Debug)]
pub enum Command {
    Publish { topic: String, data: Vec<u8> },
}

#[derive(Debug, Clone)]
pub enum BadFormat {
    Report(Vec<u8>),
    RemoteControl(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum Unsolicited {
    Connect,
    Disconnect,
    Report(report::Report),
    RemoteControl(remote_control::response::Response),
    BadFormat(BadFormat),
}

struct ClientBackend {
    company: String,
    client: rumqttc::AsyncClient,
    pending_request: Option<(String, Vec<u8>)>,
    mqtt_unsolicited_rx: mpsc::Receiver<rumqttc::Event>,
    pending_unsolicited: Option<Unsolicited>,
    command_rx: mpsc::Receiver<Command>,
    unsolicited_tx: PollSender<Unsolicited>,
}

enum MaintainResult {
    Continue,
    Pending,
    Closed,
}

impl ClientBackend {
    async fn new(
        options: rumqttc::MqttOptions,
        company: String,
        internal_queue_size: usize,
    ) -> Result<(Self, mpsc::Sender<Command>, mpsc::Receiver<Unsolicited>), rumqttc::ClientError>
    {
        let (client, mut connection) = rumqttc::AsyncClient::new(options, internal_queue_size);
        client
            .subscribe(&format!("/applink/{company}/#"), rumqttc::QoS::AtLeastOnce)
            .await?;

        let (command_tx, command_rx) = mpsc::channel(internal_queue_size);
        let (unsolicited_tx, unsolicited_rx) = mpsc::channel(internal_queue_size);
        let (mqtt_unsolicited_tx, mqtt_unsolicited_rx) = mpsc::channel(internal_queue_size);
        tokio::spawn(async move {
            loop {
                match connection.poll().await {
                    Ok(event) => mqtt_unsolicited_tx.send(event).await.unwrap(),
                    Err(e) => {
                        log::error!("MQTT connection error: {}", e);
                        break;
                    }
                }
            }
        });
        Ok((
            Self {
                company,
                client,
                pending_request: None,
                mqtt_unsolicited_rx,
                command_rx,
                unsolicited_tx: PollSender::new(unsolicited_tx),
                pending_unsolicited: None,
            },
            command_tx,
            unsolicited_rx,
        ))
    }

    fn send_next(&mut self, cx: &mut std::task::Context<'_>) -> MaintainResult {
        let (topic, data) = if let Some((topic, data)) = self.pending_request.take() {
            (topic, data)
        } else {
            match self.command_rx.poll_recv(cx) {
                std::task::Poll::Ready(Some(Command::Publish { topic, data })) => (topic, data),
                std::task::Poll::Ready(None) => return MaintainResult::Closed,
                std::task::Poll::Pending => return MaintainResult::Pending,
            }
        };
        if self
            .client
            .try_publish(&topic, rumqttc::QoS::AtLeastOnce, false, data.clone())
            .is_ok()
        {
            MaintainResult::Continue
        } else {
            self.pending_request = Some((topic, data));
            MaintainResult::Pending
        }
    }

    fn poll_recv(&mut self, cx: &mut std::task::Context<'_>) -> MaintainResult {
        let packet = match self.mqtt_unsolicited_rx.poll_recv(cx) {
            std::task::Poll::Ready(Some(packet)) => packet,
            std::task::Poll::Ready(None) => return MaintainResult::Closed,
            std::task::Poll::Pending => return MaintainResult::Pending,
        };

        let to_send = match packet {
            rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) => {
                let topic = publish.topic;
                let data = publish.payload;
                let report_topic = format!("/applink/{}/report", self.company);
                let remote_control_response_topic =
                    format!("/applink/{}/remotectrl/response/", self.company);
                if topic.starts_with(&report_topic) {
                    if let Ok(report) = report::parse(&data) {
                        Unsolicited::Report(report)
                    } else {
                        Unsolicited::BadFormat(BadFormat::Report(data.to_vec()))
                    }
                } else if topic.starts_with(&remote_control_response_topic) {
                    println!("Received: {}", topic);
                    if let Ok(response) = remote_control::response::parse(&data) {
                        Unsolicited::RemoteControl(response)
                    } else {
                        Unsolicited::BadFormat(BadFormat::RemoteControl(data.to_vec()))
                    }
                } else {
                    log::error!("Unknown topic: {}", topic);
                    return MaintainResult::Continue;
                }
            }
            rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) => Unsolicited::Connect,
            rumqttc::Event::Incoming(rumqttc::Packet::Disconnect) => Unsolicited::Disconnect,
            _ => return MaintainResult::Continue,
        };

        match self.unsolicited_tx.poll_reserve(cx) {
            std::task::Poll::Ready(Ok(_)) => {
                self.unsolicited_tx.send_item(to_send).unwrap();
            }
            std::task::Poll::Ready(Err(_)) => return MaintainResult::Closed,
            std::task::Poll::Pending => {
                self.pending_unsolicited = Some(to_send);
                return MaintainResult::Pending;
            }
        }

        MaintainResult::Continue
    }
}

impl std::future::Future for ClientBackend {
    type Output = Result<(), rumqttc::Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();

        // Process commands
        loop {
            match this.send_next(cx) {
                MaintainResult::Continue => {}
                MaintainResult::Pending => break,
                MaintainResult::Closed => return std::task::Poll::Ready(Ok(())),
            }
        }

        // Process incoming
        loop {
            match this.poll_recv(cx) {
                MaintainResult::Continue => {}
                MaintainResult::Pending => break,
                MaintainResult::Closed => return std::task::Poll::Ready(Ok(())),
            }
        }
        std::task::Poll::Pending
    }
}

pub struct Client {
    command_tx: mpsc::Sender<Command>,
    company: String,
    listeners: Arc<Mutex<Vec<mpsc::Sender<Unsolicited>>>>,
    // TODO Increment on clone
    id: usize,
    request_sn: usize,
}

#[derive(Debug)]
pub enum RequestError {
    SendBackendDead(mpsc::error::SendError<Command>),
    ReceiveBackendDead,
}

impl Client {
    pub async fn new(
        options: rumqttc::MqttOptions,
        company: String,
        internal_queue_size: usize,
    ) -> Result<Self, rumqttc::ClientError> {
        let (backend, command_tx, mut unsolicited_rx) =
            ClientBackend::new(options, company.clone(), internal_queue_size).await?;
        let listeners: Arc<Mutex<Vec<mpsc::Sender<Unsolicited>>>> =
            Arc::new(Mutex::new(Vec::new()));

        // Start client backend
        tokio::spawn(backend);

        // Start listener dispatcher
        let dispatcher_listeners = listeners.clone();
        tokio::spawn(async move {
            let mut listeners = dispatcher_listeners.lock().await;
            while let Some(unsolicited) = unsolicited_rx.recv().await {
                println!("Dispatching unsolicited: {:?}", unsolicited);
                for listener in listeners.iter_mut() {
                    listener.send(unsolicited.clone()).await.unwrap();
                }
            }
        });

        Ok(Self {
            command_tx,
            company,
            listeners,
            id: 0,
            request_sn: 0,
        })
    }

    fn request_id(&mut self) -> String {
        self.request_sn += 1;
        format!("{}-{}", self.id, self.request_sn)
    }

    pub async fn remote_control(
        &mut self,
        command: remote_control::request::Command,
    ) -> Result<remote_control::response::Response, RequestError> {
        // Subscribe to response
        let mut rx = self.unsolicited().await;

        // Build request
        let command_s = command.encode();
        let data = command_s.as_bytes().to_vec();
        let request_id = self.request_id();
        let topic = format!("/applink/{}/remotectrl/request/{request_id}", self.company);

        // Send request
        self.command_tx
            .send(Command::Publish { topic, data })
            .await
            .map_err(RequestError::SendBackendDead)?;

        // Wait for response
        println!("Waiting for response");
        while let Some(unsolicited) = rx.recv().await {
            if let Unsolicited::RemoteControl(response) = unsolicited {
                return Ok(response);
            }
        }
        Err(RequestError::ReceiveBackendDead)
    }

    pub async fn unsolicited(&mut self) -> mpsc::Receiver<Unsolicited> {
        let (tx, rx) = mpsc::channel(1);
        self.listeners.lock().await.push(tx);
        rx
    }
}
