use crate::utility::Utility;

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
    Utility = 4,
    Show = 5,
    Queue = 6,
    DB = 7,
    Job = 8,
}

pub fn get_indentation_from_tab_count(tab_count: usize) -> String {
    let mut indentation: String = String::new();
    for _ in 0..tab_count {
        indentation += r"    ";
    }
    return indentation;
}

pub fn print(
    verbosity: Verbosity,
    from_module: From,
    traceback: Utility,
    string: String,
    indent: usize,
) {
    fn print(
        verbosity_string: &str,
        from_module_string: &str,
        call_functions_string: String,
        string: String,
        indentation: String,
    ) {
        println!(
            "{}[{}][{}][{}]::{}",
            indentation, verbosity_string, from_module_string, call_functions_string, string
        );
    }
    //asdf;
    let indentation = get_indentation_from_tab_count(indent);

    //print(Verbosity::DEBUG, r"", format!(""));
    let set_output_verbosity_level = Verbosity::DEBUG as usize; //would be set as a filter in any output view
    let show_only = Verbosity::DEBUG;

    //module called from
    let from_module_string: &str;
    match from_module as usize {
        1 => from_module_string = "main",
        2 => from_module_string = "lib",
        3 => from_module_string = "content",
        4 => from_module_string = "utility",
        5 => from_module_string = "shows",
        6 => from_module_string = "queue",
        7 => from_module_string = "db",
        8 => from_module_string = "job",
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
            indentation,
        );
    } else if current_verbosity_level <= set_output_verbosity_level {
        print(
            verbosity_string,
            from_module_string,
            call_functions_string,
            string,
            indentation,
        );
    }
}
