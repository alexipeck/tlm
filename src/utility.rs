use crate::{
    print::{print, From, Verbosity},
    timer::Timer,
};

#[derive(Clone, Debug)]
pub struct Utility {
    pub traceback: Vec<String>,
    pub timers: Vec<Timer>,
    pub print_timing: bool,
}
impl Utility {
    pub fn new(created_from: &str) -> Utility {
        let mut traceback = Utility {
            traceback: Vec::new(),
            timers: Vec::new(),
            print_timing: false,
        };

        return traceback.add_traceback_location(created_from);
    }

    pub fn enable_timing_print(&mut self) {
        self.print_timing = true;
    }

    pub fn disable_timing_print(&mut self) {
        self.print_timing = false;
    }

    pub fn get_saved_timing(&self, identifier: String, utility: Utility) -> u128 {
        let utility = utility.clone_and_add_location("get_saved_timing");

        for timer in &self.timers {
            if timer.stage_task_identifier == identifier {
                return timer.saved_time.unwrap();
            }
        }
        print(
            Verbosity::ERROR,
            From::Utility,
            utility,
            format!("A timer was never created or the identifier used matches no timers."),
            0,
        );
        panic!();
    }

    fn add_traceback_location(&mut self, called_from: &str) -> Utility {
        self.traceback.push(String::from(called_from));
        return self.clone();
    }

    pub fn clone_and_add_location(&self, called_from: &str) -> Utility {
        let mut temp = self.clone();
        temp.timers = Vec::new();
        temp.add_traceback_location(called_from);
        return temp;
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
