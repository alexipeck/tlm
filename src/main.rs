use tlm::{
    database::{
        miscellaneous::db_purge,
        print::{print_contents, print_shows},
    },
    manager::FileManager,
    utility::Utility,
};

fn main() {
    //traceback and timing utility
    let mut utility = Utility::new("main");
    //utility.enable_timing_print();

    //purges the database, should be used selectively
    db_purge(utility.clone());

    //A FileManager stores working files, hashsets and supporting functions related to updating those files
    let mut file_manager: FileManager = FileManager::new(utility.clone());

    let allowed_extensions = vec!["mp4", "mkv", "webm", "MP4"];
    let ignored_paths = vec![".recycle_bin"];

    let t = file_manager.import_files(&allowed_extensions, &ignored_paths);

    file_manager.process_new_files(
        t,
        utility.clone(),
    );

    utility.disable_timing_print();

    //print_contents(file_manager.working_content.clone(), utility.clone());
    //print_shows(utility.clone());
    //print_jobs(utility.clone());

    //queue.print();
    //shows.print();
}
