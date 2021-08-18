use crate::{
    content::Content,
    database::{
        execution::{get_by_query, get_client},
        retrieve::get_uid_from_result,
    },
    print::{print, From, Verbosity},
    utility::Utility,
};
use std::ops::{Index, IndexMut};
use tokio_postgres::Row;

#[derive(Clone, Debug)]
pub struct Season {
    pub number: usize,
    pub episodes: Vec<Content>,
}

impl Season {
    pub fn new(number: usize) -> Season {
        let episodes = Vec::new();
        Season {
            number: number,
            episodes: episodes,
        }
    }

    pub fn insert_in_order(&mut self, content: Content) {
        //not in order, but that's fine for now, just doing member stuff
        self.episodes.push(content);
    }
}
#[derive(Clone, Debug)]
pub struct Show {
    pub show_uid: usize,
    pub title: String,
    pub seasons: Vec<Season>,
}

impl Show {
    pub fn new(uid: usize, title: String) -> Show {
        Show {
            show_uid: uid,
            title: title,
            seasons: Vec::new(),
        }
    }

    pub fn print_show(&self, utility: Utility) {
        let utility = utility.clone_and_add_location("print_show");
        print(
            Verbosity::DEBUG,
            From::Shows,
            utility,
            format!("[uid: {}][title: {}]", self.show_uid, self.title),
            0,
        );
    }

    pub fn show_exists(show_title: String, working_shows: Vec<Show>) -> Option<usize> {
        for show in working_shows {
            if show.title == show_title {
                return Some(show.show_uid);
            }
        }
        return None;
    }

    pub fn ensure_show_exists(
        show_title: String,
        working_shows: &mut Vec<Show>,
        utility: Utility,
    ) -> usize {
        let mut utility = utility.clone_and_add_location("ensure_show_exists");

        let show_uid = Show::show_exists(show_title.clone(), working_shows.clone());
        if show_uid.is_some() {
            return show_uid.unwrap();
        } else {
            println!("Adding a new show: {}", show_title);
            utility.start_timer(0);
            let result = get_client(utility.clone()).query(
                r"INSERT INTO show (title) VALUES ($1) RETURNING show_uid;",
                &[&show_title],
            );
            utility.print_timer_from_stage_and_task(
                0,
                "startup",
                "inserting show UID",
                4,
                utility.clone(),
            );

            let show_uid = get_uid_from_result(result, utility.clone());
            let new_show = Show {
                show_uid: show_uid,
                title: show_title.clone(),
                seasons: Vec::new(),
            };
            working_shows.push(new_show);

            Show::show_exists(show_title.clone(), working_shows.clone());

            return show_uid;
        }
    }

    pub fn from_row(row: Row, utility: Utility) -> Show {
        let mut utility = utility.clone_and_add_location("from_row");

        utility.start_timer(0);
        let show_uid_temp: i32 = row.get(0);
        let title_temp: String = row.get(1);

        //change to have it pull all info out of the db, it currently generates what it can from the filename
        let show = Show {
            show_uid: show_uid_temp as usize,
            title: title_temp,
            seasons: Vec::new(),
        };
        utility.print_timer_from_stage_and_task(
            0,
            "startup",
            "from_row: create show from row",
            1,
            utility.clone(),
        );

        return show;
    }

    pub fn get_all_shows(utility: Utility) -> Vec<Show> {
        let mut utility = utility.clone_and_add_location("get_all_shows");
        utility.start_timer(0);

        let raw_shows = get_by_query(r"SELECT show_uid, title FROM show", utility.clone());

        let mut shows: Vec<Show> = Vec::new();
        for row in raw_shows {
            shows.push(Show::from_row(row, utility.clone()));
        }

        utility.print_timer_from_stage_and_task(0, "startup", "read in shows", 0, utility.clone());

        return shows;
    }
}

pub struct Shows {
    pub shows: Vec<Show>,
}

impl Shows {
    pub fn print(&self, utility: Utility) {
        let utility = utility.clone_and_add_location("print");
        
        for show in &self.shows {
            for season in &show.seasons {
                for episode in &season.episodes {
                    print(
                        Verbosity::INFO,
                        From::Shows,
                        utility.clone(),
                        format!("{}", episode.get_filename_woe()),
                        0,
                    );
                }
            }
        }
    }
}
