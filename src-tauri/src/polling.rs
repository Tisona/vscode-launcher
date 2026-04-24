use crate::running::Poller;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const POLL_INTERVAL: Duration = Duration::from_secs(5);
pub const EVENT_RUNNING_UPDATED: &str = "running-workspaces-updated";

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        eprintln!(
            "[polling] thread started, interval={}s",
            POLL_INTERVAL.as_secs()
        );
        let mut poller = Poller::new();
        let mut tick_no: u64 = 0;
        loop {
            tick_no += 1;
            let current = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| poller.tick()))
                .unwrap_or_default();
            eprintln!(
                "[polling] tick {}: {} running workspace(s)",
                tick_no,
                current.len()
            );
            for w in &current {
                eprintln!(
                    "[polling]   - {} cpu={:.1}% ram={} MB windows={}",
                    w.path.display(),
                    w.cpu,
                    w.ram_bytes / 1_048_576,
                    w.window_count
                );
            }
            match app.emit(EVENT_RUNNING_UPDATED, &current) {
                Ok(()) => {}
                Err(e) => eprintln!("[polling] emit failed: {}", e),
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    });
}
