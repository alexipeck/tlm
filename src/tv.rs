use crate::{
    database::{create_show, establish_connection},
    diesel::prelude::*,
    generic::Generic,
    model::*,
    print::{print, From, Verbosity},
    schema::show::dsl::show as show_table,
    utility::Utility,
};

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct TV {
    pub shows: Vec<Show>,
}

impl TV {
    pub fn new(utility: Utility) -> TV {
        let utility = utility.clone_add_location("new(TV)");

        return TV {
            shows: Show::get_all_shows(utility.clone()),
        };
    }
}

#[derive(Clone, Debug)]
pub struct Episode {
    pub episode_uid: Option<usize>,
    pub generic: Generic,
    pub show_uid: usize,
    pub show_title: String,
    pub show_season: usize,
    pub show_episode: Vec<usize>,
}

impl Episode {
    pub fn new(
        generic: Generic,
        show_uid: usize,
        show_title: String,
        show_season: usize,
        show_episode: Vec<usize>,
    ) -> Self {
        return Episode {
            episode_uid: None,
            generic: generic,
            show_uid: show_uid,
            show_title: show_title,
            show_season: show_season,
            show_episode: show_episode,
        };
    }

    pub fn get_filename_from_pathbuf(pathbuf: PathBuf) -> String {
        return pathbuf.file_name().unwrap().to_str().unwrap().to_string();
    }

    pub fn get_episode_string(&self) -> String {
        let episode = self.show_episode.clone();
        if episode.len() < 1 {
            panic!("There was less than 1 episode in the thingo");
        } else {
            let mut episode_string = String::new();
            let mut first: bool = true;
            for episode in episode {
                if first {
                    episode_string.push_str(&format!("{}", episode));
                    first = false;
                } else {
                    episode_string += &format!("_{}", episode);
                }
            }
            return episode_string;
        }
    }

    pub fn print_episode(&self, utility: Utility) {
        let utility = utility.clone_add_location("print_episode(Episode)");

        //could realistically just check if it has an episode designation,
        print(
            Verbosity::DEBUG,
            From::TV,
            format!(
                "[generic_uid:'{:4}'][show_uid:'{:2}'][season:'{:2}'][episode:'{:2}'][full_path:'{}'][show_title:'{}']",
                self.generic.get_generic_uid(utility.clone()),
                self.show_uid,
                self.show_season,
                self.get_episode_string(),
                self.generic.get_full_path(),
                self.show_title,
            ),
            utility.preferences.generic_output_whitelisted,
            utility.clone(),
        );
    }
}

#[derive(Clone, Debug)]
pub struct Season {
    pub number: usize,
    pub episodes: Vec<Episode>,
}

impl Season {
    pub fn new(number: usize) -> Season {
        Season {
            number: number,
            episodes: Vec::new(),
        }
    }

    pub fn insert_in_order(&mut self, generic: Episode) {
        //not in order, but that's fine for now
        self.episodes.push(generic);
    }
}

#[derive(Clone, Debug)]
pub struct Show {
    pub show_uid: usize,
    pub title: String,
    pub seasons: Vec<Season>,
}

impl Show {
    pub fn new(uid: usize, show_title: String) -> Show {
        Show {
            show_uid: uid,
            title: show_title,
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
        let title_temp: String = show_model.show_title;

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
            .expect("Error loading show");

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
