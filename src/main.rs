use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use log::info;
use paho_mqtt as mqtt;

use mqtt_bench::cli::{Cli, Commands};
use prometheus::{linear_buckets, Encoder, Histogram, HistogramOpts, Registry, TextEncoder};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let r = Registry::new();

    let conn_histo_opts = HistogramOpts::new("conn_histo", "Connect Latency Histogram")
        .buckets(linear_buckets(0.0, 100.0, 10).unwrap());
    let conn_histo = Histogram::with_opts(conn_histo_opts).unwrap();
    r.register(Box::new(conn_histo.clone())).unwrap();

    let pub_histo_opts = HistogramOpts::new("pub_histo", "Publish MQTT Message Latency")
        .buckets(linear_buckets(0.0, 10.0, 20).unwrap());
    let pub_histo = Histogram::with_opts(pub_histo_opts).unwrap();
    r.register(Box::new(pub_histo.clone())).unwrap();

    let sub_histo_opts = HistogramOpts::new("sub_histo", "E2E MQTT Message Delivery Latency")
        .buckets(linear_buckets(0.0, 10.0, 20).unwrap());
    let sub_histo = Histogram::with_opts(sub_histo_opts).unwrap();
    r.register(Box::new(sub_histo.clone())).unwrap();

    let cli = Cli::parse();
    match cli.command {
        Some(cmd) => match cmd {
            Commands::Connect { common } => {
                let client = mqtt_bench::client::Client::new(
                    &common,
                    "rust_client_id",
                    conn_histo.clone(),
                    pub_histo.clone(),
                    pub_histo.clone(),
                )
                .context("Failed to create MQTT client")?;
                client.connect().await?;
                info!(
                    "Connection to {} established with client-id={}",
                    common.connection_string(),
                    client.client_id()
                );
            }
            Commands::Pub {
                common,
                topic,
                message_size,
                payload,
            } => {
                let client = mqtt_bench::client::Client::new(
                    &common,
                    "rust_client_id",
                    conn_histo,
                    pub_histo,
                    sub_histo,
                )
                .context("Failed to create MQTT client")?;
                client.connect().await?;
                info!(
                    "Connection to {} established with client-id={}",
                    common.connection_string(),
                    client.client_id()
                );

                let message = mqtt::MessageBuilder::new()
                    .topic(&topic)
                    .payload(payload.unwrap_or_else(|| "a".repeat(message_size as usize).into()))
                    .qos(common.qos)
                    .finalize();
                client.publish(message).await?;
                info!("Published Message OK");
            }

            Commands::Sub { common, topic } => {
                let client = mqtt_bench::client::Client::new(
                    &common,
                    "rust_client_id",
                    conn_histo.clone(),
                    pub_histo.clone(),
                    sub_histo.clone(),
                )
                .context("Failed to create MQTT client")?;
                client.connect().await?;
                info!(
                    "Connection to {} established with client-id={}",
                    common.connection_string(),
                    client.client_id()
                );
                client.subscribe(&topic, common.qos).await?;
                info!("Subscribed to topic {}", topic);
                loop {
                    sleep(Duration::from_secs(1)).await;
                }
            }
        },

        None => {
            println!("No command specified");
        }
    }

    // let server_uri = "ssl://mqtt-mw5b95mv-gz-public.mqtt.tencenttdmq.com:8883";

    // let create_opts = mqtt::CreateOptionsBuilder::new_v3()
    //     .client_id("rust_client_id")
    //     .server_uri(server_uri)
    //     .mqtt_version(4)
    //     .persistence(PersistenceType::None)
    //     .send_while_disconnected(false)
    //     .allow_disconnected_send_at_anytime(false)
    //     .finalize();

    // let connect_opts = mqtt::ConnectOptionsBuilder::new_v3()
    //     .clean_session(true)
    //     .user_name("root")
    //     .password("password")
    //     .connect_timeout(Duration::from_secs(5))
    //     .keep_alive_interval(Duration::from_secs(3))
    //     .ssl_options(SslOptionsBuilder::new().finalize())
    //     .finalize();

    // let client =
    //     mqtt::AsyncClient::new(create_opts).context("Failed to create MQTT AsyncClient")?;

    // client.set_connected_callback(|cli| {
    //     println!(
    //         "Connected to the MQTT server client-id: {}",
    //         cli.client_id()
    //     );
    // });

    // client.set_connection_lost_callback(|c| {
    //     println!("Connection lost client-id: {}", c.client_id());
    // });

    // let _connect_result = client
    //     .connect(connect_opts)
    //     .await
    //     .context("Failed to connect to the MQTT server")?;

    // let semaphore = Arc::new(Semaphore::new(0));
    // let counter = Arc::clone(&semaphore);
    // client.set_message_callback(move |_client, _msg| {
    //     counter.add_permits(1);
    // });
    // client.subscribe_many(&["home/#"], &[mqtt::QOS_1]).await?;

    // let total = 16;

    // for _ in 0..total {
    //     let instant = std::time::Instant::now();
    //     client
    //         .publish(
    //             MessageBuilder::new()
    //                 .topic("home/1")
    //                 .payload(b"test")
    //                 .qos(mqtt::QOS_1)
    //                 .finalize(),
    //         )
    //         .await
    //         .context("Failed to publish message")?;
    //     let elapsed = instant.elapsed().as_millis();
    //     histo.observe(elapsed as f64);
    // }

    sleep(Duration::from_secs(3)).await;

    {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = r.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        println!("{}", String::from_utf8(buffer).unwrap());
    }

    Ok(())
}
