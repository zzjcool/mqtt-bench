use std::{sync::Arc, time::Duration};

use anyhow::Context;
use clap::Parser;
use log::{info, trace};

use mqtt_bench::cli::{Cli, Commands};
use mqtt_bench::state::{ctrl_c, print_stats, State};

use mqtt_bench::command::{connect, publish};
use mqtt_bench::statistics::Statistics;
use tokio::sync::Semaphore;
use tokio::{sync::mpsc::channel, time::sleep};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    console_subscriber::init();

    let cli = Cli::parse();

    let state = State::new();
    let (tx, rx) = channel::<()>(1);

    ctrl_c(Arc::clone(&state));
    print_stats(Arc::clone(&state), rx);

    let statistics = Statistics::new();

    match cli.command {
        Some(cmd) => match cmd {
            Commands::Connect { common } => {
                connect(&common, &state, &statistics).await;
            }
            Commands::Pub {
                common,
                mut pub_options,
            } => {
                if 0 == pub_options.topic_number {
                    pub_options.topic_number = common.total;
                    info!(
                        "Now that --topic-number is 0, it will be set to --total={}",
                        common.total
                    );
                }

                publish(&common, &state, &statistics, &pub_options).await?;
            }

            Commands::Sub { common, topic } => {
                let semaphore = Arc::new(Semaphore::new(common.concurrency));
                let client = mqtt_bench::client::Client::new(
                    common.clone(),
                    Arc::clone(&semaphore),
                    "rust_client_id",
                    statistics.latency.clone(),
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

    // Attempt to signal task that is printing statistics.
    if let Err(_e) = tx.send(()).await {
        trace!("Should have received Ctrl-C signal");
        debug_assert!(state.stopped());
    }

    Ok(())
}
