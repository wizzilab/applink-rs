use crate::codec::{remote_control, report, wizzi_macro};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::PollSender;

macro_rules! p_debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug")]
        log::debug!( $($arg)*)
    };
}

#[derive(Debug)]
pub enum Command {
    Publish { topic: String, data: Vec<u8> },
}

#[derive(Debug, Clone)]
pub enum BadFormat {
    Utf8 { topic: String, data: Vec<u8> },
    Report(report::ReportParseError),
    RemoteControl(remote_control::response::Error),
    Macro(wizzi_macro::Error),
}

#[derive(Debug, Clone)]
pub enum Unsolicited {
    Connect,
    Disconnect,
    Report(report::Report),
    // TODO Rewrite to have a proper request/response handling instead of hijacking the unsolicited
    // feed.
    RemoteControl(remote_control::response::Response),
    Macro(wizzi_macro::response::Response),
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
            p_debug!("Sent to MQTT: {} {:?}", topic, data);
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
                p_debug!("Rcv from MQTT: {} {:?}", topic, publish.payload);
                let topic = publish.topic;
                match std::str::from_utf8(&publish.payload) {
                    Ok(data) => {
                        // TODO Make a topic tree instead
                        let report_topic = format!("/applink/{}/report", self.company);
                        let remote_control_response_topic =
                            format!("/applink/{}/remotectrl/response/", self.company);
                        let remote_control_request_topic =
                            format!("/applink/{}/remotectrl/request/", self.company);
                        let macro_request_topic =
                            format!("/applink/{}/macro/request/", self.company);
                        let macro_response_topic =
                            format!("/applink/{}/macro/response/", self.company);
                        if topic.starts_with(&report_topic) {
                            match report::parse(data) {
                                Ok(report) => Unsolicited::Report(report),
                                Err(e) => Unsolicited::BadFormat(BadFormat::Report(e)),
                            }
                        } else if topic.starts_with(&remote_control_response_topic) {
                            match remote_control::response::parse(data) {
                                Ok(response) => Unsolicited::RemoteControl(response),
                                Err(e) => Unsolicited::BadFormat(BadFormat::RemoteControl(e)),
                            }
                        } else if topic.starts_with(&macro_response_topic) {
                            match wizzi_macro::Response::parse(data) {
                                Ok(response) => Unsolicited::Macro(response),
                                Err(e) => Unsolicited::BadFormat(BadFormat::Macro(e)),
                            }
                        } else if topic.starts_with(&remote_control_request_topic)
                            || topic.starts_with(&macro_request_topic)
                        {
                            // TODO
                            return MaintainResult::Continue;
                        } else {
                            log::error!("Unknown topic: {}", topic);
                            return MaintainResult::Continue;
                        }
                    }
                    Err(_) => Unsolicited::BadFormat(BadFormat::Utf8 {
                        topic,
                        data: publish.payload.to_vec(),
                    }),
                }
            }
            rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) => {
                p_debug!("Connected");
                Unsolicited::Connect
            }
            rumqttc::Event::Incoming(rumqttc::Packet::Disconnect) => {
                p_debug!("Disconnected");
                Unsolicited::Disconnect
            }
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
    root_id: usize,
    id: usize,
    request_sn: usize,
}

