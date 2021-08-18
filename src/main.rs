use tlm::{
    database::{
        ensure::ensure_tables_exist,
        miscellaneous::db_purge,
        print::{print_contents, print_shows},
    },
    import_files, process_new_files,
    utility::Utility,
    manager::FileManager,
};

fn main() {
    /*
    load list of existing contents
    load list of existing files
    load list of existing show_uids

    load list of directories, filtering out those already stored as a file
    create content from this pathbuf
    insert content
    fill out show information if content is an episode
    insert episode from content
    */

    let mut utility = Utility::new("main");
    utility.enable_timing_print();

    //db_purge(utility.clone());
    ensure_tables_exist(utility.clone());

    let mut file_manager: FileManager = FileManager::new(utility.clone());

    let allowed_extensions = vec!["mp4", "mkv", "webm", "MP4"];
    let ignored_paths = vec![".recycle_bin"];
    
    process_new_files(
        import_files(
            &file_manager.tracked_directories.root_directories,
            &allowed_extensions,
            &ignored_paths,
            &mut file_manager.existing_files_hashset.unwrap(),
        ),
        &mut file_manager.working_content,
        &mut file_manager.working_shows,
        utility.clone(),
    );

    utility.disable_timing_print();

    print_contents(file_manager.working_content.clone(), utility.clone());
    print_shows(utility.clone());
    //print_jobs(utility.clone());

    //queue.print();
    //shows.print();
}
