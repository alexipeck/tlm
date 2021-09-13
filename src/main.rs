extern crate diesel;
extern crate websocket;

use tlm::{
    config::Config,
    scheduler::{Hash, ImportFiles, ProcessNewFiles, Scheduler, Task, TaskType},
    utility::{Traceback, Utility},
};
use websocket::sync::Server;
use websocket::OwnedMessage;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use std::io::stdout;
use tracing::{event, Level};
use tracing_appender;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::Registry;

fn main() {
    //traceback and timing utility
    let file = tracing_appender::rolling::daily("./logs", "prefix.log");
    let (writer, _guard) = tracing_appender::non_blocking(stdout());
    let (writer2, _guard) = tracing_appender::non_blocking(file);
    let layer = tracing_subscriber::fmt::layer()
        .with_writer(writer)
        .finish();

    let layer2 = tracing_subscriber::fmt::layer()
        .with_writer(writer2)
        .finish();

    let subscriber = Registry::default().with(layer).with(layer2);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    event!(Level::INFO, "Starting tlm");

    let utility = Utility::new(Traceback::Main);

    let config: Config = Config::new(&utility.preferences);

    let tasks: Arc<Mutex<VecDeque<Task>>> = Arc::new(Mutex::new(VecDeque::new()));

    let stop_scheduler = Arc::new(AtomicBool::new(false));
    let mut scheduler: Scheduler = Scheduler::new(
        config.clone(),
        utility.clone(),
        tasks.clone(),
        stop_scheduler.clone(),
    );
    let utility_inner = utility.clone();

    //Start the scheduler in it's own thread and return the scheduler at the end
    //so that we can print information before exiting
    let scheduler_handle = thread::spawn(move || {
        scheduler.start_scheduler(utility_inner.clone());
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
    server.set_nonblocking(true);

    if !utility.preferences.disable_input {
        let running = Arc::new(AtomicBool::new(true));
        let running_inner = running.clone();
        ctrlc::set_handler(move || running_inner.store(false, Ordering::SeqCst))
            .expect("Error setting Ctrl-C handler");
        while running.load(Ordering::SeqCst) {
            let result = match server.accept() {
                Ok(wsupgrade) => {
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
                _ => {}
            };
        }
    }

    stop_scheduler.store(true, Ordering::Relaxed);

    let scheduler = scheduler_handle.join().unwrap();

    scheduler
        .file_manager
        .print_number_of_generics(utility.clone());
    scheduler
        .file_manager
        .print_number_of_shows(utility.clone());
    scheduler
        .file_manager
        .print_number_of_episodes(utility.clone());
    scheduler.file_manager.print_shows(utility.clone());
    scheduler.file_manager.print_generics(utility.clone());
    scheduler.file_manager.print_episodes(utility.clone());
    //scheduler.file_manager.print_rejected_files(utility); //I'm all for it as soon as it's disabled by default
}
