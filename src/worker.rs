use directories::BaseDirs;
use std::collections::VecDeque;
use std::env;
use std::io::stdout;
use std::io::Error as IoError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::{thread::sleep, time::Duration};
use tlm::config::WorkerConfig;
use tlm::worker_manager::Encode;
use tlm::ws::run_worker;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{error, warn, Level};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::Registry;
use tracing_subscriber::Layer;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let stdout_level = match env::var("TLM_DISPLAYED_LEVEL") {
        Ok(level_str) => match level_str.to_lowercase().as_str() {
            "info" => Some(Level::INFO),
            "debug" => Some(Level::DEBUG),
            "warning" | "warn" => Some(Level::WARN),
            "trace" => Some(Level::TRACE),
            "error" | "err" => Some(Level::ERROR),
            _ => None,
        },
        Err(_) => None,
    };

    let base_dirs = BaseDirs::new().unwrap_or_else(|| {
        error!("Home directory could not be found");
        panic!();
    });
    let log_path = base_dirs.config_dir().join("tlm/logs/");

    let file = tracing_appender::rolling::daily(log_path, "tlm_worker.log");
    let (stdout_writer, _guard) = tracing_appender::non_blocking(stdout());
    let (file_writer, _guard) = tracing_appender::non_blocking(file);

    let level_filter;
    if let Some(level) = stdout_level {
        level_filter = LevelFilter::from_level(level);
    } else {
        level_filter = LevelFilter::from_level(Level::INFO);
    }
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(stdout_writer)
        .with_filter(level_filter);

    let logfile_layer = tracing_subscriber::fmt::layer().with_writer(file_writer);

    let subscriber = Registry::default().with(stdout_layer).with(logfile_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let config = WorkerConfig::new(base_dirs.config_dir().join("tlm/tlm_worker.config"));

    loop {
        let transcode_queue: Arc<Mutex<VecDeque<Encode>>> = Arc::new(Mutex::new(VecDeque::new()));
        let stop_worker = Arc::new(AtomicBool::new(false));
        let transcode_queue_inner = transcode_queue.clone();
        let stop_worker_inner = stop_worker.clone();
        let (mut tx, rx) = futures_channel::mpsc::unbounded();

        tx.start_send(Message::Text("initialise_worker".to_string()))
            .unwrap();
        //TODO: Don't create this thread until we actually have a websocket established
        //Alternatively, don't worry about it, it isn't really a problem as it is currently
        let handle = thread::spawn(move || loop {
            let inner_tx = tx.clone();
            let mut current_transcode = transcode_queue.lock().unwrap().pop_front();
            if current_transcode.is_some() {
                current_transcode.as_mut().unwrap().run();
            }
            sleep(Duration::new(1, 0));
            match tx.start_send(Message::Text("test_message".to_string())) {
                Err(err) => {
                    warn!("Failed to send message. Server is likely closed");
                    break;
                }
                Ok(_) => {}
            }

            if stop_worker_inner.load(Ordering::Relaxed) {
                break;
            }
        });
        run_worker(transcode_queue_inner, rx, config.clone()).await?;

        stop_worker.store(true, Ordering::Relaxed);
        let _ = handle.join();
    }
    Ok(())
}
