use crate::{
    print::{print, From, Verbosity},
    utility::Utility,
};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Timer {
    //The stage and task are for instance: "startup: looping through contents to do 'x'"
    //The identifier is an arbitrary string of characters you can search the code directly for, I recommend a 4 character alpha-numeric combination
    pub uid: usize,
    pub function_name: String,
    pub indent_by: usize,
    pub timer: Instant,
    pub stored_time: Option<u128>,
}

impl Timer {
    pub fn create_timer(uid: usize, function_name: String, indentation: usize) -> Timer {
        return Timer {
            uid: uid,
            function_name: function_name,
            indent_by: indentation,
            timer: Instant::now(),
            stored_time: None,
        };
    }

    pub fn store_timing(&mut self) {
        self.stored_time = Some(self.timer.elapsed().as_millis());
    }

    pub fn reset_timer(&mut self) {
        self.timer = Instant::now();
        self.stored_time = None;
    }

    pub fn print_timer(&mut self, utility: Utility) {
        let utility = utility.clone_add_location("print_timer(Timer)");

        if !utility.print_timing {
            return;
        }

        if self.stored_time.is_none() {
            self.store_timing();
        }

        if self.stored_time.unwrap() > 0 {
            print(
                Verbosity::INFO,
                From::Utility,
                utility,
                format!(
                    "{} took: {}ms",
                    self.function_name,
                    self.stored_time.unwrap()
                ),
            );
        }
    }
}
