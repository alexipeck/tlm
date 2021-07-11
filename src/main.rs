use tlm::{import_files, print, get_show_title_from_pathbuf};
mod content;
mod designation;
mod queue;
mod shows;
mod database;
use content::Content;
use designation::Designation;
use queue::Queue;
use shows::Shows;
use database::db_connect;

fn main() {
    //Queue
    let mut queue = Queue::new();

    //tracked directories - avoid crossover, it will lead to duplicate entries
    let mut tracked_root_directories: Vec<String> = Vec::new();
    if !cfg!(target_os = "windows") {
        //tracked_root_directories.push(String::from("/mnt/nas/tvshows")); //manual entry
        tracked_root_directories.push(String::from("/home/anpeck/tlm/test_files"));
    //manual entry
    } else {
        //tracked_root_directories.push(String::from("T:/")); //manual entry
        tracked_root_directories.push(String::from(
            r"C:\Users\Alexi Peck\Desktop\tlm\test_files\generics\",
        ));
        tracked_root_directories.push(String::from(
            r"C:\Users\Alexi Peck\Desktop\tlm\test_files\episodes\",
        ));
        //manual entry
    }

    //allowed video extensions
    let allowed_extensions = vec!["mp4", "mkv", "webm", "MP4"];

    //ignored directories
    let ignored_paths = vec![".recycle_bin"];

    let mut raw_filepaths = Vec::new();

    //Load all video files under tracked directories exluding all ignored paths
    import_files(
        &mut raw_filepaths,
        &tracked_root_directories,
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
        let error = db_connect(content.clone());
        match error {
            Err(err) => {
                println!("{}", err.to_string());
            }
            _ => {

            }
        }
        queue.main_queue.push_back(content);
    }
    let filenames: Vec<String> = Vec::new();
    //filenames.push(String::from(r"Weeds - S08E10 - Threshold Bluray-1080p.mkv"));
    //filenames.push(String::from(r"Weeds - S08E11 - God Willing and the Creek Don't Rise Bluray-1080p.mkv"));
    //filenames.push(String::from(r"Weeds - S08E12-13 - It's Time Bluray-1080p.mkv"));

    let uids: Vec<usize> = Vec::new();
    //uids.push(10);
    //uids.push(22);
    //uids.push(35);

    //queue.prioritise_content_by_title(filenames.clone());

    //queue.prioritise_content_by_uid(uids.clone());

    queue.print();

    /* while queue.get_full_queue_length() > 0 {
        queue.encode_and_rename_next_unreserved("NUC".to_string());
    } */

    shows.print();
    //add to db by filename, allowing the same file to be retargeted in another directory, without losing track of all the data associated with the episode

    //unify generic and episode naming (bring together)
}
