use std::{collections::HashSet, path::PathBuf};

use tlm::{
    content::Content,
    database::{
        ensure::ensure_tables_exist,
        miscellaneous::db_purge,
        print::{print_contents, print_shows},
    },
    handle_tracked_directories, import_files, process_new_files,
    show::Show,
    utility::Utility,
    TrackedDirectories,
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

    let tracked_directories: TrackedDirectories = handle_tracked_directories();

    let allowed_extensions = vec!["mp4", "mkv", "webm", "MP4"];

    //ignored directories
    let ignored_paths = vec![".recycle_bin"];

    let mut working_shows: Vec<Show> = Show::get_all_shows(utility.clone());

    let mut working_content: Vec<Content> =
        Content::get_all_contents(&mut working_shows, utility.clone());

    let mut existing_files_hashset: HashSet<PathBuf> =
        Content::get_all_filenames_as_hashset_from_contents(
            working_content.clone(),
            utility.clone(),
        );

    process_new_files(
        import_files(
            &tracked_directories.root_directories,
            &allowed_extensions,
            &ignored_paths,
            &mut existing_files_hashset,
        ),
        &mut working_content,
        &mut working_shows,
        utility.clone(),
    );

    utility.disable_timing_print();

    print_contents(working_content, utility.clone());
    print_shows(utility.clone());
    //print_jobs(utility.clone());

    //queue.print();
    //shows.print();
}
