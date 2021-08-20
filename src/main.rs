use tlm::{
    content::Content,
    database::miscellaneous::db_purge,
    manager::FileManager,
    tv::print,
    utility::Utility,
};

fn main() {
    //traceback and timing utility
    let mut utility = Utility::new("main");
    utility.enable_timing_print();

    //purges the database, should be used selectively
    db_purge(utility.clone());

    //The FileManager stores working files, hashsets and supporting functions related to updating those files
    let mut file_manager: FileManager = FileManager::new(utility.clone());

    let allowed_extensions = vec!["mp4", "mkv", "webm", "MP4"];
    let ignored_paths = vec![".recycle_bin", ".Recycle.Bin"];

    file_manager.import_files(&allowed_extensions, &ignored_paths);
    println!("{}", file_manager.new_files_queue.len());

    file_manager.process_new_files(utility.clone());

    utility.disable_timing_print();

    Content::print_contents(file_manager.working_content.clone(), utility.clone());
    print::print_shows(file_manager.tv.working_shows.clone(), utility.clone());
}
