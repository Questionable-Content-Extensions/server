//! Shared rate-limited trigger for manually kicking off the background comic updater early.

use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::Notify;

const MIN_INTERVAL: Duration = Duration::from_mins(5);

/// Lets editors request an out-of-schedule run of the background comic updater, while
/// enforcing a minimum interval since the last run (scheduled or manual).
#[derive(Debug)]
pub struct ComicUpdaterTrigger {
    last_run: Mutex<Instant>,
    notify: Notify,
}

impl Default for ComicUpdaterTrigger {
    fn default() -> Self {
        Self::new()
    }
}

impl ComicUpdaterTrigger {
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_run: Mutex::new(Instant::now()),
            notify: Notify::new(),
        }
    }

    /// Records that a run (scheduled or manual) has just started.
    pub fn record_run(&self) {
        *self.last_run.lock().expect("lock is not poisoned") = Instant::now();
    }

    /// Requests an immediate run. Returns `Err` with the remaining wait time if the minimum
    /// interval since the last run hasn't elapsed yet.
    pub fn request_run(&self) -> Result<(), Duration> {
        let elapsed = self
            .last_run
            .lock()
            .expect("lock is not poisoned")
            .elapsed();
        if let Some(remaining) = MIN_INTERVAL.checked_sub(elapsed) {
            return Err(remaining);
        }

        self.notify.notify_one();
        Ok(())
    }

    /// Waits until a manual run has been requested via [`Self::request_run`].
    pub async fn notified(&self) {
        self.notify.notified().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_run_succeeds_when_last_run_is_old_enough() {
        let trigger = ComicUpdaterTrigger {
            last_run: Mutex::new(
                Instant::now()
                    .checked_sub(Duration::from_mins(10))
                    .expect("instant far enough in the past"),
            ),
            notify: Notify::new(),
        };
        assert!(trigger.request_run().is_ok());
    }

    #[test]
    fn request_run_fails_when_rate_limited() {
        let trigger = ComicUpdaterTrigger::new();
        let result = trigger.request_run();
        assert!(result.is_err());
        assert!(result.unwrap_err() <= MIN_INTERVAL);
    }

    #[tokio::test]
    async fn request_run_wakes_a_waiter() {
        let trigger = ComicUpdaterTrigger {
            last_run: Mutex::new(
                Instant::now()
                    .checked_sub(Duration::from_mins(10))
                    .expect("instant far enough in the past"),
            ),
            notify: Notify::new(),
        };

        let wait = trigger.notified();
        trigger.request_run().expect("not rate limited");
        tokio::time::timeout(Duration::from_secs(1), wait)
            .await
            .expect("notified() should resolve after request_run()");
    }

    #[test]
    fn record_run_resets_the_rate_limit() {
        let trigger = ComicUpdaterTrigger {
            last_run: Mutex::new(
                Instant::now()
                    .checked_sub(Duration::from_mins(10))
                    .expect("instant far enough in the past"),
            ),
            notify: Notify::new(),
        };
        trigger.record_run();
        let result = trigger.request_run();
        assert!(result.is_err());
    }
}
