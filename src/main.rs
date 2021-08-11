use std::{
    collections::HashSet,
    path::PathBuf,
};

use tlm::{
    TrackedDirectories,
    process_new_files,
    import_files,
    handle_tracked_directories,
    database::{
        miscellaneous::db_purge,
        ensure::ensure_tables_exist,
        print::{print_contents, print_shows},
    },
    content::Content,
    shows::Shows,
    traceback::Traceback,
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

    let traceback = Traceback::new("main");

    db_purge(traceback.clone());

    ensure_tables_exist(traceback.clone());

    let tracked_directories: TrackedDirectories = handle_tracked_directories();

    let allowed_extensions = vec!["mp4", "mkv", "webm", "MP4"];

    //ignored directories
    let ignored_paths = vec![".recycle_bin"];

    let mut working_content: Vec<Content> = Content::get_all_contents(traceback.clone());

    let mut existing_files_hashset: HashSet<PathBuf> = Content::get_all_filenames_as_hashset_from_contents(working_content.clone(), traceback.clone());

    let shows = Shows::new();

    process_new_files(
        import_files(
            &tracked_directories.root_directories,
            &allowed_extensions,
            &ignored_paths,
            &mut existing_files_hashset,
        ),
        &mut working_content,
        traceback.clone(),
    );

    print_contents(working_content, traceback.clone());
    print_shows(traceback.clone());
    //print_jobs(traceback.clone());

    //queue.print();
    //shows.print();
}
