use crate::database::{create_show, establish_connection};
use crate::diesel::prelude::*;
use crate::model::*;
use crate::schema::show::dsl::show as show_table;
use crate::{
    generic::Generic,
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
pub struct Episode {
    pub show_uid: usize,
    pub show_title: String,
    pub show_season_episode: (usize, Vec<usize>),
}

impl Episode {
    pub fn new() -> Self {
        return Episode {
            show_title: None,
            show_season_episode: None,
            show_uid: None,
        }
    }

    pub fn seperate_season_episode(&mut self) -> Option<(usize, Vec<usize>)> {
        fn rem_first_char(value: &str) -> &str {
            let mut chars = value.chars();
            chars.next();
            return chars.as_str();
        }
        
        let episode_string: String;
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r"S[0-9]*E[0-9\-]*").unwrap();
        }

        match REGEX.find(&self.get_filename()) {
            None => return None,
            Some(val) => episode_string = String::from(rem_first_char(val.as_str())),
        }

        let mut season_episode_iter = episode_string.split('E');
        let season_temp = season_episode_iter
            .next()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let mut episodes: Vec<usize> = Vec::new();
        for episode in season_episode_iter.next().unwrap().split('-') {
            episodes.push(episode.parse::<usize>().unwrap());
        }

        return Some((season_temp, episodes));
    }

    pub fn get_filename_from_pathbuf(pathbuf: PathBuf) -> String {
        return pathbuf.file_name().unwrap().to_str().unwrap().to_string();
    }

    pub fn get_filename(&self) -> String {
        return self
            .full_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
    }

    pub fn get_full_path_with_suffix_from_pathbuf(pathbuf: PathBuf, suffix: String) -> PathBuf {
        //C:\Users\Alexi Peck\Desktop\tlm\test_files\episodes\Test Show\Season 3\Test Show - S03E02 - tf8.mp4\_encodeH4U8\mp4
        //.push(self.full_path.extension().unwrap())
        //bad way of doing it
        let new_filename = format!(
            "{}{}.{}",
            pathbuf.file_stem().unwrap().to_string_lossy().to_string(),
            &suffix,
            pathbuf.extension().unwrap().to_string_lossy().to_string(),
        );
        return pathbuf.parent().unwrap().join(new_filename);
    }

    pub fn get_full_path_with_suffix(&self, suffix: String) -> PathBuf {
        //C:\Users\Alexi Peck\Desktop\tlm\test_files\episodes\Test Show\Season 3\Test Show - S03E02 - tf8.mp4\_encodeH4U8\mp4
        //.push(self.full_path.extension().unwrap())
        //bad way of doing it
        let new_filename = format!(
            "{}{}.{}",
            self.full_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            &suffix,
            self.full_path
                .extension()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        );
        return self.full_path.parent().unwrap().join(new_filename);
    }

    pub fn get_season_number(&self) -> usize {
        return self.show_season_episode.as_ref().unwrap().0;
    }

    pub fn get_show_title(&self, utility: Utility) -> String {
        let utility = utility.clone_add_location("get_show_title(Show)");

        if self.show_title.is_some() {
            return self.show_title.clone().unwrap();
        } else {
            print(
                Verbosity::CRITICAL,
                From::Generic,
                String::from("You called get_show_title on a content that didn't have an episode designation or was incorrectly created"),
                false,
                utility,
            );
            panic!();
        }
    }

    pub fn get_show_uid(&self, utility: Utility) -> usize {
        let utility = utility.clone_add_location("get_show_uid(Show)");

        if self.show_uid.is_some() {
            return self.show_uid.unwrap();
        } else {
            print(
                Verbosity::CRITICAL,
                From::Generic,
                String::from("You called get_show_uid on a content that didn't have an episode designation or was incorrectly created"),
                false,
                utility,
            );
            panic!();
        }
    }

    pub fn get_episode_string(&self) -> String {
        if self.show_season_episode.is_some() {
            let episode = self.show_season_episode.clone().unwrap().1;
            if episode.len() < 1 {
                panic!("There was less than 1 episode in the thingo");
            } else {
                let mut prepare = String::new();
                let mut first: bool = true;
                for episode in episode {
                    if first {
                        prepare.push_str(&format!("{}", episode));
                        first = false;
                    } else {
                        prepare += &format!("_{}", episode);
                    }
                }
                return prepare;
            }
        } else {
            panic!("show_season_episode is_none");
        }
    }

    pub fn print_episode(&self, utility: Utility) {
        let utility = utility.clone_add_location("print_episode(Episode)");

        //could realistically just check if it has an episode designation,
        if self.show_uid.is_some()
            && self.show_title.is_some()
            && self.show_season_episode.is_some()
        {
            print(
                Verbosity::DEBUG,
                From::Generic,
                format!(
                    "[generic_uid:'{:4}'][designation:'{}'][show_uid:'{:2}'][season:'{:2}'][episode:'{:2}'][full_path:'{}'][show_title:'{}']",
                    self.get_generic_uid(utility.clone()),
                    self.designation as i32,
                    self.get_show_uid(utility.clone()),
                    self.get_season_number(),
                    self.get_episode_string(),
                    self.get_full_path(),
                    self.get_show_title(utility.clone()),
                ),
                utility.preferences.content_output_whitelisted,
                utility.clone(),
            );
        } else {
            print(
                Verbosity::DEBUG,
                From::DB,
                format!(
                    "[generic_uid:'{}'][designation:'{}'][full_path:'{}']",
                    self.get_generic_uid(utility.clone()),
                    self.designation as i32,
                    self.get_full_path(),
                ),
                utility.preferences.content_output_whitelisted,
                utility.clone(),
            );
        }
    }
}

#[derive(Clone, Debug)]
pub struct Season {
    pub number: usize,
    pub episodes: Vec<Generic>,
}

impl Season {
    pub fn new(number: usize) -> Season {
        let episodes = Vec::new();
        Season {
            number: number,
            episodes: episodes,
        }
    }

    pub fn insert_in_order(&mut self, c: Generic) {
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
