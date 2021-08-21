use tlm::{
    config::{Config, Preferences},
    manager::{FileManager},
    utility::Utility,
};

fn main() {
    //traceback and timing utility
    let mut utility = Utility::new("main");

    let preferences = Preferences::new();

    let config: Config = Config::ensure_config_exists_then_get(&preferences);

    if preferences.default_print || preferences.print_general {
        utility.enable_timing_print();
    }

    //The FileManager stores working files, hashsets and supporting functions related to updating those files
    let mut file_manager: FileManager = FileManager::new(utility.clone());

    file_manager.tracked_directories = config.tracked_directories;
    file_manager.import_files(&config.allowed_extensions, &config.ignored_paths);

    file_manager.process_new_files(utility.clone());

    utility.disable_timing_print();

    println!(
        "Number of contents loaded in memory: {}",
        file_manager.working_content.len()
    );
    println!(
        "Number of shows loaded in memory: {}",
        file_manager.tv.working_shows.len()
    );
}
