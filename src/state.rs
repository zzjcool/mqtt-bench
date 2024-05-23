use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

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