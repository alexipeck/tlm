use std::{
    collections::{VecDeque, HashSet},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
    path::PathBuf,
};

static WORKER_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

use tlm::{import_files, TrackedDirectories};
mod content;
mod database;
mod designation;
mod filter;
mod job;
mod print;
mod queue;
mod shows;
mod task;
mod traceback;
mod error_handling;
use content::Content;
use database::{
    db_purge,
    ensure::ensure_tables_exist,
    insert::{
        insert_content,
        insert_episode_if_episode,
        insert_job,
    },
    print_contents,
    print_jobs,
    print_shows,
};
use designation::Designation;
use print::{print, From, Verbosity};//remove from main
use queue::Queue;
use shows::Shows;
use traceback::Traceback;

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

    let mut traceback = Traceback::new();
    traceback.add_location("main");

    db_purge(traceback.clone());

    ensure_tables_exist(traceback.clone());

    let start = Instant::now();
    let existing_content: Vec<Content> = Content::get_all_contents(traceback.clone());
    print(Verbosity::INFO, From::Main, traceback.clone(), format!("startup: read in 'content' took: {}ms", start.elapsed().as_millis()));
    
    let start = Instant::now();
    let mut existing_files_hashset: HashSet<PathBuf> = Content::get_all_filenames_as_hashset(traceback.clone());
    print(Verbosity::INFO, From::Main, traceback.clone(), format!("startup: read in 'existing files hashset' took: {}ms", start.elapsed().as_millis()));

    //remote or local workers
    /*let mut encode_workers: Workers = Workers::new();
    encode_workers
        .workers
        .push_back(Worker::new("nuc".to_string()));
    let temp = encode_workers.get_and_reserve_worker();
    let mut worker: Option<(usize, String)> = None;
    if temp.is_some() {
        worker = Some(temp.unwrap());
    } else {
        panic!("No encode workers available");
    }*/

    //tracked directories - avoids duplicate entries
    let mut tracked_directories = TrackedDirectories::new();

    //manual entries
    if !cfg!(target_os = "windows") {
        //tracked_root_directories.push(String::from("/mnt/nas/tvshows")); //manual entry
        tracked_directories
            .root_directories
            .push_back(String::from(r"/home/anpeck/tlm/test_files/"));
        tracked_directories
            .root_directories
            .push_back(String::from(r"/home/alexi/tlm/test_files/"));
        tracked_directories
            .cache_directories
            .push_back(String::from(r"/home/anpeck/tlm/test_files/cache/"));
        tracked_directories
            .cache_directories
            .push_back(String::from(r"/home/alexi/tlm/test_files/cache/"));
    } else {
        //tracked_root_directories.push(String::from("T:/")); //manual entry
        tracked_directories.root_directories.push_back(String::from(
            r"C:\Users\Alexi Peck\Desktop\tlm\test_files\generics\",
        ));
        tracked_directories.root_directories.push_back(String::from(
            r"C:\Users\Alexi Peck\Desktop\tlm\test_files\shows\",
        ));
        tracked_directories
            .cache_directories
            .push_back(String::from(
                r"C:\Users\Alexi Peck\Desktop\tlm\test_files\cache\",
            ));
    }

    //queue
    //let mut queue = Queue::new(tracked_directories.cache_directories.clone());

    //allowed video extensions
    let allowed_extensions = vec!["mp4", "mkv", "webm", "MP4"];

    //ignored directories
    let ignored_paths = vec![".recycle_bin"];

    //raw_filepaths only contains the new files (those that don't already exist in the database)
    let new_files = import_files(
        &tracked_directories.root_directories,
        &allowed_extensions,
        &ignored_paths,
        &mut existing_files_hashset,
    );

    //sort out filepaths into series and seasons
    let mut shows = Shows::new();

    //loop through all new files
    for new_file in new_files {
        let mut content = Content::new(&new_file, traceback.clone());

        content.uid = insert_content(content.clone(), traceback.clone());

        //content.print(traceback.clone());
        insert_episode_if_episode(content.clone(), traceback.clone());
        /*

        let mut job = content.create_job();
        if worker.is_some() {
            job.prepare_tasks(
                worker.clone().unwrap(),
                Some(tracked_directories.cache_directories[0].clone()),
            );

            /*
            pub uid: usize,
            pub full_path: PathBuf,
            pub designation: Designation,
            //pub job_queue: VecDeque<Job>,
            pub hash: Option<u64>,
            //pub versions: Vec<FileVersion>,
            //pub metadata_dump
            pub show_uid: Option<usize>,
            pub show_title: Option<String>,
            pub show_season_episode: Option<(usize, usize)>,
            */
            insert_job(job.clone(), traceback.clone());
        }
        queue.add_job_to_queue(job);
        */
    }

    print_contents(traceback.clone());
    print_shows(traceback.clone());
    //print_jobs(traceback.clone());

    //queue.print();

    /* while queue.get_full_queue_length() > 0 {
        print::print(
            print::Verbosity::INFO,
            "queue_execution",
            format!("LIQ: {}", queue.get_full_queue_length().to_string()),
        );
        queue.run_job(worker.clone().unwrap());
    } */

    //shows.print();
}
