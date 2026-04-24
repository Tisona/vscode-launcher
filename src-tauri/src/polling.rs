use crate::commands::AppState;
use crate::running::Poller;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const POLL_INTERVAL: Duration = Duration::from_secs(5);
pub const EVENT_RUNNING_UPDATED: &str = "running-workspaces-updated";

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut poller = Poller::new();
        loop {
            let current = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| poller.tick()))
                .unwrap_or_default();
            if let Some(state) = app.try_state::<AppState>() {
                *state.running.lock().unwrap() = current.clone();
            }
            let _ = app.emit(EVENT_RUNNING_UPDATED, &current);
            std::thread::sleep(POLL_INTERVAL);
        }
    });
}
