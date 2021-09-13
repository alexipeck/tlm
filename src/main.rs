extern crate diesel;
extern crate websocket;

use tlm::{
    config::{Config, Preferences},
    scheduler::{Hash, ImportFiles, ProcessNewFiles, Scheduler, Task, TaskType},
};
use websocket::sync::Server;
use websocket::OwnedMessage;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use std::io::stdout;
use tracing::{event, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::Registry;

fn main() {
    let file = tracing_appender::rolling::daily("./logs", "tlm.log");
    let (writer, _guard) = tracing_appender::non_blocking(stdout());
    let (writer2, _guard) = tracing_appender::non_blocking(file);
    let layer = tracing_subscriber::fmt::layer().with_writer(writer);

    let layer2 = tracing_subscriber::fmt::layer().with_writer(writer2);

    let subscriber = Registry::default().with(layer).with(layer2);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    event!(Level::INFO, "Starting tlm");

    let preferences = Preferences::default();

    let config: Config = Config::new(&preferences);

    let tasks: Arc<Mutex<VecDeque<Task>>> = Arc::new(Mutex::new(VecDeque::new()));

    let stop_scheduler = Arc::new(AtomicBool::new(false));
    let mut scheduler: Scheduler =
        Scheduler::new(config.clone(), tasks.clone(), stop_scheduler.clone());

    let inner_pref = preferences.clone();
    //Start the scheduler in it's own thread and return the scheduler at the end
    //so that we can print information before exiting
    let scheduler_handle = thread::spawn(move || {
        scheduler.start_scheduler(&inner_pref);
        scheduler
    });

    //Initial setup in own scope so lock drops
    {
        let mut tasks_guard = tasks.lock().unwrap();
        tasks_guard.push_back(Task::new(TaskType::Hash(Hash::new())));

        tasks_guard.push_back(Task::new(TaskType::ImportFiles(ImportFiles::new(
            &config.allowed_extensions,
            &config.ignored_paths,
        ))));

        tasks_guard.push_back(Task::new(TaskType::ProcessNewFiles(ProcessNewFiles::new())));
    }

    let mut server = Server::bind("127.0.0.1:49200").unwrap();
    if server.set_nonblocking(true).is_err() {
        event!(Level::ERROR, "");
        panic!();
    }

    server.set_nonblocking(true);

    if !preferences.disable_input {
        let running = Arc::new(AtomicBool::new(true));
        let running_inner = running.clone();
        ctrlc::set_handler(move || {
            event!(Level::WARN, "Stop signal received shutting down");
            running_inner.store(false, Ordering::SeqCst)
        })
        .expect("Error setting Ctrl-C handler");
        while running.load(Ordering::SeqCst) {
            if let Ok(wsupgrade) = server.accept() {
                let message = wsupgrade.accept().unwrap().recv_message().unwrap();
                match message {
                    OwnedMessage::Text(text) => {
                        println!("{}", text);
                    }
                    _ => {
                        println!("Unk");
                    }
                }
            }
        }
    }

    stop_scheduler.store(true, Ordering::Relaxed);

    let scheduler = scheduler_handle.join().unwrap();

    scheduler.file_manager.print_number_of_generics();
    scheduler.file_manager.print_number_of_shows();
    scheduler.file_manager.print_number_of_episodes();
    scheduler.file_manager.print_shows(&preferences);
    scheduler.file_manager.print_generics(&preferences);
    scheduler.file_manager.print_episodes(&preferences);
    //scheduler.file_manager.print_rejected_files(preferences); //I'm all for it as soon as it's disabled by default
}
