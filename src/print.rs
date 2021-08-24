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

pub fn get_indentation_from_tab_count(tab_count: usize) -> String {
    let mut indentation: String = String::new();
    for _ in 0..tab_count {
        indentation += "\t";
    }
    return indentation;
}

pub fn print(verbosity: Verbosity, from_module: From, utility: Utility, string: String) {
    let mut utility = utility.clone_add_location_start_timing("print", 0);
    let indentation = get_indentation_from_tab_count(utility.indentation);

    //called from
    let call_functions_string: String;

    if verbosity as usize <= utility.min_verbosity as usize {
        if verbosity == Verbosity::CRITICAL || verbosity == Verbosity::ERROR {
            call_functions_string = utility.to_string();
            eprintln!(
                "{}[{}][{}][{}]::{}",
                indentation,
                verbosity.to_string(),
                from_module.to_string(),
                call_functions_string,
                string
            );
        } else {
            call_functions_string = format!("{}", utility.traceback[utility.traceback.len() - 1]);
            println!(
                "{}[{}][{}][{}]::{}",
                indentation,
                verbosity.to_string(),
                from_module.to_string(),
                call_functions_string,
                string
            );
        }
    }

    utility.print_function_timer();
}
