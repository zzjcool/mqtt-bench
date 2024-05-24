use log::{debug, info};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;

pub struct State {
    connected: AtomicUsize,
    total: AtomicUsize,
    stopped: AtomicBool,
}

impl State {
    pub fn new() -> Arc<State> {
        let state = Self {
            connected: AtomicUsize::new(0),
            total: AtomicUsize::new(0),
            stopped: AtomicBool::new(false),
        };
        Arc::new(state)
    }

    pub fn connected(&self) -> usize {
        self.connected.load(Ordering::Relaxed)
    }

    pub fn total(&self) -> usize {
        self.total.load(Ordering::Relaxed)
    }

    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Relaxed)
    }

    pub fn on_connect_failure(&self) {
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn on_connected(&self) {
        self.total.fetch_add(1, Ordering::Relaxed);
        self.connected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn stop_flag(&self) -> &AtomicBool {
        &self.stopped
    }
}

pub fn ctrl_c(state: Arc<State>) {
    let _ = tokio::task::Builder::new()
        .name("ctrl_c")
        .spawn(async move {
            if let Ok(()) = tokio::signal::ctrl_c().await {
                info!("Ctrl-C received, stopping");
                state.stop_flag().store(true, Ordering::Relaxed);
            }
            tokio::signal::ctrl_c().await.unwrap();
            state.stop_flag().store(true, Ordering::Relaxed);
        });
}

pub fn print_stats(state: Arc<State>, mut rx: Receiver<()>) {
    let _ = tokio::task::Builder::new().name("stats_printer").spawn({
        async move {
            loop {
                tokio::select! {
                    _ = rx.recv() => {
                        debug!("Received signal to stop");
                        break;
                    }
                    _ = sleep(Duration::from_secs(1)) => {
                        info!("{} client(s) connected", state.connected());
                        if state.stop_flag().load(Ordering::Relaxed) {
                            break;
                        }
                    }
                }
            }
        }
    });
}
