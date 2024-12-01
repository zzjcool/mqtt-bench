use std::sync::Arc;

use clap::Parser;
use log::{info, trace};

use mqtt_bench::cli::{Cli, Commands};
use mqtt_bench::state::{ctrl_c, print_stats, State};

use mqtt_bench::command::{benchmark, connect, publish, subscribe};
use mqtt_bench::statistics::Statistics;
use tokio::sync::mpsc::channel;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::builder().format_timestamp_millis().init();

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
                connect(&common, &state, &statistics).await?;
            }
            Commands::Pub {
                common,
                mut pub_options,
            } => {
                if 0 == pub_options.topic_number {
                    pub_options.topic_number = common.total;
                    info!(
                        "Now that --topic-number is 0, it will be set to --topic-number={}",
                        common.total
                    );
                }

                publish(&common, &state, &statistics, &pub_options).await?;
            }

            Commands::Sub {
                common,
                mut sub_options,
            } => {
                if 0 == sub_options.topic_number {
                    sub_options.topic_number = common.total;
                    info!(
                        "Now that --topic-number is 0, it will be set to --topic-number={}",
                        common.total
                    );
                }

                subscribe(&common, &state, &statistics, &sub_options).await?;
            }

            Commands::Benchmark {
                common,
                mut pub_options,
            } => {
                if 0 == pub_options.topic_number {
                    pub_options.topic_number = common.total;
                    info!(
                        "Now that --topic-number is 0, it will be set to --topic-number={}",
                        common.total
                    );
                }

                benchmark(&common, &state, &statistics, &pub_options).await?;
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
