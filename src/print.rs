use crate::utility::Utility;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

//trickle up
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Verbosity {
    CRITICAL = 1,
    ERROR = 2,
    WARNING = 3,
    INFO = 4,
    DEBUG = 5,
    NOTSET = 0,
}

pub enum From {
    NOTSET = 0,
    Main = 1,
    Lib = 2,
    Generic = 3,
    Utility = 4,
    Show = 5,
    Queue = 6,
    DB = 7,
    Job = 8,
    Manager = 9,
    TV = 10,
    Config = 11,
    Scheduler = 12,
}

impl fmt::Display for From {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            From::Main => "main",
            From::Lib => "lib",
            From::Generic => "generic",
            From::Utility => "utility",
            From::Show => "shows",
            From::Queue => "queue",
            From::DB => "db",
            From::Job => "job",
            From::Manager => "manager",
            From::Scheduler => "scheduler",
            From::TV => "tv",
            _ => "notset",
        };
        write!(f, "{}", string)
    }
}

impl fmt::Display for Verbosity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            Verbosity::CRITICAL => "CRITICAL",
            Verbosity::ERROR => "ERROR",
            Verbosity::WARNING => "WARNING",
            Verbosity::INFO => "INFO",
            Verbosity::DEBUG => "DEBUG",
            _ => "NOTSET",
        };
        write!(f, "{}", string)
    }
}

impl FromStr for Verbosity {
    type Err = ParseIntError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_uppercase().as_str() {
            "CRITICAL" => Ok(Verbosity::CRITICAL),
            "ERROR" => Ok(Verbosity::ERROR),
            "WARNING" => Ok(Verbosity::WARNING),
            "INFO" => Ok(Verbosity::INFO),
            "DEBUG" => Ok(Verbosity::DEBUG),
            _ => Ok(Verbosity::NOTSET),
        }
    }
}

pub fn print(
    verbosity: Verbosity,
    from_module: From,
    string: String,
    whitelisted: bool,
    utility: Utility,
) {
    let mut utility = utility.clone_add_location("print");

    if !utility.preferences.default_print && !whitelisted {
        return;
    }

    //called from
    let call_functions_string: String;
    //whitelisted ignores min_verbosity, I'm personally not a fan of this, another print control method needs to be talked about
    if verbosity as usize <= utility.preferences.min_verbosity as usize || whitelisted {
        if verbosity == Verbosity::CRITICAL || verbosity == Verbosity::ERROR {
            call_functions_string = utility.to_string();
            eprintln!(
                "[{}][{}][{}]::{}",
                verbosity.to_string(),
                from_module.to_string(),
                call_functions_string,
                string
            );
        } else {
            call_functions_string = utility.traceback[utility.traceback.len() - 1].to_string();
            println!(
                "[{}][{}][{}]::{}",
                verbosity.to_string(),
                from_module.to_string(),
                call_functions_string,
                string
            );
        }
    }

    utility.print_function_timer();
}
