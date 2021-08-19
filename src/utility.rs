use crate::{
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

    pub fn add_timer(&mut self, uid: usize, stage_task_identifier: &str) {
        self.timers.push(Timer::create_timer(uid, String::from(stage_task_identifier)));
    }

    pub fn store_timing_by_uid(&mut self, uid: usize) {
        for timer in &mut self.timers {
            if timer.uid == uid {
                timer.store_timing();
            }
        }
    }

    pub fn print_specific_timer_by_uid(&mut self, uid: usize, indent: usize, utility: Utility) {
        for timer in &mut self.timers {
            if timer.uid == uid {
                timer.print_timer(indent, utility.clone());
            }
        }
    }

    pub fn print_all_timers_except_one(&mut self, uid: usize, indent: usize, utility: Utility) {
        for timer in &mut self.timers {
            if !(timer.uid == uid) {
                timer.print_timer(indent, utility.clone());
            }
        }
    }

    pub fn print_all_timers_except_many(&mut self, uid: Vec<usize>, indent: usize, utility: Utility) {
        for timer in &mut self.timers {
            if !uid.contains(&timer.uid) {
                timer.print_timer(indent, utility.clone());
            }
        }
    }

    pub fn print_all_timers(&mut self, indent: usize, utility: Utility) {
        for timer in &mut self.timers {
            timer.print_timer(indent, utility.clone());
        }
    }

    pub fn enable_timing_print(&mut self) {
        self.print_timing = true;
    }

    pub fn disable_timing_print(&mut self) {
        self.print_timing = false;
    }

    fn add_traceback_location(&mut self, called_from: &str) -> Utility {
        self.traceback.push(String::from(called_from));
        return self.clone();
    }

    //wipes timers on clone
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
