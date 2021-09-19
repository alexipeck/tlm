use std::{collections::VecDeque, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, thread};
use tracing::{Level, event};

use crate::{config::{Config, Preferences}, scheduler::{Hash, ImportFiles, ProcessNewFiles, Scheduler, Task, TaskType}};

pub struct CommandController {
    
}

impl CommandController {
    pub fn new() -> Self {
        CommandController {
            
        }
    }

    pub fn run(&mut self) {
        
    }
}