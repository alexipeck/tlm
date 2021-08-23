use crate::{
    print::{print, From, Verbosity},
    timer::Timer,
};

#[derive(Clone, Debug)]
pub struct Utility {
    pub traceback: Vec<String>,
    pub timers: Vec<Timer>,
    pub indentation: usize,
    pub print_timing: bool,

    pub current_location: String,
    pub function_timer: Option<Timer>,
    pub timing_minumum_threshold: usize,
}

impl Utility {
    pub fn new(created_from: &str, timing_minimum_threshold: usize) -> Utility {
        let mut utility = Utility {
            traceback: Vec::new(),
            timers: Vec::new(),
            indentation: 0,
            print_timing: false,

            current_location: String::from(created_from),
            function_timer: None,
            timing_minumum_threshold: timing_minimum_threshold,
        };
        return utility.add_traceback_location(created_from);
    }

    pub fn add_timer(&mut self, identifier: usize, stage_task_identifier: &str, utility: Utility) {
        let utility = utility.clone_and_add_location("add_timer(Utility)", 0);

        if self.timer_exists(identifier) {
            self.delete_or_reset_single_timer(false, identifier);
        } else {
            self.timers.push(Timer::create_timer(
                identifier,
                String::from(stage_task_identifier),
                0,
            ));
        }
    }

    pub fn start_function_timer(&mut self, additional_indentation: usize) {
        self.function_timer = Some(Timer::create_timer(
            0,
            String::from(self.current_location.clone()),
            self.indentation + additional_indentation,
        ));
    }

    pub fn add_timer_with_extra_indentation(
        &mut self,
        identifier: usize,
        stage_task_identifier: &str,
        extra_indentation: usize,
        utility: Utility,
    ) {
        let utility = utility.clone_and_add_location("add_timer(Utility)", 0);

        if self.timer_exists(identifier) {
            self.delete_or_reset_single_timer(false, identifier);
        } else {
            self.timers.push(Timer::create_timer(
                identifier,
                String::from(stage_task_identifier),
                extra_indentation,
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

    pub fn print_specific_timer_by_uid(&mut self, uid: usize, utility: Utility) {
        let utility = utility.clone_add_location("print_specific_timer_by_uid(Utility)");

        for timer in &mut self.timers {
            if timer.uid == uid {
                timer.print_timer(utility.clone());
            }
        }
    }

    pub fn print_all_timers_except_one(&mut self, uid: usize, utility: Utility) {
        let utility = utility.clone_add_location("print_all_timers_except_one(Utility)");

        for timer in &mut self.timers {
            if !(timer.uid == uid) {
                timer.print_timer(utility.clone());
            }
        }
    }

    pub fn print_all_timers_except_many(&mut self, uid: Vec<usize>, utility: Utility) {
        let utility = utility.clone_and_add_location("print_all_timers_except_many(Utility)", 0);

        for timer in &mut self.timers {
            if !uid.contains(&timer.uid) {
                timer.print_timer(utility.clone());
            }
        }
    }

    pub fn print_all_timers(&mut self, utility: Utility) {
        let mut utility = utility.clone_add_location_start_timing("print_all_timers(Utility)", 0);

        for timer in &mut self.timers {
            timer.print_timer(utility.clone());
        }

        utility.print_function_timer();
    }

    pub fn print_function_timer(&mut self) {
        if self.function_timer.is_some() {
            //this doesn't take into save the timer that is saved in print_timer - non-persistent because of the clone
            self.function_timer
                .clone()
                .unwrap()
                .print_timer(self.clone());
        } else {
            print(
                Verbosity::CRITICAL,
                From::Utility,
                self.clone(),
                format!("You tried to print a timer that doesn't exist."),
            );
            panic!()
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

    pub fn increment_indendation(&mut self) {
        self.indentation += 1;
    }

    //wipes timers of clone
    //increments indentation
    pub fn clone_add_location_start_timing(
        &self,
        called_from: &str,
        additional_indentation: usize,
    ) -> Utility {
        let mut temp = self.clone();
        temp.timers = Vec::new();
        temp.add_traceback_location(called_from);
        temp.current_location = String::from(called_from);
        temp.increment_indendation();
        temp.start_function_timer(additional_indentation);
        return temp;
    }

    pub fn clone_add_location(&self, called_from: &str) -> Utility {
        let mut temp = self.clone();
        temp.timers = Vec::new();
        temp.add_traceback_location(called_from);
        temp.current_location = String::from(called_from);
        temp.increment_indendation();
        return temp;
    }

    pub fn clone_and_add_location(&self, called_from: &str, indentation: usize) -> Utility {
        let mut temp = self.clone();
        temp.timers = Vec::new();
        temp.add_traceback_location(called_from);
        for _ in 0..indentation + 1 {
            temp.increment_indendation();
        }
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
