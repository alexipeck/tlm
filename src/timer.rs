use std::time::Instant;
use crate::utility::Utility;
use crate::print::{print, From, Verbosity};

#[derive(Clone, Debug)]
pub struct Timer {
    //The stage and task are for instance: "startup: looping through contents to do 'x'"
    //The identifier is an arbitrary string of characters you can search the code directly for, I recommend a 4 character alpha-numeric combination
    pub stage_task_identifier: String,
    pub timer: Instant,
    pub saved_time: Option<u128>,
}

impl Timer {
    pub fn create_timer(stage_task_identifier: String) -> Timer {
        return Timer {
            stage_task_identifier: stage_task_identifier,
            timer: Instant::now(),
            saved_time: None,
        }
    }

    pub fn save_timing(&mut self) {
        self.saved_time = Some(self.timer.elapsed().as_millis());
    }

    pub fn print_timer_from_stage_and_task_from_saved(
        &self,
        indentation_tabs: usize,
        utility: Utility,
    ) {
        if self.saved_time.is_some() {
            let utility = utility.clone_and_add_location("print_timer_from_stage_and_task_from_saved");
            
            if 
            print(
                Verbosity::INFO,
                From::Utility,
                utility,
                format!("{} took: {}ms", self.stage_task_identifier, self.saved_time.unwrap()),
                indentation_tabs,
            );
        }
    }

    pub fn print_timer_from_stage_and_task(
        &self,
        identifier: String,
        stage: &str,
        task: &str,
        indent: usize,
        utility: Utility,
    ) {
        if self.saved_time.is_some() {
            let utility = utility.clone_and_add_location("print_timer_from_stage_and_task");

            self.save_timing();
            if timing > 0 {
                print(
                    Verbosity::INFO,
                    From::Utility,
                    self.clone(),
                    format!("{}: handling task '{}' took: {}ms", stage, task, timing,),
                    indent,
                );
            }
        }
    }
}