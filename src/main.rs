use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use anyhow::Context;
use clap::{CommandFactory, Parser};
use log::{debug, error, info, trace};
use paho_mqtt as mqtt;

use mqtt_bench::cli::{Cli, Commands};
use mqtt_bench::state::State;

use prometheus::{linear_buckets, Encoder, Histogram, HistogramOpts, Registry, TextEncoder};
use tokio::{sync::mpsc::channel, time::sleep};

use clap_help::Printer;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    console_subscriber::init();

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

    if cli.help {
        Printer::new(Cli::command()).print_help();
        return Ok(());
    }

    let state = State::new();

    let ctrl_c_state = Arc::clone(&state);
    let _ = tokio::task::Builder::new()
        .name("ctrl_c")
        .spawn(async move {
            if let Ok(()) = tokio::signal::ctrl_c().await {
                info!("Ctrl-C received, stopping");
                ctrl_c_state.stop_flag().store(true, Ordering::Relaxed);
            }
            tokio::signal::ctrl_c().await.unwrap();
            ctrl_c_state.stop_flag().store(true, Ordering::Relaxed);
        });

    match cli.command {
        Some(cmd) => match cmd {
            Commands::Connect { common } => {
                let semaphore = Arc::new(tokio::sync::Semaphore::new(common.concurrency));
                let (tx, mut rx) = channel::<()>(1);
                let stats_state = Arc::clone(&state);
                let _ = tokio::task::Builder::new().name("stats_printer").spawn({
                    async move {
                        loop {
                            tokio::select! {
                                _ = rx.recv() => {
                                    debug!("Received signal to stop");
                                    break;
                                }
                                _ = sleep(Duration::from_secs(1)) => {
                                    info!("{} client(s) connected", stats_state.connected());
                                    if stats_state.stop_flag().load(Ordering::Relaxed) {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                });

                for id in 0..common.total {
                    let client = match mqtt_bench::client::Client::new(
                        common.clone(),
                        Arc::clone(&semaphore),
                        &format!("client_{}", id),
                        conn_histo.clone(),
                        pub_histo.clone(),
                        sub_histo.clone(),
                    )
                    .context("Failed to create MQTT client")
                    {
                        Ok(client) => client,
                        Err(e) => {
                            error!("{}", e.to_string());
                            state.on_connect_failure();
                            break;
                        }
                    };

                    let client_state = Arc::clone(&state);
                    let _ =
                        tokio::task::Builder::new()
                            .name(&client.client_id())
                            .spawn(async move {
                                if let Err(e) = client.connect().await {
                                    error!("{}", e.to_string());
                                    client_state.on_connect_failure();
                                    return;
                                }
                                client_state.on_connected();

                                loop {
                                    if client_state.stopped() {
                                        break;
                                    }

                                    if client.connected() {
                                        trace!("{} ping...", client.client_id());
                                        tokio::time::sleep(Duration::from_secs(1)).await;
                                        continue;
                                    }
                                    break;
                                }
                            });
                }

                loop {
                    if state.total() < common.total {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    } else {
                        break;
                    }
                }

                if common.show_statistics {
                    show_statistics(&r);
                }

                tokio::time::sleep(Duration::from_secs(300)).await;

                // Attempt to signal task that is printing statistics.
                if let Err(_e) = tx.send(()).await {
                    trace!("Should have received Ctrl-C signal");
                    debug_assert_eq!(true, state.stopped());
                }
            }
            Commands::Pub {
                common,
                topic,
                message_size,
                payload,
            } => {
                let semaphore = Arc::new(tokio::sync::Semaphore::new(common.concurrency));
                let client = mqtt_bench::client::Client::new(
                    common.clone(),
                    Arc::clone(&semaphore),
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
                let semaphore = Arc::new(tokio::sync::Semaphore::new(common.concurrency));
                let client = mqtt_bench::client::Client::new(
                    common.clone(),
                    Arc::clone(&semaphore),
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

    Ok(())
}

fn show_statistics(r: &Registry) {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = r.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    println!("{}", String::from_utf8(buffer).unwrap());
}
