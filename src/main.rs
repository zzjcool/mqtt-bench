use std::sync::Mutex;
use std::{
    sync::{
        atomic::{
            AtomicBool, AtomicUsize,
            Ordering::{self, Relaxed, SeqCst},
        },
        Arc,
    },
    time::Duration,
};

use anyhow::Context;
use clap::{CommandFactory, Parser};
use log::{debug, error, info, trace};
use paho_mqtt as mqtt;

use mqtt_bench::cli::{Cli, Commands};
use prometheus::{linear_buckets, Encoder, Histogram, HistogramOpts, Registry, TextEncoder};
use tokio::{sync::mpsc::channel, time::sleep};

use clap_help::Printer;

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

    if cli.help {
        Printer::new(Cli::command()).print_help();
        return Ok(());
    }

    let stopped = Arc::new(AtomicBool::new(false));

    let stopped_flag = stopped.clone();
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            info!("Ctrl-C received, stopping");
            stopped_flag.store(true, Ordering::Relaxed);
        }
        tokio::signal::ctrl_c().await.unwrap();
        stopped_flag.store(true, Relaxed);
    });

    match cli.command {
        Some(cmd) => match cmd {
            Commands::Connect { common } => {
                let connected = Arc::new(AtomicUsize::new(0));
                let (tx, mut rx) = channel::<()>(1);
                tokio::spawn({
                    let connected = connected.clone();
                    let stopped_flag = stopped.clone();
                    async move {
                        loop {
                            tokio::select! {
                                _ = rx.recv() => {
                                    debug!("Received signal to stop");
                                    break;
                                }
                                _ = sleep(Duration::from_secs(1)) => {
                                    info!("{} client(s) connected", connected.load(SeqCst));
                                    if stopped_flag.load(Relaxed) {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                });

                let total = Arc::new(AtomicUsize::new(0));
                let clients = Arc::new(Mutex::new(Vec::new()));

                let semaphore = Arc::new(tokio::sync::Semaphore::new(common.concurrency));
                for id in 0..common.total {
                    let semaphore = Arc::clone(&semaphore);
                    let clients = Arc::clone(&clients);
                    let conn_histogram = conn_histo.clone();
                    let pub_histogram = pub_histo.clone();
                    let sub_histogram = sub_histo.clone();
                    let connected = Arc::clone(&connected);
                    let total = Arc::clone(&total);
                    let common = common.clone();
                    let semaphore = Arc::clone(&semaphore);
                    let stop_flag = Arc::clone(&stopped);
                    tokio::spawn(async move {
                        let client = match mqtt_bench::client::Client::new(
                            common,
                            &format!("client_{}", id),
                            conn_histogram,
                            pub_histogram,
                            sub_histogram,
                        )
                        .context("Failed to create MQTT client")
                        {
                            Ok(client) => client,
                            Err(e) => {
                                error!("{}", e.to_string());
                                total.fetch_add(1, Ordering::Relaxed);
                                return;
                            }
                        };

                        if let Ok(permit) = semaphore.acquire().await {
                            if stop_flag.load(Ordering::Relaxed) {
                                total.fetch_add(1, Ordering::Relaxed);
                                return;
                            }

                            if let Err(e) = client.connect().await {
                                error!("{}", e.to_string());
                                total.fetch_add(1, Ordering::Relaxed);
                                return;
                            }
                            drop(permit);
                        }

                        match clients.lock() {
                            Ok(mut guard) => guard.push(client),
                            Err(e) => {
                                error!("Unable to acquire lock: {}", e.to_string())
                            }
                        }
                        connected.fetch_add(1, SeqCst);
                        total.fetch_add(1, Ordering::Relaxed);
                    });
                }

                loop {
                    if total.load(Ordering::Relaxed) < common.total {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    } else {
                        break;
                    }
                }

                if common.show_statistics {
                    show_statistics(&r);
                }

                // Attempt to signal task that is printing statistics.
                if let Err(_e) = tx.send(()).await {
                    trace!("Should have received Ctrl-C signal");
                    debug_assert_eq!(true, stopped.load(Ordering::Relaxed));
                }
            }
            Commands::Pub {
                common,
                topic,
                message_size,
                payload,
            } => {
                let client = mqtt_bench::client::Client::new(
                    common.clone(),
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
                    common.clone(),
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
