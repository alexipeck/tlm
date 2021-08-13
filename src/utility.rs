use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Utility {
    pub traceback: Vec<String>,
    pub timer: Option<Instant>,
}
impl Utility {
    pub fn new(created_from: &str) -> Utility {
        let mut traceback = Utility {
            traceback: Vec::new(),
            timer: None,
        };
        traceback.add_traceback_location(created_from);
        return traceback;
    }

    pub fn start_timer(&mut self) {
        self.timer = Some(Instant::now());
    }

    pub fn get_timer_ms(&self) -> u128 {
        if self.timer.is_some() {
            return self.timer.unwrap().elapsed().as_millis();
        }
        panic!("A timer was never created, your fault.");
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
