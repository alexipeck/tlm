//trickle up
#[derive(Clone, Debug, PartialEq)]
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
    Shows = 4,
    Queue = 5,
    DB = 6,
    Job = 7,
}

pub fn print(
    verbosity: Verbosity,
    from_module: From,
    traceback: crate::traceback::Traceback,
    string: String,
) {
    fn print(
        verbosity_string: &str,
        from_module_string: &str,
        call_functions_string: String,
        string: String,
    ) {
        println!(
            "[{}][{}][{}] {}",
            verbosity_string, from_module_string, call_functions_string, string
        );
    }

    //print(Verbosity::DEBUG, r"", format!(""));
    let set_output_verbosity_level = Verbosity::DEBUG as usize; //would be set as a filter in any output view
    let show_only = Verbosity::DEBUG;

    //module called from
    let from_module_string: &str;
    match from_module as usize {
        1 => from_module_string = "main",
        2 => from_module_string = "lib",
        3 => from_module_string = "content",
        4 => from_module_string = "shows",
        5 => from_module_string = "queue",
        6 => from_module_string = "db",
        7 => from_module_string = "job",
        _ => from_module_string = "notset",
    }

    //verbosity
    let current_verbosity_level = verbosity.clone() as usize;
    let verbosity_string: &str;
    match current_verbosity_level {
        1 => verbosity_string = "CRITICAL",
        2 => verbosity_string = "ERROR",
        3 => verbosity_string = "WARNING",
        4 => verbosity_string = "INFO",
        5 => verbosity_string = "DEBUG",
        _ => verbosity_string = "NOTSET",
    }

    //called from
    let mut call_functions_string: String = String::new();
    if verbosity.clone() as usize == Verbosity::CRITICAL as usize
        || verbosity.clone() as usize == Verbosity::ERROR as usize
    {
        call_functions_string = traceback.to_string();
    } else {
        call_functions_string += &format!("{}", traceback.traceback[traceback.traceback.len() - 1]);
    }

    if verbosity == show_only {
        print(
            verbosity_string,
            from_module_string,
            call_functions_string,
            string,
        );
    } else if current_verbosity_level <= set_output_verbosity_level {
        print(
            verbosity_string,
            from_module_string,
            call_functions_string,
            string,
        );
    }
}