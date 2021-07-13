use std::collections::VecDeque;

use tlm::{content::Job, get_show_title_from_pathbuf, import_files, print};
mod content;
mod database;
mod designation;
mod queue;
mod shows;
//mod job;
use content::Content;
use database::{db_insert_content, db_purge, print_content};
use designation::Designation;
use queue::Queue;
use shows::Shows;

pub struct TrackedDirectories {
    pub root_directories: VecDeque<String>,
    pub cache_directories: VecDeque<String>,
}

impl TrackedDirectories {
    pub fn new() -> TrackedDirectories {
        TrackedDirectories {
            root_directories: VecDeque::new(),
            cache_directories: VecDeque::new(),
        }
    }
}

fn main() {
    db_purge();

    //tracked directories - avoid crossover, it will lead to duplicate entries
    let mut tracked_directories = TrackedDirectories::new();

    //manual entries
    if !cfg!(target_os = "windows") {
        //tracked_root_directories.push(String::from("/mnt/nas/tvshows")); //manual entry
        tracked_directories
            .root_directories
            .push_back(String::from(r"/home/anpeck/tlm/test_files/"));
        tracked_directories
            .cache_directories
            .push_back(String::from(r"/home/anpeck/tlm/test_files/cache/"))
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
    let mut queue = Queue::new(tracked_directories.cache_directories);

    //allowed video extensions
    let allowed_extensions = vec!["mp4", "mkv", "webm", "MP4"];

    //ignored directories
    let ignored_paths = vec![".recycle_bin"];

    let mut raw_filepaths = Vec::new();

    //Load all video files under tracked directories exluding all ignored paths
    import_files(
        &mut raw_filepaths,
        &tracked_directories.root_directories,
        &allowed_extensions,
        &ignored_paths,
    );

    //sort out filepaths into series and seasons
    let mut shows = Shows::new();

    //loop through all paths
    for raw_filepath in raw_filepaths {
        let mut content = Content::new(&raw_filepath);
        if content.show_title.is_some() {
            content.set_show_uid(
                shows
                    .ensure_show_exists_by_title(content.show_title.clone().unwrap())
                    .0,
            );
        }

        //dumping prepared values into Content struct based on Designation
        match content.designation {
            Designation::Episode => {
                content.show_title = Some(get_show_title_from_pathbuf(&raw_filepath));
                content.show_season_episode = content.show_season_episode;
                shows.add_episode(content.clone());
            }
            /*Designation::Movie => (

            ),*/
            _ => {}
        }

        db_insert_content(content.clone());
        let encode_string = content.generate_encode_string();
        let job = content.create_job(encode_string);
        queue.add_job_to_queue(job);
    }

    print_content();

    queue.print();

    while queue.get_full_queue_length() > 0 {
        print::print(
            print::Verbosity::INFO,
            "queue_execution",
            format!("LIQ: {}", queue.get_full_queue_length().to_string()),
        );
        queue.run_job("nuc".to_string());
    }

    shows.print();
}
