use crate::utility::Utility;

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
    Content = 3,
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

impl From {
    pub fn to_string(self) -> String {
        String::from(match self {
            From::Main => "main",
            From::Lib => "lib",
            From::Content => "content",
            From::Utility => "utility",
            From::Show => "shows",
            From::Queue => "queue",
            From::DB => "db",
            From::Job => "job",
            From::Manager => "manager",
            From::Scheduler => "scheduler",
            From::TV => "tv",
            _ => "notset",
        })
    }
}

impl Verbosity {
    pub fn to_string(self) -> String {
        String::from(match self {
            Verbosity::CRITICAL => "CRITICAL",
            Verbosity::ERROR => "ERROR",
            Verbosity::WARNING => "WARNING",
            Verbosity::INFO => "INFO",
            Verbosity::DEBUG => "DEBUG",
            _ => "NOTSET",
        })
    }

    pub fn from_string(input: &str) -> Verbosity {
        match input {
            "CRITICAL" => Verbosity::CRITICAL,
            "ERROR" => Verbosity::ERROR,
            "WARNING" => Verbosity::WARNING,
            "INFO" => Verbosity::INFO,
            "DEBUG" => Verbosity::DEBUG,
            _ => Verbosity::NOTSET,
        }
    }
}

pub fn print(
    verbosity: Verbosity,
    from_module: From,
    utility: Utility,
    string: String,
    whitelisted: bool,
) {
    let mut utility = utility.clone_add_location("print");
    
    if !utility.preferences.default_print && !whitelisted {
        return;
    }

    //called from
    let call_functions_string: String;
    //whitelisted ignores min_verbosity, I'm personally not a fan of this, another print control method needs to be talked about
    if verbosity as usize <= utility.min_verbosity as usize || whitelisted {
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
            call_functions_string = format!("{}", utility.traceback[utility.traceback.len() - 1]);
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
