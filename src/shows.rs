use crate::{
    content::Content,
    database::ensure::ensure_show_exists,
    print::{print, From, Verbosity},
    utility::Utility,
};
use std::ops::{Index, IndexMut};
use std::sync::atomic::{AtomicUsize, Ordering};

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

pub struct Show {
    pub uid: usize,
    pub title: String,
    pub seasons: Vec<Season>,
}

impl Show {
    pub fn new(uid: usize, title: String) -> Show {
        Show {
            uid: uid,
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
            format!("[uid: {}][title: {}]", self.uid, self.title),
        );
    }
}

//Season
impl Index<usize> for Show {
    type Output = Season;
    fn index(&self, season: usize) -> &Season {
        &self.seasons[season]
    }
}

impl IndexMut<usize> for Show {
    fn index_mut<'a>(&'a mut self, season: usize) -> &'a mut Season {
        &mut self.seasons[season]
    }
}

//Episode
impl Index<(usize, usize)> for Show {
    type Output = Content;
    fn index(&self, (season, episode): (usize, usize)) -> &Content {
        &self.seasons[season].episodes[episode]
    }
}

impl IndexMut<(usize, usize)> for Show {
    fn index_mut<'a>(&'a mut self, (season, episode): (usize, usize)) -> &'a mut Content {
        &mut self.seasons[season].episodes[episode]
    }
}

//Show
impl Index<usize> for Shows {
    type Output = Show;
    fn index(&self, show: usize) -> &Show {
        &self.shows[show]
    }
}

impl IndexMut<usize> for Shows {
    fn index_mut<'a>(&'a mut self, show: usize) -> &'a mut Show {
        &mut self.shows[show]
    }
}

pub struct Shows {
    pub shows: Vec<Show>,
}

impl Shows {
    fn find_index_by_uid(&self, uid: usize) -> Option<usize> {
        //if !is_none(show_uid)
        return self.shows.iter().position(|show| show.uid == uid);
    }

    pub fn new() -> Shows {
        Shows { shows: Vec::new() }
    }

    fn ensure_season_exists_by_show_index_and_season_number(
        &mut self,
        show_index: usize,
        season_number: usize,
    ) {
        for season in &mut self.shows[show_index].seasons {
            if season.number == season_number {
                break;
            }
        }
        self[show_index].seasons.push(Season::new(season_number));
    }

    //not actually in order
    fn insert_in_order(
        &mut self,
        show_index: usize,
        //season_number: usize,
        //_episode_number: usize,
        content: Content,
    ) {
        //remember episode_number
        //let mut inserted = false;
        for season in &mut self[show_index].seasons {
            if season.number == content.show_season_episode.unwrap().0 {
                //let mut index: usize = 0;

                season.insert_in_order(content.clone());
                /* for episode in &mut season.episodes {
                    let current = episode.show_season_episode.clone().unwrap().1.parse::<usize>().unwrap();
                    if index + 1 <= season.episodes.len() {
                        let next = season.episodes[index + 1].show_season_episode.clone().unwrap().1.parse::<usize>().unwrap();

                        if current < episode_number && next > episode_number {
                            season.episodes.insert(index, content);
                            inserted = true;
                        }
                    }
                    index += 1;
                }
                if !inserted {
                    season.episodes.push(content);
                } */
            }
        }
    }

    pub fn print(&self, utility: Utility) {
        let utility = utility.clone_and_add_location("print");

        /*
         * logic
         */
        for show in &self.shows {
            for season in &show.seasons {
                for episode in &season.episodes {
                    print(
                        Verbosity::INFO,
                        From::Shows,
                        utility.clone(),
                        format!("{}", episode.get_filename_woe()),
                    );
                }
            }
        }
        //////////
    }
}
