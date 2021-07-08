use tlm::{Content, Designation, Season, Show, Shows, Queue,
    import_files,
    encode,
    rename,
};

fn main() {
    //Queue
    let mut queue = Queue::new();

    //tracked directories - avoid crossover, it will lead to duplicate entries
    let mut tracked_root_directories: Vec<String> = Vec::new();
    if !cfg!(target_os = "windows") {
        //tracked_root_directories.push(String::from("/mnt/nas/tvshows")); //manual entry
        tracked_root_directories.push(String::from("/home/anpeck/tlm/test_files")); //manual entry
    } else {
        //tracked_root_directories.push(String::from("T:/")); //manual entry
        tracked_root_directories.push(String::from(r"C:\Users\Alexi Peck\Desktop\tlm\test_files\generic\"));
        tracked_root_directories.push(String::from(r"C:\Users\Alexi Peck\Desktop\tlm\test_files\episode\"));
        //manual entry
    }

    //allowed video extensions
    let allowed_extensions = vec!["mp4", "mkv", "webm", "MP4"];

    //ignored directories
    //currently works on both linux and windows
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
            content.set_show_uid(shows.ensure_show_exists_by_title(content.show_title.clone().unwrap()).0);
        }

        //prepare title
        let show_title = Content::get_show_title_from_pathbuf(&raw_filepath);

        //dumping prepared values into Content struct based on Designation
        match content.designation {
            Designation::Episode => {                
                let season_episode = content.show_season_episode;
                content.show_title = Some(show_title);
                content.show_season_episode = season_episode;
                shows.add_episode(content.clone());
            }
            /*Designation::Movie => (

            ),*/
            _ => {}
        }
        queue.main_queue.push(content);
    }
    let filenames: Vec<String> = Vec::new();
    //filenames.push(String::from(r"Weeds - S08E10 - Threshold Bluray-1080p.mkv"));
    //filenames.push(String::from(r"Weeds - S08E11 - God Willing and the Creek Don't Rise Bluray-1080p.mkv"));
    //filenames.push(String::from(r"Weeds - S08E12-13 - It's Time Bluray-1080p.mkv"));

    let uids: Vec<usize> = Vec::new();
    //uids.push(10);
    //uids.push(22);
    //uids.push(35);

    queue.prioritise_content_by_title(filenames.clone());

    queue.prioritise_content_by_uid(uids.clone());

    for content in &queue.priority_queue {
        println!("{}{}", content.parent_directory, content.filename);
    }

    for content in &queue.main_queue {
        println!("{}{}", content.parent_directory, content.filename);
    }

    for content in queue.main_queue {
        let source = content.get_full_path();
        let encode_target = content.get_full_path_with_suffix("_encode".to_string());//want it to use the actual extension rather than just .mp4
        let rename_target = content.get_full_path_specific_extension("mp4".to_string());
        println!("Starting encode of {}\nEncoding to {}_encode.mp4", content.get_filename(), content.get_filename_woe());
        encode(&source, &encode_target);
        rename(&encode_target, &rename_target);
    }

    shows.print();
    //add to db by filename, allowing the same file to be retargeted in another directory, without losing track of all the data associated with the episode

    //unify generic and episode naming (bring together)
}
