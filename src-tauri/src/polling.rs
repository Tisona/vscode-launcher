use crate::running::Poller;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
pub const EVENT_RUNNING_UPDATED: &str = "running-workspaces-updated";

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut poller = Poller::new();
        loop {
            std::thread::sleep(POLL_INTERVAL);
            let current = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| poller.tick()))
                .unwrap_or_default();
            let _ = app.emit(EVENT_RUNNING_UPDATED, &current);
        }
    });
}
