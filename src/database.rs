use crate::model::*;
use crate::schema::content as content_table;
use crate::schema::content::dsl::content as content_data;
use crate::schema::episode as episode_table;
use crate::schema::show as show_table;
use crate::utility::Utility;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;

///Sets up a connection to the database via DATABASE_URL environment variable
pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

///Inserts content data into the database
pub fn create_contents(conn: &PgConnection, new_contents: Vec<NewContent>) -> Vec<ContentModel> {
    diesel::insert_into(content_table::table)
        .values(&new_contents)
        .get_results(conn)
        .expect("Error saving new content")
}

///Inserts show data into the database
pub fn create_show(conn: &PgConnection, title: String) -> ShowModel {
    let new_show = NewShow { title };

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

///Get all content from the database
pub fn get_all_content(utility: Utility) -> Vec<ContentModel> {
    let mut utility = utility.clone_add_location("get_all_content (database)");
    let connection = establish_connection();

    utility.print_function_timer();
    content_data
        .load::<ContentModel>(&connection)
        .expect("Error loading content")
}
