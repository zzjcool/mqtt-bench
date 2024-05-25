use crate::cli::{Common, PubOptions};
use crate::state::State;
use crate::statistics::Statistics;
use anyhow::Context;
use log::{error, info, trace, warn};
use paho_mqtt::MessageBuilder;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

pub async fn connect(common: &Common, state: &Arc<State>, statistics: &Statistics) {
    let semaphore = Arc::new(Semaphore::new(common.concurrency));
    for id in 0..common.total {
        let client = match crate::client::Client::new(
            common.clone(),
            Arc::clone(&semaphore),
            &format!("client_{}", id),
            statistics.latency.clone(),
        )
        .context(format!("Failed to create MQTT client client_{}", id))
        {
            Ok(client) => client,
            Err(e) => {
                error!("{}", e.to_string());
                state.on_connect_failure();
                break;
            }
        };

        let client_state = Arc::clone(state);
        let _ = tokio::task::Builder::new()
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

    await_running(common, state).await;

    if common.show_statistics {
        statistics.show_statistics();
    }
}

pub async fn publish(
    common: &Common,
    state: &Arc<State>,
    statistics: &Statistics,
    pub_options: &PubOptions,
) -> Result<(), anyhow::Error> {
    let semaphore = Arc::new(Semaphore::new(common.concurrency));
    for id in 0..common.total {
        let client = match crate::client::Client::new(
            common.clone(),
            Arc::clone(&semaphore),
            &format!("client_{}", id),
            statistics.latency.clone(),
        )
        .context(format!("Failed to create MQTT client client_{}", id))
        {
            Ok(client) => client,
            Err(e) => {
                error!("{}", e.to_string());
                state.on_connect_failure();
                break;
            }
        };

        let payload = pub_options
            .payload
            .clone()
            .unwrap_or_else(|| "a".repeat(pub_options.message_size as usize));

        let message = MessageBuilder::new()
            .topic(&pub_options.topic)
            .payload(payload)
            .qos(common.qos)
            .finalize();

        let pub_interval = Duration::from_millis(common.interval);

        let client_state = Arc::clone(state);
        let _ = tokio::task::Builder::new()
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
                        info!("Benchmark has stopped");
                        break;
                    }

                    if client.connected() {
                        if let Err(e) = client.publish(message.clone()).await {
                            error!("Failed to publish message: {}", e.to_string());
                            break;
                        }
                        client_state.on_publish();

                        if pub_interval.as_millis() > 0 {
                            tokio::time::sleep(pub_interval).await;
                        }
                        continue;
                    }
                    warn!("Client {} has disconnected", client.client_id());
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

    await_running(common, state).await;

    if common.show_statistics {
        statistics.show_statistics();
    }
    Ok(())
}

async fn await_running(common: &Common, state: &Arc<State>) {
    for _ in 0..common.time {
        if state.stopped() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
