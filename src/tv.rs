use crate::diesel::prelude::*;
use crate::model::*;
use crate::{
    content::Content,
    print::{print, From, Verbosity},
    utility::Utility,
};
use crate::database::{establish_connection, create_show};
use crate::schema::show::dsl::show as show_table;

#[derive(Clone, Debug)]
pub struct TV {
    pub working_shows: Vec<Show>,
}

impl TV {
    pub fn new(utility: Utility) -> TV {
        let utility = utility.clone_add_location("new(TV)");

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

    pub fn insert_in_order(&mut self, c: Content) {
        //not in order, but that's fine for now
        self.episodes.push(c);
    }
}

#[derive(Clone, Debug)]
pub struct Show {
    pub show_uid: usize,
    pub title: String,
    pub seasons: Vec<Season>,
}

impl Show {
    pub fn new(uid: usize, t: String) -> Show {
        Show {
            show_uid: uid,
            title: t,
            seasons: Vec::new(),
        }
    }

    pub fn print_show(&self, utility: Utility) {
        let utility = utility.clone_add_location("print_show(Show)");
        print(
            Verbosity::DEBUG,
            From::Show,
            utility,
            format!("[uid: {}][title: {}]", self.show_uid, self.title),
        );
    }


    pub fn show_exists(
        show_title: String,
        working_shows: &Vec<Show>,
        utility: Utility,
    ) -> Option<usize> {
        let mut utility = utility.clone_add_location("show_exists(Show)");
        for s in working_shows {
            if s.title == show_title {
                return Some(s.show_uid);
            }
        }

        utility.print_function_timer();
        return None;
    }

    pub fn ensure_show_exists(
        show_title: String,
        working_shows: &mut Vec<Show>,
        utility: Utility,
    ) -> usize {
        let mut utility = utility.clone_add_location_start_timing("ensure_show_exists(Show)", 0);

        let show_uid = Show::show_exists(show_title.clone(), working_shows, utility.clone());
        match show_uid {
            Some(uid) => return uid,
            None => {
                let connection = establish_connection();
                if utility.print_timing {
                    print(
                        Verbosity::INFO,
                        From::TV,
                        utility.clone(),
                        format!("Adding a new show: {}", show_title),
                    );
                }
                let show_model = create_show(
                    &connection,
                    show_title.clone(),
                );
                utility.print_specific_timer_by_uid(0, utility.clone());

                let show_uid = show_model.show_uid as usize;
                let new_show = Show {
                    show_uid: show_uid,
                    title: show_title.clone(),
                    seasons: Vec::new(),
                };
                working_shows.push(new_show);

                return show_uid;
            }
        }
    }

    pub fn from_show_model(show_model: ShowModel, utility: Utility) -> Show {
        let mut utility = utility.clone_add_location_start_timing("from_show_model(Show)", 0);

        utility.add_timer(
            0,
            "startup: from_row: create show from row",
            utility.clone(),
        );
        let show_uid_temp: i32 = show_model.show_uid;
        let title_temp: String = show_model.title;

        let show = Show {
            show_uid: show_uid_temp as usize,
            title: title_temp,
            seasons: Vec::new(),
        };
        utility.print_function_timer();

        return show;
    }

    pub fn get_all_shows(utility: Utility) -> Vec<Show> {
        let mut utility = utility.clone_add_location_start_timing("get_all_shows(Show)", 0);
        utility.add_timer(0, "startup: read in shows", utility.clone());

        let connection = establish_connection();
        let raw_shows = show_table
            .load::<ShowModel>(&connection)
            .expect("Error loading content");

        let mut shows: Vec<Show> = Vec::new();
        for show in raw_shows {
            shows.push(Show::from_show_model(show, utility.clone()));
        }

        utility.print_function_timer();
        return shows;
    }
}

pub fn print_shows(shows: Vec<Show>, utility: Utility) {
    let utility = utility.clone_add_location("print_shows(Show)");

    for show in shows {
        show.print_show(utility.clone());
    }
}
