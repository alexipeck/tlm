use std::time::Instant;
use crate::utility::Utility;
use crate::print::{print, From, Verbosity};

#[derive(Clone, Debug)]
pub struct Timer {
    //The stage and task are for instance: "startup: looping through contents to do 'x'"
    //The identifier is an arbitrary string of characters you can search the code directly for, I recommend a 4 character alpha-numeric combination
    pub uid: usize,
    pub stage_task_identifier: String,
    pub timer: Instant,
    pub saved_time: Option<u128>,
}

impl Timer {
    pub fn create_timer(uid: usize, stage_task_identifier: String) -> Timer {
        return Timer {
            uid: uid,
            stage_task_identifier: stage_task_identifier,
            timer: Instant::now(),
            saved_time: None,
        }
    }

    pub fn store_timing(&mut self) {
        self.saved_time = Some(self.timer.elapsed().as_millis());
    }

    pub fn print_timer(
        &mut self,
        indent: usize,
        utility: Utility,
    ) {
        let utility = utility.clone_and_add_location("print_timer_from_stage_and_task");

        if self.saved_time.is_none() {
            self.store_timing();
        }
        
        if self.saved_time.unwrap() > 0 {
            print(
                Verbosity::INFO,
                From::Utility,
                utility,
                format!("{} took: {}ms", self.stage_task_identifier, self.saved_time.unwrap()),
                indent,
            );
        }
    }
}