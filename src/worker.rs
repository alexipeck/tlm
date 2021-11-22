use std::collections::VecDeque;
use std::io::Error as IoError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::{thread::sleep, time::Duration};
use tlm::worker_manager::Encode;
use tlm::ws::run_worker;
use tokio_tungstenite::tungstenite::protocol::Message;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let transcode_queue: Arc<Mutex<VecDeque<Encode>>> = Arc::new(Mutex::new(VecDeque::new()));
    let stop_worker = Arc::new(AtomicBool::new(false));
    let transcode_queue_inner = transcode_queue.clone();
    let stop_worker_inner = stop_worker.clone();
    let (mut tx, rx) = futures_channel::mpsc::unbounded();

    tx.start_send(Message::Text("initialise_worker".to_string())).unwrap();
    let handle = thread::spawn(move || loop {
        let inner_tx = tx.clone();
        let mut current_transcode = transcode_queue.lock().unwrap().pop_front();
        if current_transcode.is_some() {
            current_transcode.as_mut().unwrap().run();
        }
        sleep(Duration::new(1, 0));
        println!("Test");
        tx.start_send(Message::Text("test_message".to_string())).unwrap();

        if stop_worker_inner.load(Ordering::Relaxed) {
            break;
        }
    });
    run_worker(transcode_queue_inner, rx).await?;

    stop_worker.store(true, Ordering::Relaxed);
    let _ = handle.join();
    Ok(())
}
