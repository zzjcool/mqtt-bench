use super::cli::Common;
use anyhow::Context;
use log::info;
use mqtt::AsyncClient;
use paho_mqtt as mqtt;
use prometheus::Histogram;
use tokio::time::Instant;

pub struct Client<'a> {
    opts: &'a Common,
    pub inner: AsyncClient,
    conn_histo: Histogram,
    pub_histo: Histogram,
    sub_histo: Histogram,
}

impl<'a> Client<'a> {
    pub fn new(
        opts: &'a Common,
        client_id: &str,
        conn_histo: Histogram,
        pub_histo: Histogram,
        sub_histo: Histogram,
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

        let client =
            mqtt::AsyncClient::new(create_opts).context("Failed to create MQTT AsyncClient")?;
        let e2e_histo = sub_histo.clone();
        client.set_message_callback(move |_client, message| {
            if let Some(message) = message {
                info!("Received message, topic={}", message.topic());
                e2e_histo.observe(1 as f64);
            }
        });

        Ok(Self {
            opts,
            inner: client,
            conn_histo,
            pub_histo,
            sub_histo,
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
                    .verify(false)
                    .enable_server_cert_auth(false)
                    .finalize(),
            )
            .finalize();

        self.inner.set_connected_callback(|cli| {
            info!(
                "Connected to server_uri={} with client-id={}",
                cli.server_uri(),
                cli.client_id()
            );
        });

        self.inner.set_connection_lost_callback(|c| {
            info!("Connection lost client-id: {}", c.client_id());
        });
        let instant = Instant::now();
        let _connect_result = self
            .inner
            .connect(connect_opts)
            .await
            .context("Failed to connect to the MQTT server")?;
        self.conn_histo
            .observe(instant.elapsed().as_millis() as f64);
        Ok(())
    }

    pub async fn publish(&self, message: mqtt::Message) -> Result<(), anyhow::Error> {
        let instant = Instant::now();
        let pub_result = self
            .inner
            .publish(message)
            .await
            .context("Failed to publish message");
        self.pub_histo.observe(instant.elapsed().as_millis() as f64);
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
