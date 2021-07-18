use crate::print::{print as print, Verbosity};
use crate::content::Content;
use std::ops::{Index, IndexMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::database::db_ensure_show_exists;

static SHOW_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

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

    pub fn print_show(&self) {
        print(Verbosity::DEBUG, 
            "shows", 
            "print_show", 
            format!("[uid: {}][title: {}]", self.uid, self.title));
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

    //returns (uid, index)
    pub fn ensure_show_exists_by_title(&mut self, title: String) -> (usize, usize) {
        let mut index: usize = 0;
        for show in &self.shows {
            if show.title == title {
                return (show.uid, index);
            }
            index += 1;
        }
        let uid = SHOW_UID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_show = Show::new(uid, title.clone());
        db_ensure_show_exists(temp_show.title.clone());
        self.shows.push(temp_show);
        return (uid, index);
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

    //will overwrite data
    pub fn add_episode(&mut self, content: Content) {
        let show_index = self
            .ensure_show_exists_by_title(content.show_title.clone().unwrap())
            .1;
        self.ensure_season_exists_by_show_index_and_season_number(
            show_index,
            content.show_season_episode.clone().unwrap().0,
        );
        self.insert_in_order(show_index, content);
    }

    //insert show

    //exists

    //pub collect season

    //pub collect show

    pub fn print(&self) {
        for show in &self.shows {
            for season in &show.seasons {
                for episode in &season.episodes {
                    print(Verbosity::INFO, "shows", "shows.print", format!("{}", episode.get_filename_woe()));
                }
            }
        }
    }
}
