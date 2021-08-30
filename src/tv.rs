use crate::database::{create_show, establish_connection};
use crate::diesel::prelude::*;
use crate::model::*;
use crate::schema::show::dsl::show as show_table;
use crate::{
    content::Content,
    print::{print, From, Verbosity},
    utility::Utility,
};

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
        if !utility.preferences.print_shows && !utility.preferences.show_output_whitelisted {
            return;
        }
        print(
            Verbosity::DEBUG,
            From::Show,
            format!("[uid: {}][title: {}]", self.show_uid, self.title),
            false,
            utility,
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
        connection: &PgConnection,
    ) -> usize {
        let utility = utility.clone_add_location("ensure_show_exists(Show)");

        let show_uid = Show::show_exists(show_title.clone(), working_shows, utility.clone());
        match show_uid {
            Some(uid) => return uid,
            None => {
                if utility.preferences.print_shows || utility.preferences.show_output_whitelisted {
                    print(
                        Verbosity::INFO,
                        From::TV,
                        format!("Adding a new show: {}", show_title),
                        utility.preferences.show_output_whitelisted,
                        utility.clone(),
                    );
                }

                let show_model = create_show(connection, show_title.clone());

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
        let mut utility = utility.clone_add_location("from_show_model(Show)");

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
        let mut utility = utility.clone_add_location("get_all_shows(Show)");

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

    pub fn print_shows(shows: &Vec<Show>, utility: Utility) {
        let mut utility = utility.clone_add_location("print_shows(FileManager)");

        if !utility.preferences.print_shows {
            return;
        }

        for show in shows {
            show.print_show(utility.clone());
        }

        utility.print_function_timer();
    }
}
