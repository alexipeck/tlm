use crate::{
    content::Content,
    print::{print, From, Verbosity},
    utility::Utility,
};
use crate::diesel::prelude::*;
use crate::{establish_connection, create_show};
use crate::model::*;
use crate::schema::show::dsl::*;

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

    pub fn show_exists(show_title: String, working_shows: Vec<Show>, utility: Utility) -> Option<usize> {
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

        let s_uid = Show::show_exists(show_title.clone(), working_shows.clone(), utility.clone());
        if s_uid.is_some() {
            return s_uid.unwrap();
        } else {
            let connection = establish_connection();
            if utility.print_timing {
                print(
                    Verbosity::INFO,
                    From::TV,
                    utility.clone(),
                    format!("Adding a new show: {}", show_title),
                );
            }
            utility.add_timer(0, "startup: inserting show UID", utility.clone());
            let show_model = create_show(
                &connection,
                show_title.clone(),
            );
            //utility.print_specific_timer_by_uid(0, utility.clone());

            let s_uid = show_model.show_uid as usize;
            let new_show = Show {
                show_uid: s_uid,
                title: show_title.clone(),
                seasons: Vec::new(),
            };
            working_shows.push(new_show);

            return s_uid;
        }
    }

    pub fn from_show_model(showModel: ShowModel, utility: Utility) -> Show {
        let mut utility = utility.clone_add_location_start_timing("from_show_model(Show)", 0);

        utility.add_timer(
            0,
            "startup: from_row: create show from row",
            utility.clone(),
        );
        let show_uid_temp: i32 = showModel.show_uid;
        let title_temp: String = showModel.title;

        //change to have it pull all info out of the db, it currently generates what it can from the filename
        let s = Show {
            show_uid: show_uid_temp as usize,
            title: title_temp,
            seasons: Vec::new(),
        };
        utility.print_function_timer();

        return s;
    }

    pub fn get_all_shows(utility: Utility) -> Vec<Show> {
        let mut utility = utility.clone_add_location_start_timing("get_all_shows(Show)", 0);
        utility.add_timer(0, "startup: read in shows", utility.clone());
        
        let connection = establish_connection();
        let raw_shows = show
            .load::<ShowModel>(&connection)
            .expect("Error loading content");

        let mut shows: Vec<Show> = Vec::new();
        for s in raw_shows {
            shows.push(Show::from_show_model(s, utility.clone()));
        }

        utility.print_function_timer();
        return shows;
    }
}

pub fn print_shows(shows: Vec<Show>, utility: Utility) {
    let utility = utility.clone_add_location("print_shows(Show)");

    for s in shows {
        print(
            Verbosity::INFO,
            From::Show,
            utility.clone(),
            format!("[title:{}]", s.title),
        );
    }
}
