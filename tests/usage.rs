use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration as StdDuration;

use akira_billing::error::Error;
use akira_billing::types::{LicenseSyncUsageResponse, SignedLicense};
use akira_billing::usage::{MemoryBuffer, TrackerOptions, UsageBuffer, UsageTracker};
use futures::future::BoxFuture;
use tokio::sync::Mutex;

fn dummy_signed() -> SignedLicense {
    SignedLicense {
        key_id: String::new(),
        algorithm: String::new(),
        payload: String::new(),
        signature: String::new(),
        valid_until: String::new(),
    }
}

#[tokio::test]
async fn flush_pushes_and_refresh() {
    let buf: Arc<dyn UsageBuffer> = Arc::new(MemoryBuffer::new());
    let synced_deltas: Arc<Mutex<HashMap<String, u64>>> = Arc::new(Mutex::new(HashMap::new()));
    let synced_serial = Arc::new(AtomicU64::new(0));
    let refreshed = Arc::new(AtomicU64::new(0));

    let sd = Arc::clone(&synced_deltas);
    let ss = Arc::clone(&synced_serial);
    let sync = Arc::new(move |deltas: HashMap<String, u64>, serial: u64| {
        let sd = Arc::clone(&sd);
        let ss = Arc::clone(&ss);
        Box::pin(async move {
            *sd.lock().await = deltas.clone();
            ss.store(serial, Ordering::SeqCst);
            Ok(LicenseSyncUsageResponse {
                license: dummy_signed(),
                applied: deltas,
                serial: serial + 1,
            })
        }) as BoxFuture<'static, Result<LicenseSyncUsageResponse, Error>>
    });

    let serial =
        Arc::new(|| Box::pin(async { Ok(42u64) }) as BoxFuture<'static, Result<u64, Error>>);

    let r = Arc::clone(&refreshed);
    let on_refresh = Arc::new(move |_resp: LicenseSyncUsageResponse| {
        let r = Arc::clone(&r);
        Box::pin(async move {
            r.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }) as BoxFuture<'static, Result<(), Error>>
    });

    let tracker = UsageTracker::new(TrackerOptions {
        buffer: Arc::clone(&buf),
        sync,
        serial: Some(serial),
        on_refresh: Some(on_refresh),
        flush_interval: None,
    });

    tracker.track("requests_per_day", 3).await.unwrap();
    tracker.track("requests_per_day", 2).await.unwrap();
    tracker.flush().await.unwrap();

    let sd = synced_deltas.lock().await;
    assert_eq!(sd.get("requests_per_day").copied(), Some(5));
    assert_eq!(synced_serial.load(Ordering::SeqCst), 42);
    assert_eq!(refreshed.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn rollback_on_sync_error() {
    let buf: Arc<dyn UsageBuffer> = Arc::new(MemoryBuffer::new());

    let sync = Arc::new(|_: HashMap<String, u64>, _: u64| {
        Box::pin(async {
            Err(Error::Api {
                status: 0,
                code: "boom".into(),
            })
        }) as BoxFuture<'static, Result<LicenseSyncUsageResponse, Error>>
    });

    let tracker = UsageTracker::new(TrackerOptions {
        buffer: Arc::clone(&buf),
        sync,
        serial: None,
        on_refresh: None,
        flush_interval: None,
    });

    tracker.track("f", 4).await.unwrap();
    let err = tracker.flush().await.unwrap_err();
    assert!(matches!(err, Error::Api { .. }));

    let drained = buf.drain().await.unwrap();
    assert_eq!(drained.get("f").copied(), Some(4));
}

#[tokio::test]
async fn start_runs_flush_and_stop_finalizes() {
    let buf: Arc<dyn UsageBuffer> = Arc::new(MemoryBuffer::new());
    let call_count = Arc::new(AtomicU64::new(0));

    let cc = Arc::clone(&call_count);
    let sync = Arc::new(move |deltas: HashMap<String, u64>, _: u64| {
        let cc = Arc::clone(&cc);
        Box::pin(async move {
            cc.fetch_add(1, Ordering::SeqCst);
            Ok(LicenseSyncUsageResponse {
                license: dummy_signed(),
                applied: deltas,
                serial: 0,
            })
        }) as BoxFuture<'static, Result<LicenseSyncUsageResponse, Error>>
    });

    let tracker = Arc::new(UsageTracker::new(TrackerOptions {
        buffer: Arc::clone(&buf),
        sync,
        serial: None,
        on_refresh: None,
        flush_interval: Some(StdDuration::from_millis(10)),
    }));

    tracker.track("f", 1).await.unwrap();
    tracker.start().await;
    tokio::time::sleep(StdDuration::from_millis(200)).await;
    tracker.stop().await.unwrap();

    assert!(call_count.load(Ordering::SeqCst) >= 1);
}
