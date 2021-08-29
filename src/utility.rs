use crate::{
    config::Preferences,
    print::{print, From, Verbosity},
    timer::Timer,
};

#[derive(Clone, Debug)]
pub struct Utility {
    pub traceback: Vec<String>,
    pub current_location: String,
    pub function_timer: Option<Timer>,
    pub preferences: Preferences,
}

impl Utility {
    pub fn new(created_from: &str) -> Self {
        let mut utility = Utility {
            traceback: Vec::new(),
            current_location: String::from(created_from),
            function_timer: None,

            preferences: Preferences::new(),
        };
        return utility.add_traceback_location(created_from);
    }

    pub fn start_function_timer(&mut self) {
        self.function_timer = Some(Timer::create_timer(
            0,
            String::from(self.current_location.clone()),
        ));
    }

    pub fn print_function_timer(&mut self) {
        if !self.preferences.timing_enabled {
            return;
        }

        if self.function_timer.is_some() {
            //the function interally saves inside, but because of the clone, it isn't persistent
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
                false,
            );
            panic!()
        }
    }

    fn add_traceback_location(&mut self, called_from: &str) -> Utility {
        self.traceback.push(String::from(called_from));
        return self.clone();
    }

    pub fn clone_add_location(&self, called_from: &str) -> Utility {
        let mut temp = self.clone();
        temp.add_traceback_location(called_from);
        temp.current_location = String::from(called_from);
        if self.preferences.timing_enabled {
            temp.start_function_timer();
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
