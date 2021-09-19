use std::{collections::VecDeque, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, thread};
use tracing::{Level, event};

use crate::{config::{Config, Preferences}, scheduler::{Hash, ImportFiles, ProcessNewFiles, Scheduler, Task, TaskType}};

pub struct CommandController {
    pub preferences: Preferences,
    pub config: Config,
    pub stop_scheduler: Arc<AtomicBool>,
    pub scheduler: Scheduler,
}

impl CommandController {
    pub fn new() -> Self {
        let preferences: Preferences = Preferences::default();
        let config: Config = Config::new(&preferences);
        let stop_scheduler: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        
        CommandController {
            preferences,
            config: config.clone(),
            stop_scheduler: stop_scheduler.clone(),
            scheduler: Scheduler::new(config, Arc::new(Mutex::new(VecDeque::new())), stop_scheduler),
        }
    }

    pub fn run(&mut self) {
        let inner_pref = self.preferences.clone();
        //let tasks: Arc<Mutex<VecDeque<Task>>> = self.scheduler.tasks.clone();
        //Start the scheduler in it's own thread and return the scheduler at the end
        //so that we can print information before exiting
        let scheduler_handle = thread::spawn(move || {
            self.scheduler.start_scheduler(&inner_pref);
            self.scheduler
        });

        //Initial setup in own scope so lock drops
        {
            let mut tasks_guard = self.scheduler.tasks.lock().unwrap(); //TODO: Switch to a fair mutex implementation
            tasks_guard.push_back(Task::new(TaskType::Hash(Hash::default())));

            tasks_guard.push_back(Task::new(TaskType::ImportFiles(ImportFiles::default())));

            tasks_guard.push_back(Task::new(TaskType::ProcessNewFiles(
                ProcessNewFiles::default(),
            )));
        }

        if !self.preferences.disable_input {
            let running = Arc::new(AtomicBool::new(true));
            let running_inner = running.clone();
            ctrlc::set_handler(move || {
                event!(Level::WARN, "Stop signal received shutting down");
                running_inner.store(false, Ordering::SeqCst)
            })
            .expect("Error setting Ctrl-C handler");
            while running.load(Ordering::SeqCst) {}
        }
    
        self.stop_scheduler.store(true, Ordering::Relaxed);
    
        let scheduler = scheduler_handle.join().unwrap();
    
        scheduler.file_manager.print_number_of_generics();
        scheduler.file_manager.print_number_of_shows();
        scheduler.file_manager.print_number_of_episodes();
        scheduler.file_manager.print_shows(&self.preferences);
        scheduler.file_manager.print_generics(&self.preferences);
        scheduler.file_manager.print_episodes(&self.preferences);
        //scheduler.file_manager.print_rejected_files(); //I'm all for it as soon as it's disabled by default
    }
}