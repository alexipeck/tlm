use std::time::Instant;
use crate::print::{print, From, Verbosity};

#[derive(Clone, Debug)]
pub struct Utility {
    pub traceback: Vec<String>,
    pub timers: Vec<(usize, Instant, Option<u128>)>,
    pub print: bool,
}
impl Utility {
    pub fn new(created_from: &str) -> Utility {
        let mut traceback = Utility {
            traceback: Vec::new(),
            timers: Vec::new(),
            print: false,
        };
        traceback.add_traceback_location(created_from);
        return traceback;
    }

    pub fn enable_timing_print(&mut self) {
        self.print = true;
    }

    pub fn disable_timing_print(&mut self) {
        self.print = false;
    }
    
    pub fn get_saved_timing(&self, identifier: usize, utility: Utility) -> u128 {
        for timer in &self.timers {
            if timer.0 == identifier {
                return timer.2.unwrap();
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

    pub fn save_timing(&mut self, identifier: usize, utility: Utility) {
        let mut timer_exists: bool = false;
        let mut counter: usize = 0;
        for timer in &self.timers {
            if timer.0 == identifier {
                timer_exists = true;
                break;
            }
            counter += 1;
        }
        if timer_exists {     
            self.timers[counter].2 = Some(self.timers[counter].1.elapsed().as_millis());
        } else {
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
    }

    pub fn start_timer(&mut self, identifier: usize) {
        let timer_exists: bool = false;
        for (i, timer) in self.timers.iter().enumerate() {
            if timer.0 == identifier {
                self.timers[i] = (identifier, Instant::now(), None);
                break;
            }
        }
        if !timer_exists {
            self.timers.push((identifier, Instant::now(), None));
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

    pub fn print_timer_from_stage_and_task_from_saved(&self, identifier: usize, stage: &str, task: &str, indentation_tabs: usize, utility: Utility) {
        if self.print {
            let timing = self.get_saved_timing(identifier, utility);
            if timing > 0 {
                print(
                    Verbosity::INFO,
                    From::Utility,
                    self.clone(),
                    format!(
                        "{}: handling task '{}' took: {}ms",
                        stage,
                        task,
                        timing,
                    ),
                    indentation_tabs,
                );
            }
        }
    }

    pub fn print_timer_from_stage_and_task(&self, identifier: usize, stage: &str, task: &str, indent: usize, utility: Utility) {
        if self.print {
            let timing = self.get_timer_ms(identifier, utility);
            if timing > 0 {
                print(
                    Verbosity::INFO,
                    From::Utility,
                    self.clone(),
                    format!(
                        "{}: handling task '{}' took: {}ms",
                        stage,
                        task,
                        timing,
                    ),
                    indent,
                );
            }
        }
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
