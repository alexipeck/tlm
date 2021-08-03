#[derive(Clone, Debug)]
pub struct Traceback {
    pub traceback: Vec<String>,
}
impl Traceback {
    pub fn new() -> Traceback {
        return Traceback {
            traceback: Vec::new(),
        }
    }

    pub fn to_string(&self) -> String {
        let mut call_functions_string: String = String::new();
        let mut single_execute_done = false;
        for function in &self.traceback {
            if !single_execute_done {
                call_functions_string += &format!("{}", function);
                single_execute_done = true;
            } else {
                call_functions_string += &format!(" > {}", function);
            }
        }
        return call_functions_string;
    }
    
    pub fn add_location(&mut self, called_from: &str) -> Traceback {
        self.traceback.push(String::from(called_from));
        return self.clone();
    }
}
