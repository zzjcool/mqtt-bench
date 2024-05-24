use crate::cli::Common;
use crate::state::State;
use crate::statistics::Statistics;
use anyhow::Context;
use log::{error, trace};
use std::sync::Arc;
use std::time::Duration;

pub async fn connect(common: &Common, state: &Arc<State>, statistics: &Statistics) {
    let semaphore = Arc::new(tokio::sync::Semaphore::new(common.concurrency));
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

    if common.show_statistics {
        statistics.show_statistics();
    }

    tokio::time::sleep(Duration::from_secs(300)).await;
}
