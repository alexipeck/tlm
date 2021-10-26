use crate::schema::generic::designation;
use crate::{
    designation::Designation, generic::Generic, model::*, schema::episode as episode_table,
    schema::episode::dsl::episode as episode_db, schema::generic as generic_table,
    schema::generic::dsl::generic as generic_data, schema::show as show_table,
    schema::show::dsl::show as show_db, show::Episode, show::Show,
};
use diesel::{pg::PgConnection, prelude::*};
use std::env;
use tracing::error;

///Sets up a connection to the database via DATABASE_URL environment variable
pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|err| {
        error!("DATABASE_URL must be set. Err: {}", err);
        panic!();
    });
    PgConnection::establish(&database_url).unwrap_or_else(|err| {
        error!("Error connecting to {}. Err: {}", database_url, err);
        panic!();
    })
}

///Inserts generic data into the database
pub fn create_generics(conn: &PgConnection, new_generics: Vec<NewGeneric>) -> Vec<GenericModel> {
    diesel::insert_into(generic_table::table)
        .values(&new_generics)
        .get_results(conn)
        .unwrap_or_else(|err| {
            error!("Error saving new generics. Err: {}", err);
            panic!();
        })
}

///Inserts show data into the database
pub fn create_show(conn: &PgConnection, show_title: String) -> ShowModel {
    let new_show = NewShow { show_title };

    diesel::insert_into(show_table::table)
        .values(&new_show)
        .get_result(conn)
        .unwrap_or_else(|err| {
            error!("Error saving new show. Err: {}", err);
            panic!();
        })
}

///Inserts episode data into the database
pub fn create_episodes(conn: &PgConnection, new_episode: Vec<NewEpisode>) -> Vec<EpisodeModel> {
    diesel::insert_into(episode_table::table)
        .values(&new_episode)
        .get_results(conn)
        .unwrap_or_else(|err| {
            error!("Error saving new episode. Err: {}", err);
            panic!();
        })
}

///Get all generic from the database
pub fn get_all_generics() -> Vec<Generic> {
    let connection = establish_connection();

    let generic_models = generic_data
        .filter(designation.eq(Designation::Generic as i32))
        .load::<GenericModel>(&connection)
        .unwrap_or_else(|err| {
            error!("Error loading generic. Err: {}", err);
            panic!();
        });

    let mut generics: Vec<Generic> = Vec::new();
    for generic_model in generic_models {
        generics.push(Generic::from_generic_model(generic_model));
    }
    generics
}

pub fn get_all_shows() -> Vec<Show> {
    let connection = establish_connection();
    let raw_shows = show_db
        .load::<ShowModel>(&connection)
        .unwrap_or_else(|err| {
            error!("Error loading shows. Err: {}", err);
            panic!();
        });

    //these all contain the episode designation
    let generic_models = generic_data
        .filter(designation.eq(Designation::Episode as i32))
        .load::<GenericModel>(&connection)
        .unwrap_or_else(|err| {
            error!("Error loading generic. Err: {}", err);
            panic!();
        });

    let mut generics: Vec<Generic> = Vec::new();
    for generic_model in generic_models {
        generics.push(Generic::from_generic_model(generic_model));
    }

    let episode_models = episode_db
        .load::<EpisodeModel>(&connection)
        .unwrap_or_else(|err| {
            error!("Error loading episodes. Err: {}", err);
            panic!();
        });
    let mut episodes: Vec<Episode> = Vec::new();

    for episode_model in episode_models {
        for generic in &generics {
            if generic.get_generic_uid() == episode_model.generic_uid as usize {
                let episode = Episode::new(
                    generic.clone(),
                    episode_model.show_uid as usize,
                    "".to_string(),
                    episode_model.season_number as usize,
                    vec![episode_model.episode_number as usize],
                ); //temporary first episode_number
                episodes.push(episode);
                break;
            }
        }
    }

    let mut shows: Vec<Show> = Vec::new();
    for show in raw_shows {
        shows.push(Show::from_show_model(show));
    }
    for episode in episodes {
        let show_uid = episode.show_uid;
        for show in &mut shows {
            if show.show_uid == show_uid {
                show.insert_episode(episode);
                break;
            }
        }
    }
    shows
}
