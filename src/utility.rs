use crate::timer::Timer;

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

    pub fn add_timer(&mut self, identifier: usize, stage_task_identifier: &str) {
        if self.timer_exists(identifier) {
            self.delete_or_reset_single_timer(false, identifier);
        } else {
            self.timers.push(Timer::create_timer(
                identifier,
                String::from(stage_task_identifier),
            ));
        }
    }

    pub fn timer_exists(&self, uid: usize) -> bool {
        for timer in &self.timers {
            if timer.uid == uid {
                return true;
            }
        }
        return false;
    }

    pub fn delete_or_reset_single_timer(&mut self, delete: bool, timer_identifier: usize) {
        self.delete_or_reset_multiple_timers(delete, vec![timer_identifier]);
    }

    pub fn delete_or_reset_multiple_timers(&mut self, delete: bool, timer_identifiers: Vec<usize>) {
        let timer_indexes: Vec<usize> = self.get_timer_indexes_based_by_uids(timer_identifiers);
        for timer_index in timer_indexes {
            if delete {
                self.timers.remove(timer_index);
            } else {
                self.timers[timer_index].reset_timer();
            }
        }
    }

    fn get_timer_indexes_based_by_uids(&self, uids: Vec<usize>) -> Vec<usize> {
        let mut prepare: Vec<usize> = Vec::new();
        let mut counter: usize = 0;
        for timer in &self.timers {
            if uids.contains(&timer.uid) {
                prepare.push(counter);
            }

            counter += 1;
        }

        return prepare;
    }

    fn get_timer_indexes_based_excluding_by_uids(&self, uids: Vec<usize>) -> Vec<usize> {
        let mut prepare: Vec<usize> = Vec::new();
        let mut counter: usize = 0;
        for timer in &self.timers {
            if !uids.contains(&timer.uid) {
                prepare.push(counter);
            }

            counter += 1;
        }

        return prepare;
    }

    fn get_timer_uids_based_on_exclusion(&self, uids: Vec<usize>) -> Vec<usize> {
        let mut prepare: Vec<usize> = Vec::new();
        let mut counter: usize = 0;
        for timer in &self.timers {
            if !uids.contains(&timer.uid) {
                prepare.push(counter);
            }

            counter += 1;
        }

        return prepare;
    }

    pub fn delete_or_reset_all_timers_except_one(&mut self, delete: bool, ignored_timer: usize) {
        let uids_of_timers_to_reset_or_delete =
            self.get_timer_uids_based_on_exclusion(vec![ignored_timer]);
        self.delete_or_reset_multiple_timers(delete, uids_of_timers_to_reset_or_delete);
    }

    pub fn delete_or_reset_all_timers_except_many(
        &mut self,
        delete: bool,
        ignored_timers: Vec<usize>,
    ) {
        let uids_of_timers_to_reset_or_delete =
            self.get_timer_uids_based_on_exclusion(ignored_timers);
        self.delete_or_reset_multiple_timers(delete, uids_of_timers_to_reset_or_delete);
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

    pub fn print_all_timers_except_many(
        &mut self,
        uid: Vec<usize>,
        indent: usize,
        utility: Utility,
    ) {
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

    //wipes timers of clone
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
