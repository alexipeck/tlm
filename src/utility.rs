use std::time::Instant;
use crate::print::{print, From, Verbosity};

#[derive(Clone, Debug)]
pub struct Utility {
    pub traceback: Vec<String>,
    pub timers: Vec<(usize, Instant)>,
}
impl Utility {
    pub fn new(created_from: &str) -> Utility {
        let mut traceback = Utility {
            traceback: Vec::new(),
            timers: Vec::new(),
        };
        traceback.add_traceback_location(created_from);
        return traceback;
    }
    
    pub fn start_timer(&mut self, identifier: usize) {
        let timer_exists: bool = false;
        for (i, timer) in self.timers.iter().enumerate() {
            if timer.0 == identifier {
                self.timers[i] = (identifier, Instant::now());
                break;
            }
        }
        if !timer_exists {
            self.timers.push((identifier, Instant::now()));
        }
    }

    pub fn get_timer_ms(&self, identifier: usize, utility: Utility) -> u128 {
        for timer in &self.timers {
            if timer.0 == identifier {
                return timer.1.elapsed().as_millis();
            }
        }
        print(
            Verbosity::ERROR,
            From::Utility,
            utility,
            format!(
                "A timer was never created or the identifier used matches no timers."
            ),
            0,
        );
        panic!();
    }

    pub fn print_timer_from_stage_and_task(&self, identifier: usize, stage: &str, task: &str, indent: usize, utility: Utility) {
        print(
            Verbosity::INFO,
            From::Utility,
            self.clone(),
            format!(
                "{}: handling task '{}' took: {}ms",
                stage,
                task,
                self.get_timer_ms(identifier, utility),
            ),
            indent,
        );
    }

    fn add_traceback_location(&mut self, called_from: &str) -> Utility {
        self.traceback.push(String::from(called_from));
        return self.clone();
    }

    pub fn clone_and_add_location(&self, called_from: &str) -> Utility {
        return self.clone().add_traceback_location(called_from);
    }

    pub fn to_string(&self) -> String {
        let mut call_functions_string: String = String::new();
        let mut single_execute_done = false;
        for function in &self.traceback {
            if !single_execute_done {
                call_functions_string += &format!("'{}'", function);
                single_execute_done = true;
            } else {
                call_functions_string += &format!(">'{}'", function);
            }
        }
        return call_functions_string;
    }
}
