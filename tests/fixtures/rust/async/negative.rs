use tokio::sync::Mutex;

static LOCK: Mutex<i32> = Mutex::const_new(0);

async fn some_future() {}

async fn good_async() {
    {
        let mut guard = LOCK.lock().await;
        *guard += 1;
    }

    let shutdown = tokio_util::sync::CancellationToken::new();
    tokio::select! {
        _ = shutdown.cancelled() => {}
        _ = some_future() => {}
    }
}