#[derive(Debug)]
pub enum RequestError {
    Dash7boardError {
        msg: String,
        trace: Vec<wizzi_macro::Response>,
    },
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
            while let Some(unsolicited) = unsolicited_rx.recv().await {
                let mut listeners = dispatcher_listeners.lock().await;
                let mut to_rm = vec![];
                for (i, listener) in listeners.iter_mut().enumerate() {
                    if listener.send(unsolicited.clone()).await.is_err() {
                        to_rm.push(i);
                    }
                }
                for i in to_rm.into_iter().rev() {
                    listeners.remove(i);
                }
            }
        });

        Ok(Self {
            command_tx,
            company,
            listeners,
            root_id: rand::random(),
            id: 0,
            request_sn: 0,
        })
    }

    fn request_id(&mut self) -> String {
        self.request_sn += 1;
        format!("{}-{}-{}", self.root_id, self.id, self.request_sn)
    }

    pub async fn remote_control(
        &mut self,
        command: remote_control::request::Request,
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
        while let Some(unsolicited) = rx.recv().await {
            if let Unsolicited::RemoteControl(response) = unsolicited {
                if response.meta.rid == request_id {
                    return Ok(response);
                }
            }
        }
        Err(RequestError::ReceiveBackendDead)
    }

    pub async fn unsolicited(&mut self) -> mpsc::Receiver<Unsolicited> {
        let (tx, rx) = mpsc::channel(1);
        self.listeners.lock().await.push(tx);
        rx
    }

    pub async fn real_time_wizzi_macro(
        &mut self,
        request: wizzi_macro::Request,
    ) -> Result<mpsc::Receiver<wizzi_macro::Response>, RequestError> {
        let (out_tx, out_rx) = mpsc::channel(1);

        // Subscribe to response
        let mut rx = self.unsolicited().await;

        // Build request
        let request_s = request.encode();
        let data = request_s.as_bytes().to_vec();
        let request_id = self.request_id();
        let topic = format!("/applink/{}/macro/request/{request_id}", self.company);

        // Send request
        self.command_tx
            .send(Command::Publish { topic, data })
            .await
            .map_err(RequestError::SendBackendDead)?;

        // Wait for response
        tokio::spawn(async move {
            while let Some(unsolicited) = rx.recv().await {
                if let Unsolicited::Macro(response) = unsolicited {
                    if response.meta.rid == request_id {
                        let done = matches!(
                            response.msg,
                            wizzi_macro::Message::Status {
                                status: wizzi_macro::Status::End,
                            }
                        ) || matches!(
                            response.msg,
                            wizzi_macro::Message::Status {
                                status: wizzi_macro::Status::Err { .. },
                            }
                        );
                        if out_tx.send(response).await.is_err() || done {
                            break;
                        }
                    }
                }
            }
        });
        Ok(out_rx)
    }

    pub async fn raw_wizzi_macro(
        &mut self,
        request: wizzi_macro::Request,
    ) -> Result<Vec<wizzi_macro::Response>, RequestError> {
        let mut out = vec![];
        let mut rx = self.real_time_wizzi_macro(request).await?;
        let mut err = None;
        while let Some(response) = rx.recv().await {
            if let wizzi_macro::Message::Status {
                status: wizzi_macro::Status::Err { err: e },
            } = &response.msg
            {
                err = Some(e.to_string());
            }
            out.push(response);
        }
        if let Some(err) = err {
            return Err(RequestError::Dash7boardError {
                msg: err,
                trace: out,
            });
        }
        Ok(out)
    }

    pub async fn wizzi_macro(
        &mut self,
        request: wizzi_macro::Request,
    ) -> Result<HashMap<String, Result<(), String>>, RequestError> {
        let mut ret = HashMap::new();
        for response in self.raw_wizzi_macro(request).await? {
            match response.msg {
                wizzi_macro::Message::DstatusOk { uid } => {
                    ret.insert(uid, Ok(()));
                }
                wizzi_macro::Message::DstatusError { uid, err } => {
                    ret.insert(uid, Err(err));
                }
                _ => {}
            }
        }
        Ok(ret)
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            command_tx: self.command_tx.clone(),
            company: self.company.clone(),
            listeners: self.listeners.clone(),
            root_id: self.root_id,
            id: self.id + 1,
            request_sn: 0,
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::common::test::{load_config, TestConfig};
    use std::sync::{Arc, Mutex, MutexGuard};

    lazy_static! {
        static ref RUNNING: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
    }

    async fn client() -> (Client, TestConfig, MutexGuard<'static, ()>) {
        #![allow(clippy::await_holding_lock)]
        let lock = RUNNING.lock().unwrap();

        let conf = load_config().await;

        let mut options =
            rumqttc::MqttOptions::new(conf.client_id.clone(), conf.roger_server.clone(), 8883);
        options.set_credentials(conf.username.clone(), conf.password.clone());
        options.set_transport(rumqttc::Transport::tls_with_default_config());

        let client = Client::new(options, conf.company.clone(), 1).await.unwrap();
        (client, conf, lock)
    }

    #[tokio::test]
    async fn test_read_uid() {
        #![allow(clippy::await_holding_lock)]
        let (mut client, conf, lock) = client().await;

        let request = remote_control::Request {
            action: remote_control::Action::Read,
            user_type: remote_control::Dash7boardPermission::Admin,
            gmuid: remote_control::GatewayModemUid::Uid(conf.gateway.clone()),
            uid: conf.device.clone(),
            fid: 0,
            field_name: "uid".to_string(),
        };

        let data = hex::decode(conf.device.clone()).unwrap();

        let response = client.remote_control(request).await.unwrap();

        assert_eq!(
            response,
            remote_control::Response {
                meta: remote_control::Meta {
                    uid: Some(conf.device.clone()),
                    guid: Some(conf.gateway.clone()),
                    gmuid: Some(conf.gateway.clone()),
                    rid: response.meta.rid.clone(),
                },
                msg: Ok(remote_control::Message {
                    value: Some(remote_control::Value::Binary(data)),
                }),
            }
        );

        drop(lock);
    }

    #[tokio::test]
    async fn test_wizzi_macro() {
        #![allow(clippy::await_holding_lock)]
        let (mut client, conf, lock) = client().await;
        let request = wizzi_macro::Request {
            site_id: conf.site_id,
            user_type: wizzi_macro::Dash7boardPermission::Admin,
            name: "wp_ping_no_security".to_string(),
            shared_vars: std::collections::HashMap::new(),
            device_vars: std::collections::HashMap::new(),
            device_uids: vec![conf.device.clone()],
            gateway_mode: wizzi_macro::GatewayMode::Best,
        };

        let response = client.raw_wizzi_macro(request).await.unwrap();
        let rid = response[0].meta.rid.clone();

        let expected_sequence = [
            wizzi_macro::Message::Status {
                status: wizzi_macro::Status::Start,
            },
            wizzi_macro::Message::Log { progress: 0.0 },
            wizzi_macro::Message::DstatusOk {
                uid: conf.device.clone(),
            },
            wizzi_macro::Message::Log { progress: 100.0 },
            wizzi_macro::Message::Status {
                status: wizzi_macro::Status::End,
            },
        ];
        let expected = expected_sequence
            .into_iter()
            .map(|msg| wizzi_macro::Response {
                meta: wizzi_macro::Meta { rid: rid.clone() },
                msg,
            })
            .collect::<Vec<_>>();

        assert_eq!(response, expected);

        drop(lock);
    }
}
