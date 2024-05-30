use super::cli::Common;
use crate::statistics::LatencyHistogram;
use anyhow::Context;
use log::{debug, info, trace};
use mqtt::AsyncClient;
use paho_mqtt as mqtt;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::Instant;

pub struct Client {
    opts: Common,
    connect_semaphore: Arc<Semaphore>,
    pub inner: AsyncClient,
    latency: LatencyHistogram,
}

impl Client {
    pub fn new(
        opts: Common,
        connect_semaphore: Arc<Semaphore>,
        client_id: &str,
        latency: LatencyHistogram,
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
        client.set_message_callback(move |_client, message| {
            if let Some(message) = message {
                info!("Received message, topic={}", message.topic());
                e2e_histogram.observe(1f64);
            }
        });

        Ok(Self {
            opts,
            connect_semaphore,
            inner: client,
            latency,
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
        let instant = Instant::now();
        let _connect_result = self
            .inner
            .connect(connect_opts)
            .await
            .context("Failed to connect to the MQTT server")?;
        self.latency
            .connect
            .observe(instant.elapsed().as_millis() as f64);
        Ok(())
    }

    pub fn connected(&self) -> bool {
        self.inner.is_connected()
    }

    pub async fn publish(&self, message: mqtt::Message) -> Result<(), anyhow::Error> {
        let topic = message.topic().to_owned();
        let instant = Instant::now();
        let pub_result = self
            .inner
            .publish(message)
            .await
            .context("Failed to publish message");
        self.latency
            .publish
            .observe(instant.elapsed().as_millis() as f64);
        trace!("{} published a message to {}", self.client_id(), topic);
        pub_result
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
