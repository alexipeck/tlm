use std::collections::VecDeque;
use std::io::Error as IoError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::{thread::sleep, time::Duration};
use tlm::scheduler::Encode;
use tlm::ws::run_worker;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let transcode_queue: Arc<Mutex<VecDeque<Encode>>> = Arc::new(Mutex::new(VecDeque::new()));
    let stop_worker = Arc::new(AtomicBool::new(false));
    let transcode_queue_inner = transcode_queue.clone();
    let stop_worker_inner = stop_worker.clone();

    let handle = thread::spawn(move || loop {
        let mut current_transcode = transcode_queue.lock().unwrap().pop_front();
        if current_transcode.is_some() {
            current_transcode.as_mut().unwrap().run();
        }
        sleep(Duration::new(1, 0));
        println!("Test");

        if stop_worker_inner.load(Ordering::Relaxed) {
            break;
        }
    });
    run_worker(9999, transcode_queue_inner).await?;

    stop_worker.store(true, Ordering::Relaxed);
    let _ = handle.join();
    Ok(())
}
