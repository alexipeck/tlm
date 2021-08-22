use crate::{
    content::Content,
    database::{
        execution::{get_by_query, get_client},
        retrieve::get_uid_from_result,
    },
    print::{print, From, Verbosity},
    utility::Utility,
};
use tokio_postgres::Row;

#[derive(Clone, Debug)]
pub struct TV {
    pub working_shows: Vec<Show>,
}

impl TV {
    pub fn new(utility: Utility) -> TV {
        let utility = utility.clone_and_add_location("new(TV)");

        return TV {
            working_shows: Show::get_all_shows(utility.clone()),
        };
    }
}

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
        //not in order, but that's fine for now
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
        let utility = utility.clone_and_add_location("print_show(Show)");
        print(
            Verbosity::DEBUG,
            From::Show,
            utility,
            format!("[uid: {}][title: {}]", self.show_uid, self.title),
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
        let mut utility = utility.clone_and_add_location("ensure_show_exists(Show)");

        let show_uid = Show::show_exists(show_title.clone(), working_shows.clone());
        if show_uid.is_some() {
            return show_uid.unwrap();
        } else {
            print(
                Verbosity::INFO,
                From::TV,
                utility.clone(),
                format!("Adding a new show: {}", show_title),
            );
            utility.add_timer(0, "startup: inserting show UID", utility.clone());
            let result = get_client(utility.clone()).query(
                r"INSERT INTO show (title) VALUES ($1) RETURNING show_uid;",
                &[&show_title],
            );
            utility.print_specific_timer_by_uid(0, utility.clone());

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
        let mut utility = utility.clone_and_add_location("from_row(Show)");

        utility.add_timer(
            0,
            "startup: from_row: create show from row",
            utility.clone(),
        );
        let show_uid_temp: i32 = row.get(0);
        let title_temp: String = row.get(1);

        //change to have it pull all info out of the db, it currently generates what it can from the filename
        let show = Show {
            show_uid: show_uid_temp as usize,
            title: title_temp,
            seasons: Vec::new(),
        };
        utility.print_specific_timer_by_uid(0, utility.clone());

        return show;
    }

    pub fn get_all_shows(utility: Utility) -> Vec<Show> {
        let mut utility = utility.clone_and_add_location("get_all_shows(Show)");
        utility.add_timer(0, "startup: read in shows", utility.clone());

        let raw_shows = get_by_query(r"SELECT show_uid, title FROM show", utility.clone());

        let mut shows: Vec<Show> = Vec::new();
        for row in raw_shows {
            shows.push(Show::from_row(row, utility.clone()));
        }

        utility.print_specific_timer_by_uid(0, utility.clone());

        return shows;
    }
}

pub fn print_shows(shows: Vec<Show>, utility: Utility) {
    let utility = utility.clone_and_add_location("print_shows(Show)");

    for show in shows {
        print(
            Verbosity::INFO,
            From::Show,
            utility.clone(),
            format!("[title:{}]", show.title),
        );
    }
}
