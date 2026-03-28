use std::collections::HashMap;
use std::io::BufRead;
use std::sync::Mutex;

struct Semaphore;
impl Semaphore {
    async fn acquire(&self) -> Permit {
        Permit
    }
}

struct Permit;

async fn some_future() {}
async fn fetch_next() {}

static LOCK_A: Mutex<i32> = Mutex::new(0);
static LOCK_B: Mutex<i32> = Mutex::new(0);

async fn bad_async(reader: impl BufRead, mut sink: std::fs::File) {
    let guard = LOCK_A.lock().unwrap();
    let _ = std::fs::read_to_string("/tmp/data.txt");
    for line in reader.lines() {
        let text = line.unwrap();
        sink.write_all(text.as_bytes()).unwrap();
        let _cache: HashMap<String, String> = HashMap::new();
    }
    some_future().await;
    drop(guard);
}

async fn permit_await(semaphore: &Semaphore) {
    let permit = semaphore.acquire().await;
    some_future().await;
    drop(permit);
}

async fn spawn_without_cancel() {
    tokio::spawn(async move {
        let _ = std::fs::read_to_string("/tmp/spawned.txt");
        some_future().await;
    });
}

async fn select_loop() {
    loop {
        tokio::select! {
            _ = fetch_next() => {}
        }
    }
}

fn lock_order_forward() {
    let _a = LOCK_A.lock().unwrap();
    let _b = LOCK_B.lock().unwrap();
}

fn lock_order_reverse() {
    let _b = LOCK_B.lock().unwrap();
    let _a = LOCK_A.lock().unwrap();
}