use super::cli::Common;
use crate::state::State;
use crate::statistics::LatencyHistogram;
use anyhow::Context;
use byteorder::ReadBytesExt;
use bytes::Buf;
use log::{debug, error, info, trace};
use mqtt::AsyncClient;
use paho_mqtt as mqtt;
use std::io::Cursor;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Semaphore;
use tokio::time::Instant;

pub struct Client {
    opts: Common,
    connect_semaphore: Arc<Semaphore>,
    pub inner: AsyncClient,
    latency: LatencyHistogram,
    state: Arc<State>,
}

impl Client {
    pub fn new(
        opts: Common,
        connect_semaphore: Arc<Semaphore>,
        client_id: &str,
        latency: LatencyHistogram,
        state: Arc<State>,
    ) -> Result<Self, anyhow::Error> {
        let server_uri = if opts.ssl {
            format!("ssl://{}:{}", opts.host, opts.port.unwrap_or(8883))
        } else {
            format!("tcp://{}:{}", opts.host, opts.port.unwrap_or(1883))
        };

        let create_opts = mqtt::CreateOptionsBuilder::new_v3()
            .client_id(client_id)
            .server_uri(server_uri)
            .mqtt_version(4)
            .persistence(mqtt::PersistenceType::None)
            .send_while_disconnected(false)
            .allow_disconnected_send_at_anytime(false)
            .finalize();

        let client = AsyncClient::new(create_opts).context("Failed to create MQTT AsyncClient")?;
        let e2e_histogram = latency.subscribe.clone();
        let _state = Arc::clone(&state);
        client.set_message_callback(move |_client, message| {
            if let Some(message) = message {
                _state.on_receive();
                let payload = message.payload();
                let mut cursor = Cursor::new(payload);
                if cursor.remaining() > std::mem::size_of::<u128>() {
                    match cursor.read_u128::<byteorder::LittleEndian>() {
                        Ok(ts) => {
                            let now = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            if now >= ts {
                                e2e_histogram.observe((now - ts) as f64);
                            }
                        }
                        Err(e) => {
                            error!("Failed to read timestamp from payload: {}", e);
                        }
                    }
                }
                trace!("Received message, topic={}", message.topic());
            }
        });

        Ok(Self {
            opts,
            connect_semaphore,
            inner: client,
            latency,
            state,
        })
    }

    pub fn client_id(&self) -> String {
        self.inner.client_id()
    }

    pub async fn connect(&self) -> Result<(), anyhow::Error> {
        let connect_opts = mqtt::ConnectOptionsBuilder::new_v3()
            .clean_session(true)
            .user_name(&self.opts.user_name)
            .password(&self.opts.password)
            .connect_timeout(std::time::Duration::from_secs(5))
            .keep_alive_interval(std::time::Duration::from_secs(3))
            .ssl_options(
                mqtt::SslOptionsBuilder::new()
                    .verify(self.opts.verify)
                    .enable_server_cert_auth(self.opts.auth_server_certificate)
                    .ssl_version(mqtt::SslVersion::Tls_1_2)
                    .finalize(),
            )
            .finalize();

        self.inner.set_connected_callback(|cli| {
            debug!(
                "Connected to server_uri={} with client-id={}",
                cli.server_uri(),
                cli.client_id()
            );
        });

        self.inner.set_connection_lost_callback(|c| {
            info!("Connection lost client-id: {}", c.client_id());
        });

        let _permit = self
            .connect_semaphore
            .acquire()
            .await
            .context("Failed to acquire connect permit")?;

        if self.state.stopped() {
            return Ok(());
        }

        let instant = Instant::now();
        if let Err(e) = self
            .inner
            .connect(connect_opts)
            .await
            .context("Failed to connect to the MQTT server")
        {
            self.state.on_connect_failure();
            return Err(e);
        }
        self.latency
            .connect
            .observe(instant.elapsed().as_millis() as f64);
        self.state.on_connected();
        Ok(())
    }

    pub fn connected(&self) -> bool {
        self.inner.is_connected()
    }

    pub async fn publish(&self, message: mqtt::Message) -> Result<(), anyhow::Error> {
        let topic = message.topic().to_owned();
        let instant = Instant::now();
        if let Err(e) = self
            .inner
            .publish(message)
            .await
            .context("Failed to publish message")
        {
            self.state.on_publish_failure();
            return Err(e);
        }

        self.latency
            .publish
            .observe(instant.elapsed().as_millis() as f64);
        self.state.on_publish();
        trace!("{} published a message to {}", self.client_id(), topic);
        Ok(())
    }

    pub async fn subscribe(&self, topic: &str, qos: i32) -> Result<(), anyhow::Error> {
        let _sub_result = self.inner.subscribe(topic, qos).await.context(format!(
            "Failed to subscribe to the topic={}, qos={}",
            topic, qos
        ))?;
        info!("{} subscribed {} with qos={}", self.client_id(), topic, qos);
        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if self.connected() {
            if let Err(e) = self.inner.disconnect(None).wait() {
                error!("Failed to disconnect client: {}", e);
            }
        }
    }
}
