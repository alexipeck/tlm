use crate::{
    model::*, schema::episode as episode_table, schema::generic as generic_table,
    schema::generic::dsl::generic as generic_data, schema::show as show_table, utility::Utility,
};
use diesel::{pg::PgConnection, prelude::*};
use std::env;

///Sets up a connection to the database via DATABASE_URL environment variable
pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

///Inserts generic data into the database
pub fn create_generics<'a>(
    conn: &PgConnection,
    new_generics: Vec<NewGeneric>,
) -> Vec<GenericModel> {
    diesel::insert_into(generic_table::table)
        .values(&new_generics)
        .get_results(conn)
        .expect("Error saving new generic")
}

///Inserts show data into the database
pub fn create_show<'a>(conn: &PgConnection, title: String) -> ShowModel {
    let new_show = NewShow { title: title };

    diesel::insert_into(show_table::table)
        .values(&new_show)
        .get_result(conn)
        .expect("Error saving new show")
}

///Inserts episode data into the database
pub fn create_episodes<'a>(conn: &PgConnection, new_episode: Vec<NewEpisode>) -> Vec<EpisodeModel> {
    diesel::insert_into(episode_table::table)
        .values(&new_episode)
        .get_results(conn)
        .expect("Error saving new episode")
}

///Get all generic from the database
pub fn get_all_generics(utility: Utility) -> Vec<GenericModel> {
    let mut utility = utility.clone_add_location("get_all_generic (database)");
    let connection = establish_connection();

    utility.print_function_timer();
    let data = generic_data
        .load::<GenericModel>(&connection)
        .expect("Error loading generic");
    return data;
}
