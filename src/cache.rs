use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{Mutex, Notify},
    time::Instant,
};

use crate::{AppState, metrics::SnapshotEntry};

pub struct PriceCache {
    inner: Mutex<PriceMutex>,
    duration: Duration,
}

pub struct PriceMutex {
    value: Option<Arc<SnapshotEntry>>,
    expires_at: Option<Instant>,
    computing: bool,
    notify: Arc<Notify>,
}

impl PriceCache {
    pub fn new(duration: Duration) -> Self {
        Self {
            inner: Mutex::new(PriceMutex {
                value: None,
                expires_at: None,
                computing: false,
                notify: Arc::new(Notify::new()),
            }),
            duration,
        }
    }

    pub async fn get_or_compute(&self, state: Arc<AppState>) -> anyhow::Result<Arc<SnapshotEntry>> {
        loop {
            let (maybe_cached, should_compute, notify) = {
                let mut guard = self.inner.lock().await;
                let now = Instant::now();

                if let (Some(value), Some(expires_at)) = (&guard.value, guard.expires_at) {
                    if now < expires_at {
                        return Ok(value.clone());
                    }
                }

                if guard.computing {
                    (None, false, guard.notify.clone())
                } else {
                    guard.computing = true;
                    (None, true, guard.notify.clone())
                }
            };

            if let Some(v) = maybe_cached {
                return Ok(v);
            }

            if should_compute {
                let result = state.metrics.compute(state.clone()).await;

                let mut guard = self.inner.lock().await;
                guard.computing = false;

                match result {
                    Ok(value) => {
                        let value = Arc::new(value);
                        guard.value = Some(value.clone());
                        guard.expires_at = Some(Instant::now() + self.duration);
                        notify.notify_waiters();
                        return Ok(value);
                    }
                    Err(err) => {
                        // wake waiters so they can retry / observe failure path
                        notify.notify_waiters();
                        return Err(err);
                    }
                }
            } else {
                notify.notified().await;
            }
        }
    }
}
