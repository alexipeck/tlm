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
    Manager = 9,
    TV = 10,
}

pub fn get_indentation_from_tab_count(tab_count: usize) -> String {
    let mut indentation: String = String::new();
    for _ in 0..tab_count {
        indentation += r"    ";
    }
    return indentation;
}

pub fn print(verbosity: Verbosity, from_module: From, utility: Utility, string: String) {
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
    let mut utility = utility.clone_add_location_start_timing("print", 0);
    
    let indentation = get_indentation_from_tab_count(utility.indentation);

    //print(Verbosity::DEBUG, r"", format!(""));
    let set_output_verbosity_level = Verbosity::DEBUG as usize; //would be set as a filter in any output view
    let show_only = Verbosity::DEBUG;

    //module called from
    let from_module_string: &str;
    match from_module {
        From::Main => from_module_string = "main",
        From::Lib => from_module_string = "lib",
        From::Content => from_module_string = "content",
        From::Utility => from_module_string = "utility",
        From::Show => from_module_string = "shows",
        From::Queue => from_module_string = "queue",
        From::DB => from_module_string = "db",
        From::Job => from_module_string = "job",
        From::Manager => from_module_string = "manager",
        From::TV => from_module_string = "tv",
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
        call_functions_string = utility.to_string();
    } else {
        call_functions_string += &format!("{}", utility.traceback[utility.traceback.len() - 1]);
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

    utility.print_function_timer();
}
