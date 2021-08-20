use tlm::{
    content::Content,
    database::miscellaneous::db_purge,
    manager::FileManager,
    show::Show,
    utility::Utility,
};

fn main() {
    //traceback and timing utility
    let mut utility = Utility::new("main");
    utility.enable_timing_print();

    //purges the database, should be used selectively
    //db_purge(utility.clone());

    //The FileManager stores working files, hashsets and supporting functions related to updating those files
    let mut file_manager: FileManager = FileManager::new(utility.clone());

    let allowed_extensions = vec!["mp4", "mkv", "webm", "MP4"];
    let ignored_paths = vec![".recycle_bin", ".Recycle.Bin"];

    let t = file_manager.import_files(&allowed_extensions, &ignored_paths);

    file_manager.process_new_files(
        t,
        utility.clone(),
    );

    utility.disable_timing_print();

    Content::print_contents(file_manager.working_content.clone(), utility.clone());
    Show::print_shows(file_manager.working_shows.clone(), utility.clone());
}
