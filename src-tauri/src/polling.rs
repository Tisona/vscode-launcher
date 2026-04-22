use crate::running;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
pub const EVENT_RUNNING_UPDATED: &str = "running-workspaces-updated";

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last: HashSet<PathBuf> = HashSet::new();
        loop {
            let current = std::panic::catch_unwind(running::running_workspaces)
                .unwrap_or_default();
            if current != last {
                let payload: Vec<PathBuf> = current.iter().cloned().collect();
                let _ = app.emit(EVENT_RUNNING_UPDATED, payload);
                last = current;
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    });
}
