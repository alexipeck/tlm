use crate::schema::generic::designation;
use crate::{
    designation::Designation, generic::Generic, model::*, schema::episode as episode_table,
    schema::generic as generic_table, schema::generic::dsl::generic as generic_data,
    schema::show as show_table, schema::show::dsl::show as show_db, tv::Show, utility::Utility,
};
use diesel::{pg::PgConnection, prelude::*};
use std::env;

///Sets up a connection to the database via DATABASE_URL environment variable
pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

///Inserts generic data into the database
pub fn create_generics(conn: &PgConnection, new_generics: Vec<NewGeneric>) -> Vec<GenericModel> {
    diesel::insert_into(generic_table::table)
        .values(&new_generics)
        .get_results(conn)
        .expect("Error saving new generic")
}

///Inserts show data into the database
pub fn create_show(conn: &PgConnection, show_title: String) -> ShowModel {
    let new_show = NewShow { show_title };

    diesel::insert_into(show_table::table)
        .values(&new_show)
        .get_result(conn)
        .expect("Error saving new show")
}

///Inserts episode data into the database
pub fn create_episodes(conn: &PgConnection, new_episode: Vec<NewEpisode>) -> Vec<EpisodeModel> {
    diesel::insert_into(episode_table::table)
        .values(&new_episode)
        .get_results(conn)
        .expect("Error saving new episode")
}

///Get all generic from the database
pub fn get_all_generics(utility: Utility) -> Vec<Generic> {
    let mut utility = utility.clone_add_location("get_all_generic(database)");
    let connection = establish_connection();

    utility.print_function_timer();
    let generic_models = generic_data
        .filter(designation.eq(Designation::Generic as i32))
        .load::<GenericModel>(&connection)
        .expect("Error loading generic");

    let mut generics: Vec<Generic> = Vec::new();
    for generic_model in generic_models {
        generics.push(Generic::from_generic_model(generic_model, utility.clone()));
    }
    generics
}

pub fn get_all_shows(utility: Utility) -> Vec<Show> {
    let mut utility = utility.clone_add_location("get_all_shows(Show)");

    let connection = establish_connection();
    let raw_shows = show_db
        .load::<ShowModel>(&connection)
        .expect("Error loading show");

    let mut shows: Vec<Show> = Vec::new();
    for show in raw_shows {
        shows.push(Show::from_show_model(show, utility.clone()));
    }

    utility.print_function_timer();
    shows
}
