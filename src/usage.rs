use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::future::BoxFuture;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::error::Error;
use crate::types::LicenseSyncUsageResponse;

#[async_trait]
pub trait UsageBuffer: Send + Sync {
    async fn add(&self, feature: &str, delta: u64) -> Result<(), Error>;
    async fn drain(&self) -> Result<HashMap<String, u64>, Error>;
    async fn restore(&self, deltas: HashMap<String, u64>) -> Result<(), Error>;
}

pub struct MemoryBuffer {
    state: Mutex<HashMap<String, u64>>,
}

impl MemoryBuffer {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MemoryBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UsageBuffer for MemoryBuffer {
    async fn add(&self, feature: &str, delta: u64) -> Result<(), Error> {
        let mut guard = self.state.lock().await;
        *guard.entry(feature.to_string()).or_insert(0) += delta;
        Ok(())
    }

    async fn drain(&self) -> Result<HashMap<String, u64>, Error> {
        let mut guard = self.state.lock().await;
        Ok(std::mem::take(&mut *guard))
    }

    async fn restore(&self, deltas: HashMap<String, u64>) -> Result<(), Error> {
        let mut guard = self.state.lock().await;
        for (k, v) in deltas {
            *guard.entry(k).or_insert(0) += v;
        }
        Ok(())
    }
}

pub type SyncFn = Arc<
    dyn Fn(HashMap<String, u64>, u64) -> BoxFuture<'static, Result<LicenseSyncUsageResponse, Error>>
        + Send
        + Sync,
>;

pub type SerialFn = Arc<dyn Fn() -> BoxFuture<'static, Result<u64, Error>> + Send + Sync>;

pub type RefreshFn =
    Arc<dyn Fn(LicenseSyncUsageResponse) -> BoxFuture<'static, Result<(), Error>> + Send + Sync>;

pub struct TrackerOptions {
    pub buffer: Arc<dyn UsageBuffer>,
    pub sync: SyncFn,
    pub serial: Option<SerialFn>,
    pub on_refresh: Option<RefreshFn>,
    pub flush_interval: Option<Duration>,
}

pub struct UsageTracker {
    buffer: Arc<dyn UsageBuffer>,
    sync: SyncFn,
    serial: Option<SerialFn>,
    on_refresh: Option<RefreshFn>,
    flush_interval: Duration,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl UsageTracker {
    pub fn new(opts: TrackerOptions) -> Self {
        let flush_interval = opts
            .flush_interval
            .filter(|d| !d.is_zero())
            .unwrap_or_else(|| Duration::from_secs(300));
        Self {
            buffer: opts.buffer,
            sync: opts.sync,
            serial: opts.serial,
            on_refresh: opts.on_refresh,
            flush_interval,
            handle: Mutex::new(None),
        }
    }

    pub async fn track(&self, feature: &str, delta: u64) -> Result<(), Error> {
        if delta == 0 {
            return Ok(());
        }
        self.buffer.add(feature, delta).await
    }

    pub async fn flush(&self) -> Result<(), Error> {
        let deltas = self.buffer.drain().await?;
        if deltas.is_empty() {
            return Ok(());
        }

        let serial = if let Some(serial_fn) = &self.serial {
            match serial_fn().await {
                Ok(s) => s,
                Err(err) => {
                    let _ = self.buffer.restore(deltas).await;
                    return Err(err);
                }
            }
        } else {
            0
        };

        let resp = match (self.sync)(deltas.clone(), serial).await {
            Ok(r) => r,
            Err(err) => {
                let _ = self.buffer.restore(deltas).await;
                return Err(err);
            }
        };

        if let Some(on_refresh) = &self.on_refresh {
            on_refresh(resp).await?;
        }
        Ok(())
    }

    pub async fn start(self: &Arc<Self>) {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return;
        }
        let interval_dur = self.flush_interval;
        let me = Arc::clone(self);
        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval_dur);
            ticker.tick().await;
            loop {
                ticker.tick().await;
                let _ = me.flush().await;
            }
        });
        *guard = Some(handle);
    }

    pub async fn stop(&self) -> Result<(), Error> {
        let mut guard = self.handle.lock().await;
        if let Some(handle) = guard.take() {
            handle.abort();
        }
        drop(guard);
        self.flush().await
    }
}
