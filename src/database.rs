use crate::schema::content as content_table;
use crate::schema::show as show_table;
use crate::schema::episode as episode_table;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use crate::model::*;
use std::env;

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn create_content<'a>(
    conn: &PgConnection,
    full_path: String,
    designation: i32,
) -> ContentModel {


    let new_content = NewContent {
        full_path: full_path,
        designation: designation,
    };

    diesel::insert_into(content_table::table)
        .values(&new_content)
        .get_result(conn)
        .expect("Error saving new content")
}

pub fn create_show<'a>(
    conn: &PgConnection,
    title: String,
) -> ShowModel {


    let new_show = NewShow {
        title: title,
    };

    diesel::insert_into(show_table::table)
        .values(&new_show)
        .get_result(conn)
        .expect("Error saving new show")
}

pub fn create_episode<'a>(
    conn: &PgConnection,
    content_uid: i32,
    show_uid: i32,
    episode_title: String,
    season_number: i32,
    episode_number: i32,
) -> EpisodeModel {

    let new_episode = NewEpisode {
        content_uid: content_uid,
        show_uid: show_uid,
        episode_title: episode_title,
        season_number: season_number,
        episode_number: episode_number,
    };

    diesel::insert_into(episode_table::table)
        .values(&new_episode)
        .get_result(conn)
        .expect("Error saving new episode")
}